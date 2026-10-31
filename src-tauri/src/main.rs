//! NOEMA desktop shell: Tauri v2 window over the loopback Studio served by
//! `noema-app`. The wrapper never serves HTTP and never builds the UI; it opens
//! `http://127.0.0.1:NOEMA_APP_PORT` (default 4173) and exposes three commands.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

use serde_json::{Value, json};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

const DEFAULT_PORT: u16 = 4173;

struct Server(Mutex<Option<Child>>);

fn port() -> u16 {
    std::env::var("NOEMA_APP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

fn get_json(path: &str) -> Result<Value, String> {
    let port = port();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_millis(800)).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    )
    .map_err(|e| e.to_string())?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw).map_err(|e| e.to_string())?;
    let (head, body) = raw.split_once("\r\n\r\n").ok_or("malformed bridge response")?;
    if !head.starts_with("HTTP/1.1 200") {
        return Err(format!("bridge {path}: {}", head.lines().next().unwrap_or("no status")));
    }
    serde_json::from_str(body).map_err(|e| e.to_string())
}

fn healthy() -> bool {
    get_json("/api/health").is_ok()
}

/// The app binary sits next to the packaged shell (Tauri externalBin) unless
/// `NOEMA_APP_BIN` overrides it for development.
fn app_bin() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("NOEMA_APP_BIN") {
        return Some(PathBuf::from(path));
    }
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let name = if cfg!(windows) { "noema-app.exe" } else { "noema-app" };
    let bin = dir.join(name);
    bin.exists().then_some(bin)
}

fn spawn_app(app: &tauri::AppHandle) -> Option<Child> {
    let bin = app_bin()?;
    let mut cmd = Command::new(bin);
    cmd.args(["serve", "--port", &port().to_string()]);
    if let Ok(dir) = app.path().resource_dir() {
        let ui = dir.join("ui");
        let assets = dir.join("assets");
        if ui.join("index.html").exists() {
            cmd.arg("--ui").arg(ui);
        }
        if assets.exists() {
            cmd.arg("--assets").arg(assets);
        }
    }
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let child = cmd.spawn().ok()?;
    for _ in 0..60 {
        if healthy() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Some(child)
}

/// `{ok, version}` from `GET /api/health`.
#[tauri::command]
fn health() -> Result<Value, String> {
    get_json("/api/health")
}

/// `{port, url}` of the loopback Studio the window is bound to.
#[tauri::command]
fn bridge_info() -> Value {
    json!({ "port": port(), "url": format!("http://127.0.0.1:{}", port()) })
}

/// Last session object from `GET /api/studio` (`null` when none exists).
#[tauri::command]
fn session_resume() -> Result<Value, String> {
    let studio = get_json("/api/studio")?;
    Ok(studio.get("session").cloned().unwrap_or(Value::Null))
}

fn main() {
    let app = tauri::Builder::default()
        .manage(Server(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![health, bridge_info, session_resume])
        .setup(|app| {
            if !healthy() {
                if let Some(child) = spawn_app(app.handle()) {
                    *app.state::<Server>().0.lock().unwrap() = Some(child);
                }
            }
            let url = if healthy() {
                let url = format!("http://127.0.0.1:{}", port());
                WebviewUrl::External(url.parse().expect("loopback url"))
            } else {
                WebviewUrl::App("index.html".into())
            };
            WebviewWindowBuilder::new(app, "main", url)
                .title("NOEMA")
                .inner_size(1120.0, 700.0)
                .min_inner_size(900.0, 560.0)
                .build()?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build NOEMA desktop");

    app.run(|app, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            if let Some(mut child) = app.state::<Server>().0.lock().unwrap().take() {
                let _ = child.kill();
            }
        }
    });
}
