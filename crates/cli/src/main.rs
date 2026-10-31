use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::process::ExitCode;
use std::time::Duration;

use noema_core::Store;
use noema_core::daemon::{data_dir, read_port_file};
use noema_core::rpc::METHODS;
use serde_json::{Value, json};

mod agents_cmd;
mod chat_cmd;
mod diff;
mod doctor;
mod mcp_cmd;
mod theme;

use theme::Styler;

const HELP: &str = "\
noema - the anatomical mind (Code CLI)

usage: noema <command> [options]

  Core & Diagnostics:
    status                      live system & session resume state
    doctor                      full system, hardware, database & toolchain health audit
    health                      app bridge and daemon health probe
    system profile [--refresh]  hardware profile & inference tuning
    system override JSON        deep-merge hardware & inference tuning patch
    system reset                reset hardware overrides and cached profile

  Code & Workspace:
    diff [fileA fileB | ref]    unified code diff viewer with syntax highlighting
    chat [prompt]               interactive AI coding assistant with streaming thoughts
    theme [list|set <name>]     inspect or configure terminal theme (cavity, bone, nord, etc.)
    path [status|add|clean]     manage PATH and companion toolchain enrollment (ArtCraft, Driftwatch)

  Agents & Anatomy:
    anatomy                     living anatomical model, agents & heart telemetry
    agents                      roster of the 8 specialized agents and domain assignments
    anima [set JSON]            read or patch heart/Anima identity system prompt

  Memory & Cognition:
    brain                       semantic thought graph snapshot
    brain telemetry             streaming logprob surprisal telemetry
    brain drag JSON             persist 3D thought spatial coordinate
    brain dream JSON            record an autonomous memory consolidation dream

  Studio & Sessions:
    studio                      Studio snapshot: active session, roles, widgets, FinOps
    studio session ID           switch active session focus
    studio role CAP MODEL       assign model to capability
    studio widget JSON          upsert BI or monitoring widget
    studio widget-remove ID     delete a widget
    studio layout JSON          persist grid layout
    session list [--limit N]    daemon session index
    session open JSON           open or update session
    session resume              retrieve resume checkpoint

  Ecosystem & Supervisor:
    mcp [list|status|test PATH] supervise MCP servers, stdio/SSE/WS lifecycle
    methods                     every registered daemon RPC method
    rpc <method> [json]         raw RPC invocation

  Maintenance:
    update [--check|--force]    check for updates and auto-update NOEMA from GitHub
    help                        this text

Flags:
  --json                        force raw JSON output for scripting and pipes
  --no-color                    disable ANSI terminal colors

App actions use the Studio bridge on NOEMA_APP_PORT (default 4173).
Every GUI action is reachable here. Noemad owns the database.";

