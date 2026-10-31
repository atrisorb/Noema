//! Machine discovery and hardware-derived defaults.
//!
//! One profile per host, cached in `settings` (scope `hardware`), with a live
//! refresh path. Every derived value is a plain number/boolean so the Studio
//! and `noema system override` can patch it without touching code.

use std::collections::BTreeSet;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::platform::Platform;
use crate::store::now_ms;

pub const PROFILE_KEY: &str = "hardware.profile";
pub const OVERRIDES_KEY: &str = "hardware.overrides";
pub const SCOPE: &str = "hardware";
/// Hard refresh after 12 h; docking stations and eGPUs change the profile.
pub const CACHE_TTL_MS: i64 = 12 * 60 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GpuKind {
    Discrete,
    Integrated,
    Software,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub kind: GpuKind,
    pub vram_mb: Option<u64>,
    pub driver: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub platform: Platform,
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub cpu_brand: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub ram_mb: u64,
    pub gpus: Vec<GpuInfo>,
    pub free_disk_mb: Option<u64>,
    pub total_disk_mb: Option<u64>,
    pub data_dir: String,
    pub detected_at_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quality {
    Ultra,
    High,
    Balanced,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderTuning {
    pub pixel_ratio_cap: f32,
    pub particle_budget: u32,
    pub shadows: bool,
    pub max_texture_mb: u32,
    pub lod_bias: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceTuning {
    pub max_context_tokens: u32,
    pub parallel_requests: u32,
    pub gpu_layers: i32,
    pub vram_budget_mb: u64,
    pub recommended_runtime: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DaemonTuning {
    pub worker_threads: u32,
    pub sqlite_cache_mb: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tuning {
    pub quality: Quality,
    pub render: RenderTuning,
    pub inference: InferenceTuning,
    pub daemon: DaemonTuning,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareReport {
    pub profile: HardwareProfile,
    pub tuning: Tuning,
    pub overrides: Value,
    pub cached: bool,
    pub generated_at_ms: i64,
}

fn read_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1 AND scope = ?2",
        params![key, SCOPE],
        |r| r.get::<_, String>(0),
    )
    .optional()
    .ok()
    .flatten()
}

fn write_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings(key, value, scope) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, scope = excluded.scope",
        params![key, value, SCOPE],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

fn clear_setting(conn: &Connection, key: &str) -> Result<(), String> {
    conn.execute("DELETE FROM settings WHERE key = ?1", [key])
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Derive every default from the profile. Pure: same profile, same tuning.
pub fn derive_tuning(p: &HardwareProfile) -> Tuning {
    let best = p
        .gpus
        .iter()
        .filter(|g| g.kind != GpuKind::Software)
        .max_by_key(|g| {
            let rank = match g.kind {
                GpuKind::Discrete => 3,
                GpuKind::Integrated => 2,
                GpuKind::Unknown => 1,
                GpuKind::Software => 0,
            };
            (rank, g.vram_mb.unwrap_or(0))
        });
    let vram = best.and_then(|g| g.vram_mb).unwrap_or(0);
    let ram = p.ram_mb;
    let cores = p.logical_cores.max(1);

    let quality = if vram >= 8_000 && ram >= 32_000 {
        Quality::Ultra
    } else if vram >= 6_000 || (vram >= 4_000 && ram >= 16_000) {
        Quality::High
    } else if best.is_some_and(|g| g.kind != GpuKind::Software) && ram >= 8_000 {
        Quality::Balanced
    } else {
        Quality::Low
    };
    let (pixel_ratio_cap, particle_budget, shadows, max_texture_mb, lod_bias) = match quality {
        Quality::Ultra => (2.0, 4_000, true, 2_048, 1.0),
        Quality::High => (1.75, 2_500, true, 1_024, 1.0),
        Quality::Balanced => (1.5, 1_500, false, 512, 1.15),
        Quality::Low => (1.0, 600, false, 256, 1.4),
    };

    let max_context_tokens = match vram {
        v if v >= 16_000 => 32_768,
        v if v >= 8_000 => 16_384,
        v if v >= 4_000 => 8_192,
        _ => 4_096,
    };
    let max_context_tokens = if ram < 16_000 {
        max_context_tokens.min(8_192)
    } else {
        max_context_tokens
    };
    let gpu_layers = if vram >= 4_000 { -1 } else { 0 };
    let recommended_runtime = if vram >= 4_000 {
        "ollama"
    } else if ram >= 8_000 {
        "llama.cpp"
    } else {
        "remote-only"
    };
    let parallel_requests = (cores / 2).clamp(1, 8);
    let worker_threads = cores.saturating_sub(1).clamp(1, 16);
    let sqlite_cache_mb = (ram / 32).clamp(64, 1_024) as u32;

    let mut reasons = vec![
        format!("cpu: {} ({} logical / {} physical)", p.cpu_brand, cores, p.physical_cores),
        format!("ram: {} MB", ram),
        format!(
            "data volume: {} MB free of {} MB",
            p.free_disk_mb.unwrap_or(0),
            p.total_disk_mb.unwrap_or(0)
        ),
    ];
    if let Some(g) = best {
        reasons.push(format!(
            "gpu: {} ({:?}, {} MB)",
            g.name,
            g.kind,
            g.vram_mb.unwrap_or(0)
        ));
    } else {
        reasons.push("gpu: none detected, software rendering path".into());
    }

    Tuning {
        quality,
        render: RenderTuning {
            pixel_ratio_cap,
            particle_budget,
            shadows,
            max_texture_mb,
            lod_bias,
        },
        inference: InferenceTuning {
            max_context_tokens,
            parallel_requests,
            gpu_layers,
            vram_budget_mb: vram * 85 / 100,
            recommended_runtime: recommended_runtime.into(),
        },
        daemon: DaemonTuning {
            worker_threads,
            sqlite_cache_mb,
        },
        reasons,
    }
}

/// Recursive object patch; `null` deletes a key. Unknown keys stay in the
/// stored patch but are rejected by `serde` when the effective tuning is built.
fn merge(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(base), Value::Object(patch)) => {
            for (key, value) in patch {
                if value.is_null() {
                    base.remove(key);
                } else {
                    merge(base.entry(key.clone()).or_insert(Value::Null), value);
                }
            }
        }
        (base, patch) => *base = patch.clone(),
    }
}

pub fn effective(profile: &HardwareProfile, overrides: &Value) -> Result<Tuning, String> {
    let mut value = serde_json::to_value(derive_tuning(profile)).map_err(|e| e.to_string())?;
    merge(&mut value, overrides);
    serde_json::from_value(value).map_err(|e| format!("invalid hardware override: {e}"))
}

fn cached_profile(conn: &Connection) -> Option<HardwareProfile> {
    read_setting(conn, PROFILE_KEY).and_then(|raw| serde_json::from_str(&raw).ok())
}

fn overrides(conn: &Connection) -> Value {
    read_setting(conn, OVERRIDES_KEY)
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}))
}

