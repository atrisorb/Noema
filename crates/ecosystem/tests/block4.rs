//! Block 4 evidence: manifest validation, bundled installs, MCP lifecycle on a
//! real stdio server, WebSocket and SSE round-trips, git and pins.

use noema_ecosystem::{
    GitRepo, Kind, Manifest, Marketplace, McpConfig, McpSupervisor, Repo, RepoPin, ServerStatus,
    Transport, json_schema, parse_skill_md,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("noema-{tag}-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn node_available() -> bool {
    std::process::Command::new("node")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

#[test]
fn valid_mcp_manifest_round_trips() {
    let source = r#"{
        "name": "memory",
        "version": "1.0.0",
        "kind": "mcp",
        "description": "Semantic introspection",
        "transport": "stdio",
        "mcp": {"command": "node", "args": ["server.mjs"]}
    }"#;
    let manifest = Manifest::from_json(source).unwrap();
    assert_eq!(manifest.kind, Kind::Mcp);
    assert_eq!(Manifest::from_json(&manifest.to_json()).unwrap().name, "memory");
}

#[test]
fn validation_errors_name_the_field() {
    let bad_version = json!({
        "name": "memory", "version": "1.0", "kind": "mcp", "description": "x",
        "transport": "stdio", "mcp": {"command": "node"}
    });
    let err = Manifest::from_json(&bad_version.to_string()).unwrap_err();
    assert_eq!(err.path, "version");
    assert!(err.to_string().starts_with("version: "));

    let no_command = json!({
        "name": "memory", "version": "1.0.0", "kind": "mcp", "description": "x",
        "transport": "stdio", "mcp": {}
    });
    let err = Manifest::from_json(&no_command.to_string()).unwrap_err();
    assert_eq!(err.path, "mcp.command");

    let bad_url = json!({
        "name": "remote", "version": "1.0.0", "kind": "mcp", "description": "x",
        "transport": "sse", "mcp": {"url": "ftp://example.com"}
    });
    let err = Manifest::from_json(&bad_url.to_string()).unwrap_err();
    assert_eq!(err.path, "mcp.url");
}

#[test]
fn skill_frontmatter_is_claude_code_compatible() {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../marketplace/system-visual-qa/SKILL.md"),
    )
    .unwrap();
    let doc = parse_skill_md(&source).unwrap();
    assert_eq!(doc.manifest.name, "system-visual-qa");
    assert_eq!(doc.manifest.kind, Kind::Skill);
    assert_eq!(doc.manifest.skill.as_ref().unwrap().allowed_tools, vec!["Bash", "Read", "Write"]);
    assert!(doc.instructions.contains("screenshot path"));
}

#[test]
fn schemas_are_published_per_kind() {
    assert_eq!(json_schema(Kind::Mcp)["properties"]["kind"]["const"], "mcp");
    assert_eq!(json_schema(Kind::Plugin)["required"], json!(["name", "version", "kind", "description", "entry"]));
    assert_eq!(json_schema(Kind::Skill)["$schema"], "https://json-schema.org/draft/2020-12/schema");
}

#[test]
fn bundled_marketplace_installs_every_package() {
    let root = temp_dir("market");
    let market = Marketplace::new(&root);

    let memory = market.install("memory-mcp").unwrap();
    assert_eq!(memory.kind, Kind::Mcp);
    assert!(Path::new(&memory.path).join("server.mjs").exists());
    assert!(Manifest::from_json(&std::fs::read_to_string(Path::new(&memory.path).join("noema.json")).unwrap()).is_ok());

    let skill = market.install("system-visual-qa").unwrap();
    assert_eq!(skill.kind, Kind::Skill);
    assert!(Path::new(&skill.path).join("scripts/visual-qa.mjs").exists());

    let gh = market.install("github-plugin").unwrap();
    assert_eq!(gh.kind, Kind::Plugin);

    let installed = market.installed().unwrap();
    assert_eq!(installed.len(), 3);
    assert!(installed.iter().any(|item| item.name == "memory-mcp"));
}

#[test]
fn repo_json_and_pins_round_trip() {
    let repo: Repo = serde_json::from_str(
        r#"{"full_name":"acme/private-lab","private":true,"default_branch":"trunk","html_url":"https://github.com/acme/private-lab"}"#,
    )
    .unwrap();
    assert_eq!(repo.owner(), "acme");
    assert_eq!(repo.name(), "private-lab");
    assert!(repo.private);

    let dir = temp_dir("pin");
    let path = dir.join(".noema/pin.json");
    let mut pin = RepoPin::new("acme/private-lab", "trunk", dir.display().to_string());
    pin.pin_session("session-7");
    pin.save(&path).unwrap();
    let loaded = RepoPin::load(&path).unwrap();
    assert_eq!(loaded, pin);
    assert_eq!(loaded.session.as_deref(), Some("session-7"));
}

fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