fn main() -> ExitCode {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    let as_json = raw_args.iter().any(|a| a == "--json");
    let no_color = raw_args.iter().any(|a| a == "--no-color");

    let styler = Styler::new(as_json, no_color);

    // Filter out global flags
    let args: Vec<String> = raw_args
        .into_iter()
        .filter(|a| a != "--json" && a != "--no-color")
        .collect();

    let command = args.first().map(String::as_str).unwrap_or("status");

    let result = match command {
        "status" => return status(&styler, as_json),
        "doctor" => doctor::run(&styler, as_json),
        "diff" => diff::handle(&args[1..], &styler, as_json),
        "mcp" => mcp_cmd::handle(&args[1..], &styler, as_json),
        "agents" => agents_cmd::handle(&styler, as_json),
        "chat" | "ask" => chat_cmd::handle(args.get(1).map(String::as_str), &styler, as_json),
        "theme" => theme_cmd(&args[1..], &styler, as_json),
        "health" => health(&styler, as_json),
        "anatomy" => http("GET", "/api/anatomy", None),
        "anima" => match args.get(1).map(String::as_str) {
            Some("set") => {
                parse_body(args.get(2)).and_then(|b| http("POST", "/api/anima", Some(b)))
            }
            None => http("GET", "/api/anatomy", None),
            Some(_) => parse_body(args.get(1)).and_then(|b| http("POST", "/api/anima", Some(b))),
        },
        "brain" => brain(&args[1..]),
        "system" => system(&args[1..]),
        "studio" => studio(&args[1..]),
        "session" => session(&args[1..], &styler, as_json),
        "path" => path(&args[1..], &styler, as_json),
        "update" => update(&args[1..], &styler, as_json),
        "methods" => {
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!(METHODS)).unwrap_or_default()
                );
            } else {
                let headers = ["RPC Method", "Domain"];
                let rows: Vec<Vec<String>> = METHODS
                    .iter()
                    .map(|m| {
                        let domain = m.split('.').next().unwrap_or("core");
                        vec![styler.bold(&styler.artery(m)), styler.vein(domain)]
                    })
                    .collect();
                println!("\n{}", styler.bold(&styler.artery("▶ Registered Daemon RPC Methods:")));
                println!("{}", styler.table(&headers, &rows));
                println!("  Total methods: {}\n", styler.lymph(&METHODS.len().to_string()));
            }
            return ExitCode::SUCCESS;
        }
        "rpc" => return rpc(&args[1..]),
        "help" | "-h" | "--help" => {
            print_help(&styler);
            return ExitCode::SUCCESS;
        }
        other => {
            if as_json {
                eprintln!("{}", json!({ "error": format!("unknown command: {other}") }));
            } else {
                let lines = vec![
                    format!("Unknown command: '{}'", styler.danger(other)),
                    styler.dim("──────────────────────────────────────────────────────────"),
                    format!("Run '{}' to explore the command catalog", styler.nerve("noema --help")),
                    format!("Run '{}' to audit system health", styler.lymph("noema doctor")),
                ];
                eprintln!("{}", styler.card("Command Not Recognized", &lines));
            }
            return ExitCode::FAILURE;
        }
    };

    match result {
        Ok(value) => {
            if as_json {
                print_json(&value);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            if as_json {
                eprintln!("{}", json!({ "error": e }));
            } else {
                let lines = vec![
                    format!("Error: {}", styler.danger(&e)),
                    styler.dim("──────────────────────────────────────────────────────────"),
                    "Troubleshooting Suggestions:".to_string(),
                    format!("  {} Run '{}' to diagnose system status", styler.nerve("➜"), styler.bold("noema doctor")),
                    format!("  {} Check background daemon with '{}'", styler.nerve("➜"), styler.bold("noema status")),
                    format!("  {} Pass '{}' to view raw machine payload", styler.nerve("➜"), styler.bold("--json")),
                ];
                eprintln!("{}", styler.card("NOEMA Execution Failure", &lines));
            }
            ExitCode::FAILURE
        }
    }
}

fn print_help(styler: &Styler) {
    println!("\n{}\n", styler.banner(Some("The anatomical mind and agentic coding environment")));
    println!("{HELP}\n");
}

fn health(styler: &Styler, as_json: bool) -> Result<Value, String> {
    let res = http("GET", "/api/health", None);
    match res {
        Ok(v) => {
            if as_json {
                return Ok(v);
            }
            let lines = vec![
                format!("Bridge Status: {}", styler.pass_badge()),
                format!("Port: {}", styler.vein(&app_port().to_string())),
                format!("Host: {}", styler.ink_soft("127.0.0.1 (loopback Studio bridge)")),
                format!("Response: {}", styler.lymph("Healthy (HTTP 200 OK)")),
            ];
            println!("{}", styler.card("Studio Bridge Health", &lines));
            Ok(v)
        }
        Err(e) => {
            if as_json {
                return Err(e);
            }
            let lines = vec![
                format!("Bridge Status: {}", styler.fail_badge()),
                format!("Target: 127.0.0.1:{}", app_port()),
                format!("Detail: {}", styler.ink_muted(&e)),
                styler.dim("──────────────────────────────────────────────────────────"),
                format!("➜ Launch desktop application or run '{}'", styler.nerve("noema-app.exe")),
            ];
            println!("{}", styler.card("Studio Bridge Offline", &lines));
            Err(e)
        }
    }
}

