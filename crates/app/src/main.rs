//! NOEMA desktop shell: serves the 3D world and its local bridge.
//! Tauri/WebView2 wraps this same server in Block 8 packaging.

mod api;
mod http;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::api::Api;
use crate::http::Config;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("serve");
    match command {
        "serve" => {
            if let Err(e) = serve(&args[1..]) {
                eprintln!("noema-app: {e}");
                std::process::exit(1);
            }
        }
        "help" | "--help" | "-h" => usage(),
        other => {
            eprintln!("noema-app: unknown command {other}");
            usage();
            std::process::exit(2);
        }
    }
}

fn usage() {
    println!(
        "noema-app {}\n\n  serve [--demo] [--port N] [--db PATH] [--ui DIR] [--assets DIR]\n",
        noema_core::VERSION
    );
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn serve(args: &[String]) -> Result<(), String> {
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.parent().and_then(|p| p.parent()).ok_or("bad manifest path")?.to_path_buf();
    let demo = args.iter().any(|a| a == "--demo");
    let ui = flag(args, "--ui").map(PathBuf::from).unwrap_or_else(|| {
        if let Some(ref d) = exe_dir {
            let candidate = d.join("ui");
            if candidate.join("index.html").exists() {
                return candidate;
            }
        }
        manifest.join("ui").join("dist")
    });
    let assets = flag(args, "--assets").map(PathBuf::from).unwrap_or_else(|| {
        if let Some(ref d) = exe_dir {
            let candidate = d.join("assets");
            if candidate.exists() {
                return candidate;
            }
        }
        workspace.join("assets")
    });
    let port: u16 = flag(args, "--port").and_then(|p| p.parse().ok()).unwrap_or(4173);
    let db = flag(args, "--db").map(PathBuf::from).unwrap_or_else(|| {
        if demo {
            workspace.join("target").join("block5").join("demo.db")
        } else {
            noema_core::daemon::data_dir().join("noema.db")
        }
    });
    if let Some(dir) = db.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    if demo {
        api::seed_demo(&db)?;
    } else if !db.exists() {
        noema_core::Store::open(&db).map_err(|e| e.to_string())?;
    }
    let conn = Connection::open(&db).map_err(|e| e.to_string())?;
    let api = Arc::new(Api { db: Arc::new(Mutex::new(conn)), live: Default::default() });
    let cfg = Config { ui, assets, api };
    println!("noema-app {} db={} demo={demo}", noema_core::VERSION, db.display());
    http::serve(cfg, port).map_err(|e| format!("bind 127.0.0.1:{port}: {e}"))
}
