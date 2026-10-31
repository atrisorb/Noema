//! SQLite persistence: versioned migrations, WAL, session continuity and the
//! vector-store fallback used until Block 3 writes embeddings.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};
use thiserror::Error;

use crate::ids::{AgentId, ModelId, ProjectId, SessionId};
use crate::session::{ResumeState, Session, SessionStore};

const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../migrations/0001_init.sql")),
    (2, include_str!("../migrations/0002_studio.sql")),
];

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("hardware: {0}")]
    Hardware(String),
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Which vector path is active. `SqliteVec` is only reported when the
/// extension actually loaded and answers `vec_version()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorBackend {
    SqliteVec,
    BruteForce,
}

pub struct Store {
    conn: Connection,
    vector: VectorBackend,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        Self::init(Connection::open(path)?)
    }

    pub fn open_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self, StoreError> {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA synchronous=NORMAL;",
        )?;
        let mut store = Self {
            conn,
            vector: VectorBackend::BruteForce,
        };
        store.vector = store.load_vec_extension();
        store.migrate()?;
        Ok(store)
    }

    /// Optional `sqlite-vec`: loaded only when `NOEMA_SQLITE_VEC` points to the
    /// shared library. Any failure keeps the brute-force cosine path.
    fn load_vec_extension(&self) -> VectorBackend {
        let Ok(path) = std::env::var("NOEMA_SQLITE_VEC") else {
            return VectorBackend::BruteForce;
        };
        // Safety: extension loading is gated by the operator's env var above.
        let loaded = unsafe {
            self.conn
                .load_extension_enable()
                .and_then(|()| self.conn.load_extension(&path, None::<&str>))
        }
        .and_then(|()| {
            self.conn
                .query_row("SELECT vec_version()", [], |r| r.get::<_, String>(0))
                .map(|_| ())
        });
        match loaded {
            Ok(()) => VectorBackend::SqliteVec,
            Err(e) => {
                tracing::warn!(%e, "sqlite-vec unavailable, using brute-force cosine");
                VectorBackend::BruteForce
            }
        }
    }

    pub fn vector_backend(&self) -> VectorBackend {
        self.vector
    }

    /// Cached hardware profile + effective tuning (12 h TTL, `refresh` forces).
    pub fn hardware_report(
        &self,
        refresh: bool,
    ) -> Result<crate::hardware::HardwareReport, StoreError> {
        crate::hardware::report(&self.conn, refresh).map_err(StoreError::Hardware)
    }

    /// Deep-merge a tuning patch into `hardware.overrides`.
    pub fn set_hardware_overrides(
        &self,
        patch: &serde_json::Value,
    ) -> Result<crate::hardware::HardwareReport, StoreError> {
        crate::hardware::set_overrides(&self.conn, patch).map_err(StoreError::Hardware)
    }

    /// Drop the cached profile and every override, then detect again.
    pub fn reset_hardware(&self) -> Result<crate::hardware::HardwareReport, StoreError> {
        crate::hardware::reset(&self.conn).map_err(StoreError::Hardware)
    }

    fn migrate(&mut self) -> Result<(), StoreError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);",
        )?;
        for (version, sql) in MIGRATIONS {
            if self.applied(*version)? {
                continue;
            }
            let tx = self.conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                params![version, now_ms()],
            )?;
            tx.commit()?;
            tracing::info!(version, "migration applied");
        }
        Ok(())
    }

    fn applied(&self, version: i64) -> Result<bool, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT 1 FROM schema_migrations WHERE version = ?1",
                [version],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    pub fn resume(&self) -> Result<ResumeState, StoreError> {
        let mut state = ResumeState::default();
        let row = self
            .conn
            .query_row(
                "SELECT id, project_id, agent_id, cwd, repo, branch, widget_layout
                 FROM sessions ORDER BY updated_at DESC LIMIT 1",
                [],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, Option<String>>(4)?,
                        r.get::<_, Option<String>>(5)?,
                        r.get::<_, Option<String>>(6)?,
                    ))
                },
            )
            .optional()?;
        if let Some((id, project, agent, cwd, repo, branch, layout)) = row {
            state.diff_stat = diff_stat(&cwd);
            state.last_session = Some(SessionId(id));
            state.last_project = project.map(ProjectId);
            state.last_agent = Some(AgentId(agent));
            state.last_cwd = Some(cwd);
            state.last_repo = repo;
            state.last_branch = branch;
            state.widget_layout = layout;
        }
        state.role_models = self.role_models()?;
        Ok(state)
    }

    fn role_models(&self) -> Result<BTreeMap<String, ModelId>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT capability, id FROM models ORDER BY created_at")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
        let mut out = BTreeMap::new();
        for row in rows {
            let (capability, id) = row?;
            out.entry(capability).or_insert(ModelId(id));
        }
        Ok(out)
    }

    pub fn save(&self, s: &Session) -> Result<(), StoreError> {
        let now = now_ms();
        self.conn.execute(
            "INSERT INTO sessions
               (id, project_id, agent_id, model_id, cwd, repo, branch, widget_layout, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
             ON CONFLICT(id) DO UPDATE SET
               project_id = excluded.project_id, agent_id = excluded.agent_id,
               model_id = excluded.model_id, cwd = excluded.cwd, repo = excluded.repo,
               branch = excluded.branch, widget_layout = excluded.widget_layout,
               updated_at = excluded.updated_at",
            params![
                s.id.0,
                s.project.as_ref().map(|p| p.0.clone()),
                s.agent.0,
                s.model.0,
                s.cwd,
                s.repo,
                s.branch,
                s.widget_layout,
                now
            ],
        )?;
        Ok(())
    }

    /// Foreign-key pre-check so RPC callers get a params error, not a sqlite one.
    pub fn project_exists(&self, id: &str) -> Result<bool, StoreError> {
        Ok(self
            .conn
            .query_row("SELECT 1 FROM projects WHERE id = ?1", [id], |_| Ok(()))
            .optional()?
            .is_some())
    }

    pub fn load(&self, id: &SessionId) -> Result<Option<Session>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, project_id, agent_id, model_id, cwd, repo, branch, widget_layout, updated_at
                 FROM sessions WHERE id = ?1",
                [&id.0],
                |r| {
                    Ok(Session {
                        id: SessionId(r.get(0)?),
                        project: r.get::<_, Option<String>>(1)?.map(ProjectId),
                        agent: AgentId(r.get(2)?),
                        model: ModelId(r.get(3)?),
                        cwd: r.get(4)?,
                        repo: r.get(5)?,
                        branch: r.get(6)?,
                        widget_layout: r.get(7)?,
                        updated_at: r.get(8)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn list_sessions(&self, limit: u32) -> Result<Vec<Session>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, agent_id, model_id, cwd, repo, branch, widget_layout, updated_at
             FROM sessions ORDER BY updated_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit], |r| {
            Ok(Session {
                id: SessionId(r.get(0)?),
                project: r.get::<_, Option<String>>(1)?.map(ProjectId),
                agent: AgentId(r.get(2)?),
                model: ModelId(r.get(3)?),
                cwd: r.get(4)?,
                repo: r.get(5)?,
                branch: r.get(6)?,
                widget_layout: r.get(7)?,
                updated_at: r.get(8)?,
            })
        })?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    /// Brute-force cosine over `thoughts.embedding`; used until the vector
    /// table is populated in Block 3.
    pub fn nearest(&self, query: &[f32], limit: usize) -> Result<Vec<(String, f32)>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, embedding FROM thoughts WHERE embedding IS NOT NULL")?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?))
        })?;
        let mut scored = Vec::new();
        for row in rows {
            let (id, blob) = row?;
            let vec = crate::vector::from_blob(&blob);
            if vec.len() == query.len() {
                scored.push((id, crate::vector::cosine(query, &vec)));
            }
        }
        scored.sort_by(|a, b| b.1.total_cmp(&a.1));
        scored.truncate(limit);
        Ok(scored)
    }
}