fn status(styler: &Styler, as_json: bool) -> ExitCode {
    let value = call("session.resume", json!({})).or_else(|offline| {
        let db = data_dir().join("noema.db");
        Store::open(&db)
            .and_then(|store| store.resume())
            .map(|resume| json!({ "resume": resume, "source": "local", "offline_reason": offline }))
            .map_err(|e| e.to_string())
    });

    match value {
        Ok(value) => {
            if as_json {
                print_json(&value);
                return ExitCode::SUCCESS;
            }

            println!("\n{}\n", styler.banner(Some("System & Session Status")));

            let source = value.get("source").and_then(Value::as_str).unwrap_or("daemon");
            let is_daemon = source == "daemon";

            let mut lines = Vec::new();
            lines.push(format!(
                "Daemon Status: {}",
                if is_daemon { styler.pass_badge() } else { styler.warn_badge() }
            ));
            lines.push(format!(
                "Source: {}",
                if is_daemon { styler.lymph("Live daemon (RPC port active)") } else { styler.warning("Local SQLite WAL store (daemon offline)") }
            ));

            if let Some(resume) = value.get("resume") {
                if let Some(session) = resume.get("session") {
                    let id = session.get("id").and_then(Value::as_str).unwrap_or("-");
                    let title = session.get("title").and_then(Value::as_str).unwrap_or("active session");
                    lines.push(format!("Active Session: {} ({})", styler.bold(&styler.artery(id)), styler.ink(title)));
                } else if let Some(last_session) = resume.get("last_session").and_then(Value::as_str) {
                    let agent = resume.get("last_agent").and_then(Value::as_str).unwrap_or("noema-cortex");
                    lines.push(format!("Active Session: {} (Agent: {})", styler.bold(&styler.artery(last_session)), styler.nerve(agent)));
                }
            }

            lines.push(styler.dim("──────────────────────────────────────────────────────────"));
            lines.push(format!("Database: {}", styler.ink_soft(&data_dir().join("noema.db").display().to_string())));
            lines.push(format!("Run '{}' for complete diagnostic audit", styler.nerve("noema doctor")));

            println!("{}", styler.card("NOEMA System & Session Status", &lines));
            ExitCode::SUCCESS
        }
        Err(e) => {
            if as_json {
                eprintln!("{}", json!({ "error": e }));
            } else {
                let lines = vec![
                    format!("Status Error: {}", styler.danger(&e)),
                    "Neither daemon nor local store could be read.".to_string(),
                ];
                eprintln!("{}", styler.card("Status Error", &lines));
            }
            ExitCode::FAILURE
        }
    }
}

