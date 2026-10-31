//! Daemon integration: token auth, request/response, health stream, frame cap.

use std::path::Path;
use std::time::Duration;

use noema_core::VERSION;
use noema_core::daemon;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

struct Conn {
    out: OwnedWriteHalf,
    lines: tokio::io::Lines<BufReader<OwnedReadHalf>>,
}

impl Conn {
    async fn connect(port: u16) -> Self {
        let (reader, out) = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("connect to noemad")
            .into_split();
        Self {
            out,
            lines: BufReader::new(reader).lines(),
        }
    }

    async fn send(&mut self, value: Value) {
        self.out
            .write_all(format!("{value}\n").as_bytes())
            .await
            .expect("write frame");
    }

    async fn recv(&mut self) -> Option<Value> {
        let line = tokio::time::timeout(Duration::from_secs(5), self.lines.next_line())
            .await
            .expect("frame within 5s")
            .expect("socket readable")?;
        Some(serde_json::from_str(&line).expect("ndjson frame"))
    }

    /// Read until the reply with `id` arrives, returning it plus any events seen first.
    async fn reply(&mut self, id: i64) -> (Value, Vec<Value>) {
        let mut events = Vec::new();
        for _ in 0..10 {
            let frame = self.recv().await.expect("reply before eof");
            if frame.get("id").and_then(Value::as_i64) == Some(id) {
                return (frame, events);
            }
            events.push(frame);
        }
        panic!("no reply for id {id}");
    }
}

async fn wait_port(dir: &Path) -> daemon::PortFile {
    for _ in 0..100 {
        if let Some(port) = daemon::read_port_file(dir) {
            return port;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("daemon did not publish daemon.json");
}

#[tokio::test]
async fn auth_ping_validation_events_and_frame_cap() {
    let dir = std::env::temp_dir().join(format!("noema-daemon-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    // Safety: one test per binary, so this process-global var has a single writer.
    unsafe { std::env::set_var("NOEMA_DATA_DIR", &dir) };

    let db = dir.join("noema.db");
    let server = tokio::spawn(async move { daemon::run(db).await });
    let port = wait_port(&dir).await;
    let mut conn = Conn::connect(port.port).await;

    let health = conn.recv().await.expect("health frame");
    assert_eq!(health["method"], "event.health");
    assert_eq!(
        health["params"]["pid"].as_u64(),
        Some(u64::from(std::process::id()))
    );

    conn.send(json!({"jsonrpc":"2.0","id":1,"method":"daemon.ping","params":{}}))
        .await;
    let (denied, _) = conn.reply(1).await;
    assert_eq!(denied["error"]["code"], -32001);

    conn.send(json!({"jsonrpc":"2.0","id":2,"method":"daemon.ping",
        "params":{"_auth": port.token}}))
        .await;
    let (pong, _) = conn.reply(2).await;
    assert_eq!(pong["result"]["version"], VERSION);

    let cwd = dir.to_string_lossy().to_string();
    let session_id = uuid::Uuid::new_v4().to_string();
    conn.send(
        json!({"jsonrpc":"2.0","id":3,"method":"session.open","params":{
        "_auth": port.token,
        "session":{"id":"not-a-uuid","agent":"agent:test","model":"test/model","cwd":cwd,"updated_at":0}}}),
    )
    .await;
    let (bad_id, _) = conn.reply(3).await;
    assert_eq!(bad_id["error"]["code"], -32602);
    assert!(
        bad_id["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("uuid")
    );

    conn.send(
        json!({"jsonrpc":"2.0","id":4,"method":"session.open","params":{
        "_auth": port.token,
        "session":{"id":session_id,"project":"missing-project","agent":"agent:test",
        "model":"test/model","cwd":cwd,"updated_at":0}}}),
    )
    .await;
    let (bad_project, _) = conn.reply(4).await;
    assert_eq!(bad_project["error"]["code"], -32602);
    assert!(
        bad_project["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("project_id")
    );

    conn.send(
        json!({"jsonrpc":"2.0","id":5,"method":"session.open","params":{
        "_auth": port.token,
        "session":{"id":session_id,"agent":"agent:test","model":"test/model",
        "cwd":cwd,"branch":"main","updated_at":0}}}),
    )
    .await;
    let (opened, events) = conn.reply(5).await;
    assert_eq!(opened["result"]["session"]["id"], session_id);
    let updated = events
        .iter()
        .chain(std::iter::once(&opened))
        .any(|f| f["method"] == "event.session.updated");
    if !updated {
        let event = conn.recv().await.expect("event frame");
        assert_eq!(event["method"], "event.session.updated");
    }

    // One byte over the cap: the daemon must answer -32700 and hang up.
    let oversized = vec![b'x'; daemon::MAX_FRAME + 1];
    let _ = conn.out.write_all(&oversized).await;
    let _ = conn.out.write_all(b"\n").await;
    // EOF is acceptable too: a reset mid-write still means the cap rejected the frame.
    if let Some(frame) = conn.recv().await {
        assert_eq!(frame["error"]["code"], -32700);
    }

    server.abort();
    let _ = std::fs::remove_dir_all(&dir);
}
