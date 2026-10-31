use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionTool {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub on_path: bool,
    pub executable: String,
}

/// Directory where the current process binary lives.
pub fn current_exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

/// Get user-level PATH entries.
pub fn get_user_path() -> Result<Vec<PathBuf>, String> {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", "[Environment]::GetEnvironmentVariable('PATH', 'User')"])
            .output()
            .map_err(|e| format!("failed to read user PATH via PowerShell: {e}"))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("PowerShell PATH read error: {err}"));
        }
        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(raw
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect())
    }
    #[cfg(not(windows))]
    {
        let env_path = std::env::var("PATH").unwrap_or_default();
        Ok(env_path
            .split(':')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect())
    }
}

/// Set user-level PATH entries permanently.
pub fn set_user_path(paths: &[PathBuf]) -> Result<(), String> {
    #[cfg(windows)]
    {
        let joined = paths
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join(";");
        // Escape single quotes for PowerShell string
        let escaped = joined.replace('\'', "''");
        let script = format!("[Environment]::SetEnvironmentVariable('PATH', '{escaped}', 'User')");
        let status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .status()
            .map_err(|e| format!("failed to set user PATH via PowerShell: {e}"))?;
        if !status.success() {
            return Err("PowerShell exited with non-zero status while updating PATH".into());
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        // On Unix, persist export PATH to ~/.profile if not present
        if let Some(home) = std::env::var_os("HOME") {
            let profile = PathBuf::from(home).join(".profile");
            let joined = paths
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join(":");
            let line = format!("\nexport PATH=\"{joined}:$PATH\"\n");
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&profile)
                .map_err(|e| format!("failed to open {}: {e}", profile.display()))?;
            file.write_all(line.as_bytes())
                .map_err(|e| format!("failed to write {}: {e}", profile.display()))?;
        }
        Ok(())
    }
}

/// Check if a directory is present in PATH (case-insensitively on Windows).
pub fn is_dir_in_path(target: &Path, paths: &[PathBuf]) -> bool {
    let target_canonical = target.canonicalize().unwrap_or_else(|_| target.to_path_buf());
    paths.iter().any(|p| {
        if p == target {
            return true;
        }
        if let Ok(c) = p.canonicalize() {
            if c == target_canonical {
                return true;
            }
        }
        #[cfg(windows)]
        {
            p.to_string_lossy().eq_ignore_ascii_case(&target.to_string_lossy())
        }
        #[cfg(not(windows))]
        {
            false
        }
    })
}

/// Detect companion tools and whether they are installed and on PATH.
pub fn scan_companion_tools(paths: &[PathBuf]) -> Vec<CompanionTool> {
    let mut tools = Vec::new();

    // 1. SolidCraft
    let solidcraft_paths = [
        "F:\\Programmi\\ArtCraft\\SolidCraft",
        "C:\\Programmi\\ArtCraft\\SolidCraft",
    ];
    let solidcraft_found = solidcraft_paths
        .iter()
        .find(|p| Path::new(p).join("solidcraft-cli.exe").exists())
        .copied()
        .unwrap_or(solidcraft_paths[0]);
    tools.push(CompanionTool {
        name: "SolidCraft (CAD CLI)".into(),
        path: solidcraft_found.into(),
        exists: Path::new(solidcraft_found).join("solidcraft-cli.exe").exists(),
        on_path: is_dir_in_path(Path::new(solidcraft_found), paths),
        executable: "solidcraft-cli.exe".into(),
    });

    // 2. CADCraft
    let cadcraft_paths = [
        "F:\\Programmi\\ArtCraft\\CADCraft",
        "C:\\Programmi\\ArtCraft\\CADCraft",
    ];
    let cadcraft_found = cadcraft_paths
        .iter()
        .find(|p| Path::new(p).join("cadcraft-cli.exe").exists())
        .copied()
        .unwrap_or(cadcraft_paths[0]);
    tools.push(CompanionTool {
        name: "CADCraft (Drafting CLI)".into(),
        path: cadcraft_found.into(),
        exists: Path::new(cadcraft_found).join("cadcraft-cli.exe").exists(),
        on_path: is_dir_in_path(Path::new(cadcraft_found), paths),
        executable: "cadcraft-cli.exe".into(),
    });

    // 3. Driftwatch
    let driftwatch_paths = [
        "F:\\tools\\driftwatch\\bin",
        "C:\\tools\\driftwatch\\bin",
    ];
    let driftwatch_found = driftwatch_paths
        .iter()
        .find(|p| Path::new(p).join("driftwatch").exists() || Path::new(p).join("driftwatch.cmd").exists())
        .copied()
        .unwrap_or(driftwatch_paths[0]);
    tools.push(CompanionTool {
        name: "Driftwatch (Changelog & Drift CLI)".into(),
        path: driftwatch_found.into(),
        exists: Path::new(driftwatch_found).exists(),
        on_path: is_dir_in_path(Path::new(driftwatch_found), paths),
        executable: "driftwatch".into(),
    });

    // 4. Cargo / Rust
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let cargo_bin = PathBuf::from(home).join(".cargo").join("bin");
        tools.push(CompanionTool {
            name: "Cargo (Rust toolchain)".into(),
            path: cargo_bin.to_string_lossy().to_string(),
            exists: cargo_bin.exists(),
            on_path: is_dir_in_path(&cargo_bin, paths),
            executable: "cargo".into(),
        });
    }

    tools
}