fn theme_cmd(args: &[String], styler: &Styler, as_json: bool) -> Result<Value, String> {
    let sub = args.first().map(String::as_str).unwrap_or("status");
    match sub {
        "status" => {
            let cfg = theme::load_config();
            if as_json {
                return Ok(json!({ "active_theme": cfg.theme, "no_color": cfg.no_color }));
            }
            let mut lines = Vec::new();
            lines.push(format!("Active Theme: {}", styler.bold(&styler.artery(&cfg.theme))));
            lines.push(format!("Palette Tokens Preview:"));
            lines.push(format!("  {} Artery (brand & actions)", styler.artery("■■■■")));
            lines.push(format!("  {} Nerve (focus & alerts)", styler.nerve("■■■■")));
            lines.push(format!("  {} Vein (links & secondary)", styler.vein("■■■■")));
            lines.push(format!("  {} Lymph (success & healthy)", styler.lymph("■■■■")));
            lines.push(format!("  {} Danger (destructive error)", styler.danger("■■■■")));
            lines.push(format!("  {} Ink (warm bone foreground)", styler.ink("■■■■")));
            lines.push(styler.dim("──────────────────────────────────────────────────────────"));
            lines.push(format!("Available themes: cavity, bone, nord, catppuccin, monokai"));
            lines.push(format!("Set theme with: {}", styler.nerve("noema theme set <name>")));
            println!("{}", styler.card("Vitreous Terminal Theme Status", &lines));
            Ok(json!({ "theme": cfg.theme }))
        }
        "list" => {
            let themes = ["cavity", "bone", "nord", "catppuccin", "monokai"];
            if as_json {
                return Ok(json!({ "themes": themes }));
            }
            let headers = ["Theme Name", "Description", "Mode"];
            let rows = vec![
                vec![styler.artery("cavity"), "Editorial anatomy dark cavity (default)".into(), "Dark".into()],
                vec![styler.nerve("bone"), "Warm ivory and frosted bone glass".into(), "Light".into()],
                vec![styler.vein("nord"), "Arctic, north-bluish palette".into(), "Dark".into()],
                vec![styler.lymph("catppuccin"), "Soothing warm pastel mocha".into(), "Dark".into()],
                vec![styler.danger("monokai"), "High-contrast developer icon".into(), "Dark".into()],
            ];
            println!("\n{}", styler.bold(&styler.artery("▶ Available NOEMA Themes:")));
            println!("{}", styler.table(&headers, &rows));
            Ok(json!({ "themes": themes }))
        }
        "set" => {
            let Some(name) = args.get(1) else {
                return Err("usage: noema theme set <cavity|bone|nord|catppuccin|monokai>".into());
            };
            let mut cfg = theme::load_config();
            cfg.theme = name.to_lowercase();
            theme::save_config(&cfg)?;
            if as_json {
                return Ok(json!({ "updated": true, "theme": cfg.theme }));
            }
            println!("{}", styler.lymph(&format!("✓ Theme updated to '{}'. Enjoy the new palette!", cfg.theme)));
            Ok(json!({ "theme": cfg.theme }))
        }
        other => Err(format!("unknown theme command '{other}'. Expected status, list, set")),
    }
}

fn path(args: &[String], styler: &Styler, as_json: bool) -> Result<Value, String> {
    let sub = args.first().map(String::as_str).unwrap_or("status");
    let result = match sub {
        "status" => noema_core::path_env::status()?,
        "add" => {
            let target = args.get(1).map(std::path::PathBuf::from).or_else(noema_core::path_env::current_exe_dir);
            let Some(dir) = target else {
                return Err("could not determine directory to add".into());
            };
            noema_core::path_env::add(&dir)?
        }
        "remove" => {
            let Some(target) = args.get(1) else {
                return Err("usage: noema path remove <directory>".into());
            };
            noema_core::path_env::remove(std::path::Path::new(target))?
        }
        "clean" => noema_core::path_env::clean()?,
        "setup" => noema_core::path_env::setup()?,
        other => return Err(format!("unknown path command: {other}. Expected status, add, remove, clean, setup")),
    };

    if as_json {
        return Ok(result);
    }

    // Render interactive styled view of PATH status
    let current_dir = result.get("current_exe_dir").and_then(Value::as_str).unwrap_or("unknown");
    let enrolled = result.get("noema_enrolled_in_path").and_then(Value::as_bool).unwrap_or(false);

    let mut lines = Vec::new();
    lines.push(format!(
        "NOEMA in PATH: {}",
        if enrolled { styler.pass_badge() } else { styler.warn_badge() }
    ));
    lines.push(format!("Binary Directory: {}", styler.ink_soft(current_dir)));

    if let Some(companions) = result.get("companion_tools").and_then(Value::as_array) {
        lines.push(styler.dim("──────────────────────────────────────────────────────────"));
        lines.push(format!("Companion Toolchain Detection:"));
        for c in companions {
            let name = c.get("name").and_then(Value::as_str).unwrap_or("");
            let on_path = c.get("on_path").and_then(Value::as_bool).unwrap_or(false);
            let exists = c.get("exists").and_then(Value::as_bool).unwrap_or(false);
            let path = c.get("path").and_then(Value::as_str).unwrap_or("");
            let bullet = if on_path {
                styler.lymph("●")
            } else if exists {
                styler.warning("○")
            } else {
                styler.danger("✗")
            };
            let status_note = if on_path {
                "on PATH"
            } else if exists {
                "installed (not on PATH)"
            } else {
                "not found"
            };
            lines.push(format!("  {bullet} {name}  {} ({status_note})", styler.dim(path)));
        }
    }

    println!("{}", styler.card("PATH & Toolchain Enrollment", &lines));
    Ok(result)
}

