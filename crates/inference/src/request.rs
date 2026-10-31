//! One request shape for every provider, plus the normalised stream events the
//! brain and the telemetry path consume.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::Usage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<Value>,
}

impl Message {
    pub fn text(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            name: None,
            tool_calls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effort {
    Low,
    Medium,
    High,
}

/// Thinking control. Providers with no native switch still receive the budget
/// through extra when the caller sets one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Reasoning {
    #[default]
    Off,
    Effort {
        effort: Effort,
    },
    Budget {
        tokens: u32,
    },
}

/// Full sampling surface. None means "omit the field", never "send null".
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Sampling {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub min_p: Option<f32>,
    pub typical_p: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub max_tokens: Option<u32>,
    pub seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop: Vec<String>,
    #[serde(default)]
    pub reasoning: Reasoning,
    /// Top-N logprobs. None disables the field.
    pub logprobs: Option<u8>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub template_vars: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub sampling: Sampling,
    #[serde(default)]
    pub stream: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Value>,
    /// Provider-specific payload merged last (overrides normalised fields).
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub extra: Map<String, Value>,
    /// Additional HTTP headers (custom auth, routing tags).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: Vec::new(),
            sampling: Sampling::default(),
            stream: true,
            tools: Vec::new(),
            extra: Map::new(),
            headers: Vec::new(),
        }
    }

    pub fn system_text(&self) -> Option<String> {
        let parts: Vec<&str> = self
            .messages
            .iter()
            .filter(|m| m.role == Role::System)
            .map(|m| m.content.as_str())
            .collect();
        (!parts.is_empty()).then(|| parts.join("\n"))
    }
}

/// Normalised stream. Every adapter emits only these five shapes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    Delta { text: String },
    Reasoning { text: String },
    Logprob { token: String, logprob: f64 },
    Usage(Usage),
    Done { reason: String },
}
