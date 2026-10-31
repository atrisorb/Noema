//! MCP supervisor: one lifecycle for stdio, SSE and WebSocket servers.
//!
//! Secrets never reach logs: `Secret` redacts itself in `Debug`, argv vectors
//! are never string-interpolated, and every server keeps its own log ring.

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Url};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::time::{Instant, sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use crate::manifest::{Manifest, Transport};

const LOG_CAP: usize = 512;

#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

#[derive(Clone, Default)]
pub enum Auth {
    #[default]
    None,
    Bearer(Secret),
    OAuth2 {
        access_token: Secret,
    },
    Mtls {
        cert: PathBuf,
        key: PathBuf,
    },
    Headers(BTreeMap<String, String>),
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("None"),
            Self::Bearer(_) => f.write_str("Bearer(***)"),
            Self::OAuth2 { .. } => f.write_str("OAuth2(***)"),
            Self::Mtls { cert, .. } => f.debug_struct("Mtls").field("cert", cert).finish(),
            Self::Headers(h) => f.debug_tuple("Headers").field(&h.keys().collect::<Vec<_>>()).finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self { max_attempts: 3, backoff: Duration::from_millis(200) }
    }
}

#[derive(Debug, Clone)]
pub struct McpConfig {
    pub name: String,
    pub transport: Transport,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub cwd: Option<String>,
    pub url: Option<String>,
    pub headers: BTreeMap<String, String>,
    pub auth: Auth,
    pub timeout: Duration,
    pub retry: RetryPolicy,
}

impl McpConfig {
    pub fn stdio(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: Transport::Stdio,
            command: Some(command.into()),
            ..Self::defaults()
        }
    }

    pub fn remote(name: impl Into<String>, transport: Transport, url: impl Into<String>) -> Self {
        Self { name: name.into(), transport, url: Some(url.into()), ..Self::defaults() }
    }

    fn defaults() -> Self {
        Self {
            name: String::new(),
            transport: Transport::Stdio,
            command: None,
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
            url: None,
            headers: BTreeMap::new(),
            auth: Auth::None,
            timeout: Duration::from_secs(20),
            retry: RetryPolicy::default(),
        }
    }

    /// Runtime config from a validated MCP manifest.
    pub fn from_manifest(manifest: &Manifest) -> Result<Self, McpError> {
        let transport = manifest
            .transport
            .ok_or_else(|| McpError::Invalid("transport: required".into()))?;
        let spec = manifest.mcp.clone().unwrap_or_default();
        Ok(Self {
            name: manifest.name.clone(),
            transport,
            command: spec.command,
            args: spec.args,
            env: spec.env,
            cwd: spec.cwd,
            url: spec.url,
            headers: spec.headers,
            ..Self::defaults()
        })
    }
}

#[derive(Debug)]
pub enum McpError {
    NotFound(String),
    Invalid(String),
    Spawn(String),
    Io(std::io::Error),
    Timeout(String),
    Rpc { code: i64, message: String },
    Transport(String),
    Unsupported(String),
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(n) => write!(f, "mcp server '{n}' is not running"),
            Self::Invalid(m) => write!(f, "invalid mcp config: {m}"),
            Self::Spawn(m) => write!(f, "spawn failed: {m}"),
            Self::Io(e) => write!(f, "io: {e}"),
            Self::Timeout(m) => write!(f, "timeout: {m}"),
            Self::Rpc { code, message } => write!(f, "rpc {code}: {message}"),
            Self::Transport(m) => write!(f, "transport: {m}"),
            Self::Unsupported(m) => write!(f, "unsupported: {m}"),
        }
    }
}

impl std::error::Error for McpError {}

impl From<std::io::Error> for McpError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    Running,
    Stopped,
}

struct Entry {
    session: Session,
    auth: Auth,
    headers: BTreeMap<String, String>,
    timeout: Duration,
    retry: RetryPolicy,
    logs: Arc<Mutex<VecDeque<String>>>,
}

enum Session {
    Stdio {
        child: Box<Child>,
        stdin: ChildStdin,
        stdout: BufReader<ChildStdout>,
        next_id: u64,
    },
    Http {
        http: Client,
        endpoint: String,
        ws: Option<mpsc::UnboundedSender<String>>,
        pending: Arc<Mutex<HashMap<u64, Value>>>,
        next_id: u64,
    },
}

struct Server {
    entry: Mutex<Entry>,
}

