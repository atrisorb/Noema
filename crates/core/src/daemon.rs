//! `noemad` server: loopback NDJSON JSON-RPC, token auth, port file lifecycle.

use std::io::Write;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::broadcast::error::RecvError;

use crate::VERSION;
use crate::event::{self, Bus, Event};
use crate::rpc::{self, Request, Response};
use crate::store::Store;

/// Hard cap for one NDJSON frame, enforced while reading (ARCHITECTURE 3).
pub const MAX_FRAME: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortFile {
    pub port: u16,
    pub token: String,
    pub pid: u32,
    pub version: String,
}

/// Per-user data root; see [`crate::platform::data_dir`].
pub fn data_dir() -> PathBuf {
    crate::platform::data_dir()
}

pub fn port_file(dir: &Path) -> PathBuf {
    dir.join("daemon.json")
}

pub fn read_port_file(dir: &Path) -> Option<PortFile> {
    let body = std::fs::read_to_string(port_file(dir)).ok()?;
    serde_json::from_str(&body).ok()
}

/// 32 random bytes as hex: two v4 uuids, no extra RNG dependency.
pub fn new_token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

/// Constant-time comparison for equal-length tokens.
pub fn token_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn write_port_file(dir: &Path, pf: &PortFile) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = port_file(dir);
    let body = serde_json::to_vec_pretty(pf).map_err(std::io::Error::other)?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    file.write_all(&body)?;
    file.sync_all()?;
    #[cfg(windows)]
    harden_port_file(&path);
    Ok(())
}

/// Best-effort owner-only ACL on Windows: drop inherited ACEs, grant the current
/// user full control. icacls ships with Windows.
#[cfg(windows)]
fn harden_port_file(path: &Path) {
    use std::process::Command;
    let user = std::env::var("USERNAME").unwrap_or_default();
    if user.is_empty() {
        tracing::warn!("daemon.json ACL: USERNAME unset, hardening skipped");
        return;
    }
    let grant = format!("{user}:F");
    match Command::new("icacls")
        .arg(path)
        .args(["/inheritance:r", "/grant:r"])
        .arg(&grant)
        .output()
    {
        Ok(out) if out.status.success() => {}
        Ok(out) => tracing::warn!(
            status = ?out.status,
            stderr = %String::from_utf8_lossy(&out.stderr).trim(),
            "daemon.json ACL hardening failed"
        ),
        Err(e) => tracing::warn!(%e, "icacls unavailable, daemon.json ACL hardening skipped"),
    }
}

pub async fn run(db_path: PathBuf) -> std::io::Result<()> {
    let dir = data_dir();
    let store = Arc::new(Mutex::new(
        Store::open(&db_path).map_err(std::io::Error::other)?,
    ));
    let bus = event::bus();
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
    let addr = listener.local_addr()?;
    let port = PortFile {
        port: addr.port(),
        token: new_token(),
        pid: std::process::id(),
        version: VERSION.into(),
    };
    write_port_file(&dir, &port)?;
    tracing::info!(port = addr.port(), db = %db_path.display(), backend = ?store.lock().unwrap().vector_backend(), "noemad listening");

    let started = Instant::now();
    let heartbeat = bus.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            let _ = heartbeat.send(Event::new(
                "event.health",
                json!({ "pid": std::process::id(), "uptime_s": started.elapsed().as_secs() }),
            ));
        }
    });

    let auth = Arc::new(port.token.clone());
    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _) = accepted?;
                let (store, bus, auth) = (store.clone(), bus.clone(), auth.clone());
                tokio::spawn(async move {
                    if let Err(e) = serve(stream, store, bus, auth, started).await {
                        tracing::debug!(%e, "connection closed");
                    }
                });
            }
            _ = tokio::signal::ctrl_c() => break,
        }
    }

    let _ = std::fs::remove_file(port_file(&dir));
    Ok(())
}

async fn serve(
    stream: tokio::net::TcpStream,
    store: Arc<Mutex<Store>>,
    bus: Bus,
    auth: Arc<String>,
    started: Instant,
) -> std::io::Result<()> {
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut out = BufWriter::new(writer);
    let mut events = bus.subscribe();

    // Liveness frame: the stream can be proven before any bus event exists.
    let health = Event::new(
        "event.health",
        json!({ "pid": std::process::id(), "uptime_s": started.elapsed().as_secs() }),
    );
    out.write_all(event::frame(&health).as_bytes()).await?;
    out.flush().await?;

    loop {
        tokio::select! {
            frame = read_frame(&mut reader, MAX_FRAME) => {
                let line = match frame {
                    Ok(Some(bytes)) => String::from_utf8_lossy(&bytes).into_owned(),
                    Ok(None) => return Ok(()),
                    Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                        out.write_all(Response::err(None, rpc::PARSE_CODE, e.to_string()).to_line().as_bytes()).await?;
                        out.flush().await?;
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                };
                if line.trim().is_empty() {
                    continue;
                }
                let response = match serde_json::from_str::<Request>(&line) {
                    Ok(req) => {
                        let authorized = req
                            .params
                            .get("_auth")
                            .and_then(|v| v.as_str())
                            .is_some_and(|t| token_eq(t, &auth));
                        if authorized {
                            match store.lock() {
                                Ok(mut guard) => rpc::dispatch(&mut guard, &bus, &req),
                                Err(_) => Response::err(req.id.clone(), rpc::INTERNAL_CODE, "store poisoned"),
                            }
                        } else {
                            Response::err(req.id.clone(), rpc::AUTH_CODE, "auth")
                        }
                    }
                    Err(e) => Response::err(None, rpc::PARSE_CODE, format!("parse: {e}")),
                };
                out.write_all(response.to_line().as_bytes()).await?;
                out.flush().await?;
            }
            event = events.recv() => {
                match event {
                    Ok(event) => {
                        out.write_all(event::frame(&event).as_bytes()).await?;
                        out.flush().await?;
                    }
                    Err(RecvError::Lagged(skipped)) => {
                        tracing::debug!(skipped, "event stream lagged");
                    }
                    Err(RecvError::Closed) => return Ok(()),
                }
            }
        }
    }
}

/// Read one newline-terminated frame without buffering more than `max` bytes.
/// Oversized input aborts with `InvalidData`; the caller answers and hangs up.
async fn read_frame<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    max: usize,
) -> std::io::Result<Option<Vec<u8>>> {
    let mut frame = Vec::new();
    loop {
        let buf = reader.fill_buf().await?;
        if buf.is_empty() {
            return Ok((!frame.is_empty()).then_some(frame));
        }
        let newline = buf.iter().position(|b| *b == b'\n');
        let take = newline.unwrap_or(buf.len());
        frame.extend_from_slice(&buf[..take]);
        reader.consume(take + usize::from(newline.is_some()));
        if newline.is_some() {
            return Ok(Some(frame));
        }
        if frame.len() > max {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("frame exceeds {max} bytes"),
            ));
        }
    }
}