#[test]
fn git_branch_checkout_and_diff() {
    let dir = temp_dir("git");
    git(&dir, &["init", "-b", "main"]);
    git(&dir, &["config", "user.email", "noema@example.com"]);
    git(&dir, &["config", "user.name", "NOEMA"]);
    std::fs::write(dir.join("part.rs"), "fn main() {}\n").unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-m", "init"]);

    let repo = GitRepo::open(&dir).unwrap();
    assert_eq!(repo.current_branch().unwrap(), "main");
    git(&dir, &["checkout", "-b", "anatomy"]);
    assert!(repo.branches().unwrap().contains(&"anatomy".to_string()));
    repo.checkout("main").unwrap();
    assert_eq!(repo.current_branch().unwrap(), "main");

    std::fs::write(dir.join("part.rs"), "fn main() { let _ = 1; }\n").unwrap();
    assert!(repo.diff(None).unwrap().contains("let _ = 1"));
    assert!(!repo.diff_stat().unwrap().is_empty());
}

#[tokio::test]
async fn stdio_mcp_lifecycle_on_bundled_server() {
    if !node_available() {
        eprintln!("skip: node not available");
        return;
    }
    let server = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../marketplace/memory-mcp/server.mjs");
    let mut cfg = McpConfig::stdio("memory", "node");
    cfg.args = vec![server.display().to_string()];
    cfg.env.insert("NOEMA_DB".into(), temp_dir("db").join("none.db").display().to_string());
    cfg.timeout = Duration::from_secs(10);

    let supervisor = McpSupervisor::new();
    supervisor.start(cfg).await.unwrap();
    let init = supervisor.initialize("memory").await.unwrap();
    assert_eq!(init["result"]["serverInfo"]["name"], "noema-memory");

    let tools = supervisor.list_tools("memory").await.unwrap();
    let names: Vec<&str> = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();
    assert!(names.contains(&"memory.search"));
    assert!(names.contains(&"memory.stats"));

    let response = supervisor.call_tool("memory", "memory.stats", json!({})).await.unwrap();
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let stats: Value = serde_json::from_str(text).unwrap();
    assert_eq!(stats["thoughts"], 0);

    assert_eq!(supervisor.status("memory").await.unwrap(), ServerStatus::Running);
    assert!(supervisor.logs("memory").await.unwrap().is_empty());
    supervisor.stop("memory").await.unwrap();
    assert!(supervisor.status("memory").await.is_err());
}

#[tokio::test]
async fn websocket_transport_round_trip() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        while let Some(Ok(message)) = ws.next().await {
            if let Message::Text(text) = message {
                let value: Value = serde_json::from_str(&text).unwrap();
                if let Some(id) = value.get("id") {
                    let reply = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {"protocolVersion": "2025-06-18", "serverInfo": {"name": "ws-test"}}
                    });
                    ws.send(Message::Text(reply.to_string().into())).await.unwrap();
                }
            }
        }
    });

    let cfg = McpConfig::remote("ws", Transport::WebSocket, format!("ws://{addr}"));
    let supervisor = McpSupervisor::new();
    supervisor.start(cfg).await.unwrap();
    let init = supervisor.initialize("ws").await.unwrap();
    assert_eq!(init["result"]["serverInfo"]["name"], "ws-test");
    supervisor.stop("ws").await.unwrap();
}

#[tokio::test]
async fn sse_transport_round_trip() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    async fn read_http(stream: &mut TcpStream) -> (String, String) {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            let read = stream.read(&mut chunk).await.unwrap();
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..read]);
            if let Some(header_end) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&buffer[..header_end]).to_string();
                let length = head
                    .lines()
                    .find_map(|line| {
                        let lower = line.to_ascii_lowercase();
                        lower
                            .strip_prefix("content-length:")
                            .and_then(|value| value.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                if buffer.len() >= header_end + 4 + length {
                    let body =
                        String::from_utf8_lossy(&buffer[header_end + 4..header_end + 4 + length]).to_string();
                    return (head, body);
                }
            }
        }
        (String::new(), String::new())
    }

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut sse, _) = listener.accept().await.unwrap();
        let _ = read_http(&mut sse).await;
        sse.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: keep-alive\r\n\r\n",
        )
        .await
        .unwrap();
        sse.write_all(b"event: endpoint\ndata: /messages\n\n").await.unwrap();
        sse.flush().await.unwrap();

        let (mut post, _) = listener.accept().await.unwrap();
        let (_, body) = read_http(&mut post).await;
        let request: Value = serde_json::from_str(&body).unwrap();
        let reply = json!({"jsonrpc": "2.0", "id": request["id"], "result": {"tools": []}});
        let body = reply.to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        post.write_all(response.as_bytes()).await.unwrap();
    });

    let cfg = McpConfig::remote("sse", Transport::Sse, format!("http://{addr}/sse"));
    let supervisor = McpSupervisor::new();
    supervisor.start(cfg).await.unwrap();
    let tools = supervisor.list_tools("sse").await.unwrap();
    assert_eq!(tools["result"]["tools"], json!([]));
    supervisor.stop("sse").await.unwrap();
}

#[test]
fn pkce_matches_rfc7636_vector() {
    assert_eq!(
        noema_ecosystem::mcp::pkce::challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
}
