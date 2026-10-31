use serde_json::{Value, json};

use noema_core::Store;
use noema_core::daemon::{data_dir, read_port_file};
use crate::theme::Styler;

#[derive(Debug, Clone)]
pub struct CheckItem {
    pub category: &'static str,
    pub title: &'static str,
    pub status: CheckStatus,
    pub details: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

pub fn run(styler: &Styler, as_json: bool) -> Result<Value, String> {
    let mut items = Vec::new();

    // 1. Database & Local Store
    let db_path = data_dir().join("noema.db");
    if db_path.exists() {
        match Store::open(&db_path) {
            Ok(store) => {
                let resume = store.resume();
                items.push(CheckItem {
                    category: "Storage",
                    title: "SQLite State Store",
                    status: CheckStatus::Pass,
                    details: format!("WAL database online at {}", db_path.display()),
                    hint: None,
                });
                if let Ok(r) = resume {
                    let sess_id = r.last_session.map(|s| s.to_string()).unwrap_or_else(|| "none".into());
                    items.push(CheckItem {
                        category: "Storage",
                        title: "Session Persistence",
                        status: CheckStatus::Pass,
                        details: format!("Last active session: {sess_id}"),
                        hint: None,
                    });
                }
            }
            Err(e) => {
                items.push(CheckItem {
                    category: "Storage",
                    title: "SQLite State Store",
                    status: CheckStatus::Fail,
                    details: format!("Failed to open store: {e}"),
                    hint: Some(format!("Check file permissions or restore {}", db_path.display())),
                });
            }
        }
    } else {
        items.push(CheckItem {
            category: "Storage",
            title: "SQLite State Store",
            status: CheckStatus::Warn,
            details: format!("Database not initialized at {}", db_path.display()),
            hint: Some("Run 'noema status' or launch noema-app to seed initial tables".into()),
        });
    }

    // 2. Background Daemon
    let port_opt = read_port_file(&data_dir()).map(|pf| pf.port);
    match port_opt {
        Some(port) => {
            // Test connection
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
            match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(600)) {
                Ok(_) => {
                    items.push(CheckItem {
                        category: "Daemon",
                        title: "noemad IPC / RPC",
                        status: CheckStatus::Pass,
                        details: format!("Running and listening on 127.0.0.1:{port}"),
                        hint: None,
                    });
                }
                Err(e) => {
                    items.push(CheckItem {
                        category: "Daemon",
                        title: "noemad IPC / RPC",
                        status: CheckStatus::Warn,
                        details: format!("Port file indicates {port}, but TCP connect timed out: {e}"),
                        hint: Some("Start noemad with 'noema status' or restart noema-app".into()),
                    });
                }
            }
        }
        None => {
            items.push(CheckItem {
                category: "Daemon",
                title: "noemad IPC / RPC",
                status: CheckStatus::Warn,
                details: "Daemon is currently offline (no port file found)".into(),
                hint: Some("Start daemon with 'noema status' or launch noema-app.exe".into()),
            });
        }
    }

    // 3. Hardware Acceleration & Probe
    let hw = noema_core::hardware::detect();
    let tuning = noema_core::hardware::derive_tuning(&hw);
    let gpu_name = hw.gpus.first().map(|g| g.name.as_str()).unwrap_or("Integrated");
    let gpu_ok = !gpu_name.to_lowercase().contains("basic") && !gpu_name.is_empty();

    items.push(CheckItem {
        category: "Hardware",
        title: "Compute & Memory",
        status: CheckStatus::Pass,
        details: format!(
            "CPU: {} ({} logical cores) · RAM: {:.1} GiB total",
            hw.cpu_brand,
            hw.logical_cores,
            hw.ram_mb as f64 / 1024.0
        ),
        hint: None,
    });
    items.push(CheckItem {
        category: "Hardware",
        title: "GPU & Graphics",
        status: if gpu_ok { CheckStatus::Pass } else { CheckStatus::Warn },
        details: format!("GPU: {} (Tier: {:?})", gpu_name, tuning.quality),
        hint: if gpu_ok { None } else { Some("Check graphics drivers for hardware-accelerated 3D rendering".into()) },
    });

    // 4. PATH Enrollment & Environment
    let path_status = noema_core::path_env::status();
    match path_status {
        Ok(v) => {
            let enrolled = v.get("noema_enrolled_in_path").and_then(Value::as_bool).unwrap_or(false);
            let broken = v.get("broken_paths").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
            let dupes = v.get("duplicates").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
            let current_dir = v.get("current_exe_dir").and_then(Value::as_str).unwrap_or("unknown");

            items.push(CheckItem {
                category: "Environment",
                title: "NOEMA in PATH",
                status: if enrolled { CheckStatus::Pass } else { CheckStatus::Warn },
                details: if enrolled {
                    format!("Registered in User PATH ({current_dir})")
                } else {
                    format!("Running from {current_dir}, but not found in User PATH")
                },
                hint: if enrolled { None } else { Some("Run 'noema path setup' to enroll permanently".into()) },
            });

            if broken > 0 || dupes > 0 {
                items.push(CheckItem {
                    category: "Environment",
                    title: "PATH Cleanliness",
                    status: CheckStatus::Warn,
                    details: format!("{dupes} duplicate entries, {broken} broken directories found in PATH"),
                    hint: Some("Run 'noema path clean' to safely prune stale entries".into()),
                });
            } else {
                items.push(CheckItem {
                    category: "Environment",
                    title: "PATH Cleanliness",
                    status: CheckStatus::Pass,
                    details: "Clean: 0 duplicates, 0 dead directories in User PATH".into(),
                    hint: None,
                });
            }
        }
        Err(e) => {
            items.push(CheckItem {
                category: "Environment",
                title: "PATH Status",
                status: CheckStatus::Fail,
                details: format!("Failed to read PATH: {e}"),
                hint: None,
            });
        }
    }