impl SessionStore for Store {
    type Error = StoreError;

    fn resume_state(&self) -> Result<ResumeState, Self::Error> {
        self.resume()
    }

    fn load(&self, id: &SessionId) -> Result<Option<Session>, Self::Error> {
        Store::load(self, id)
    }

    fn save(&self, session: &Session) -> Result<(), Self::Error> {
        Store::save(self, session)
    }
}

fn diff_stat(cwd: &str) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["-C", cwd, "diff", "--stat"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .last()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(cwd: &str) -> Session {
        Session {
            id: SessionId::new(),
            project: None,
            agent: AgentId("agent:noema-axon".into()),
            model: ModelId("deepseek/deepseek-flash".into()),
            cwd: cwd.into(),
            repo: Some("owner/repo".into()),
            branch: Some("main".into()),
            widget_layout: Some(r#"[{"id":"w1"}]"#.into()),
            updated_at: 0,
        }
    }

    #[test]
    fn migrations_apply_once() {
        let mut store = Store::open_memory().unwrap();
        store.migrate().unwrap();
        let n: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, MIGRATIONS.len() as i64);
    }

    #[test]
    fn resume_returns_last_session_and_layout() {
        let store = Store::open_memory().unwrap();
        let s = session(".");
        store.save(&s).unwrap();
        let state = store.resume().unwrap();
        assert_eq!(state.last_session, Some(s.id.clone()));
        assert_eq!(
            state.last_agent.as_ref().map(|a| a.0.as_str()),
            Some("agent:noema-axon")
        );
        assert_eq!(state.widget_layout.as_deref(), Some(r#"[{"id":"w1"}]"#));
        assert_eq!(
            store.load(&s.id).unwrap().unwrap().branch.as_deref(),
            Some("main")
        );
    }
}