fn update(args: &[String], styler: &Styler, as_json: bool) -> Result<Value, String> {
    let check_only = args.iter().any(|a| a == "--check");
    let force = args.iter().any(|a| a == "--force");
    let repo = std::env::var("NOEMA_GITHUB_REPO").ok();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("failed to initialize tokio runtime: {e}"))?;

    rt.block_on(async {
        let check = noema_core::update::check_update(repo.as_deref()).await?;
        if check_only {
            let val = serde_json::to_value(&check).map_err(|e| e.to_string())?;
            if as_json {
                return Ok(val);
            }
            let mut lines = Vec::new();
            lines.push(format!("Installed Version: {}", styler.bold(&styler.ink(&check.current_version))));
            lines.push(format!("Latest GitHub Release: {}", styler.bold(&styler.artery(&check.latest_version))));
            lines.push(format!(
                "Update Status: {}",
                if check.has_update {
                    styler.warn_badge()
                } else {
                    styler.pass_badge()
                }
            ));
            if check.has_update {
                lines.push(styler.dim("──────────────────────────────────────────────────────────"));
                lines.push(format!("⚡ A new release is available!"));
                lines.push(format!("Run '{}' to upgrade in-place.", styler.nerve("noema update")));
            } else {
                lines.push(format!("You are on the latest stable release."));
            }
            if let Some(asset) = &check.matching_asset {
                lines.push(format!("Platform Asset: {} ({:.1} MB)", styler.dim(&asset.name), asset.size as f64 / 1_048_576.0));
            }
            println!("{}", styler.card("NOEMA GitHub Release Update Check", &lines));
            return Ok(val);
        }
        let applied = noema_core::update::apply_update(&check, force).await?;
        if !as_json {
            println!("{}", styler.lymph(&format!("✓ NOEMA updated successfully to v{}!", check.latest_version)));
        }
        Ok(applied)
    })
}

fn brain(args: &[String]) -> Result<Value, String> {
    match args.first().map(String::as_str) {
        None => http("GET", "/api/brain", None),
        Some("telemetry") => http("GET", "/api/brain/telemetry", None),
        Some("drag") => {
            let body = parse_body(args.get(1))?;
            http("POST", "/api/brain/drag", Some(body))
        }
        Some("dream") => {
            let body = parse_body(args.get(1))?;
            http("POST", "/api/dream", Some(body))
        }
        Some(other) => Err(format!("unknown brain command: {other}")),
    }
}

