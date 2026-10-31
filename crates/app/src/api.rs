//! Anatomy projection for the 3D world. Reads the NOEMA database and composes
//! every greeting here, deterministically: no network client exists in this crate.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use noema_inference::{Activation, ActivationWindow};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

pub const DEFAULT_IDENTITY: &str = "NOEMA is the living map of this machine: seven agents, one body, one memory.";
pub const DEFAULT_PROMPT: &str = "You are NOEMA. Answer from the local database first. Never invent telemetry. Keep the owner's session, repo and branch in context.";

const LIVE_TTL_MS: i64 = 30_000;

pub struct Api {
    pub db: Arc<Mutex<Connection>>,
    pub live: Arc<Mutex<LiveTelemetry>>,
}

/// Latest activation window fed by Thalamus. Fresh samples override the
/// derived pulse; older samples fall back to stored importance/cosine.
#[derive(Default)]
pub struct LiveTelemetry {
    pub window: ActivationWindow,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Default)]
pub struct AgentFacts {
    pub id: String,
    pub name: String,
    pub role: String,
    pub model: String,
    pub greeting: String,
    pub skills: Vec<String>,
    pub plugins: Vec<String>,
    pub mcp: Vec<String>,
    pub events: usize,
    pub tokens: i64,
    pub cost: f64,
    pub recent: Vec<(String, i64, f64, i64)>,
    pub models: Vec<(String, i64, f64)>,
    pub cwd: Option<String>,
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub last_seen: i64,
}

const ACCENTS: [&str; 7] = [
    "#7ee7ff", "#ffb86b", "#b9a2ff", "#7dffb0", "#ff7d9c", "#ffd166", "#8fb3ff",
];

fn thousands(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 { format!("-{out}") } else { out }
}

fn parse_list(raw: Option<String>) -> Vec<String> {
    let Some(raw) = raw else { return Vec::new() };
    match serde_json::from_str::<Vec<String>>(&raw) {
        Ok(list) => list,
        Err(_) => raw.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect(),
    }
}

/// The whole greeting is a pure function of database rows.
pub fn compose_greeting(f: &AgentFacts) -> String {
    let opener = if f.greeting.trim().is_empty() {
        format!("{} online.", f.name)
    } else {
        f.greeting.trim().to_string()
    };
    let place = f
        .repo
        .clone()
        .or_else(|| f.cwd.clone())
        .unwrap_or_else(|| "no workspace".into());
    format!(
        "{opener} {place} · {} events · {} tokens · ${:.2} · {} skills / {} plugins / {} MCP · model {}.",
        f.events,
        thousands(f.tokens),
        f.cost,
        f.skills.len(),
        f.plugins.len(),
        f.mcp.len(),
        f.model,
    )
}

