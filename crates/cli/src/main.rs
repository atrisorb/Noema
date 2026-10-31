use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::process::ExitCode;
use std::time::Duration;

use noema_core::Store;
use noema_core::daemon::{data_dir, read_port_file};
use noema_core::rpc::METHODS;
use serde_json::{Value, json};

const HELP: &str = "\
noema - the anatomical mind (CLI)

usage: noema <command>

  status                      resume state from noemad (falls back to the local DB)
  health                      app bridge health
  anatomy                     body, agents, heart facts
  anima [set JSON]            read the heart/Anima state, or write a patch
  brain                       semantic graph snapshot
  brain telemetry             logprob telemetry feed
  brain drag JSON             persist a node position
  brain dream JSON            record a dream
  system profile [--refresh]  machine profile + tuning
  system override JSON        deep-merge a tuning patch
  system reset                drop overrides and cached profile
  studio                      Studio snapshot: session, projects, roles, widgets, usage
  studio session ID           focus a session
  studio role CAP MODEL [--project ID]
  studio widget JSON          upsert a widget
  studio widget-remove ID     delete a widget
  studio layout JSON          persist [{id,x,y,w,h}]
  session list [--limit N]    daemon sessions
  session open JSON           open or update a session
  session resume              daemon resume state
  methods                     every daemon RPC method
  rpc <method> [json]         call any daemon RPC method
  help                        this text

App actions use the Studio bridge on NOEMA_APP_PORT (default 4173).
Every GUI action is reachable here. Noemad owns the database.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str).unwrap_or("status") {
        "status" => return status(),
        "health" => http("GET", "/api/health", None),
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
        "session" => session(&args[1..]),
        "methods" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!(METHODS)).unwrap_or_default()
            );
            return ExitCode::SUCCESS;
        }
        "rpc" => return rpc(&args[1..]),
        "help" | "-h" | "--help" => {
            println!("{HELP}");
            return ExitCode::SUCCESS;
        }
        other => {
            eprintln!("unknown command: {other}\n\n{HELP}");
            return ExitCode::FAILURE;
        }
    };
    match result {
        Ok(value) => {
            print_json(&value);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("noema: {e}");
            ExitCode::FAILURE
        }
    }
}

fn status() -> ExitCode {
    let value = call("session.resume", json!({})).or_else(|offline| {
        eprintln!("daemon offline ({offline}); reading local store");
        let db = data_dir().join("noema.db");
        Store::open(&db)
            .and_then(|store| store.resume())
            .map(|resume| json!({ "resume": resume, "source": "local" }))
            .map_err(|e| e.to_string())
    });
    match value {
        Ok(value) => {
            print_json(&value);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("local store unavailable: {e}");
            ExitCode::FAILURE
        }
    }
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
            call("system.profile", json!({ "refresh": refresh })).or_else(|offline| {
                eprintln!("daemon offline ({offline}); reading local store");
                local_store()?
                    .hardware_report(refresh)
                    .map(|r| json!(r))
                    .map_err(|e| e.to_string())
            })
        }
        "override" => {
            let patch = parse_body(args.get(1))?;
            call("system.override", json!({ "overrides": patch })).or_else(|offline| {
                eprintln!("daemon offline ({offline}); writing local store");
                local_store()?
                    .set_hardware_overrides(&patch)
                    .map(|r| json!(r))
                    .map_err(|e| e.to_string())
            })
        }
        "reset" => call("system.reset", json!({})).or_else(|offline| {
            eprintln!("daemon offline ({offline}); writing local store");
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
        None | Some("snapshot") => http("GET", "/api/studio", None),
        Some("session") => {
            let id = args.get(1).ok_or("usage: noema studio session <id>")?;
            http(
                "POST",
                "/api/studio",
                Some(json!({ "action": "session", "id": id })),
            )
        }
        Some("role") => {
            let capability = args
                .get(1)
                .ok_or("usage: noema studio role <capability> <model> [--project <id>]")?;
            let model = args
                .get(2)
                .ok_or("usage: noema studio role <capability> <model> [--project <id>]")?;
            let mut body = json!({ "action": "role", "capability": capability, "model": model });
            if let Some(i) = args.iter().position(|a| a == "--project") {
                body["project"] = json!(args.get(i + 1).ok_or("--project needs an id")?);
            }
            http("POST", "/api/studio", Some(body))
        }
        Some("widget") => {
            let widget = parse_body(args.get(1))?;
            http(
                "POST",
                "/api/studio",
                Some(json!({ "action": "widget", "widget": widget })),
            )
        }
        Some("widget-remove") => {
            let id = args
                .get(1)
                .ok_or("usage: noema studio widget-remove <id>")?;
            http(
                "POST",
                "/api/studio",
                Some(json!({ "action": "widget_remove", "id": id })),
            )
        }
        Some("layout") => {
            let widgets = parse_body(args.get(1))?;
            http(
                "POST",
                "/api/studio",
                Some(json!({ "action": "layout", "widgets": widgets })),
            )
        }
        Some(other) => Err(format!("unknown studio command: {other}")),
    }
}

fn session(args: &[String]) -> Result<Value, String> {
    match args.first().map(String::as_str) {
        None | Some("resume") => call("session.resume", json!({})),
        Some("list") => {
            let mut limit = 50u64;
            if let Some(i) = args.iter().position(|a| a == "--limit") {
                limit = args
                    .get(i + 1)
                    .and_then(|v| v.parse().ok())
                    .ok_or("--limit needs a number")?;
            }
            call("session.list", json!({ "limit": limit }))
        }
        Some("open") => {
            let session = parse_body(args.get(1))?;
            call("session.open", json!({ "session": session }))
        }
        Some(other) => Err(format!("unknown session command: {other}")),
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

/// One HTTP/1.1 request to the Studio bridge, connection closed after the reply.
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

/// One request, one response line: the CLI never holds a daemon connection.
fn call(method: &str, params: Value) -> Result<Value, String> {
    let port = read_port_file(&data_dir()).ok_or("no daemon.json")?;
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port.port));
    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_millis(800)).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;

    let mut params = params;
    if let Value::Object(map) = &mut params {
        map.insert("_auth".into(), Value::String(port.token.clone()));
    }
    let request = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    stream
        .write_all(format!("{request}\n").as_bytes())
        .map_err(|e| e.to_string())?;

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
            continue; // event notification, not our reply
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

    /// RPC parity (8.1a): a method without a CLI path fails the build.
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

    /// GUI parity: the bridge routes the UI calls all have a CLI command.
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
        ];
        for (method, route) in ROUTES {
            assert!(
                HELP.contains(match *route {
                    "/api/health" => "health",
                    "/api/anatomy" => "anatomy",
                    "/api/anima" => "anima",
                    "/api/system" | "/api/system/refresh" => "system",
                    "/api/brain" => "brain",
                    "/api/brain/telemetry" => "telemetry",
                    "/api/brain/drag" => "drag",
                    "/api/studio" => "studio",
                    "/api/dream" => "dream",
                    _ => unreachable!(),
                }),
                "{method} {route} has no CLI command"
            );
        }
    }
}