/// Comprehensive PATH status inspect.
pub fn status() -> Result<Value, String> {
    let user_paths = get_user_path()?;
    let exe_dir = current_exe_dir();
    let exe_enrolled = exe_dir.as_ref().map(|d| is_dir_in_path(d, &user_paths)).unwrap_or(false);

    let mut duplicates = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for p in &user_paths {
        let key = p.to_string_lossy().to_lowercase();
        if !seen.insert(key) {
            duplicates.push(p.to_string_lossy().to_string());
        }
    }

    let broken_paths: Vec<String> = user_paths
        .iter()
        .filter(|p| !p.exists())
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let companions = scan_companion_tools(&user_paths);

    Ok(json!({
        "current_exe_dir": exe_dir.map(|d| d.to_string_lossy().to_string()),
        "noema_enrolled_in_path": exe_enrolled,
        "user_path_count": user_paths.len(),
        "user_paths": user_paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
        "duplicates": duplicates,
        "broken_paths": broken_paths,
        "companion_tools": companions,
    }))
}

/// Enroll a directory into user PATH.
pub fn add(dir: &Path) -> Result<Value, String> {
    let mut paths = get_user_path()?;
    if is_dir_in_path(dir, &paths) {
        return Ok(json!({
            "added": false,
            "message": format!("{} is already in PATH", dir.display()),
            "status": status()?
        }));
    }
    paths.push(dir.to_path_buf());
    set_user_path(&paths)?;
    Ok(json!({
        "added": true,
        "directory": dir.to_string_lossy().to_string(),
        "status": status()?
    }))
}

/// Remove a directory from user PATH.
pub fn remove(dir: &Path) -> Result<Value, String> {
    let paths = get_user_path()?;
    let before_count = paths.len();
    let target_canonical = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());

    let filtered: Vec<PathBuf> = paths
        .into_iter()
        .filter(|p| {
            if p == dir {
                return false;
            }
            if let Ok(c) = p.canonicalize() {
                if c == target_canonical {
                    return false;
                }
            }
            #[cfg(windows)]
            {
                !p.to_string_lossy().eq_ignore_ascii_case(&dir.to_string_lossy())
            }
            #[cfg(not(windows))]
            {
                true
            }
        })
        .collect();

    if filtered.len() == before_count {
        return Ok(json!({
            "removed": false,
            "message": format!("{} was not found in PATH", dir.display()),
            "status": status()?
        }));
    }

    set_user_path(&filtered)?;
    Ok(json!({
        "removed": true,
        "directory": dir.to_string_lossy().to_string(),
        "status": status()?
    }))
}

/// Clean up duplicates and non-existent paths from user PATH.
pub fn clean() -> Result<Value, String> {
    let paths = get_user_path()?;
    let mut cleaned: Vec<PathBuf> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut removed_broken = Vec::new();
    let mut removed_duplicates = Vec::new();

    for p in paths {
        let key = p.to_string_lossy().to_lowercase();
        if !seen.insert(key) {
            removed_duplicates.push(p.to_string_lossy().to_string());
            continue;
        }
        if !p.exists() {
            removed_broken.push(p.to_string_lossy().to_string());
            continue;
        }
        cleaned.push(p);
    }

    set_user_path(&cleaned)?;

    Ok(json!({
        "cleaned": true,
        "removed_duplicates": removed_duplicates,
        "removed_broken_paths": removed_broken,
        "new_count": cleaned.len(),
        "status": status()?
    }))
}

/// One-shot setup: enroll current NOEMA exe directory and clean duplicates.
pub fn setup() -> Result<Value, String> {
    let exe_dir = current_exe_dir().ok_or("unable to determine current executable directory")?;
    let mut paths = get_user_path()?;
    let already = is_dir_in_path(&exe_dir, &paths);
    if !already {
        paths.push(exe_dir.clone());
    }

    // Deduplicate
    let mut unique = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for p in paths {
        let key = p.to_string_lossy().to_lowercase();
        if seen.insert(key) {
            unique.push(p);
        }
    }

    set_user_path(&unique)?;

    Ok(json!({
        "setup": true,
        "enrolled_noema": exe_dir.to_string_lossy().to_string(),
        "was_already_enrolled": already,
        "status": status()?
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_dir_in_path_finds_exact_and_canonical() {
        let p1 = PathBuf::from("C:\\tools\\noema");
        let list = vec![PathBuf::from("C:\\bin"), PathBuf::from("C:\\tools\\noema")];
        assert!(is_dir_in_path(&p1, &list));
        assert!(!is_dir_in_path(&PathBuf::from("C:\\missing"), &list));
    }

    #[test]
    #[cfg(windows)]
    fn is_dir_in_path_is_case_insensitive_on_windows() {
        let p1 = PathBuf::from("c:\\Tools\\Noema");
        let list = vec![PathBuf::from("C:\\tools\\noema")];
        assert!(is_dir_in_path(&p1, &list));
    }
}
