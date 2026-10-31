//! Asynchronous event bus: one broadcast channel shared by every daemon
//! connection (CLI, Studio, 3D world) and every internal producer.

use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub method: String,
    pub params: Value,
}

impl Event {
    pub fn new(method: &str, params: Value) -> Self {
        Self { method: method.into(), params }
    }
}

/// Capacity 1024 per the IPC contract; overflow drops the oldest frames.
pub type Bus = tokio::sync::broadcast::Sender<Event>;

pub fn bus() -> Bus {
    tokio::sync::broadcast::channel(1024).0
}

/// NDJSON notification frame for one event.
pub fn frame(event: &Event) -> String {
    let mut line = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "method": event.method,
        "params": event.params,
    }))
    .unwrap_or_default();
    line.push('\n');
    line
}
