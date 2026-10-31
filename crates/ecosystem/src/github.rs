//! Proprietary GitHub plugin: device flow or PAT auth, `user/repo` search
//! (public and private), branches, checkout, diff and session pinning.
//!
//! System `git` only, argv vectors only, token never logged.

use crate::mcp::Secret;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

const API: &str = "https://api.github.com";

#[derive(Debug)]
pub enum GithubError {
    Http(String),
    Status(u16, String),
    Json(String),
    Git(String),
    Io(std::io::Error),
    Invalid(String),
}

impl fmt::Display for GithubError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(e) => write!(f, "http: {e}"),
            Self::Status(code, message) => write!(f, "github {code}: {message}"),
            Self::Json(e) => write!(f, "json: {e}"),
            Self::Git(e) => write!(f, "git: {e}"),
            Self::Io(e) => write!(f, "io: {e}"),
            Self::Invalid(e) => write!(f, "invalid: {e}"),
        }
    }
}

impl std::error::Error for GithubError {}

impl From<std::io::Error> for GithubError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// PAT resolution order: `NOEMA_GITHUB_TOKEN`, then `GITHUB_TOKEN`.
/// The Studio stores the token in the OS keychain and injects it here.
pub fn token_from_env() -> Option<Secret> {
    ["NOEMA_GITHUB_TOKEN", "GITHUB_TOKEN"]
        .iter()
        .find_map(|key| std::env::var(key).ok())
        .filter(|value| !value.trim().is_empty())
        .map(Secret::new)
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Repo {
    pub full_name: String,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub default_branch: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub html_url: Option<String>,
}

impl Repo {
    pub fn owner(&self) -> &str {
        self.full_name.split_once('/').map(|(owner, _)| owner).unwrap_or("")
    }

    pub fn name(&self) -> &str {
        self.full_name.split_once('/').map(|(_, name)| name).unwrap_or("")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Branch {
    pub name: String,
    #[serde(default)]
    pub commit: Option<CommitRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommitRef {
    pub sha: String,
}

pub struct GithubClient {
    http: Client,
    token: Secret,
}

impl GithubClient {
    pub fn new(token: Secret) -> Result<Self, GithubError> {
        let http = Client::builder()
            .user_agent(format!("NOEMA/{}", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|e| GithubError::Http(e.to_string()))?;
        Ok(Self { http, token })
    }

    /// Live `user/repo` autocomplete. Private repos appear when the token can read them.
    pub async fn search_repos(&self, query: &str, limit: usize) -> Result<Vec<Repo>, GithubError> {
        let url = format!(
            "{API}/search/repositories?q={}&per_page={}",
            encode(query),
            limit.clamp(1, 100)
        );
        let value = self.get_json(&url).await?;
        let items = value
            .get("items")
            .and_then(Value::as_array)
            .ok_or_else(|| GithubError::Json("missing items array".into()))?;
        items
            .iter()
            .map(|item| serde_json::from_value(item.clone()).map_err(|e| GithubError::Json(e.to_string())))
            .collect()
    }

    pub async fn branches(&self, full_name: &str) -> Result<Vec<Branch>, GithubError> {
        let (owner, repo) = split_repo(full_name)?;
        let url = format!("{API}/repos/{owner}/{repo}/branches?per_page=100");
        let value = self.get_json(&url).await?;
        serde_json::from_value(value).map_err(|e| GithubError::Json(e.to_string()))
    }

    async fn get_json(&self, url: &str) -> Result<Value, GithubError> {
        let response = self
            .http
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .bearer_auth(self.token.expose())
            .send()
            .await
            .map_err(|e| GithubError::Http(e.to_string()))?;
        let status = response.status();
        let body = response.text().await.map_err(|e| GithubError::Http(e.to_string()))?;
        if !status.is_success() {
            return Err(GithubError::Status(status.as_u16(), error_message(&body)));
        }
        serde_json::from_str(&body).map_err(|e| GithubError::Json(e.to_string()))
    }
}

fn split_repo(full_name: &str) -> Result<(&str, &str), GithubError> {
    full_name
        .split_once('/')
        .filter(|(owner, repo)| !owner.is_empty() && !repo.is_empty())
        .ok_or_else(|| GithubError::Invalid(format!("repo '{full_name}' must be owner/name")))
}

fn error_message(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v.get("message").and_then(Value::as_str).map(str::to_string))
        .unwrap_or_else(|| body.chars().take(200).collect())
}

fn encode(input: &str) -> String {
    input
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => (b as char).to_string(),
            b' ' => "+".to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// Local git operations. Uses the system binary; never a shell string.
#[derive(Debug, Clone)]
pub struct GitRepo {
    pub root: PathBuf,
}

impl GitRepo {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GithubError> {
        let output = run_git(path.as_ref(), &["rev-parse", "--show-toplevel"])?;
        Ok(Self { root: PathBuf::from(output.trim()) })
    }

    pub fn branches(&self) -> Result<Vec<String>, GithubError> {
        let output = run_git(
            &self.root,
            &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
        )?;
        Ok(output.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect())
    }

    pub fn current_branch(&self) -> Result<String, GithubError> {
        Ok(run_git(&self.root, &["rev-parse", "--abbrev-ref", "HEAD"])?.trim().to_string())
    }

    pub fn checkout(&self, branch: &str) -> Result<(), GithubError> {
        if branch.is_empty() || branch.starts_with('-') || branch.chars().any(char::is_whitespace) {
            return Err(GithubError::Invalid(format!("branch '{branch}'")));
        }
        run_git(&self.root, &["checkout", branch])?;
        Ok(())
    }

    pub fn diff(&self, base: Option<&str>) -> Result<String, GithubError> {
        match base {
            Some(base) => {
                if base.starts_with('-') {
                    return Err(GithubError::Invalid("diff base".into()));
                }
                run_git(&self.root, &["diff", "--no-color", base])
            }
            None => run_git(&self.root, &["diff", "--no-color"]),
        }
    }

    pub fn diff_stat(&self) -> Result<String, GithubError> {
        run_git(&self.root, &["diff", "--stat", "--no-color"])
    }

    pub fn status(&self) -> Result<String, GithubError> {
        run_git(&self.root, &["status", "--short"])
    }
}

fn run_git(dir: &Path, args: &[&str]) -> Result<String, GithubError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| GithubError::Git(format!("git unavailable: {e}")))?;
    if !output.status.success() {
        return Err(GithubError::Git(String::from_utf8_lossy(&output.stderr).trim().to_string()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Repo + branch + session triple. Booting the Studio restores it verbatim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepoPin {
    pub full_name: String,
    pub branch: String,
    pub dir: String,
    #[serde(default)]
    pub session: Option<String>,
    pub pinned_at: i64,
}

impl RepoPin {
    pub fn new(full_name: impl Into<String>, branch: impl Into<String>, dir: impl Into<String>) -> Self {
        Self {
            full_name: full_name.into(),
            branch: branch.into(),
            dir: dir.into(),
            session: None,
            pinned_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn pin_session(&mut self, session: impl Into<String>) {
        self.session = Some(session.into());
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), GithubError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self).map_err(|e| GithubError::Json(e.to_string()))?)?;
        std::fs::rename(tmp, path)?;
        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, GithubError> {
        let source = std::fs::read_to_string(path)?;
        serde_json::from_str(&source).map_err(|e| GithubError::Json(e.to_string()))
    }
}

/// GitHub OAuth device flow (no client secret, no redirect URI).
#[derive(Debug, Clone)]
pub struct DeviceAuthorization {
    pub device_code: Secret,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DevicePoll {
    Pending,
    SlowDown,
    Token(Secret),
    Denied,
    Expired,
}

pub async fn start_device_flow(
    http: &Client,
    client_id: &str,
    scope: &str,
) -> Result<DeviceAuthorization, GithubError> {
    let response = http
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", client_id), ("scope", scope)])
        .send()
        .await
        .map_err(|e| GithubError::Http(e.to_string()))?;
    let status = response.status();
    let body = response.text().await.map_err(|e| GithubError::Http(e.to_string()))?;
    if !status.is_success() {
        return Err(GithubError::Status(status.as_u16(), error_message(&body)));
    }
    let value: Value = serde_json::from_str(&body).map_err(|e| GithubError::Json(e.to_string()))?;
    Ok(DeviceAuthorization {
        device_code: Secret::new(
            value.get("device_code").and_then(Value::as_str).ok_or_else(|| GithubError::Json("missing device_code".into()))?,
        ),
        user_code: value.get("user_code").and_then(Value::as_str).unwrap_or("").to_string(),
        verification_uri: value
            .get("verification_uri")
            .and_then(Value::as_str)
            .unwrap_or("https://github.com/login/device")
            .to_string(),
        expires_in: value.get("expires_in").and_then(Value::as_u64).unwrap_or(900),
        interval: value.get("interval").and_then(Value::as_u64).unwrap_or(5),
    })
}

pub async fn poll_device_flow(
    http: &Client,
    client_id: &str,
    device_code: &Secret,
) -> Result<DevicePoll, GithubError> {
    let response = http
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("device_code", device_code.expose()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await
        .map_err(|e| GithubError::Http(e.to_string()))?;
    let status = response.status();
    let body = response.text().await.map_err(|e| GithubError::Http(e.to_string()))?;
    if !status.is_success() {
        return Err(GithubError::Status(status.as_u16(), error_message(&body)));
    }
    let value: Value = serde_json::from_str(&body).map_err(|e| GithubError::Json(e.to_string()))?;
    if let Some(token) = value.get("access_token").and_then(Value::as_str) {
        return Ok(DevicePoll::Token(Secret::new(token)));
    }
    Ok(match value.get("error").and_then(Value::as_str).unwrap_or("") {
        "authorization_pending" => DevicePoll::Pending,
        "slow_down" => DevicePoll::SlowDown,
        "access_denied" => DevicePoll::Denied,
        "expired_token" => DevicePoll::Expired,
        other => return Err(GithubError::Json(format!("device flow error: {other}"))),
    })
}
