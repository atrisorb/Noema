//! Loopback HTTP for the 3D world: static UI, anatomy JSON, Anima persistence.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use serde_json::{Value, json};

use crate::api::Api;

#[derive(Clone)]
pub struct Config {
    pub ui: PathBuf,
    pub assets: PathBuf,
    pub api: Arc<Api>,
}

pub fn serve(cfg: Config, port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("noema-app listening on http://127.0.0.1:{port}");
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let cfg = cfg.clone();
        thread::spawn(move || {
            if let Err(e) = handle(stream, cfg) {
                eprintln!("connection error: {e}");
            }
        });
    }
    Ok(())
}

fn handle(mut stream: TcpStream, cfg: Config) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let target = parts.next().unwrap_or("/").to_string();

    let mut length = 0usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            break;
        }
        if header == "\r\n" || header == "\n" {
            break;
        }
        if let Some(value) = header.to_ascii_lowercase().strip_prefix("content-length:") {
            length = value.trim().parse().unwrap_or(0);
        }
    }
    if length > 1_000_000 {
        return respond(&mut stream, 413, "application/json", br#"{"error":"body too large"}"#);
    }
    let mut body = vec![0u8; length];
    if length > 0 {
        reader.read_exact(&mut body)?;
    }

    let path = target.split('?').next().unwrap_or("/");
    match (method.as_str(), path) {
        ("GET", "/api/health") => {
            json_response(&mut stream, 200, json!({"ok": true, "version": noema_core::VERSION}))
        }
        ("GET", "/api/anatomy") => match cfg.api.anatomy() {
            Ok(payload) => json_response(&mut stream, 200, payload),
            Err(e) => json_response(&mut stream, 500, json!({"error": e})),
        },
        ("POST", "/api/anima") => {
            let parsed: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            match cfg.api.set_anima(&parsed) {
                Ok(payload) => json_response(&mut stream, 200, payload),
                Err(e) => json_response(&mut stream, 400, json!({"error": e})),
            }
        }
        ("GET", "/api/system") => match cfg.api.system(false) {
            Ok(payload) => json_response(&mut stream, 200, payload),
            Err(e) => json_response(&mut stream, 500, json!({ "error": e })),
        },
        ("GET", "/api/system/refresh") => match cfg.api.system(true) {
            Ok(payload) => json_response(&mut stream, 200, payload),
            Err(e) => json_response(&mut stream, 500, json!({ "error": e })),
        },
        ("POST", "/api/system") => {
            let parsed: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            match cfg.api.set_system_overrides(&parsed) {
                Ok(payload) => json_response(&mut stream, 200, payload),
                Err(e) => json_response(&mut stream, 400, json!({ "error": e })),
            }
        }
        ("GET", "/api/brain") => match cfg.api.brain() {
            Ok(payload) => json_response(&mut stream, 200, payload),
            Err(e) => json_response(&mut stream, 500, json!({ "error": e })),
        },
        ("GET", "/api/brain/telemetry") => match cfg.api.telemetry() {
            Ok(payload) => json_response(&mut stream, 200, payload),
            Err(e) => json_response(&mut stream, 500, json!({ "error": e })),
        },
        ("POST", "/api/brain/telemetry") => {
            let parsed: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            match cfg.api.ingest_telemetry(&parsed) {
                Ok(payload) => json_response(&mut stream, 200, payload),
                Err(e) => json_response(&mut stream, 400, json!({ "error": e })),
            }
        }
        ("POST", "/api/brain/drag") => {
            let parsed: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            match cfg.api.set_drag(&parsed) {
                Ok(payload) => json_response(&mut stream, 200, payload),
                Err(e) => json_response(&mut stream, 400, json!({ "error": e })),
            }
        }
        ("GET", "/api/studio") => match cfg.api.studio() {
            Ok(payload) => json_response(&mut stream, 200, payload),
            Err(e) => json_response(&mut stream, 500, json!({ "error": e })),
        },
        ("POST", "/api/studio") => {
            let parsed: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            match cfg.api.studio_update(&parsed) {
                Ok(payload) => json_response(&mut stream, 200, payload),
                Err(e) => json_response(&mut stream, 400, json!({ "error": e })),
            }
        }
        ("POST", "/api/dream") => {
            let parsed: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            match cfg.api.record_dream(&parsed) {
                Ok(payload) => json_response(&mut stream, 200, payload),
                Err(e) => json_response(&mut stream, 400, json!({ "error": e })),
            }
        }
        ("GET", _) => {
            let rel = path.trim_start_matches('/');
            let rel = if rel.is_empty() { "index.html" } else { rel };
            // UI bundle first, then the shared anatomy assets.
            let file = resolve(&cfg.ui, rel).or_else(|| {
                path.strip_prefix("/assets/")
                    .and_then(|rest| resolve(&cfg.assets, rest))
            });
            let Some(file) = file else {
                return respond(&mut stream, 404, "text/plain", b"not found");
            };
            let Ok(bytes) = std::fs::read(&file) else {
                return respond(&mut stream, 404, "text/plain", b"not found");
            };
            respond(&mut stream, 200, mime(&file), &bytes)
        }
        _ => respond(&mut stream, 405, "text/plain", b"method not allowed"),
    }
}

fn resolve(root: &Path, rel: &str) -> Option<PathBuf> {
    if rel.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        return None;
    }
    if rel.contains('\\') {
        return None;
    }
    let root = root.canonicalize().ok()?;
    let path = root.join(rel).canonicalize().ok()?;
    path.starts_with(&root).then_some(path)
}

fn mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "glb" => "model/gltf-binary",
        "gltf" => "model/gltf+json",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn json_response(stream: &mut TcpStream, status: u16, value: Value) -> std::io::Result<()> {
    respond(
        stream,
        status,
        "application/json",
        value.to_string().as_bytes(),
    )
}

fn respond(stream: &mut TcpStream, status: u16, mime: &str, body: &[u8]) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        _ => "Internal Server Error",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    stream.flush()
}
