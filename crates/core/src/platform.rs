//! Host platform facts: identity, per-OS data paths and secret backends.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Macos,
    Linux,
    Other,
}

impl Platform {
    pub fn current() -> Self {
        match std::env::consts::OS {
            "windows" => Self::Windows,
            "macos" => Self::Macos,
            "linux" => Self::Linux,
            _ => Self::Other,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::Linux => "linux",
            Self::Other => "other",
        }
    }
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
}

/// NOEMA data root. `NOEMA_DATA_DIR` wins on every OS.
///
/// Windows `%APPDATA%\noema`; macOS `~/Library/Application Support/noema`;
/// Linux `$XDG_DATA_HOME/noema`, else `~/.local/share/noema`.
pub fn data_dir() -> PathBuf {
    if let Some(dir) = env_path("NOEMA_DATA_DIR") {
        return dir;
    }
    match Platform::current() {
        Platform::Windows => env_path("APPDATA")
            .map(|p| p.join("noema"))
            .unwrap_or_else(|| home().join(".noema")),
        Platform::Macos => home()
            .join("Library")
            .join("Application Support")
            .join("noema"),
        Platform::Linux => env_path("XDG_DATA_HOME")
            .map(|p| p.join("noema"))
            .unwrap_or_else(|| home().join(".local").join("share").join("noema")),
        Platform::Other => home().join(".noema"),
    }
}

/// Secret backend the keychain adapter must use on this host.
pub fn keychain_backend() -> &'static str {
    match Platform::current() {
        Platform::Windows => "dpapi",
        Platform::Macos => "keychain",
        Platform::Linux => "secret-service",
        Platform::Other => "env",
    }
}