/// Cached profile or a fresh probe. `refresh` forces the probe.
pub fn report(conn: &Connection, refresh: bool) -> Result<HardwareReport, String> {
    let cached = cached_profile(conn);
    let fresh = cached
        .as_ref()
        .is_some_and(|p| now_ms() - p.detected_at_ms < CACHE_TTL_MS);
    let (profile, cached_hit) = if !refresh && fresh {
        (cached.unwrap(), true)
    } else {
        let profile = detect();
        write_setting(
            conn,
            PROFILE_KEY,
            &serde_json::to_string(&profile).map_err(|e| e.to_string())?,
        )?;
        (profile, false)
    };
    let overrides = overrides(conn);
    let tuning = effective(&profile, &overrides)?;
    Ok(HardwareReport {
        profile,
        tuning,
        overrides,
        cached: cached_hit,
        generated_at_ms: now_ms(),
    })
}

/// Merge and persist a patch, then return the effective report.
pub fn set_overrides(conn: &Connection, patch: &Value) -> Result<HardwareReport, String> {
    if !patch.is_object() {
        return Err("overrides must be a JSON object".into());
    }
    let mut current = overrides(conn);
    merge(&mut current, patch);
    // Validate before writing so a typo never poisons the stored patch.
    effective(&detect_or_cached(conn), &current)?;
    write_setting(
        conn,
        OVERRIDES_KEY,
        &serde_json::to_string(&current).map_err(|e| e.to_string())?,
    )?;
    report(conn, false)
}

/// Drop overrides and the cached profile, then detect again.
pub fn reset(conn: &Connection) -> Result<HardwareReport, String> {
    clear_setting(conn, OVERRIDES_KEY)?;
    clear_setting(conn, PROFILE_KEY)?;
    report(conn, true)
}

fn detect_or_cached(conn: &Connection) -> HardwareProfile {
    cached_profile(conn).unwrap_or_else(detect)
}

