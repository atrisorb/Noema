//! NOEMA core: identifiers, session state, storage, event bus and the
//! contracts shared by `noemad`, the CLI and the Studio UI.

pub mod daemon;
pub mod event;
pub mod hardware;
pub mod ids;
pub mod path_env;
pub mod platform;
pub mod rpc;
pub mod session;
pub mod store;
pub mod update;
pub mod vector;

pub use hardware::{HardwareProfile, HardwareReport, Tuning};
pub use ids::{AgentId, ModelId, ProjectId, SessionId, ThoughtId};
pub use platform::Platform;
pub use session::{ResumeState, Session, SessionStore};
pub use store::{Store, StoreError, UsageRecord, VectorBackend};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
