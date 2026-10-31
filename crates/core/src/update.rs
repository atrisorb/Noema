use std::path::Path;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::VERSION;
use crate::path_env::current_exe_dir;

pub const DEFAULT_REPO: &str = "atrisorb/noema";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheck {
    pub current_version: String,
    pub latest_version: String,
    pub tag_name: String,
    pub has_update: bool,
    pub published_at: Option<String>,
    pub html_url: String,
    pub body: Option<String>,
    pub matching_asset: Option<ReleaseAsset>,
    pub all_assets: Vec<ReleaseAsset>,
}

/// Parse version string into numeric tuple (major, minor, patch).
fn parse_version(raw: &str) -> (u64, u64, u64) {
    let clean = raw.trim().trim_start_matches('v');
    let mut parts = clean.split('.');
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

/// Find the appropriate asset for the current OS and architecture.
fn select_asset(assets: &[ReleaseAsset]) -> Option<ReleaseAsset> {
    #[cfg(target_os = "windows")]
    {
        assets
            .iter()
            .find(|a| a.name.ends_with("windows-x64.zip"))
            .or_else(|| assets.iter().find(|a| a.name.ends_with(".zip")))
            .cloned()
    }
    #[cfg(target_os = "linux")]
    {
        assets
            .iter()
            .find(|a| a.name.ends_with("linux-x86_64.tar.gz"))
            .or_else(|| assets.iter().find(|a| a.name.ends_with(".tar.gz")))
            .or_else(|| assets.iter().find(|a| a.name.ends_with(".AppImage")))
            .cloned()
    }
    #[cfg(target_os = "macos")]
    {
        assets
            .iter()
            .find(|a| a.name.contains("universal.dmg"))
            .or_else(|| assets.iter().find(|a| a.name.contains("macos-arm64.dmg")))
            .or_else(|| assets.iter().find(|a| a.name.ends_with(".dmg")))
            .cloned()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        assets.first().cloned()
    }
}

/// Check GitHub releases for the latest release.
pub async fn check_update(repo: Option<&str>) -> Result<UpdateCheck, String> {
    let repo_slug = repo.unwrap_or(DEFAULT_REPO);
    let url = format!("https://api.github.com/repos/{repo_slug}/releases/latest");

    let client = reqwest::Client::builder()
        .user_agent(format!("NOEMA-CLI/{VERSION}"))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    let res = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("failed to fetch releases from GitHub ({url}): {e}"))?;

    if !res.status().is_success() {
        return Err(format!("GitHub API returned HTTP {}: {}", res.status(), url));
    }

    let payload: Value = res
        .json()
        .await
        .map_err(|e| format!("failed to parse GitHub release JSON: {e}"))?;

    let tag_name = payload
        .get("tag_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let latest_version = tag_name.trim_start_matches('v').to_string();
    let html_url = payload
        .get("html_url")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let published_at = payload
        .get("published_at")
        .and_then(Value::as_str)
        .map(String::from);
    let body = payload
        .get("body")
        .and_then(Value::as_str)
        .map(String::from);

    let mut all_assets = Vec::new();
    if let Some(assets_arr) = payload.get("assets").and_then(Value::as_array) {
        for a in assets_arr {
            let name = a.get("name").and_then(Value::as_str).unwrap_or("").to_string();
            let size = a.get("size").and_then(Value::as_u64).unwrap_or(0);
            let browser_download_url = a
                .get("browser_download_url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if !name.is_empty() && !browser_download_url.is_empty() {
                all_assets.push(ReleaseAsset {
                    name,
                    size,
                    browser_download_url,
                });
            }
        }
    }

    let current = parse_version(VERSION);
    let latest = parse_version(&latest_version);
    let has_update = latest > current;
    let matching_asset = select_asset(&all_assets);

    Ok(UpdateCheck {
        current_version: VERSION.to_string(),
        latest_version,
        tag_name,
        has_update,
        published_at,
        html_url,
        body,
        matching_asset,
        all_assets,
    })
}

/// Download matching asset and apply in-place update.
pub async fn apply_update(check: &UpdateCheck, force: bool) -> Result<Value, String> {
    if !check.has_update && !force {
        return Ok(json!({
            "updated": false,
            "message": format!("NOEMA is already up to date ({VERSION})"),
            "current_version": VERSION,
        }));
    }

    let asset = check
        .matching_asset
        .as_ref()
        .ok_or("no matching asset found for current OS and architecture")?;

    let target_dir = current_exe_dir().ok_or("could not determine NOEMA installation directory")?;

    // Download to temp
    let temp_dir = std::env::temp_dir().join(format!("noema-update-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("failed to create temp dir {}: {e}", temp_dir.display()))?;

    let downloaded_file = temp_dir.join(&asset.name);
    let client = reqwest::Client::builder()
        .user_agent(format!("NOEMA-CLI/{VERSION}"))
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    let res = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(|e| format!("failed to download asset {}: {e}", asset.browser_download_url))?;

    if !res.status().is_success() {
        return Err(format!("download failed with status {}", res.status()));
    }

    let bytes = res
        .bytes()
        .await
        .map_err(|e| format!("failed to read asset payload: {e}"))?;

    std::fs::write(&downloaded_file, bytes)
        .map_err(|e| format!("failed to write {}: {e}", downloaded_file.display()))?;

    // Extract
    let extract_dir = temp_dir.join("extract");
    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("failed to create {}: {e}", extract_dir.display()))?;

    #[cfg(windows)]
    {
        // Try tar -xf first, fallback to PowerShell Expand-Archive
        let tar_status = std::process::Command::new("tar")
            .args(["-xf", downloaded_file.to_str().unwrap(), "-C", extract_dir.to_str().unwrap()])
            .status();
        let success = tar_status.map(|s| s.success()).unwrap_or(false);
        if !success {
            let ps_script = format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                downloaded_file.to_string_lossy().replace('\'', "''"),
                extract_dir.to_string_lossy().replace('\'', "''")
            );
            let ps_status = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &ps_script])
                .status()
                .map_err(|e| format!("failed to run Expand-Archive: {e}"))?;
            if !ps_status.success() {
                return Err("failed to extract update zip archive".into());
            }
        }
    }
    #[cfg(not(windows))]
    {
        let tar_status = std::process::Command::new("tar")
            .args(["-xzf", downloaded_file.to_str().unwrap(), "-C", extract_dir.to_str().unwrap()])
            .status()
            .map_err(|e| format!("failed to unpack tar.gz: {e}"))?;
        if !tar_status.success() {
            return Err("failed to extract update tar.gz archive".into());
        }
    }

    // In-place copy to target_dir
    let mut files_updated = Vec::new();
    let entries = std::fs::read_dir(&extract_dir)
        .map_err(|e| format!("failed to read extract dir: {e}"))?;

    for entry in entries.flatten() {
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = target_dir.join(&file_name);

        if src_path.is_file() {
            #[cfg(windows)]
            {
                // If destination exists, rename it to .old first to avoid Windows file locks
                if dest_path.exists() {
                    let old_path = dest_path.with_extension("old");
                    std::fs::remove_file(&old_path).ok();
                    std::fs::rename(&dest_path, &old_path).ok();
                }
            }
            std::fs::copy(&src_path, &dest_path)
                .map_err(|e| format!("failed to copy {} to {}: {e}", src_path.display(), dest_path.display()))?;
            files_updated.push(file_name.to_string_lossy().to_string());
        } else if src_path.is_dir() {
            // Replace directory (e.g. ui/ or assets/)
            std::fs::remove_dir_all(&dest_path).ok();
            copy_dir_all(&src_path, &dest_path)?;
            files_updated.push(format!("{}/", file_name.to_string_lossy()));
        }
    }

    // Clean temp
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(json!({
        "updated": true,
        "previous_version": VERSION,
        "new_version": check.latest_version,
        "target_directory": target_dir.to_string_lossy().to_string(),
        "files_updated": files_updated,
    }))
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("failed to create dir {}: {e}", dst.display()))?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("failed to read dir {}: {e}", src.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_orders_semver_numerically() {
        assert_eq!(parse_version("0.5.21"), (0, 5, 21));
        assert_eq!(parse_version("v0.5.21"), (0, 5, 21));
        assert!(parse_version("0.5.22") > parse_version("0.5.21"));
        assert!(parse_version("0.6.0") > parse_version("0.5.21"));
        assert!(parse_version("1.0.0") > parse_version("0.5.21"));
        assert_eq!(parse_version("0.5.21"), parse_version("v0.5.21"));
    }

    #[test]
    fn asset_selector_matches_platform_family() {
        let assets = vec![
            ReleaseAsset {
                name: "NOEMA-0.5.21-1.x86_64.rpm".into(),
                size: 100,
                browser_download_url: "https://example.com/rpm".into(),
            },
            ReleaseAsset {
                name: "noema-0.5.21-windows-x64.zip".into(),
                size: 200,
                browser_download_url: "https://example.com/zip".into(),
            },
            ReleaseAsset {
                name: "noema-0.5.21-linux-x86_64.tar.gz".into(),
                size: 300,
                browser_download_url: "https://example.com/tar.gz".into(),
            },
            ReleaseAsset {
                name: "NOEMA_0.5.21_universal.dmg".into(),
                size: 400,
                browser_download_url: "https://example.com/dmg".into(),
            },
        ];

        let selected = select_asset(&assets).expect("should select asset");
        #[cfg(target_os = "windows")]
        assert!(selected.name.ends_with(".zip"));
        #[cfg(target_os = "linux")]
        assert!(selected.name.ends_with(".tar.gz"));
        #[cfg(target_os = "macos")]
        assert!(selected.name.ends_with(".dmg"));
    }
}