/// Supervisor owns every MCP runtime. One entry per server name.
#[derive(Default)]
pub struct McpSupervisor {
    servers: Mutex<HashMap<String, Arc<Server>>>,
}

impl McpSupervisor {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(&self, cfg: McpConfig) -> Result<(), McpError> {
        let logs = Arc::new(Mutex::new(VecDeque::new()));
        let session = match cfg.transport {
            Transport::Stdio => spawn_stdio(&cfg, logs.clone())?,
            Transport::Sse => connect_sse(&cfg, logs.clone()).await?,
            Transport::WebSocket => connect_ws(&cfg, logs.clone()).await?,
        };
        let server = Arc::new(Server {
            entry: Mutex::new(Entry {
                session,
                auth: cfg.auth,
                headers: cfg.headers,
                timeout: cfg.timeout,
                retry: cfg.retry,
                logs,
            }),
        });
        let mut servers = self.servers.lock().await;
        if let Some(old) = servers.insert(cfg.name.clone(), server) {
            stop_server(&old).await.ok();
        }
        Ok(())
    }

    pub async fn stop(&self, name: &str) -> Result<(), McpError> {
        let server = self.servers.lock().await.remove(name);
        match server {
            Some(server) => stop_server(&server).await,
            None => Err(McpError::NotFound(name.into())),
        }
    }

    pub async fn status(&self, name: &str) -> Result<ServerStatus, McpError> {
        let server = self.get(name).await?;
        let mut entry = server.entry.lock().await;
        Ok(match &mut entry.session {
            Session::Stdio { child, .. } => match child.try_wait() {
                Ok(Some(_)) => ServerStatus::Stopped,
                _ => ServerStatus::Running,
            },
            Session::Http { .. } => ServerStatus::Running,
        })
    }

    pub async fn logs(&self, name: &str) -> Result<Vec<String>, McpError> {
        let server = self.get(name).await?;
        let logs = server.entry.lock().await.logs.clone();
        Ok(logs.lock().await.iter().cloned().collect())
    }

    /// JSON-RPC call with timeout and retry. Retries transport failures only.
    pub async fn request(&self, name: &str, method: &str, params: Value) -> Result<Value, McpError> {
        let server = self.get(name).await?;
        let mut entry = server.entry.lock().await;
        let retry = entry.retry.clone();
        let dur = entry.timeout;
        let logs = entry.logs.clone();
        let auth = entry.auth.clone();
        let headers = entry.headers.clone();

        let mut attempt = 0;
        loop {
            let outcome = match &mut entry.session {
                Session::Stdio { stdin, stdout, next_id, .. } => {
                    stdio_request(stdin, stdout, next_id, method, &params, dur, &logs).await
                }
                Session::Http { http, endpoint, ws, pending, next_id } => {
                    http_request(http, endpoint, ws, pending, next_id, method, &params, dur, &auth, &headers, &logs)
                        .await
                }
            };
            match outcome {
                Ok(value) => return Ok(value),
                Err(McpError::Rpc { .. }) => return outcome,
                Err(err) if attempt < retry.max_attempts => {
                    let pause = retry.backoff * 2u32.pow(attempt);
                    push_log(&logs, format!("{name}: retry {attempt} after {err}")).await;
                    attempt += 1;
                    sleep(pause).await;
                }
                Err(err) => return Err(err),
            }
        }
    }

    /// Standard MCP handshake: initialize, initialized notification, tools/list.
    pub async fn initialize(&self, name: &str) -> Result<Value, McpError> {
        let info = self
            .request(
                name,
                "initialize",
                json!({
                    "protocolVersion": "2025-06-18",
                    "capabilities": {},
                    "clientInfo": {"name": "noema", "version": env!("CARGO_PKG_VERSION")}
                }),
            )
            .await?;
        let _ = self.notify(name, "notifications/initialized", json!({})).await;
        Ok(info)
    }

    pub async fn list_tools(&self, name: &str) -> Result<Value, McpError> {
        self.request(name, "tools/list", json!({})).await
    }

    pub async fn call_tool(&self, name: &str, tool: &str, arguments: Value) -> Result<Value, McpError> {
        self.request(name, "tools/call", json!({"name": tool, "arguments": arguments})).await
    }