fn system(args: &[String]) -> Result<Value, String> {
    let sub = args.first().map(String::as_str).unwrap_or("profile");
    match sub {
        "profile" => {
            let refresh = args.iter().any(|a| a == "--refresh");
            call("system.profile", json!({ "refresh": refresh })).or_else(|_offline| {
                local_store()?
                    .hardware_report(refresh)
                    .map(|r| json!(r))
                    .map_err(|e| e.to_string())
            })
        }
        "override" => {
            let patch = parse_body(args.get(1))?;
            call("system.override", json!({ "overrides": patch })).or_else(|_offline| {
                local_store()?
                    .set_hardware_overrides(&patch)
                    .map(|r| json!(r))
                    .map_err(|e| e.to_string())
            })
        }
        "reset" => call("system.reset", json!({})).or_else(|_offline| {
            local_store()?
                .reset_hardware()
                .map(|r| json!(r))
                .map_err(|e| e.to_string())
        }),
        other => Err(format!("unknown system command: {other}")),
    }
}

fn studio(args: &[String]) -> Result<Value, String> {
    match args.first().map(String::as_str) {
        None => http("GET", "/api/studio", None),
        Some("session") => {
            let id = args.get(1).ok_or("missing session id")?;
            http("POST", "/api/studio", Some(json!({ "action": "session", "id": id })))
        }
        Some("role") => {
            let cap = args.get(1).ok_or("usage: noema studio role CAP MODEL [--project ID]")?;
            let model = args.get(2).ok_or("usage: noema studio role CAP MODEL [--project ID]")?;
            let project = args.iter().position(|a| a == "--project").and_then(|i| args.get(i + 1));
            let mut payload = json!({ "action": "role", "capability": cap, "model": model });
            if let Some(p) = project {
                payload["project"] = json!(p);
            }
            http("POST", "/api/studio", Some(payload))
        }
        Some("widget") => {
            let body = parse_body(args.get(1))?;
            let mut payload = json!({ "action": "widget" });
            payload["widget"] = body;
            http("POST", "/api/studio", Some(payload))
        }
        Some("widget-remove") => {
            let id = args.get(1).ok_or("usage: noema studio widget-remove ID")?;
            http("POST", "/api/studio", Some(json!({ "action": "widget_remove", "id": id })))
        }
        Some("layout") => {
            let body = parse_body(args.get(1))?;
            http("POST", "/api/studio", Some(json!({ "action": "layout", "layout": body })))
        }
        Some(other) => Err(format!("unknown studio command: {other}")),
    }
}