fn facts(conn: &Connection, id: &str) -> rusqlite::Result<Option<AgentFacts>> {
    let base = conn
        .query_row(
            "SELECT name, role, model_id, greeting, skills, plugins, mcp_servers
             FROM agents WHERE id = ?1",
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                ))
            },
        )
        .optional()?;
    let Some((name, role, model, greeting, skills, plugins, mcp)) = base else {
        return Ok(None);
    };
    let mut f = AgentFacts {
        id: id.to_string(),
        name,
        role: role.unwrap_or_default(),
        model: model.unwrap_or_else(|| "unassigned".into()),
        greeting,
        skills: parse_list(skills),
        plugins: parse_list(plugins),
        mcp: parse_list(mcp),
        ..AgentFacts::default()
    };
    let (events, tokens, cost): (i64, i64, f64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(tokens),0), COALESCE(SUM(cost_usd),0)
         FROM usage_events WHERE agent_id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    f.events = events as usize;
    f.tokens = tokens;
    f.cost = cost;

    let mut stmt = conn.prepare(
        "SELECT kind, tokens, cost_usd, created_at FROM usage_events
         WHERE agent_id = ?1 ORDER BY created_at DESC LIMIT 6",
    )?;
    f.recent = stmt
        .query_map([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<Result<_, _>>()?;

    let mut stmt = conn.prepare(
        "SELECT model_id, COALESCE(SUM(tokens),0), COALESCE(SUM(cost_usd),0)
         FROM usage_events WHERE agent_id = ?1 GROUP BY model_id ORDER BY 3 DESC",
    )?;
    f.models = stmt
        .query_map([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<_, _>>()?;

    let session = conn
        .query_row(
            "SELECT cwd, repo, branch, updated_at FROM sessions
             WHERE agent_id = ?1 ORDER BY updated_at DESC LIMIT 1",
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?;
    if let Some((cwd, repo, branch, last_seen)) = session {
        f.cwd = Some(cwd);
        f.repo = repo;
        f.branch = branch;
        f.last_seen = last_seen;
    }
    Ok(Some(f))
}

fn settings(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1 AND scope = 'anima'",
        [key],
        |r| r.get(0),
    )
    .optional()
}

fn put_settings(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings(key, value, scope) VALUES (?1, ?2, 'anima')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

const STUDIO_ROLES: [&str; 5] = ["embedding", "router", "chat", "coding", "dreaming"];

fn studio_setting(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()
}

fn put_studio_setting(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings(key, value, scope) VALUES (?1, ?2, 'studio')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, scope = excluded.scope",
        params![key, value],
    )?;
    Ok(())
}

fn role_key(capability: &str, project: Option<&str>) -> String {
    match project {
        Some(p) => format!("role.{capability}@project:{p}"),
        None => format!("role.{capability}@global"),
    }
}

fn first_model(conn: &Connection, capability: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT id FROM models WHERE capability = ?1 ORDER BY created_at LIMIT 1",
        [capability],
        |r| r.get(0),
    )
    .optional()
}

fn grid_value(w: &Value, key: &str, default: i64) -> i64 {
    w.get(key).and_then(Value::as_i64).unwrap_or(default)
}

fn ensure_widgets(conn: &Connection) -> rusqlite::Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM widgets", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }
    let defaults = [
        ("widget-tokens-agent", "tokens by agent", json!({"metric": "tokens", "dimension": "agent", "filters": []}), 0, 0, 6, 4),
        ("widget-cost-model", "cost by model", json!({"metric": "cost", "dimension": "model", "filters": []}), 6, 0, 6, 4),
        ("widget-latency-peak", "latency by peak band", json!({"metric": "latency", "dimension": "peak", "filters": []}), 0, 4, 6, 3),
        ("widget-thinking-kind", "tokens by thinking", json!({"metric": "tokens", "dimension": "thinking", "filters": []}), 6, 4, 6, 3),
    ];
    for (id, name, spec, x, y, w, h) in defaults {
        upsert_widget(conn, &json!({"id": id, "name": name, "chart": "bars", "spec": spec, "x": x, "y": y, "w": w, "h": h}))?;
    }
    Ok(())
}

fn upsert_widget(conn: &Connection, w: &Value) -> rusqlite::Result<()> {
    let id = w
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("widget-{}", now_ms()));
    let name = w.get("name").and_then(Value::as_str).unwrap_or("widget").trim();
    let chart = w.get("chart").and_then(Value::as_str).unwrap_or("bars");
    let spec = w.get("spec").map(Value::to_string).unwrap_or_else(|| "{}".into());
    let x = grid_value(w, "x", 0).clamp(0, 11);
    let width = grid_value(w, "w", 4).clamp(2, 12).min(12 - x);
    let y = grid_value(w, "y", 0).max(0);
    let height = grid_value(w, "h", 3).clamp(2, 8);
    let pinned = i64::from(w.get("pinned").and_then(Value::as_bool).unwrap_or(false));
    conn.execute(
        "INSERT INTO widgets(id,name,chart,spec,x,y,w,h,pinned) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name, chart=excluded.chart, spec=excluded.spec,
           x=excluded.x, y=excluded.y, w=excluded.w, h=excluded.h, pinned=excluded.pinned",
        params![id, name, chart, spec, x, y, width, height, pinned],
    )?;
    Ok(())
}