    async fn notify(&self, name: &str, method: &str, params: Value) -> Result<(), McpError> {
        let server = self.get(name).await?;
        let mut entry = server.entry.lock().await;
        let frame = json!({"jsonrpc": "2.0", "method": method, "params": params}).to_string();
        match &mut entry.session {
            Session::Stdio { stdin, .. } => {
                stdin.write_all(frame.as_bytes()).await?;
                stdin.write_all(b"\n").await?;
                stdin.flush().await?;
            }
            Session::Http { ws: Some(tx), .. } => {
                tx.send(frame).map_err(|_| McpError::Transport("websocket closed".into()))?;
            }
            Session::Http { .. } => {
                return Err(McpError::Unsupported("notifications need a websocket session".into()));
            }
        }
        Ok(())
    }

    async fn get(&self, name: &str) -> Result<Arc<Server>, McpError> {
        self.servers
            .lock()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| McpError::NotFound(name.into()))
    }
}

fn spawn_stdio(cfg: &McpConfig, logs: Arc<Mutex<VecDeque<String>>>) -> Result<Session, McpError> {
    let command = cfg
        .command
        .as_deref()
        .filter(|c| !c.trim().is_empty())
        .ok_or_else(|| McpError::Invalid("mcp.command: stdio requires a command".into()))?;
    let mut cmd = Command::new(command);
    cmd.args(&cfg.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let Some(cwd) = &cfg.cwd {
        cmd.current_dir(cwd);
    }
    for (key, value) in &cfg.env {
        cmd.env(key, value);
    }
    let mut child = cmd.spawn().map_err(|e| McpError::Spawn(format!("{command}: {e}")))?;
    let stdin = child.stdin.take().ok_or_else(|| McpError::Spawn("stdin unavailable".into()))?;
    let stdout = child.stdout.take().ok_or_else(|| McpError::Spawn("stdout unavailable".into()))?;
    if let Some(stderr) = child.stderr.take() {
        let logs = logs.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                push_log(&logs, line).await;
            }
        });
    }
    Ok(Session::Stdio { child: Box::new(child), stdin, stdout: BufReader::new(stdout), next_id: 1 })
}

async fn stop_server(server: &Server) -> Result<(), McpError> {
    let mut entry = server.entry.lock().await;
    if let Session::Stdio { child, .. } = &mut entry.session {
        child.start_kill().ok();
        let _ = timeout(Duration::from_secs(2), child.wait()).await;
    }
    Ok(())
}

async fn stdio_request(
    stdin: &mut ChildStdin,
    stdout: &mut BufReader<ChildStdout>,
    next_id: &mut u64,
    method: &str,
    params: &Value,
    dur: Duration,
    logs: &Arc<Mutex<VecDeque<String>>>,
) -> Result<Value, McpError> {
    let id = *next_id;
    *next_id += 1;
    let frame = format!("{}\n", json_rpc_frame(id, method, params));

    let exchange = async {
        stdin.write_all(frame.as_bytes()).await?;
        stdin.flush().await?;
        socket_request(stdout, id, logs).await
    };
    timeout(dur, exchange)
        .await
        .map_err(|_| McpError::Timeout(format!("{method} (id {id})")))?
}