fn session(args: &[String], styler: &Styler, as_json: bool) -> Result<Value, String> {
    let sub = args.first().map(String::as_str).unwrap_or("resume");
    match sub {
        "resume" => {
            let res = call("session.resume", json!({})).or_else(|offline| {
                let db = data_dir().join("noema.db");
                Store::open(&db)
                    .and_then(|store| store.resume())
                    .map(|resume| json!({ "resume": resume, "source": "local", "offline_reason": offline }))
                    .map_err(|e| e.to_string())
            })?;

            if as_json {
                return Ok(res);
            }

            println!("\n{}\n", styler.banner(Some("Session Resume Checkpoint")));

            let mut lines = Vec::new();
            let session_data = res.get("session").or_else(|| res.get("resume"));
            if let Some(session) = session_data {
                let id = session.get("id").or_else(|| session.get("last_session")).and_then(Value::as_str).unwrap_or("none");
                let agent = session.get("agent").or_else(|| session.get("last_agent")).and_then(Value::as_str).unwrap_or("noema-cortex");
                let model = session.get("model").and_then(Value::as_str).unwrap_or("claude-sonnet-4-6");
                let cwd = session.get("cwd").or_else(|| session.get("last_cwd")).and_then(Value::as_str).unwrap_or("-");
                let branch = session.get("branch").or_else(|| session.get("last_branch")).and_then(Value::as_str).unwrap_or("main");

                lines.push(format!("Session ID: {}", styler.bold(&styler.artery(id))));
                lines.push(format!("Agent:      {}", styler.nerve(agent)));
                lines.push(format!("Model:      {}", styler.vein(model)));
                lines.push(format!("Branch:     {}", styler.lymph(branch)));
                lines.push(format!("Workspace:  {}", styler.ink_muted(cwd)));

                if let Some(diff) = res.get("diff_stat").or_else(|| session.get("diff_stat")) {
                    let files = diff.get("files_changed").and_then(Value::as_u64).unwrap_or(0);
                    let ins = diff.get("insertions").and_then(Value::as_u64).unwrap_or(0);
                    let del = diff.get("deletions").and_then(Value::as_u64).unwrap_or(0);
                    lines.push(format!(
                        "Diff Stat:  {} files · {} insertions (+) · {} deletions (-)",
                        styler.ink(&files.to_string()),
                        styler.lymph(&ins.to_string()),
                        styler.danger(&del.to_string())
                    ));
                }
            } else {
                lines.push(styler.dim("No active working session found."));
            }

            lines.push(styler.dim("──────────────────────────────────────────────────────────"));
            lines.push(format!("Resume interactive chat with: {}", styler.nerve("noema chat")));
            lines.push(format!("List previous sessions with:     {}", styler.lymph("noema session list")));

            println!("{}", styler.card("Active Working Session State", &lines));
            Ok(res)
        }
        "list" => {
            let mut limit = 50u64;
            if let Some(i) = args.iter().position(|a| a == "--limit") {
                limit = args
                    .get(i + 1)
                    .and_then(|v| v.parse().ok())
                    .ok_or("--limit needs a number")?;
            }
            let res = call("session.list", json!({ "limit": limit })).or_else(|_offline| {
                let db = data_dir().join("noema.db");
                let store = Store::open(&db).map_err(|e| e.to_string())?;
                store.list_sessions(limit as u32).map(|sessions| json!({ "sessions": sessions })).map_err(|e| e.to_string())
            })?;

            if as_json {
                return Ok(res);
            }

            println!("\n{}\n", styler.banner(Some("Session History Index")));

            let empty = Vec::new();
            let sessions = res.get("sessions").and_then(Value::as_array).unwrap_or(&empty);

            if sessions.is_empty() {
                println!("  No previous sessions recorded in store.\n");
            } else {
                let headers = ["Session ID", "Agent", "Model", "Branch", "Workspace"];
                let rows: Vec<Vec<String>> = sessions.iter().map(|s| {
                    let id = s.get("id").and_then(Value::as_str).unwrap_or("-");
                    let agent = s.get("agent").and_then(Value::as_str).unwrap_or("-");
                    let model = s.get("model").and_then(Value::as_str).unwrap_or("-");
                    let branch = s.get("branch").and_then(Value::as_str).unwrap_or("main");
                    let cwd = s.get("cwd").and_then(Value::as_str).unwrap_or("-");
                    vec![
                        styler.bold(&styler.artery(id)),
                        styler.nerve(agent),
                        styler.vein(model),
                        styler.lymph(branch),
                        styler.ink_muted(cwd),
                    ]
                }).collect();

                println!("{}", styler.table(&headers, &rows));
                println!("  Total recorded sessions: {}\n", styler.lymph(&sessions.len().to_string()));
            }

            Ok(res)
        }
        "open" => {
            let session = parse_body(args.get(1))?;
            call("session.open", json!({ "session": session }))
        }
        other => Err(format!("unknown session command: {other}")),
    }
}

fn local_store() -> Result<Store, String> {
    Store::open(&data_dir().join("noema.db")).map_err(|e| e.to_string())
}

fn parse_body(raw: Option<&String>) -> Result<Value, String> {
    let raw = raw.ok_or("missing JSON argument")?;
    serde_json::from_str(raw).map_err(|e| format!("invalid JSON: {e}"))
}

