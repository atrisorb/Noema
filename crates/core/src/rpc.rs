//! JSON-RPC 2.0 dispatch shared by `noemad` and the CLI client.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::VERSION;
use crate::event::Bus;
use crate::session::Session;
use crate::store::Store;

pub const AUTH_CODE: i32 = -32001;
pub const PARSE_CODE: i32 = -32700;
pub const METHOD_CODE: i32 = -32601;
pub const PARAMS_CODE: i32 = -32602;
pub const INTERNAL_CODE: i32 = -32603;

#[derive(Debug, Deserialize)]
pub struct Request {
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: &'static str,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl Response {
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }

    pub fn to_line(&self) -> String {
        let mut line = serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialize"}}"#.into()
        });
        line.push('\n');
        line
    }
}

/// Longest accepted agent/model reference; ids are opaque labels in the catalog.
const MAX_REF_LEN: usize = 128;

fn valid_ref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_REF_LEN
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':' | '/' | '@'))
}

/// Reject malformed or dangling references before they reach the database.
fn validate_session(store: &Store, session: &Session) -> Result<(), String> {
    if uuid::Uuid::parse_str(&session.id.0).is_err() {
        return Err("session.id must be a uuid".into());
    }
    if !valid_ref(&session.agent.0) {
        return Err("session.agent must be 1-128 chars of [A-Za-z0-9._:/@-]".into());
    }
    if !valid_ref(&session.model.0) {
        return Err("session.model must be 1-128 chars of [A-Za-z0-9._:/@-]".into());
    }
    if !std::path::Path::new(&session.cwd).is_absolute() {
        return Err("session.cwd must be an absolute path".into());
    }
    if let Some(project) = &session.project
        && !store
            .project_exists(&project.0)
            .map_err(|e| e.to_string())?
    {
        return Err(format!("unknown project_id: {}", project.0));
    }
    Ok(())
}

/// Every RPC method the daemon dispatches, in one place.
///
/// `rpc.methods` exposes this list and the CLI parity test asserts each entry has
/// a CLI path, so adding a method without a client is a test failure.
pub const METHODS: &[&str] = &[
    "rpc.methods",
    "daemon.ping",
    "session.resume",
    "session.list",
    "session.open",
    "system.profile",
    "system.override",
    "system.reset",
];

/// Dispatch one authenticated request. Errors never kill the connection.
pub fn dispatch(store: &mut Store, bus: &Bus, req: &Request) -> Response {
    let id = req.id.clone();
    match req.method.as_str() {
        "rpc.methods" => Response::ok(id, json!({ "methods": METHODS })),
        "daemon.ping" => Response::ok(id, json!({ "version": VERSION, "pid": std::process::id() })),
        "session.resume" => match store.resume() {
            Ok(resume) => {
                let session = resume
                    .last_session
                    .as_ref()
                    .and_then(|sid| store.load(sid).ok().flatten());
                Response::ok(
                    id,
                    json!({ "session": session, "diff_stat": resume.diff_stat, "resume": resume }),
                )
            }
            Err(e) => Response::err(id, INTERNAL_CODE, e.to_string()),
        },
        "session.list" => {
            let limit = req
                .params
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(50)
                .min(500) as u32;
            match store.list_sessions(limit) {
                Ok(list) => Response::ok(id, json!({ "sessions": list })),
                Err(e) => Response::err(id, INTERNAL_CODE, e.to_string()),
            }
        }
        "session.open" => {
            let Some(session) = req
                .params
                .get("session")
                .cloned()
                .and_then(|v| serde_json::from_value::<Session>(v).ok())
            else {
                return Response::err(id, PARAMS_CODE, "session.open requires params.session");
            };
            if let Err(message) = validate_session(store, &session) {
                return Response::err(id, PARAMS_CODE, message);
            }
            match store.save(&session) {
                Ok(()) => {
                    let _ = bus.send(crate::event::Event::new(
                        "event.session.updated",
                        json!({ "session_id": session.id }),
                    ));
                    Response::ok(id, json!({ "session": session }))
                }
                Err(e) => Response::err(id, INTERNAL_CODE, e.to_string()),
            }
        }
        "system.profile" => {
            let refresh = req
                .params
                .get("refresh")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            match store.hardware_report(refresh) {
                Ok(report) => Response::ok(id, json!(report)),
                Err(e) => Response::err(id, INTERNAL_CODE, e.to_string()),
            }
        }
        "system.override" => {
            let Some(patch) = req.params.get("overrides") else {
                return Response::err(id, PARAMS_CODE, "system.override requires params.overrides");
            };
            match store.set_hardware_overrides(patch) {
                Ok(report) => Response::ok(id, json!(report)),
                Err(e) => Response::err(id, PARAMS_CODE, e.to_string()),
            }
        }
        "system.reset" => match store.reset_hardware() {
            Ok(report) => Response::ok(id, json!(report)),
            Err(e) => Response::err(id, INTERNAL_CODE, e.to_string()),
        },
        other => Response::err(id, METHOD_CODE, format!("method not found: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_method_reaches_the_dispatch_table() {
        let mut store = Store::open_memory().expect("memory store");
        let bus = crate::event::bus();
        for method in METHODS {
            let req = Request {
                id: Some(json!(1)),
                method: (*method).into(),
                params: json!({}),
            };
            let res = dispatch(&mut store, &bus, &req);
            assert_ne!(
                res.error.as_ref().map(|e| e.code),
                Some(METHOD_CODE),
                "{method} is listed but not dispatched"
            );
        }
    }
}