impl Api {
    pub fn anatomy(&self) -> Result<Value, String> {
        let conn = self.db.lock().map_err(|_| "db poisoned".to_string())?;
        let identity = settings(&conn, "anima.identity")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| DEFAULT_IDENTITY.into());
        let system_prompt = settings(&conn, "anima.system_prompt")
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| DEFAULT_PROMPT.into());
        let ids: Vec<String> = conn
            .prepare("SELECT id FROM agents ORDER BY name")
            .and_then(|mut s| s.query_map([], |r| r.get::<_, String>(0))?.collect())
            .map_err(|e| e.to_string())?;
        let mut agents = Vec::with_capacity(ids.len());
        for (index, id) in ids.iter().enumerate() {
            let Some(f) = facts(&conn, id).map_err(|e| e.to_string())? else { continue };
            let accent = ACCENTS[index % ACCENTS.len()];
            let recent: Vec<Value> = f
                .recent
                .iter()
                .map(|(kind, tokens, cost, at)| json!({"kind": kind, "tokens": tokens, "cost_usd": cost, "at": at}))
                .collect();
            let by_model: Vec<Value> = f
                .models
                .iter()
                .map(|(model, tokens, cost)| json!({"model": model, "tokens": tokens, "cost_usd": cost}))
                .collect();
            agents.push(json!({
                "id": f.id,
                "name": f.name,
                "role": f.role,
                "model": f.model,
                "accent": accent,
                "greeting": compose_greeting(&f),
                "tabs": {
                    "activity": {"events": recent},
                    "tokens": {"total": f.tokens, "by_model": by_model},
                    "costs": {"total_usd": f.cost, "by_model": by_model},
                    "ecosystem": {"skills": f.skills, "plugins": f.plugins, "mcp": f.mcp},
                    "workspace": {
                        "cwd": f.cwd, "repo": f.repo, "branch": f.branch, "last_seen": f.last_seen
                    }
                }
            }));
        }
        Ok(json!({
            "version": noema_core::VERSION,
            "anima": {"identity": identity, "system_prompt": system_prompt},
            "agents": agents
        }))
    }

    pub fn set_anima(&self, body: &Value) -> Result<Value, String> {
        let identity = body.get("identity").and_then(Value::as_str).unwrap_or(DEFAULT_IDENTITY).trim().to_string();
        let system_prompt = body.get("system_prompt").and_then(Value::as_str).unwrap_or(DEFAULT_PROMPT).trim().to_string();
        if identity.len() > 8_000 || system_prompt.len() > 16_000 {
            return Err("anima text too long".into());
        }
        let conn = self.db.lock().map_err(|_| "db poisoned".to_string())?;
        put_settings(&conn, "anima.identity", &identity).map_err(|e| e.to_string())?;
        put_settings(&conn, "anima.system_prompt", &system_prompt).map_err(|e| e.to_string())?;
        Ok(json!({"identity": identity, "system_prompt": system_prompt}))
    }

    /// Hardware profile + derived tuning. `refresh` bypasses the 12 h cache.
    pub fn system(&self, refresh: bool) -> Result<Value, String> {
        let conn = self.db.lock().map_err(|_| "db poisoned".to_string())?;
        let report = noema_core::hardware::report(&conn, refresh)?;
        serde_json::to_value(report).map_err(|e| e.to_string())
    }

    /// Persist a tuning patch; the body is either `{overrides:{...}}` or the patch itself.
    pub fn set_system_overrides(&self, body: &Value) -> Result<Value, String> {
        let patch = body.get("overrides").unwrap_or(body);
        let conn = self.db.lock().map_err(|_| "db poisoned".to_string())?;
        let report = noema_core::hardware::set_overrides(&conn, patch)?;
        serde_json::to_value(report).map_err(|e| e.to_string())
    }

    /// Thought graph for the brain view: positions come from the incremental
    /// projection, edges from cosine kNN, pulse from derived activation telemetry.
    pub fn brain(&self) -> Result<Value, String> {
        let conn = self.db.lock().map_err(|_| "db poisoned".to_string())?;
        let mut stmt = conn
            .prepare("SELECT id,title,body,agent_id,created_at,importance,cluster_id,embedding,pos_x,pos_y,pos_z FROM thoughts")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, f64>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<Vec<u8>>>(7)?,
                    r.get::<_, Option<f64>>(8)?,
                    r.get::<_, Option<f64>>(9)?,
                    r.get::<_, Option<f64>>(10)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut thoughts = Vec::new();
        for row in rows {
            let (id, title, body, agent_id, created_at, importance, cluster, blob, px, py, pz) = row.map_err(|e| e.to_string())?;
            let Some(blob) = blob else { continue };
            let stored = match (px, py, pz) {
                (Some(x), Some(y), Some(z)) => Some([x as f32, y as f32, z as f32]),
                _ => None,
            };
            thoughts.push((id, title, body, agent_id, created_at, importance, cluster, noema_core::vector::from_blob(&blob), stored));
        }
        let inputs: Vec<noema_hippocampus::Input> = thoughts
            .iter()
            .map(|t| noema_hippocampus::Input {
                id: t.0.clone(),
                embedding: t.7.clone(),
                region: region_of(t.6.as_deref()),
            })
            .collect();
        let pos = noema_hippocampus::project(&inputs, 160);
        let live_activation = {
            let live = self.live.lock().map_err(|_| "telemetry poisoned".to_string())?;
            let fresh = now_ms() - live.updated_at_ms <= LIVE_TTL_MS;
            (fresh && !live.window.is_empty()).then(|| (live.window.activation(), live.updated_at_ms))
        };
        let mut edges = Vec::new();
        let mut nodes = Vec::with_capacity(thoughts.len());
        for (i, t) in thoughts.iter().enumerate() {
            let mut scored: Vec<(usize, f32)> = (0..thoughts.len())
                .filter(|&j| j != i)
                .map(|j| (j, noema_core::vector::cosine(&t.7, &thoughts[j].7)))
                .collect();
            scored.sort_by(|a, b| b.1.total_cmp(&a.1));
            scored.truncate(3);
            let sim = if scored.is_empty() { 0.0 } else { scored.iter().map(|s| s.1).sum::<f32>() / scored.len() as f32 };
            for (j, weight) in &scored {
                if *weight > 0.35 && i < *j {
                    edges.push(json!({"from": t.0, "to": thoughts[*j].0, "weight": weight}));
                }
            }
            let derived = noema_hippocampus::Activation {
                logprob_entropy: (t.5.clamp(0.0, 1.0) as f32) * 2.6,
                thinking_density: t.5.clamp(0.0, 1.0) as f32,
                cosine_drift: (1.0 - sim).clamp(0.0, 1.0),
            };
            let pulse = match live_activation {
                Some((activation, _)) => brain_activation(activation).pulse(),
                None => derived.pulse(),
            };
            nodes.push(json!({
                "id": t.0, "title": t.1, "body": t.2, "agent_id": t.3, "created_at": t.4,
                "importance": t.5, "region": region_of(t.6.as_deref()).label(),
                "pos": t.8.unwrap_or_else(|| pos.get(i).copied().unwrap_or([0.0, 0.0, 0.0])),
                "pulse": pulse,
            }));
        }
        Ok(json!({
            "nodes": nodes,
            "edges": edges,
            "regions": noema_hippocampus::Region::ALL.iter()
                .map(|r| json!({"id": r.label(), "anchor": r.anchor()}))
                .collect::<Vec<_>>(),
            "telemetry": match live_activation {
                Some((activation, updated_at_ms)) => json!({
                    "source": "live",
                    "activation": activation,
                    "pulse": brain_activation(activation).pulse(),
                    "updated_at_ms": updated_at_ms,
                }),
                None => json!({"source": "derived", "updated_at_ms": 0}),
            },
            "generated_at_ms": now_ms(),
        }))
    }

    /// Thalamus pushes normalised StreamEvent batches here; the next brain read
    /// and the telemetry endpoint use the same window.
    pub fn ingest_telemetry(&self, body: &Value) -> Result<Value, String> {
        let mut live = self.live.lock().map_err(|_| "telemetry poisoned".to_string())?;
        if body.get("reset").and_then(Value::as_bool).unwrap_or(false) {
            live.window.reset();
        }
        if let Some(events) = body.get("events") {
            let events: Vec<noema_inference::StreamEvent> =
                serde_json::from_value(events.clone()).map_err(|e| format!("events: {e}"))?;
            for event in &events {
                live.window.ingest(event);
            }
        }
        if let Some(drift) = body.get("cosine_drift").and_then(Value::as_f64) {
            live.window.set_cosine_drift(drift as f32);
        }
        live.updated_at_ms = now_ms();
        let activation = live.window.activation();
        let source = if live.window.is_empty() { "empty" } else { "live" };
        Ok(json!({
            "source": source,
            "activation": activation,
            "pulse": brain_activation(activation).pulse(),
            "updated_at_ms": live.updated_at_ms,
        }))
    }

    pub fn telemetry(&self) -> Result<Value, String> {
        let live = self.live.lock().map_err(|_| "telemetry poisoned".to_string())?;
        let fresh = now_ms() - live.updated_at_ms <= LIVE_TTL_MS;
        let activation = live.window.activation();
        let source = if fresh && !live.window.is_empty() { "live" } else { "empty" };
        Ok(json!({
            "source": source,
            "activation": activation,
            "pulse": brain_activation(activation).pulse(),
            "updated_at_ms": live.updated_at_ms,
        }))
    }

    /// Persist the final local position of a dragged thought. The projection is
    /// only a seed; stored coordinates win on the next brain read.
    pub fn set_drag(&self, body: &Value) -> Result<Value, String> {
        let id = body.get("id").and_then(Value::as_str).filter(|s| !s.is_empty()).ok_or("id required")?;
        let values = body.get("pos").and_then(Value::as_array).ok_or("pos[3] required")?;
        if values.len() != 3 {
            return Err("pos[3] required".into());
        }
        let mut pos = [0.0f32; 3];
        for (i, value) in values.iter().enumerate() {
            let n = value.as_f64().ok_or("pos must be numeric")? as f32;
            if !n.is_finite() {
                return Err("pos must be finite".into());
            }
            pos[i] = n;
        }
        let len = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
        if len > 1.0 {
            let scale = 1.0 / len;
            for c in &mut pos {
                *c *= scale;
            }
        }
        let conn = self.db.lock().map_err(|_| "db poisoned".to_string())?;
        let changed = conn
            .execute(
                "UPDATE thoughts SET pos_x=?2,pos_y=?3,pos_z=?4 WHERE id=?1",
                params![id, pos[0], pos[1], pos[2]],
            )
            .map_err(|e| e.to_string())?;
        if changed == 0 {
            return Err("unknown thought".into());
        }
        Ok(json!({"id": id, "pos": pos, "persisted": true}))
    }

    /// Dream runs are persisted topology, not a UI illusion.
    pub fn record_dream(&self, body: &Value) -> Result<Value, String> {
        let summary = body.get("summary").and_then(Value::as_str).unwrap_or("dream run").trim();
        if summary.len() > 4_000 { return Err("dream summary too long".into()); }
        let count = body.get("thought_count").and_then(Value::as_u64).unwrap_or(0);
        let insights = body.get("insights").cloned().unwrap_or_else(|| json!([]));
        let id = noema_core::ThoughtId::new().to_string();
        let conn = self.db.lock().map_err(|_| "db poisoned".to_string())?;
        let now = now_ms();
        conn.execute(
            "INSERT INTO dreams(id,session_id,started_at,finished_at,summary,insights,input_thoughts,output_thoughts)
             VALUES (?1,NULL,?2,?3,?4,NULL,?5,?6)",
            params![id, now, now, summary, count.to_string(), insights.to_string()],
        ).map_err(|e| e.to_string())?;
        Ok(json!({"id": id, "persisted": true, "thought_count": count}))
    }

    /// Studio snapshot: restored session, projects, sessions, role models,
    /// widget layout and the aggregated BI feed.
    pub fn studio(&self) -> Result<Value, String> {
        let conn = self.db.lock().map_err(|_| "db poisoned".to_string())?;
        ensure_widgets(&conn).map_err(|e| e.to_string())?;

        let projects: Vec<(String, String, String)> = {
            let mut stmt = conn
                .prepare("SELECT id,name,root_path FROM projects ORDER BY updated_at DESC LIMIT 50")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map_err(|e| e.to_string())?;
            rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?
        };

        let sessions: Vec<Value> = {
            let mut stmt = conn
                .prepare(
                    "SELECT id,project_id,agent_id,model_id,cwd,repo,branch,updated_at
                     FROM sessions ORDER BY updated_at DESC LIMIT 25",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| {
                    Ok(json!({
                        "id": r.get::<_, String>(0)?,
                        "project_id": r.get::<_, Option<String>>(1)?,
                        "agent_id": r.get::<_, String>(2)?,
                        "model_id": r.get::<_, String>(3)?,
                        "cwd": r.get::<_, String>(4)?,
                        "repo": r.get::<_, Option<String>>(5)?,
                        "branch": r.get::<_, Option<String>>(6)?,
                        "updated_at": r.get::<_, i64>(7)?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?
        };

        let models: Vec<Value> = {
            let mut stmt = conn
                .prepare("SELECT id,name,capability FROM models ORDER BY created_at")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| {
                    Ok(json!({
                        "id": r.get::<_, String>(0)?,
                        "name": r.get::<_, String>(1)?,
                        "capability": r.get::<_, String>(2)?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?
        };

        let mut roles = serde_json::Map::new();
        for cap in STUDIO_ROLES {
            let global = studio_setting(&conn, &role_key(cap, None))
                .map_err(|e| e.to_string())?
                .or(first_model(&conn, cap).map_err(|e| e.to_string())?);
            let mut overrides = serde_json::Map::new();
            for (id, _, _) in &projects {
                if let Some(model) =
                    studio_setting(&conn, &role_key(cap, Some(id))).map_err(|e| e.to_string())?
                {
                    overrides.insert(id.clone(), json!(model));
                }
            }
            roles.insert(cap.to_string(), json!({"global": global, "projects": overrides}));
        }

        let widgets: Vec<Value> = {
            let mut stmt = conn
                .prepare("SELECT id,name,chart,spec,x,y,w,h,pinned FROM widgets ORDER BY y,x")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| {
                    let spec = r.get::<_, String>(3)?;
                    Ok(json!({
                        "id": r.get::<_, String>(0)?,
                        "name": r.get::<_, String>(1)?,
                        "chart": r.get::<_, String>(2)?,
                        "spec": serde_json::from_str::<Value>(&spec).unwrap_or_else(|_| json!({})),
                        "x": r.get::<_, i64>(4)?,
                        "y": r.get::<_, i64>(5)?,
                        "w": r.get::<_, i64>(6)?,
                        "h": r.get::<_, i64>(7)?,
                        "pinned": r.get::<_, i64>(8)? != 0,
                    }))
                })
                .map_err(|e| e.to_string())?;
            rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?
        };

        let usage: Vec<Value> = {
            let mut stmt = conn
                .prepare(
                    "SELECT u.agent_id, u.model_id, COALESCE(p.name, ''), u.kind, u.peak, u.thinking,
                            COUNT(*), SUM(u.tokens), SUM(u.cost_usd), AVG(u.latency_ms), MAX(u.created_at)
                     FROM usage_events u LEFT JOIN projects p ON p.id = u.project_id
                     GROUP BY u.agent_id, u.model_id, p.name, u.kind, u.peak, u.thinking
                     ORDER BY MAX(u.created_at) DESC LIMIT 400",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| {
                    Ok(json!({
                        "agent": r.get::<_, Option<String>>(0)?,
                        "model": r.get::<_, String>(1)?,
                        "project": r.get::<_, String>(2)?,
                        "kind": r.get::<_, String>(3)?,
                        "peak": r.get::<_, i64>(4)?,
                        "thinking": r.get::<_, Option<String>>(5)?,
                        "events": r.get::<_, i64>(6)?,
                        "tokens": r.get::<_, i64>(7)?,
                        "cost_usd": r.get::<_, f64>(8)?,
                        "latency_ms": r.get::<_, Option<f64>>(9)?,
                        "at": r.get::<_, i64>(10)?,
                    }))
                })
                .map_err(|e| e.to_string())?;
            rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?
        };

        let session = sessions.first().cloned().unwrap_or(Value::Null);
        Ok(json!({
            "session": session,
            "projects": projects.iter().map(|(id, name, root)| json!({"id": id, "name": name, "root_path": root})).collect::<Vec<_>>(),
            "sessions": sessions,
            "models": models,
            "roles": roles,
            "widgets": widgets,
            "usage": usage,
        }))
    }

    /// Studio writes: session focus, role-model assignment (global or project
    /// scoped), widget upsert/remove and layout persistence.
    pub fn studio_update(&self, body: &Value) -> Result<Value, String> {
        let action = body.get("action").and_then(Value::as_str).ok_or("missing action")?;
        {
            let conn = self.db.lock().map_err(|_| "db poisoned".to_string())?;
            match action {
                "session" => {
                    let id = body.get("id").and_then(Value::as_str).ok_or("missing session id")?;
                    let changed = conn
                        .execute("UPDATE sessions SET updated_at = ?1 WHERE id = ?2", params![now_ms(), id])
                        .map_err(|e| e.to_string())?;
                    if changed == 0 {
                        return Err("unknown session".into());
                    }
                }
                "role" => {
                    let capability = body
                        .get("capability")
                        .and_then(Value::as_str)
                        .ok_or("missing capability")?;
                    if !STUDIO_ROLES.contains(&capability) {
                        return Err(format!("unknown capability {capability}"));
                    }
                    let model = body.get("model").and_then(Value::as_str).ok_or("missing model")?;
                    if model.trim().is_empty() {
                        return Err("empty model".into());
                    }
                    let project = body.get("project").and_then(Value::as_str).filter(|p| !p.is_empty());
                    put_studio_setting(&conn, &role_key(capability, project), model)
                        .map_err(|e| e.to_string())?;
                }
                "widget" => {
                    let widget = body.get("widget").ok_or("missing widget")?;
                    upsert_widget(&conn, widget).map_err(|e| e.to_string())?;
                }
                "widget_remove" => {
                    let id = body.get("id").and_then(Value::as_str).ok_or("missing widget id")?;
                    conn.execute("DELETE FROM widgets WHERE id = ?1", [id])
                        .map_err(|e| e.to_string())?;
                }
                "layout" => {
                    let list = body.get("widgets").and_then(Value::as_array).ok_or("missing widgets")?;
                    for w in list {
                        let id = w.get("id").and_then(Value::as_str).ok_or("layout item missing id")?;
                        let x = grid_value(w, "x", 0).clamp(0, 11);
                        let y = grid_value(w, "y", 0).max(0);
                        let width = grid_value(w, "w", 4).clamp(2, 12).min(12 - x);
                        let height = grid_value(w, "h", 3).clamp(2, 8);
                        conn.execute(
                            "UPDATE widgets SET x=?1,y=?2,w=?3,h=?4 WHERE id=?5",
                            params![x, y, width, height, id],
                        )
                        .map_err(|e| e.to_string())?;
                    }
                }
                other => return Err(format!("unknown action {other}")),
            }
        }
        self.studio()
    }
}

fn brain_activation(activation: Activation) -> noema_hippocampus::Activation {
    noema_hippocampus::Activation {
        logprob_entropy: activation.logprob_entropy,
        thinking_density: activation.thinking_density,
        cosine_drift: activation.cosine_drift,
    }
}

fn region_of(cluster: Option<&str>) -> noema_hippocampus::Region {
    use noema_hippocampus::Region;
    match cluster.unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "motor" => Region::Motor,
        "broca" => Region::Broca,
        "wernicke" => Region::Wernicke,
        "hippocampus" => Region::Hippocampus,
        _ => Region::Prefrontal,
    }
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

const DEMO: &[(&str, &str, &str, &str, &str)] = &[
    ("axon", "Axon", "Core, storage and QA", "claudeapikey/claude-haiku-5-5", "Axon online. The spine is stable."),
    ("cortex", "Cortex", "Architecture and review", "claudeapikey/claude-opus-5-5", "Cortex online. Plans checked against the repo."),
    ("hippocampus", "Hippocampus", "Semantic graph and Dream Mode", "claudeapikey/claude-sonnet-5-5", "Hippocampus online. Memory indexed."),
    ("retina", "Retina", "Studio and BI widgets", "claudeapikey/claude-sonnet-5-5", "Retina online. Studio aligned."),
    ("soma", "Soma", "3D bio-anatomical and shader engine", "claudeapikey/claude-sonnet-5-5", "Soma online. Body map warm."),
    ("synapse", "Synapse", "Ecosystem, MCP and plugins", "claudeapikey/claude-haiku-5-5", "Synapse online. Marketplace open."),
    ("thalamus", "Thalamus", "Inference adapters and FinOps", "claudeapikey/claude-sonnet-5-5", "Thalamus online. Rate cards loaded."),
];

const DEMO_SKILLS: &[&str] = &["\"anatomy-glb\"", "\"visual-qa\""];
const DEMO_MCP: &[&str] = &["\"driftwatch\""];

/// Local-only fixture database for visual QA. Never touches the owner database.
pub fn seed_demo(path: &Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    noema_core::Store::open(path).map_err(|e| e.to_string())?;
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO projects(id,name,root_path,created_at,updated_at) VALUES (?1,?2,?3,?4,?4)",
        params!["demo-project", "agentic-anatomy-workspace", "F:\\Progetti\\agentic-anatomy-workspace", now],
    ).map_err(|e| e.to_string())?;
    const DEMO_MODELS: &[(&str, &str, &str)] = &[
        ("llamalocal/nomic-embed-text-v1.5", "nomic-embed-text v1.5", "embedding"),
        ("llamalocal/FrogNano-4B-2609-Q4_K_M", "FrogNano 4B", "router"),
        ("claudeapikey/claude-sonnet-5-5", "Claude Sonnet 5.5", "chat"),
        ("llamalocal/Qwen2.5-Coder-7B-Instruct-abliterated-Q4_K_M", "Qwen2.5 Coder 7B", "coding"),
        ("claudeapikey/claude-opus-5-5", "Claude Opus 5.5", "dreaming"),
    ];
    for (id, name, capability) in DEMO_MODELS {
        tx.execute(
            "INSERT INTO models(id,provider_id,name,capability,context_window,params,created_at)
             VALUES (?1,NULL,?2,?3,NULL,NULL,?4)",
            params![id, name, capability, now],
        ).map_err(|e| e.to_string())?;
    }
    for (i, (id, name, role, model, greeting)) in DEMO.iter().enumerate() {
        tx.execute(
            "INSERT INTO agents(id,name,role,model_id,workdirs,skills,plugins,mcp_servers,greeting,created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                id, name, role, model,
                "[\"F:\\\\Progetti\\\\agentic-anatomy-workspace\"]",
                format!("[{}]", DEMO_SKILLS.join(",")),
                format!("[{}]", if i == 5 { "\"github\"" } else { "\"none\"" }),
                format!("[{}]", DEMO_MCP.join(",")),
                greeting,
                now + i as i64
            ],
        ).map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO sessions(id,project_id,agent_id,model_id,cwd,repo,branch,widget_layout,created_at,updated_at)
             VALUES (?1,'demo-project',?2,?3,?4,?5,?6,NULL,?7,?8)",
            params![
                format!("00000000-0000-4000-8000-{:012}", i + 1),
                id, model, "F:\\Progetti\\agentic-anatomy-workspace",
                "agentic-anatomy-workspace", "main", now, now + i as i64
            ],
        ).map_err(|e| e.to_string())?;
        for k in 0..6 {
            let kind = ["chat", "embedding", "dream"][k % 3];
            let thinking = if kind == "chat" {
                Some(if (i + k) % 2 == 0 { "on" } else { "off" })
            } else {
                None
            };
            tx.execute(
                "INSERT INTO usage_events(id,session_id,agent_id,model_id,kind,tokens,cost_usd,peak,latency_ms,project_id,thinking,created_at)
                 VALUES (?1,NULL,?2,?3,?4,?5,?6,?7,?8,'demo-project',?9,?10)",
                params![
                    format!("demo-{id}-{k}"), id, model, kind,
                    1_200 + (i as i64 + 1) * (k as i64 + 2) * 830,
                    ((i + 1) as f64 * (k + 1) as f64) * 0.017,
                    (k % 2) as i64,
                    420 + (i as i64 + k as i64) * 70,
                    thinking,
                    now - (6 - k) as i64 * 60_000
                ],
            ).map_err(|e| e.to_string())?;
        }
    }
    for (i, (id, name, role, _, _)) in DEMO.iter().enumerate() {
        for k in 0..3 {
            let region = noema_hippocampus::Region::ALL[(i + k) % noema_hippocampus::Region::ALL.len()];
            let topic = ["storage spine", "render budget", "memory index"][k];
            tx.execute(
                "INSERT INTO thoughts(id,session_id,agent_id,title,body,concepts,cluster_id,embedding,importance,pos_x,pos_y,pos_z,created_at)
                 VALUES (?1,NULL,?2,?3,?4,'[]',?5,?6,?7,NULL,NULL,NULL,?8)",
                params![
                    format!("demo-thought-{i}-{k}"),
                    id,
                    format!("{name} - {topic}"),
                    format!("{role}. Observation {k}: {name} keeps the local graph warm and the session durable."),
                    region.label(),
                    demo_embedding(&format!("{id}-{k}"), region),
                    0.4 + ((i + k) % 6) as f64 * 0.1,
                    now - (i as i64 * 3 + k as i64) * 60_000
                ],
            ).map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn demo_embedding(seed: &str, region: noema_hippocampus::Region) -> Vec<u8> {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in seed.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    let index = noema_hippocampus::Region::ALL.iter().position(|r| *r == region).unwrap_or(0);
    let mut v = Vec::with_capacity(16);
    for d in 0..16 {
        h = h.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let noise = ((h >> 40) as f32 / (1u64 << 24) as f32) * 0.1 - 0.05;
        v.push(if d == index { 1.0 } else { noise });
    }
    noema_core::vector::to_blob(&v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> AgentFacts {
        AgentFacts {
            id: "soma".into(),
            name: "Soma".into(),
            role: "3D".into(),
            model: "m".into(),
            greeting: "Soma online.".into(),
            skills: vec!["anatomy".into()],
            plugins: vec![],
            mcp: vec!["driftwatch".into()],
            events: 6,
            tokens: 12_345,
            cost: 0.42,
            repo: Some("agentic-anatomy-workspace".into()),
            ..AgentFacts::default()
        }
    }

    #[test]
    fn greeting_is_stable_and_db_shaped() {
        let a = compose_greeting(&facts());
        let b = compose_greeting(&facts());
        assert_eq!(a, b);
        assert!(a.starts_with("Soma online. agentic-anatomy-workspace"));
        assert!(a.contains("12,345 tokens"));
        assert!(a.contains("1 skills / 0 plugins / 1 MCP"));
    }

    #[test]
    fn greeting_without_greeting_row_falls_back() {
        let mut f = facts();
        f.greeting = "  ".into();
        assert!(compose_greeting(&f).starts_with("Soma online."));
    }

    #[test]
    fn demo_brain_projects_every_thought_and_persists_dreams() {
        let dir = std::env::temp_dir().join(format!("noema-block6-{}.db", std::process::id()));
        seed_demo(&dir).unwrap();
        let conn = Connection::open(&dir).unwrap();
        let api = Api { db: Arc::new(Mutex::new(conn)), live: Default::default() };
        let brain = api.brain().unwrap();
        let nodes = brain["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 21);
        assert!(!brain["edges"].as_array().unwrap().is_empty());
        for node in nodes {
            let pos = node["pos"].as_array().unwrap();
            let r = (pos[0].as_f64().unwrap().powi(2) + pos[1].as_f64().unwrap().powi(2) + pos[2].as_f64().unwrap().powi(2)).sqrt();
            assert!(r <= 1.001, "projection radius {r}");
            assert!(node["pulse"]["hz"].as_f64().unwrap() > 0.0);
        }
        let dream = api.record_dream(&json!({"summary":"test dream","thought_count":21})).unwrap();
        assert_eq!(dream["persisted"], true);
        let count: i64 = Connection::open(&dir).unwrap()
            .query_row("SELECT COUNT(*) FROM dreams", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn live_telemetry_overrides_derived_pulse() {
        let dir = std::env::temp_dir().join(format!("noema-block6-live-{}.db", std::process::id()));
        seed_demo(&dir).unwrap();
        let conn = Connection::open(&dir).unwrap();
        let api = Api { db: Arc::new(Mutex::new(conn)), live: Default::default() };
        let fed = api
            .ingest_telemetry(&json!({
                "events": [
                    {"type":"logprob","token":"a","logprob":-0.2},
                    {"type":"logprob","token":"b","logprob":-0.8},
                    {"type":"usage","input":1,"output":4,"reasoning":2,"cache_read":0,"cache_write":0}
                ]
            }))
            .unwrap();
        assert_eq!(fed["source"], "live");
        assert!((fed["activation"]["logprob_entropy"].as_f64().unwrap() - 0.5).abs() < 1e-6);
        let brain = api.brain().unwrap();
        assert_eq!(brain["telemetry"]["source"], "live");
        let pulse = brain["nodes"][0]["pulse"]["hz"].as_f64().unwrap();
        assert!((pulse - fed["pulse"]["hz"].as_f64().unwrap()).abs() < 1e-6);
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn drag_position_persists_and_wins_over_projection() {
        let dir = std::env::temp_dir().join(format!("noema-block6-drag-{}.db", std::process::id()));
        seed_demo(&dir).unwrap();
        let conn = Connection::open(&dir).unwrap();
        let api = Api { db: Arc::new(Mutex::new(conn)), live: Default::default() };
        let brain = api.brain().unwrap();
        let id = brain["nodes"][0]["id"].as_str().unwrap().to_string();
        api.set_drag(&json!({"id": id, "pos": [0.1, 0.2, 0.3]})).unwrap();
        let again = api.brain().unwrap();
        let node = again["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"].as_str() == Some(id.as_str()))
            .unwrap();
        for (got, want) in node["pos"].as_array().unwrap().iter().zip([0.1f64, 0.2, 0.3]) {
            assert!((got.as_f64().unwrap() - want).abs() < 1e-6);
        }
        assert!(api.set_drag(&json!({"id":"missing","pos":[0.0,0.0,0.0]})).is_err());
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn demo_seed_feeds_every_tab() {
        let dir = std::env::temp_dir().join(format!("noema-block5-{}.db", std::process::id()));
        seed_demo(&dir).unwrap();
        let conn = Connection::open(&dir).unwrap();
        let api = Api { db: Arc::new(Mutex::new(conn)), live: Default::default() };
        let payload = api.anatomy().unwrap();
        let agents = payload["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 7);
        let soma = agents.iter().find(|a| a["id"] == "soma").unwrap();
        assert!(soma["greeting"].as_str().unwrap().contains("tokens"));
        assert_eq!(soma["tabs"]["ecosystem"]["skills"].as_array().unwrap().len(), 2);
        assert_eq!(soma["tabs"]["activity"]["events"].as_array().unwrap().len(), 6);
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn studio_restores_session_and_round_trips_roles_widgets_layout() {
        let dir = std::env::temp_dir().join(format!("noema-block7-{}.db", std::process::id()));
        seed_demo(&dir).unwrap();
        let conn = Connection::open(&dir).unwrap();
        let api = Api { db: Arc::new(Mutex::new(conn)), live: Default::default() };

        let state = api.studio().unwrap();
        assert_eq!(state["projects"].as_array().unwrap().len(), 1);
        assert_eq!(state["widgets"].as_array().unwrap().len(), 4);
        assert!(state["usage"].as_array().unwrap().len() >= 7);
        assert_eq!(state["roles"]["chat"]["global"], "claudeapikey/claude-sonnet-5-5");
        let session = state["session"]["id"].as_str().unwrap().to_string();

        api.studio_update(&json!({"action":"role","capability":"chat","model":"demo-chat"})).unwrap();
        let scoped = api.studio_update(&json!({
            "action":"role","capability":"coding","model":"demo-coding","project":"demo-project"
        })).unwrap();
        assert_eq!(scoped["roles"]["chat"]["global"], "demo-chat");
        assert_eq!(scoped["roles"]["coding"]["projects"]["demo-project"], "demo-coding");

        let added = api.studio_update(&json!({
            "action":"widget",
            "widget":{"id":"w-test","name":"cost by soma","chart":"bars",
                      "spec":{"metric":"cost","dimension":"agent","filters":[{"dimension":"agent","value":"soma"}]},
                      "x":1,"y":2,"w":5,"h":3}
        })).unwrap();
        assert!(added["widgets"].as_array().unwrap().iter().any(|w| w["id"] == "w-test"));

        let moved = api.studio_update(&json!({
            "action":"layout","widgets":[{"id":"w-test","x":4,"y":1,"w":6,"h":3}]
        })).unwrap();
        let widget = moved["widgets"].as_array().unwrap().iter().find(|w| w["id"] == "w-test").unwrap();
        assert_eq!(widget["x"], 4);
        assert_eq!(widget["w"], 6);

        let restored = api.studio_update(&json!({"action":"session","id":session})).unwrap();
        assert_eq!(restored["session"]["id"], session);

        let removed = api.studio_update(&json!({"action":"widget_remove","id":"w-test"})).unwrap();
        assert!(!removed["widgets"].as_array().unwrap().iter().any(|w| w["id"] == "w-test"));
        assert!(api.studio_update(&json!({"action":"role","capability":"nope","model":"x"})).is_err());
        let _ = std::fs::remove_file(&dir);
    }
}