fn rpc(args: &[String]) -> ExitCode {
    let Some(method) = args.first() else {
        eprintln!("usage: noema rpc <method> [json-params]");
        return ExitCode::FAILURE;
    };
    let params = args
        .get(1)
        .map(|raw| serde_json::from_str::<Value>(raw).unwrap_or_else(|_| json!({ "raw": raw })))
        .unwrap_or_else(|| json!({}));
    match call(method, params) {
        Ok(value) => {
            print_json(&value);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("rpc failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn app_port() -> u16 {
    std::env::var("NOEMA_APP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4173)
}

fn http(method: &str, path: &str, body: Option<Value>) -> Result<Value, String> {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, app_port()));
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(800))
        .map_err(|e| format!("app offline on {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    let body = body.map(|v| v.to_string()).unwrap_or_default();
    let head = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        app_port(),
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .and_then(|_| stream.write_all(body.as_bytes()))
        .map_err(|e| e.to_string())?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(|e| e.to_string())?;
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("malformed HTTP response")?;
    let status = String::from_utf8_lossy(&raw[..split])
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    let payload: Value =
        serde_json::from_slice(&raw[split + 4..]).map_err(|e| format!("{status}: {e}"))?;
    if !status.contains(" 200 ") {
        let message = payload
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("request failed");
        return Err(format!("{message} ({status})"));
    }
    Ok(payload)
}

fn call(method: &str, params: Value) -> Result<Value, String> {
    let port = read_port_file(&data_dir())
        .map(|pf| pf.port)
        .ok_or("daemon port file unavailable")?;
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(600))
        .map_err(|e| format!("daemon unreachable on {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });
    let mut line = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    line.push('\n');
    stream.write_all(line.as_bytes()).map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(stream);
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
            return Err("daemon closed the connection".into());
        }
        let Ok(response) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if response.get("method").is_some() {
            continue;
        }
        if let Some(error) = response.get("error") {
            return Err(error.to_string());
        }
        return Ok(response.get("result").cloned().unwrap_or(Value::Null));
    }
}

fn print_json(value: &Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli_path(method: &str) -> Option<&'static str> {
        match method {
            "rpc.methods" => Some("noema methods"),
            "daemon.ping" => Some("noema health"),
            "session.resume" => Some("noema status | noema session resume"),
            "session.list" => Some("noema session list"),
            "session.open" => Some("noema session open '<json>'"),
            "system.profile" => Some("noema system profile"),
            "system.override" => Some("noema system override '<json>'"),
            "system.reset" => Some("noema system reset"),
            _ => None,
        }
    }

    #[test]
    fn every_rpc_method_has_a_cli_path() {
        for method in METHODS {
            assert!(cli_path(method).is_some(), "no CLI path for {method}");
        }
    }

    #[test]
    fn every_gui_route_has_a_cli_command() {
        const ROUTES: &[(&str, &str)] = &[
            ("GET", "/api/health"),
            ("GET", "/api/anatomy"),
            ("POST", "/api/anima"),
            ("GET", "/api/system"),
            ("GET", "/api/system/refresh"),
            ("POST", "/api/system"),
            ("GET", "/api/brain"),
            ("GET", "/api/brain/telemetry"),
            ("POST", "/api/brain/telemetry"),
            ("POST", "/api/brain/drag"),
            ("GET", "/api/studio"),
            ("POST", "/api/studio"),
            ("POST", "/api/dream"),
            ("GET", "/api/system/path"),
            ("POST", "/api/system/path"),
            ("GET", "/api/system/update"),
            ("POST", "/api/system/update"),
        ];
        for (method, route) in ROUTES {
            assert!(
                HELP.contains(match *route {
                    "/api/health" => "health",
                    "/api/anatomy" => "anatomy",
                    "/api/anima" => "anima",
                    "/api/system" | "/api/system/refresh" => "system",
                    "/api/system/path" => "path",
                    "/api/system/update" => "update",
                    "/api/brain" => "brain",
                    "/api/brain/telemetry" => "telemetry",
                    "/api/brain/drag" => "drag",
                    "/api/studio" => "studio",
                    "/api/dream" => "dream",
                    _ => unreachable!(),
                }),
                "{method} {route} has no CLI command in HELP"
            );
        }
    }
}