async fn socket_request(
    stdout: &mut BufReader<ChildStdout>,
    id: u64,
    logs: &Arc<Mutex<VecDeque<String>>>,
) -> Result<Value, McpError> {
    let mut incoming = String::new();
    loop {
        incoming.clear();
        let read = stdout.read_line(&mut incoming).await?;
        if read == 0 {
            return Err(McpError::Transport("server closed stdout".into()));
        }
        let value: Value = match serde_json::from_str(incoming.trim()) {
            Ok(value) => value,
            Err(_) => {
                push_log(logs, incoming.trim()).await;
                continue;
            }
        };
        if value.get("id").and_then(Value::as_u64) == Some(id) {
            return response_payload(value);
        }
        if let Some(method) = value.get("method").and_then(Value::as_str) {
            push_log(logs, format!("notification {method}")).await;
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn http_request(
    http: &Client,
    endpoint: &str,
    ws: &Option<mpsc::UnboundedSender<String>>,
    pending: &Arc<Mutex<HashMap<u64, Value>>>,
    next_id: &mut u64,
    method: &str,
    params: &Value,
    dur: Duration,
    auth: &Auth,
    headers: &BTreeMap<String, String>,
    logs: &Arc<Mutex<VecDeque<String>>>,
) -> Result<Value, McpError> {
    let id = *next_id;
    *next_id += 1;
    let frame = json_rpc_frame(id, method, params);

    if let Some(tx) = ws {
        tx.send(frame.to_string())
            .map_err(|_| McpError::Transport("websocket closed".into()))?;
        return wait_pending(pending, id, dur, method).await;
    }

    let mut req = http
        .post(endpoint)
        .header(CONTENT_TYPE, "application/json")
        .json(&frame);
    req = apply_headers(req, auth, headers)?;
    let resp = timeout(dur, req.send())
        .await
        .map_err(|_| McpError::Timeout(format!("{method} (id {id})")))?
        .map_err(|e| McpError::Transport(e.to_string()))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(McpError::Transport(format!("{method}: HTTP {status}")));
    }
    let body = resp.text().await.map_err(|e| McpError::Transport(e.to_string()))?;
    if let Ok(value) = serde_json::from_str::<Value>(&body)
        && value.get("id").and_then(Value::as_u64) == Some(id)
    {
        return response_payload(value);
    }
    push_log(logs, format!("{method}: awaiting SSE reply")).await;
    wait_pending(pending, id, dur, method).await
}

async fn wait_pending(
    pending: &Arc<Mutex<HashMap<u64, Value>>>,
    id: u64,
    dur: Duration,
    method: &str,
) -> Result<Value, McpError> {
    let deadline = Instant::now() + dur;
    loop {
        if let Some(value) = pending.lock().await.remove(&id) {
            return response_payload(value);
        }
        if Instant::now() >= deadline {
            return Err(McpError::Timeout(format!("{method} (id {id})")));
        }
        sleep(Duration::from_millis(10)).await;
    }
}

fn response_payload(value: Value) -> Result<Value, McpError> {
    if let Some(error) = value.get("error") {
        return Err(McpError::Rpc {
            code: error.get("code").and_then(Value::as_i64).unwrap_or(-32000),
            message: error.get("message").and_then(Value::as_str).unwrap_or("unknown").into(),
        });
    }
    Ok(value)
}

async fn connect_sse(cfg: &McpConfig, logs: Arc<Mutex<VecDeque<String>>>) -> Result<Session, McpError> {
    let url = cfg.url.as_deref().ok_or_else(|| McpError::Invalid("mcp.url: required".into()))?;
    let http = http_client(&cfg.auth)?;
    let mut req = http.get(url).header(ACCEPT, "text/event-stream");
    req = apply_headers(req, &cfg.auth, &cfg.headers)?;
    let resp = timeout(cfg.timeout, req.send())
        .await
        .map_err(|_| McpError::Timeout("sse connect".into()))?
        .map_err(|e| McpError::Transport(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(McpError::Transport(format!("sse connect: HTTP {}", resp.status())));
    }

    let base = Url::parse(url).map_err(|e| McpError::Invalid(format!("mcp.url: {e}")))?;
    let pending = Arc::new(Mutex::new(HashMap::new()));
    let (endpoint_tx, endpoint_rx) = oneshot::channel::<String>();
    let reader_pending = pending.clone();
    let reader_logs = logs.clone();
    tokio::spawn(async move {
        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut endpoint_tx = Some(endpoint_tx);
        while let Some(chunk) = stream.next().await {
            let Ok(bytes) = chunk else { break };
            buffer.push_str(&String::from_utf8_lossy(&bytes));
            while let Some((event, data)) = take_event(&mut buffer) {
                if event == "endpoint" {
                    if let Some(tx) = endpoint_tx.take() {
                        let _ = tx.send(data);
                    }
                } else if event == "message"
                    && let Ok(value) = serde_json::from_str::<Value>(&data)
                    && let Some(id) = value.get("id").and_then(Value::as_u64)
                {
                    reader_pending.lock().await.insert(id, value);
                }
            }
        }
        push_log(&reader_logs, "sse stream closed").await;
    });

    let endpoint = timeout(cfg.timeout, endpoint_rx)
        .await
        .map_err(|_| McpError::Timeout("sse endpoint event".into()))?
        .map_err(|_| McpError::Transport("sse closed before endpoint event".into()))?;
    let endpoint = match Url::parse(&endpoint) {
        Ok(_) => endpoint,
        Err(_) => base
            .join(&endpoint)
            .map_err(|e| McpError::Invalid(format!("sse endpoint: {e}")))?
            .to_string(),
    };

    Ok(Session::Http { http, endpoint, ws: None, pending, next_id: 1 })
}

async fn connect_ws(cfg: &McpConfig, logs: Arc<Mutex<VecDeque<String>>>) -> Result<Session, McpError> {
    let url = cfg.url.as_deref().ok_or_else(|| McpError::Invalid("mcp.url: required".into()))?;
    if matches!(cfg.auth, Auth::Mtls { .. }) {
        return Err(McpError::Unsupported("mTLS is only wired for SSE/HTTP clients".into()));
    }
    let url = if let Some(rest) = url.strip_prefix("http://") {
        format!("ws://{rest}")
    } else if let Some(rest) = url.strip_prefix("https://") {
        format!("wss://{rest}")
    } else {
        url.to_string()
    };

    let mut request = url
        .clone()
        .into_client_request()
        .map_err(|e| McpError::Invalid(format!("mcp.url: {e}")))?;
    let mut headers = cfg.headers.clone();
    if let Auth::Bearer(secret) = &cfg.auth {
        headers.insert("Authorization".into(), format!("Bearer {}", secret.expose()));
    }
    for (key, value) in headers {
        let name: tokio_tungstenite::tungstenite::http::HeaderName = key
            .parse()
            .map_err(|_| McpError::Invalid(format!("header name {key}")))?;
        let value: tokio_tungstenite::tungstenite::http::HeaderValue = value
            .parse()
            .map_err(|_| McpError::Invalid(format!("header value for {key}")))?;
        request.headers_mut().insert(name, value);
    }

    let (stream, _) = timeout(cfg.timeout, tokio_tungstenite::connect_async(request))
        .await
        .map_err(|_| McpError::Timeout("websocket connect".into()))?
        .map_err(|e| McpError::Transport(e.to_string()))?;
    let (mut sink, mut source) = stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let pending = Arc::new(Mutex::new(HashMap::new()));
    let reader_pending = pending.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(text) = rx.recv() => {
                    if sink.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                message = source.next() => match message {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(value) = serde_json::from_str::<Value>(&text)
                            && let Some(id) = value.get("id").and_then(Value::as_u64)
                        {
                            reader_pending.lock().await.insert(id, value);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
        push_log(&logs, "websocket closed").await;
    });

    Ok(Session::Http { http: http_client(&Auth::None)?, endpoint: url, ws: Some(tx), pending, next_id: 1 })
}

fn take_event(buffer: &mut String) -> Option<(String, String)> {
    let index = buffer.find("\n\n")?;
    let frame: String = buffer.drain(..index + 2).collect();
    let mut event = "message".to_string();
    let mut data = String::new();
    for line in frame.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(value) = line.strip_prefix("event:") {
            event = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(value.trim_start());
        }
    }
    Some((event, data))
}

fn http_client(auth: &Auth) -> Result<Client, McpError> {
    let mut builder = Client::builder().connect_timeout(Duration::from_secs(10));
    if let Auth::Mtls { cert, key } = auth {
        let mut pem = std::fs::read(cert).map_err(McpError::Io)?;
        pem.extend(std::fs::read(key).map_err(McpError::Io)?);
        let identity = reqwest::Identity::from_pem(&pem)
            .map_err(|e| McpError::Invalid(format!("mTLS identity: {e}")))?;
        builder = builder.identity(identity);
    }
    builder.build().map_err(|e| McpError::Transport(e.to_string()))
}

fn apply_headers(
    mut req: reqwest::RequestBuilder,
    auth: &Auth,
    headers: &BTreeMap<String, String>,
) -> Result<reqwest::RequestBuilder, McpError> {
    for (key, value) in headers {
        req = req.header(key, value);
    }
    req = match auth {
        Auth::Bearer(secret) => req.header(AUTHORIZATION, format!("Bearer {}", secret.expose())),
        Auth::OAuth2 { access_token } => {
            req.header(AUTHORIZATION, format!("Bearer {}", access_token.expose()))
        }
        Auth::Headers(extra) => {
            let mut req = req;
            for (key, value) in extra {
                req = req.header(key, value);
            }
            req
        }
        Auth::None | Auth::Mtls { .. } => req,
    };
    Ok(req)
}

fn json_rpc_frame(id: u64, method: &str, params: &Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
}

async fn push_log(logs: &Arc<Mutex<VecDeque<String>>>, line: impl Into<String>) {
    let mut logs = logs.lock().await;
    if logs.len() == LOG_CAP {
        logs.pop_front();
    }
    logs.push_back(line.into());
}

/// RFC 7636 PKCE helpers. Verifier is 64 hex chars (43-128 allowed).
pub mod pkce {
    use super::*;
    use sha2::Digest;

    pub fn verifier() -> String {
        format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        )
    }

    pub fn challenge(verifier: &str) -> String {
        let digest = sha2::Sha256::digest(verifier.as_bytes());
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
    }
}