    // 5. Companion Toolchain (ArtCraft, Driftwatch, Cargo)
    let user_paths = noema_core::path_env::get_user_path().unwrap_or_default();
    let companions = noema_core::path_env::scan_companion_tools(&user_paths);
    for tool in companions {
        let status = if tool.exists && tool.on_path {
            CheckStatus::Pass
        } else if tool.exists {
            CheckStatus::Warn
        } else {
            CheckStatus::Warn
        };

        let details = if tool.exists && tool.on_path {
            format!("Installed at {} (available on PATH)", tool.path)
        } else if tool.exists {
            format!("Installed at {} (not on PATH)", tool.path)
        } else {
            format!("Not installed at default path ({})", tool.path)
        };

        let hint = if tool.exists && !tool.on_path {
            Some(format!("Run 'noema path add \"{}\"' to add to PATH", tool.path))
        } else {
            None
        };

        items.push(CheckItem {
            category: "Toolchain",
            title: Box::leak(tool.name.into_boxed_str()),
            status,
            details,
            hint,
        });
    }

    // 6. Ecosystem & MCP Packages
    let bundled = noema_ecosystem::marketplace::bundled();
    items.push(CheckItem {
        category: "Ecosystem",
        title: "MCP Package Registry",
        status: CheckStatus::Pass,
        details: format!("{} verified bundled marketplace packages loaded", bundled.len()),
        hint: None,
    });

    // 7. Inference Model Catalog & API Keys
    let catalog = noema_inference::default_catalog();
    let anthropic_key = std::env::var_os("ANTHROPIC_API_KEY").is_some();
    let openai_key = std::env::var_os("OPENAI_API_KEY").is_some();
    let gemini_key = std::env::var_os("GEMINI_API_KEY").is_some();
    let keys_count = [anthropic_key, openai_key, gemini_key].iter().filter(|&&k| k).count();

    items.push(CheckItem {
        category: "Inference",
        title: "Model Catalog & FinOps",
        status: CheckStatus::Pass,
        details: format!("{} upstream models mapped with token rate cards", catalog.len()),
        hint: None,
    });

    items.push(CheckItem {
        category: "Inference",
        title: "Provider API Keys",
        status: if keys_count > 0 { CheckStatus::Pass } else { CheckStatus::Warn },
        details: format!("{keys_count} provider keys detected in environment (Anthropic, OpenAI, Gemini)"),
        hint: if keys_count > 0 { None } else { Some("Set ANTHROPIC_API_KEY or OPENAI_API_KEY for cloud inference".into()) },
    });

    let passes = items.iter().filter(|i| i.status == CheckStatus::Pass).count();
    let warns = items.iter().filter(|i| i.status == CheckStatus::Warn).count();
    let fails = items.iter().filter(|i| i.status == CheckStatus::Fail).count();

    if as_json {
        return Ok(json!({
            "total": items.len(),
            "passed": passes,
            "warned": warns,
            "failed": fails,
            "items": items.iter().map(|i| json!({
                "category": i.category,
                "title": i.title,
                "status": match i.status {
                    CheckStatus::Pass => "pass",
                    CheckStatus::Warn => "warn",
                    CheckStatus::Fail => "fail",
                },
                "details": i.details,
                "hint": i.hint,
            })).collect::<Vec<_>>()
        }));
    }

    // Render interactive styled output
    println!("\n{}\n", styler.banner(Some("System & Code Diagnostics")));

    let mut lines = Vec::new();
    let mut current_cat = "";

    for item in &items {
        if item.category != current_cat {
            current_cat = item.category;
            lines.push(format!("\n{}", styler.bold(&styler.artery(&format!("▶ {current_cat}")))));
        }

        let badge = match item.status {
            CheckStatus::Pass => styler.pass_badge(),
            CheckStatus::Warn => styler.warn_badge(),
            CheckStatus::Fail => styler.fail_badge(),
        };

        lines.push(format!("  {badge}  {}  {}", styler.bold(&styler.ink(item.title)), styler.ink_muted(&item.details)));
        if let Some(hint) = &item.hint {
            lines.push(format!("         {} {}", styler.nerve("➜ Suggestion:"), styler.ink_soft(hint)));
        }
    }

    let summary_line = format!(
        "\n  Summary: {} passed · {} warnings · {} failures\n",
        styler.lymph(&passes.to_string()),
        styler.warning(&warns.to_string()),
        styler.danger(&fails.to_string()),
    );
    lines.push(summary_line);

    println!("{}", styler.card("NOEMA System & Code Doctor", &lines));

    Ok(json!({
        "passed": passes,
        "warned": warns,
        "failed": fails,
    }))
}