// ---------------------------------------------------------------------------
// Probes. One process invocation per OS; no extra crate, no telemetry.
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct RawProbe {
    hostname: String,
    cpu_brand: String,
    physical_cores: u32,
    logical_cores: u32,
    ram_mb: u64,
    gpus: Vec<GpuInfo>,
    free_disk_mb: Option<u64>,
    total_disk_mb: Option<u64>,
}

fn base_probe() -> RawProbe {
    RawProbe {
        hostname: std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "unknown".into()),
        cpu_brand: std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_default(),
        physical_cores: 0,
        logical_cores: std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1),
        ram_mb: 0,
        gpus: Vec::new(),
        free_disk_mb: None,
        total_disk_mb: None,
    }
}

/// Public detection entry point (no cache read/write).
pub fn detect() -> HardwareProfile {
    let raw = probe();
    let dir = crate::platform::data_dir();
    HardwareProfile {
        platform: Platform::current(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        hostname: raw.hostname,
        cpu_brand: if raw.cpu_brand.trim().is_empty() {
            "unknown".into()
        } else {
            raw.cpu_brand
        },
        physical_cores: raw.physical_cores,
        logical_cores: raw.logical_cores.max(1),
        ram_mb: raw.ram_mb,
        gpus: raw.gpus,
        free_disk_mb: raw.free_disk_mb,
        total_disk_mb: raw.total_disk_mb,
        data_dir: dir.display().to_string(),
        detected_at_ms: now_ms(),
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn run(program: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(program).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn vendor_of(name: &str) -> String {
    let n = name.to_ascii_lowercase();
    if n.contains("nvidia") {
        "nvidia"
    } else if n.contains("amd") || n.contains("radeon") || n.contains("ati ") {
        "amd"
    } else if n.contains("intel") {
        "intel"
    } else if n.contains("apple") {
        "apple"
    } else if n.contains("qualcomm") || n.contains("adreno") {
        "qualcomm"
    } else {
        "unknown"
    }
    .into()
}

fn gpu_kind(name: &str, vram_mb: u64) -> GpuKind {
    let n = name.to_ascii_lowercase();
    if ["microsoft basic", "remote display", "parsec", "llvmpipe", "swiftshader", "virtual"]
        .iter()
        .any(|k| n.contains(k))
    {
        return GpuKind::Software;
    }
    if ["intel", "apple", "iris", "uhd graphics", "radeon graphics", "vega 8", "adreno"]
        .iter()
        .any(|k| n.contains(k))
    {
        return GpuKind::Integrated;
    }
    if ["nvidia", "geforce", "rtx", "quadro", "radeon rx", "arc a", "arc b"]
        .iter()
        .any(|k| n.contains(k))
    {
        return GpuKind::Discrete;
    }
    if vram_mb >= 2_048 {
        GpuKind::Discrete
    } else {
        GpuKind::Unknown
    }
}

fn dedup_gpus(gpus: Vec<GpuInfo>) -> Vec<GpuInfo> {
    let mut seen = BTreeSet::new();
    gpus.into_iter()
        .filter(|g| seen.insert(g.name.to_ascii_lowercase()))
        .collect()
}

#[cfg(windows)]
fn num(value: &Value, key: &str) -> u64 {
    value
        .get(key)
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .max(0.0)
        .round() as u64
}

#[cfg(windows)]
fn probe() -> RawProbe {
    let mut raw = base_probe();
    let dir = crate::platform::data_dir();
    let script = r#"
$ErrorActionPreference='SilentlyContinue'
$cpu=Get-CimInstance Win32_Processor | Select-Object -First 1
$cs=Get-CimInstance Win32_ComputerSystem
$gpus=@(Get-CimInstance Win32_VideoController | ForEach-Object { [pscustomobject]@{ name=$_.Name; vram_mb=[math]::Round([double]$_.AdapterRAM/1MB); driver=$_.DriverVersion } })
$drive=Split-Path -Qualifier $env:NOEMA_PROBE_DIR
$disk=Get-CimInstance Win32_LogicalDisk -Filter "DeviceID='$drive'"
[pscustomobject]@{ host=$env:COMPUTERNAME; cpu=$cpu.Name; physical=[int]$cpu.NumberOfCores; logical=[int]$cpu.NumberOfLogicalProcessors; ram_mb=[math]::Round([double]$cs.TotalPhysicalMemory/1MB); gpus=$gpus; free_mb=[math]::Round([double]$disk.FreeSpace/1MB); total_mb=[math]::Round([double]$disk.Size/1MB) } | ConvertTo-Json -Depth 4 -Compress
"#;
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .env("NOEMA_PROBE_DIR", &dir)
        .output()
        .ok()
        .filter(|o| o.status.success());
    let Some(out) = out else { return raw };
    let text = String::from_utf8_lossy(&out.stdout);
    let text = text.trim().trim_start_matches('\u{feff}');
    let (Some(start), Some(end)) = (text.find('{'), text.rfind('}')) else {
        return raw;
    };
    let Ok(v) = serde_json::from_str::<Value>(&text[start..=end]) else {
        return raw;
    };

    raw.hostname = v
        .get("host")
        .and_then(Value::as_str)
        .unwrap_or(&raw.hostname)
        .to_string();
    raw.cpu_brand = v
        .get("cpu")
        .and_then(Value::as_str)
        .unwrap_or(&raw.cpu_brand)
        .trim()
        .to_string();
    raw.physical_cores = num(&v, "physical") as u32;
    raw.logical_cores = (num(&v, "logical") as u32).max(1);
    raw.ram_mb = num(&v, "ram_mb");
    raw.free_disk_mb = v
        .get("free_mb")
        .and_then(Value::as_f64)
        .map(|n| n.max(0.0).round() as u64);
    raw.total_disk_mb = v
        .get("total_mb")
        .and_then(Value::as_f64)
        .map(|n| n.max(0.0).round() as u64);

    let list = match v.get("gpus") {
        Some(Value::Array(items)) => items.clone(),
        Some(Value::Object(_)) => vec![v.get("gpus").cloned().unwrap_or(Value::Null)],
        _ => Vec::new(),
    };
    for g in list {
        let name = g
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("GPU")
            .trim()
            .to_string();
        let vram = num(&g, "vram_mb");
        raw.gpus.push(GpuInfo {
            vendor: vendor_of(&name),
            kind: gpu_kind(&name, vram),
            vram_mb: (vram > 0).then_some(vram),
            driver: g
                .get("driver")
                .and_then(Value::as_str)
                .map(str::to_string)
                .filter(|s| !s.is_empty()),
            name,
        });
    }
    raw.gpus = dedup_gpus(raw.gpus);
    raw
}

#[cfg(target_os = "macos")]
fn probe() -> RawProbe {
    let mut raw = base_probe();
    raw.hostname = run("sysctl", &["-n", "kern.hostname"])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(raw.hostname);
    raw.cpu_brand = run("sysctl", &["-n", "machdep.cpu.brand_string"])
        .or_else(|| run("sysctl", &["-n", "hw.model"]))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    raw.physical_cores = run("sysctl", &["-n", "hw.physicalcpu"])
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    raw.logical_cores = run("sysctl", &["-n", "hw.logicalcpu"])
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(raw.logical_cores)
        .max(1);
    raw.ram_mb = run("sysctl", &["-n", "hw.memsize"])
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|bytes| bytes / (1024 * 1024))
        .unwrap_or(0);

    if let Some(out) = run("system_profiler", &["SPDisplaysDataType", "-json"])
        && let Ok(v) = serde_json::from_str::<Value>(&out)
        && let Some(Value::Array(items)) = v.get("SPDisplaysDataType")
    {
        for item in items {
            let name = ["sppci_model", "_name", "spdisplays_chipset_model"]
                .iter()
                .find_map(|k| item.get(*k).and_then(Value::as_str))
                .unwrap_or("GPU")
                .trim()
                .to_string();
            let vendor = item
                .get("sppci_vendor")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let mut vram = ["spdisplays_vram", "spdisplays_vram_shared"]
                .iter()
                .find_map(|k| item.get(*k).and_then(Value::as_str))
                .and_then(parse_mem_mb);
            if vram.is_none() && vendor.to_ascii_lowercase().contains("apple") {
                // Unified memory: the GPU can address roughly 75% of RAM.
                vram = Some(raw.ram_mb * 3 / 4).filter(|mb| *mb > 0);
            }
            raw.gpus.push(GpuInfo {
                kind: gpu_kind(&format!("{name} {vendor}"), vram.unwrap_or(0)),
                vendor: if vendor.is_empty() { vendor_of(&name) } else { vendor },
                vram_mb: vram,
                driver: None,
                name,
            });
        }
    }
    raw.gpus = dedup_gpus(raw.gpus);
    let (free, total) = unix_disk(&crate::platform::data_dir());
    raw.free_disk_mb = free;
    raw.total_disk_mb = total;
    raw
}

#[cfg(target_os = "linux")]
fn probe() -> RawProbe {
    let mut raw = base_probe();
    raw.hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(raw.hostname);

    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        raw.cpu_brand = cpuinfo
            .lines()
            .find_map(|l| {
                ["model name", "Hardware", "Processor"].iter().find_map(|k| {
                    l.strip_prefix(&format!("{k}\t:")).map(str::trim)
                })
            })
            .unwrap_or_default()
            .to_string();
        let mut pairs = BTreeSet::new();
        let mut phys = 0u32;
        for line in cpuinfo.lines() {
            if let Some(v) = line.strip_prefix("physical id\t:") {
                phys = v.trim().parse().unwrap_or(0);
            } else if let Some(v) = line.strip_prefix("core id\t:") {
                let core = v.trim().parse().unwrap_or(0);
                pairs.insert((phys, core));
            }
        }
        if !pairs.is_empty() {
            raw.physical_cores = pairs.len() as u32;
        }
    }
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        raw.ram_mb = meminfo
            .lines()
            .find_map(|l| l.strip_prefix("MemTotal:"))
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|kb| kb.parse::<u64>().ok())
            .map(|kb| kb / 1024)
            .unwrap_or(0);
    }

    if let Some(out) = run(
        "nvidia-smi",
        &[
            "--query-gpu=name,memory.total,driver_version",
            "--format=csv,noheader,nounits",
        ],
    ) {
        for line in out.lines() {
            let parts: Vec<&str> = line.split(',').map(str::trim).collect();
            let name = parts.first().copied().unwrap_or("NVIDIA GPU").to_string();
            let vram = parts.get(1).and_then(|s| s.parse::<u64>().ok());
            raw.gpus.push(GpuInfo {
                vendor: "nvidia".into(),
                kind: GpuKind::Discrete,
                name,
                vram_mb: vram,
                driver: parts.get(2).map(|s| s.to_string()),
            });
        }
    }
    if let Some(out) = run("lspci", &[]) {
        for line in out.lines() {
            let lower = line.to_ascii_lowercase();
            if !(lower.contains("vga compatible controller")
                || lower.contains("3d controller")
                || lower.contains("display controller"))
            {
                continue;
            }
            let Some((_, rest)) = line.split_once(": ") else {
                continue;
            };
            let name = rest.split(" (rev").next().unwrap_or(rest).trim().to_string();
            if name.to_ascii_lowercase().contains("nvidia")
                && raw.gpus.iter().any(|g| g.vendor == "nvidia")
            {
                continue;
            }
            let vram = (lower.contains("amd") || lower.contains("ati"))
                .then(first_amdgpu_vram)
                .flatten();
            raw.gpus.push(GpuInfo {
                vendor: vendor_of(&name),
                kind: gpu_kind(&name, vram.unwrap_or(0)),
                vram_mb: vram,
                driver: None,
                name,
            });
        }
    }
    if raw.gpus.is_empty()
        && let Some(driver) = std::fs::read_to_string("/sys/class/drm/card0/device/uevent")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find_map(|l| l.strip_prefix("DRIVER=").map(str::to_string))
            })
    {
        raw.gpus.push(GpuInfo {
            name: format!("{} GPU (card0)", driver),
            vendor: vendor_of(&driver),
            kind: gpu_kind(&driver, 0),
            vram_mb: first_amdgpu_vram(),
            driver: Some(driver),
        });
    }
    raw.gpus = dedup_gpus(raw.gpus);
    let (free, total) = unix_disk(&crate::platform::data_dir());
    raw.free_disk_mb = free;
    raw.total_disk_mb = total;
    raw
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
fn probe() -> RawProbe {
    base_probe()
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn unix_disk(path: &Path) -> (Option<u64>, Option<u64>) {
    let Some(out) = run("df", &["-kP", &path.display().to_string()]) else {
        return (None, None);
    };
    let Some(line) = out.lines().nth(1) else {
        return (None, None);
    };
    let fields: Vec<&str> = line.split_whitespace().collect();
    let total = fields
        .get(1)
        .and_then(|v| v.parse::<u64>().ok())
        .map(|kb| kb / 1024);
    let free = fields
        .get(3)
        .and_then(|v| v.parse::<u64>().ok())
        .map(|kb| kb / 1024);
    (free, total)
}

#[cfg(target_os = "linux")]
fn first_amdgpu_vram() -> Option<u64> {
    for i in 0..8 {
        let path = format!("/sys/class/drm/card{i}/device/mem_info_vram_total");
        if let Ok(raw) = std::fs::read_to_string(&path)
            && let Ok(bytes) = raw.trim().parse::<u64>()
            && bytes > 0
        {
            return Some(bytes / (1024 * 1024));
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn parse_mem_mb(raw: &str) -> Option<u64> {
    let raw = raw.trim();
    let number: f64 = raw
        .split_whitespace()
        .next()?
        .replace(',', "")
        .parse()
        .ok()?;
    let unit = raw
        .split_whitespace()
        .nth(1)
        .unwrap_or("MB")
        .to_ascii_uppercase();
    let mb = match unit.as_str() {
        "GB" | "GIB" => number * 1024.0,
        "TB" | "TIB" => number * 1024.0 * 1024.0,
        "KB" | "KIB" => number / 1024.0,
        _ => number,
    };
    Some(mb.round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(kind: Option<(GpuKind, u64)>, ram_mb: u64, logical: u32) -> HardwareProfile {
        HardwareProfile {
            platform: Platform::Linux,
            os: "linux".into(),
            arch: "x86_64".into(),
            hostname: "test".into(),
            cpu_brand: "Test CPU".into(),
            physical_cores: logical / 2,
            logical_cores: logical,
            ram_mb,
            gpus: kind
                .map(|(kind, vram)| {
                    vec![GpuInfo {
                        name: "Test GPU".into(),
                        vendor: "test".into(),
                        kind,
                        vram_mb: Some(vram),
                        driver: None,
                    }]
                })
                .unwrap_or_default(),
            free_disk_mb: Some(100_000),
            total_disk_mb: Some(500_000),
            data_dir: "/tmp/noema".into(),
            detected_at_ms: 0,
        }
    }

    #[test]
    fn quality_follows_gpu_and_ram() {
        let ultra = derive_tuning(&profile(Some((GpuKind::Discrete, 12_000)), 64_000, 24));
        assert_eq!(ultra.quality, Quality::Ultra);
        assert_eq!(ultra.render.pixel_ratio_cap, 2.0);
        assert_eq!(ultra.inference.max_context_tokens, 16_384);
        assert_eq!(ultra.daemon.worker_threads, 16);

        let high = derive_tuning(&profile(Some((GpuKind::Discrete, 8_000)), 16_000, 12));
        assert_eq!(high.quality, Quality::High);
        assert_eq!(high.inference.max_context_tokens, 16_384);

        let balanced = derive_tuning(&profile(Some((GpuKind::Integrated, 2_000)), 16_000, 8));
        assert_eq!(balanced.quality, Quality::Balanced);
        assert_eq!(balanced.inference.gpu_layers, 0);
        assert_eq!(balanced.inference.recommended_runtime, "llama.cpp");

        let low = derive_tuning(&profile(None, 4_000, 4));
        assert_eq!(low.quality, Quality::Low);
        assert_eq!(low.inference.max_context_tokens, 4_096);
        assert_eq!(low.render.particle_budget, 600);
    }

    #[test]
    fn overrides_patch_nested_values_and_reject_typos() {
        let p = profile(Some((GpuKind::Discrete, 7_000)), 32_000, 16);
        let tuning = effective(
            &p,
            &json!({ "render": { "particle_budget": 999 }, "inference": { "parallel_requests": 3 } }),
        )
        .unwrap();
        assert_eq!(tuning.render.particle_budget, 999);
        assert_eq!(tuning.inference.parallel_requests, 3);
        assert_eq!(tuning.render.pixel_ratio_cap, 1.75);

        let err = effective(&p, &json!({ "render": { "pixel_ratio": 1.0 } })).unwrap_err();
        assert!(err.contains("invalid hardware override"));
    }

    #[test]
    fn merge_deletes_with_null_and_keeps_objects() {
        let mut base = json!({ "a": { "b": 1, "c": 2 }, "d": 3 });
        merge(&mut base, &json!({ "a": { "b": null }, "e": 4 }));
        assert_eq!(base, json!({ "a": { "c": 2 }, "d": 3, "e": 4 }));
    }

    #[test]
    fn settings_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, scope TEXT NOT NULL);",
        )
        .unwrap();
        write_setting(&conn, PROFILE_KEY, "{\"platform\":\"linux\"}").unwrap();
        assert_eq!(
            read_setting(&conn, PROFILE_KEY).as_deref(),
            Some("{\"platform\":\"linux\"}")
        );
        clear_setting(&conn, PROFILE_KEY).unwrap();
        assert_eq!(read_setting(&conn, PROFILE_KEY), None);
    }
}
