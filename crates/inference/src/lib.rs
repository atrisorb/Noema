//! Universal inference surface: every provider is normalised to one request /
//! usage shape so pricing, telemetry and the brain see identical data.

pub mod adapter;
pub mod cluster;
pub mod cost;
pub mod embed;
pub mod request;
pub mod telemetry;

use serde::{Deserialize, Serialize};

/// Wire protocols NOEMA speaks. CustomHttp accepts any OpenAI-compatible
/// server; local runtimes keep their own quirks in the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Standard {
    OpenAiChat,
    OpenAiResponses,
    AnthropicMessages,
    Gemini,
    DeepSeek,
    Ollama,
    LlamaCpp,
    Vllm,
    LmStudio,
    CustomHttp,
}

/// Token accounting. Reasoning and cache tokens are billed separately.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

impl Usage {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.reasoning + self.cache_read + self.cache_write
    }
}

pub use adapter::{Adapter, HttpCall};
pub use cost::{LocalCost, ModelCatalogEntry, RateCard, RateWindow, default_catalog};
pub use request::{Effort, Message, Reasoning, Request, Role, Sampling, StreamEvent};
pub use telemetry::{Activation, ActivationWindow};
