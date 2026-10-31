use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::ids::{AgentId, ModelId, ProjectId, SessionId};

/// One persistent working session. Everything needed to restore the Studio
/// exactly where the owner left it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub project: Option<ProjectId>,
    pub agent: AgentId,
    pub model: ModelId,
    pub cwd: String,
    pub repo: Option<String>,
    pub branch: Option<String>,
    /// Serialized widget grid (id, chart, spec, x, y, w, h, pinned).
    pub widget_layout: Option<String>,
    pub updated_at: i64,
}

/// Startup payload. The Studio and CLI both boot from this: no landing page.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResumeState {
    pub last_session: Option<SessionId>,
    pub last_project: Option<ProjectId>,
    pub last_agent: Option<AgentId>,
    pub last_cwd: Option<String>,
    pub last_repo: Option<String>,
    pub last_branch: Option<String>,
    /// Widget grid of the last session, restored verbatim.
    pub widget_layout: Option<String>,
    /// Model assigned to each role (`embedding`, `router`, `chat`, `coding`, `dreaming`).
    pub role_models: BTreeMap<String, ModelId>,
    /// Last known `git diff --stat` summary of the session working tree.
    pub diff_stat: Option<String>,
}

/// Persistence port. Implemented by the SQLite store; faked in tests.
pub trait SessionStore {
    type Error;

    fn resume_state(&self) -> Result<ResumeState, Self::Error>;
    fn load(&self, id: &SessionId) -> Result<Option<Session>, Self::Error>;
    fn save(&self, session: &Session) -> Result<(), Self::Error>;
}
