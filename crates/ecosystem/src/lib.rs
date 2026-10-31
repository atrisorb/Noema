//! NOEMA ecosystem: validated manifests, MCP supervisor, bundled marketplace
//! and the proprietary GitHub plugin.

pub mod github;
pub mod manifest;
pub mod marketplace;
pub mod mcp;

pub use github::{DevicePoll, GithubClient, GitRepo, Repo, RepoPin};
pub use manifest::{json_schema, parse_skill_md, Kind, Manifest, Transport, ValidationError};
pub use marketplace::{Marketplace, MarketError};
pub use mcp::{Auth, McpConfig, McpError, McpSupervisor, RetryPolicy, Secret, ServerStatus};
