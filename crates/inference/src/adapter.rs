//! Raw HTTP codecs per provider: build the request body, normalise the stream.
//! No provider SDKs, no hidden retries: one call in, StreamEvent out.

use serde_json::{Map, Value, json};

use crate::request::{Effort, Reasoning, Request, StreamEvent};
use crate::{Standard, Usage};

pub struct Adapter {
    pub standard: Standard,
    pub base_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HttpCall {
    pub method: &'static str,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Value,
}

impl Adapter {
    pub fn new(standard: Standard, base_url: impl Into<String>) -> Self {
        Self {
            standard,
            base_url: base_url.into(),
            api_key: None,
        }
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn default_base(standard: Standard) -> &'static str {
        match standard {
            Standard::OpenAiChat | Standard::OpenAiResponses => "https://api.openai.com",
            Standard::AnthropicMessages => "https://api.anthropic.com",
            Standard::Gemini => "https://generativelanguage.googleapis.com",
            Standard::DeepSeek => "https://api.deepseek.com",
            Standard::Ollama => "http://127.0.0.1:11434",
            Standard::LlamaCpp => "http://127.0.0.1:8080",
            Standard::Vllm => "http://127.0.0.1:8000",
            Standard::LmStudio => "http://127.0.0.1:1234",
            Standard::CustomHttp => "http://127.0.0.1:8080",
        }
    }

    /// Build the wire call. Paths follow each provider's current API.
    pub fn encode(&self, req: &Request) -> HttpCall {
        let path = match self.standard {
            Standard::OpenAiChat
            | Standard::LlamaCpp
            | Standard::Vllm
            | Standard::LmStudio
            | Standard::CustomHttp => "/v1/chat/completions".to_string(),
            Standard::OpenAiResponses => "/v1/responses".to_string(),
            Standard::AnthropicMessages => "/v1/messages".to_string(),
            Standard::DeepSeek => "/chat/completions".to_string(),
            Standard::Gemini => format!(
                "/v1beta/models/{}:streamGenerateContent?alt=sse",
                req.model
            ),
            Standard::Ollama => "/api/chat".to_string(),
        };
        let body = match self.standard {
            Standard::AnthropicMessages => anthropic_body(req),
            Standard::Gemini => gemini_body(req),
            Standard::Ollama => ollama_body(req),
            Standard::OpenAiResponses => responses_body(req),
            _ => openai_body(req, self.standard),
        };
        let mut headers = vec![("content-type".to_string(), "application/json".to_string())];
        if let Some(key) = &self.api_key {
            match self.standard {
                Standard::AnthropicMessages => {
                    headers.push(("x-api-key".into(), key.clone()));
                    headers.push(("anthropic-version".into(), "2023-06-01".into()));
                }
                Standard::Gemini => headers.push(("x-goog-api-key".into(), key.clone())),
                _ => headers.push(("authorization".into(), format!("Bearer {key}"))),
            }
        }
        headers.extend(req.headers.iter().cloned());
        HttpCall {
            method: "POST",
            url: format!("{}{}", self.base_url.trim_end_matches('/'), path),
            headers,
            body,
        }
    }

    /// One SSE payload or NDJSON line -> normalised events. Unknown or partial
    /// lines yield an empty vector; adapters never panic on provider noise.
    pub fn decode_chunk(&self, chunk: &str) -> Vec<StreamEvent> {
        let line = chunk.trim();
        let line = line.strip_prefix("data:").map(str::trim).unwrap_or(line);
        if line.is_empty() || line.starts_with("event:") || line == "[DONE]" {
            return Vec::new();
        }
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            return Vec::new();
        };
        match self.standard {
            Standard::AnthropicMessages => anthropic_events(&v),
            Standard::Gemini => gemini_events(&v),
            Standard::Ollama => ollama_events(&v),
            Standard::OpenAiResponses => responses_events(&v),
            _ => openai_events(&v),
        }
    }

    pub fn is_terminal(event: &StreamEvent) -> bool {
        matches!(event, StreamEvent::Done { .. })
    }
}

fn messages_json(req: &Request) -> Vec<Value> {
    req.messages
        .iter()
        .map(|m| {
            let mut o = Map::new();
            o.insert("role".into(), json!(m.role));
            o.insert("content".into(), json!(m.content));
            if let Some(name) = &m.name {
                o.insert("name".into(), json!(name));
            }
            if !m.tool_calls.is_empty() {
                o.insert("tool_calls".into(), json!(m.tool_calls));
            }
            Value::Object(o)
        })
        .collect()
}

fn merge(map: &mut Map<String, Value>, req: &Request) {
    for (k, v) in &req.extra {
        map.insert(k.clone(), v.clone());
    }
}

fn openai_body(req: &Request, standard: Standard) -> Value {
    let s = &req.sampling;
    let local = matches!(
        standard,
        Standard::LlamaCpp | Standard::Vllm | Standard::LmStudio | Standard::CustomHttp
    );
    let mut m = Map::new();
    m.insert("model".into(), json!(req.model));
    m.insert("messages".into(), json!(messages_json(req)));
    m.insert("stream".into(), json!(req.stream));
    if req.stream {
        m.insert("stream_options".into(), json!({ "include_usage": true }));
    }
    if let Some(v) = s.temperature {
        m.insert("temperature".into(), json!(v));
    }
    if let Some(v) = s.top_p {
        m.insert("top_p".into(), json!(v));
    }
    if let Some(v) = s.max_tokens {
        m.insert("max_tokens".into(), json!(v));
    }
    if let Some(v) = s.presence_penalty {
        m.insert("presence_penalty".into(), json!(v));
    }
    if let Some(v) = s.frequency_penalty {
        m.insert("frequency_penalty".into(), json!(v));
    }
    if let Some(v) = s.seed {
        m.insert("seed".into(), json!(v));
    }
    if !s.stop.is_empty() {
        m.insert("stop".into(), json!(s.stop));
    }
    if let Some(n) = s.logprobs {
        m.insert("logprobs".into(), json!(true));
        m.insert("top_logprobs".into(), json!(n));
    }
    if local {
        if let Some(v) = s.top_k {
            m.insert("top_k".into(), json!(v));
        }
        if let Some(v) = s.min_p {
            m.insert("min_p".into(), json!(v));
        }
        if let Some(v) = s.typical_p {
            m.insert("typical_p".into(), json!(v));
        }
        if let Some(v) = s.repeat_penalty {
            let key = if standard == Standard::Vllm {
                "repetition_penalty"
            } else {
                "repeat_penalty"
            };
            m.insert(key.into(), json!(v));
        }
    }
    if let Reasoning::Effort { effort } = s.reasoning {
        m.insert("reasoning_effort".into(), json!(effort_name(effort)));
    }
    if !req.tools.is_empty() {
        m.insert("tools".into(), json!(req.tools));
    }
    merge(&mut m, req);
    Value::Object(m)
}

fn responses_body(req: &Request) -> Value {
    let s = &req.sampling;
    let input: Vec<Value> = req
        .messages
        .iter()
        .map(|m| json!({ "role": m.role, "content": m.content }))
        .collect();
    let mut m = Map::new();
    m.insert("model".into(), json!(req.model));
    m.insert("input".into(), json!(input));
    m.insert("stream".into(), json!(req.stream));
    if let Some(v) = s.max_tokens {
        m.insert("max_output_tokens".into(), json!(v));
    }
    if let Some(v) = s.temperature {
        m.insert("temperature".into(), json!(v));
    }
    if let Some(v) = s.top_p {
        m.insert("top_p".into(), json!(v));
    }
    if let Reasoning::Effort { effort } = s.reasoning {
        m.insert("reasoning".into(), json!({ "effort": effort_name(effort) }));
    }
    if !req.tools.is_empty() {
        m.insert("tools".into(), json!(req.tools));
    }
    merge(&mut m, req);
    Value::Object(m)
}

fn anthropic_body(req: &Request) -> Value {
    let s = &req.sampling;
    let mut m = Map::new();
    m.insert("model".into(), json!(req.model));
    m.insert("stream".into(), json!(true));
    m.insert("max_tokens".into(), json!(s.max_tokens.unwrap_or(1024)));
    if let Some(system) = req.system_text() {
        m.insert("system".into(), json!(system));
    }
    let messages: Vec<Value> = req
        .messages
        .iter()
        .filter(|m| m.role != crate::Role::System)
        .map(|m| json!({ "role": m.role, "content": m.content }))
        .collect();
    m.insert("messages".into(), json!(messages));
    if let Some(v) = s.temperature {
        m.insert("temperature".into(), json!(v));
    }
    if let Some(v) = s.top_p {
        m.insert("top_p".into(), json!(v));
    }
    if let Some(v) = s.top_k {
        m.insert("top_k".into(), json!(v));
    }
    if !s.stop.is_empty() {
        m.insert("stop_sequences".into(), json!(s.stop));
    }
    match s.reasoning {
        Reasoning::Budget { tokens } => {
            m.insert(
                "thinking".into(),
                json!({ "type": "enabled", "budget_tokens": tokens }),
            );
        }
        Reasoning::Effort { effort } => {
            m.insert(
                "thinking".into(),
                json!({ "type": "enabled", "budget_tokens": effort_budget(effort) }),
            );
        }
        Reasoning::Off => {}
    }
    if !req.tools.is_empty() {
        m.insert("tools".into(), json!(req.tools));
    }
    merge(&mut m, req);
    Value::Object(m)
}

fn gemini_body(req: &Request) -> Value {
    let s = &req.sampling;
    let contents: Vec<Value> = req
        .messages
        .iter()
        .filter(|m| m.role != crate::Role::System)
        .map(|m| {
            let role = if m.role == crate::Role::Assistant {
                "model"
            } else {
                "user"
            };
            json!({ "role": role, "parts": [{ "text": m.content }] })
        })
        .collect();
    let mut config = Map::new();
    if let Some(v) = s.temperature {
        config.insert("temperature".into(), json!(v));
    }
    if let Some(v) = s.top_p {
        config.insert("topP".into(), json!(v));
    }
    if let Some(v) = s.top_k {
        config.insert("topK".into(), json!(v));
    }
    if let Some(v) = s.max_tokens {
        config.insert("maxOutputTokens".into(), json!(v));
    }
    if let Some(v) = s.seed {
        config.insert("seed".into(), json!(v));
    }
    if !s.stop.is_empty() {
        config.insert("stopSequences".into(), json!(s.stop));
    }
    if let Some(v) = s.presence_penalty {
        config.insert("presencePenalty".into(), json!(v));
    }
    if let Some(v) = s.frequency_penalty {
        config.insert("frequencyPenalty".into(), json!(v));
    }
    if let Some(n) = s.logprobs {
        config.insert("responseLogprobs".into(), json!(true));
        config.insert("logprobs".into(), json!(n));
    }
    let mut m = Map::new();
    m.insert("contents".into(), json!(contents));
    m.insert("generationConfig".into(), Value::Object(config));
    if let Some(system) = req.system_text() {
        m.insert(
            "systemInstruction".into(),
            json!({ "parts": [{ "text": system }] }),
        );
    }
    if let Reasoning::Budget { tokens } = s.reasoning {
        m.insert("thinkingConfig".into(), json!({ "thinkingBudget": tokens }));
    }
    merge(&mut m, req);
    Value::Object(m)
}

fn ollama_body(req: &Request) -> Value {
    let s = &req.sampling;
    let mut options = Map::new();
    for (key, value) in [
        ("temperature", s.temperature.map(f64::from)),
        ("top_p", s.top_p.map(f64::from)),
        ("min_p", s.min_p.map(f64::from)),
        ("typical_p", s.typical_p.map(f64::from)),
        ("repeat_penalty", s.repeat_penalty.map(f64::from)),
    ] {
        if let Some(v) = value {
            options.insert(key.into(), json!(v));
        }
    }
    if let Some(v) = s.top_k {
        options.insert("top_k".into(), json!(v));
    }
    if let Some(v) = s.max_tokens {
        options.insert("num_predict".into(), json!(v));
    }
    if let Some(v) = s.seed {
        options.insert("seed".into(), json!(v));
    }
    if !s.stop.is_empty() {
        options.insert("stop".into(), json!(s.stop));
    }
    let messages: Vec<Value> = req
        .messages
        .iter()
        .map(|m| json!({ "role": m.role, "content": m.content }))
        .collect();
    let mut m = Map::new();
    m.insert("model".into(), json!(req.model));
    m.insert("messages".into(), json!(messages));
    m.insert("stream".into(), json!(true));
    m.insert("options".into(), Value::Object(options));
    if s.reasoning != Reasoning::Off {
        m.insert("think".into(), json!(true));
    }
    if !req.tools.is_empty() {
        m.insert("tools".into(), json!(req.tools));
    }
    merge(&mut m, req);
    Value::Object(m)
}

fn effort_name(effort: Effort) -> &'static str {
    match effort {
        Effort::Low => "low",
        Effort::Medium => "medium",
        Effort::High => "high",
    }
}

fn effort_budget(effort: Effort) -> u32 {
    match effort {
        Effort::Low => 1024,
        Effort::Medium => 4096,
        Effort::High => 16384,
    }
}

fn openai_events(v: &Value) -> Vec<StreamEvent> {
    let mut out = Vec::new();
    if let Some(choice) = v["choices"].as_array().and_then(|c| c.first()) {
        let delta = &choice["delta"];
        if let Some(text) = delta["content"].as_str().filter(|t| !t.is_empty()) {
            out.push(StreamEvent::Delta { text: text.into() });
        }
        let reasoning = delta["reasoning_content"]
            .as_str()
            .or_else(|| delta["reasoning"].as_str());
        if let Some(text) = reasoning.filter(|t| !t.is_empty()) {
            out.push(StreamEvent::Reasoning { text: text.into() });
        }
        if let Some(items) = delta["logprobs"]["content"].as_array() {
            for item in items {
                if let (Some(token), Some(lp)) = (
                    item["token"].as_str(),
                    item["logprob"].as_f64(),
                ) {
                    out.push(StreamEvent::Logprob {
                        token: token.into(),
                        logprob: lp,
                    });
                }
            }
        }
        if let Some(reason) = choice["finish_reason"].as_str() {
            out.push(StreamEvent::Done { reason: reason.into() });
        }
    }
    if v["usage"].is_object() {
        out.push(StreamEvent::Usage(usage_openai(&v["usage"])));
    }
    out
}

fn responses_events(v: &Value) -> Vec<StreamEvent> {
    let kind = v["type"].as_str().unwrap_or_default();
    match kind {
        "response.output_text.delta" => v["delta"]
            .as_str()
            .map(|t| vec![StreamEvent::Delta { text: t.into() }])
            .unwrap_or_default(),
        "response.reasoning_summary_text.delta" | "response.reasoning_text.delta" => v["delta"]
            .as_str()
            .map(|t| vec![StreamEvent::Reasoning { text: t.into() }])
            .unwrap_or_default(),
        "response.completed" | "response.incomplete" => {
            let mut out = Vec::new();
            let u = &v["response"]["usage"];
            if u.is_object() {
                out.push(StreamEvent::Usage(Usage {
                    input: u["input_tokens"].as_u64().unwrap_or(0),
                    output: u["output_tokens"].as_u64().unwrap_or(0),
                    reasoning: u["output_tokens_details"]["reasoning_tokens"]
                        .as_u64()
                        .unwrap_or(0),
                    cache_read: u["input_tokens_details"]["cached_tokens"]
                        .as_u64()
                        .unwrap_or(0),
                    cache_write: 0,
                }));
            }
            out.push(StreamEvent::Done {
                reason: if kind == "response.completed" {
                    "stop".into()
                } else {
                    "incomplete".into()
                },
            });
            out
        }
        "response.failed" => vec![StreamEvent::Done {
            reason: "error".into(),
        }],
        _ => Vec::new(),
    }
}

fn anthropic_events(v: &Value) -> Vec<StreamEvent> {
    match v["type"].as_str().unwrap_or_default() {
        "message_start" => {
            let u = &v["message"]["usage"];
            if u.is_object() {
                vec![StreamEvent::Usage(Usage {
                    input: u["input_tokens"].as_u64().unwrap_or(0),
                    output: u["output_tokens"].as_u64().unwrap_or(0),
                    reasoning: 0,
                    cache_read: u["cache_read_input_tokens"].as_u64().unwrap_or(0),
                    cache_write: u["cache_creation_input_tokens"].as_u64().unwrap_or(0),
                })]
            } else {
                Vec::new()
            }
        }
        "content_block_delta" => {
            let d = &v["delta"];
            match d["type"].as_str().unwrap_or_default() {
                "text_delta" => d["text"]
                    .as_str()
                    .map(|t| vec![StreamEvent::Delta { text: t.into() }])
                    .unwrap_or_default(),
                "thinking_delta" => d["thinking"]
                    .as_str()
                    .map(|t| vec![StreamEvent::Reasoning { text: t.into() }])
                    .unwrap_or_default(),
                _ => Vec::new(),
            }
        }
        "message_delta" => {
            let mut out = Vec::new();
            let u = &v["usage"];
            if u.is_object() {
                out.push(StreamEvent::Usage(Usage {
                    output: u["output_tokens"].as_u64().unwrap_or(0),
                    ..Usage::default()
                }));
            }
            out
        }
        "message_stop" => vec![StreamEvent::Done {
            reason: v["delta"]["stop_reason"]
                .as_str()
                .unwrap_or("stop")
                .into(),
        }],
        _ => Vec::new(),
    }
}

fn gemini_events(v: &Value) -> Vec<StreamEvent> {
    let mut out = Vec::new();
    if let Some(parts) = v["candidates"][0]["content"]["parts"].as_array() {
        for part in parts {
            if let Some(text) = part["text"].as_str().filter(|t| !t.is_empty()) {
                if part["thought"].as_bool().unwrap_or(false) {
                    out.push(StreamEvent::Reasoning { text: text.into() });
                } else {
                    out.push(StreamEvent::Delta { text: text.into() });
                }
            }
        }
    }
    if let Some(reason) = v["candidates"][0]["finishReason"].as_str() {
        out.push(StreamEvent::Done {
            reason: reason.to_lowercase(),
        });
    }
    let u = &v["usageMetadata"];
    if u.is_object() {
        let cached = u["cachedContentTokenCount"].as_u64().unwrap_or(0);
        out.push(StreamEvent::Usage(Usage {
            input: u["promptTokenCount"].as_u64().unwrap_or(0).saturating_sub(cached),
            output: u["candidatesTokenCount"].as_u64().unwrap_or(0),
            reasoning: u["thoughtsTokenCount"].as_u64().unwrap_or(0),
            cache_read: cached,
            cache_write: 0,
        }));
    }
    out
}

fn ollama_events(v: &Value) -> Vec<StreamEvent> {
    let mut out = Vec::new();
    if let Some(text) = v["message"]["content"].as_str().filter(|t| !t.is_empty()) {
        out.push(StreamEvent::Delta { text: text.into() });
    }
    if let Some(text) = v["message"]["thinking"].as_str().filter(|t| !t.is_empty()) {
        out.push(StreamEvent::Reasoning { text: text.into() });
    }
    if v["done"].as_bool().unwrap_or(false) {
        out.push(StreamEvent::Usage(Usage {
            input: v["prompt_eval_count"].as_u64().unwrap_or(0),
            output: v["eval_count"].as_u64().unwrap_or(0),
            ..Usage::default()
        }));
        out.push(StreamEvent::Done {
            reason: v["done_reason"].as_str().unwrap_or("stop").into(),
        });
    }
    out
}

fn usage_openai(u: &Value) -> Usage {
    let cached = u["prompt_tokens_details"]["cached_tokens"]
        .as_u64()
        .unwrap_or(0);
    let prompt = u["prompt_tokens"].as_u64().unwrap_or(0);
    let reasoning = u["completion_tokens_details"]["reasoning_tokens"]
        .as_u64()
        .unwrap_or(0);
    let completion = u["completion_tokens"].as_u64().unwrap_or(0);
    Usage {
        input: prompt.saturating_sub(cached),
        output: completion.saturating_sub(reasoning),
        reasoning,
        cache_read: cached,
        cache_write: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{Message, Role};

    fn sample_request() -> Request {
        let mut req = Request::new("gpt-5");
        req.messages.push(Message::text(Role::System, "be terse"));
        req.messages.push(Message::text(Role::User, "hi"));
        req.sampling.temperature = Some(0.2);
        req.sampling.top_p = Some(0.9);
        req.sampling.top_k = Some(40);
        req.sampling.min_p = Some(0.05);
        req.sampling.max_tokens = Some(256);
        req.sampling.seed = Some(7);
        req.sampling.stop = vec!["END".into()];
        req.sampling.logprobs = Some(3);
        req.sampling.reasoning = Reasoning::Effort {
            effort: Effort::High,
        };
        req
    }

    #[test]
    fn openai_chat_encodes_sampling_and_auth() {
        let adapter = Adapter::new(Standard::OpenAiChat, "https://api.openai.com/").with_key("sk-test");
        let call = adapter.encode(&sample_request());
        assert_eq!(call.url, "https://api.openai.com/v1/chat/completions");
        assert!(call
            .headers
            .iter()
            .any(|(k, v)| k == "authorization" && v == "Bearer sk-test"));
        let body = &call.body;
        assert_eq!(body["model"], "gpt-5");
        assert_eq!(body["stream"], true);
        assert_eq!(body["stream_options"]["include_usage"], true);
        let top_p = body["top_p"].as_f64().unwrap();
        assert!((top_p - 0.9).abs() < 1e-6);
        assert_eq!(body["top_k"], Value::Null); // cloud OpenAI never gets top_k
        assert_eq!(body["reasoning_effort"], "high");
        assert_eq!(body["logprobs"], true);
        assert_eq!(body["top_logprobs"], 3);
        assert_eq!(body["stop"][0], "END");
        assert_eq!(body["messages"][0]["role"], "system");
    }

    #[test]
    fn anthropic_splits_system_and_thinking_budget() {
        let mut req = sample_request();
        req.sampling.reasoning = Reasoning::Budget { tokens: 2048 };
        req.model = "claude-sonnet-5-5".into();
        let call = Adapter::new(Standard::AnthropicMessages, "https://api.anthropic.com")
            .with_key("k")
            .encode(&req);
        assert_eq!(call.url, "https://api.anthropic.com/v1/messages");
        assert!(call.headers.iter().any(|(k, _)| k == "x-api-key"));
        assert!(call.headers.iter().any(|(k, v)| k == "anthropic-version" && v == "2023-06-01"));
        assert_eq!(call.body["system"], "be terse");
        assert_eq!(call.body["messages"].as_array().unwrap().len(), 1);
        assert_eq!(call.body["thinking"]["budget_tokens"], 2048);
        assert_eq!(call.body["max_tokens"], 256);
    }

    #[test]
    fn gemini_uses_model_scoped_stream_path() {
        let call = Adapter::new(Standard::Gemini, "https://generativelanguage.googleapis.com")
            .with_key("g")
            .encode(&sample_request());
        assert_eq!(
            call.url,
            "https://generativelanguage.googleapis.com/v1beta/models/gpt-5:streamGenerateContent?alt=sse"
        );
        assert_eq!(call.body["contents"][0]["role"], "user");
        assert_eq!(call.body["systemInstruction"]["parts"][0]["text"], "be terse");
        assert_eq!(call.body["generationConfig"]["topK"], 40);
        assert_eq!(call.body["generationConfig"]["maxOutputTokens"], 256);
    }

    #[test]
    fn ollama_maps_options_and_thinking() {
        let call = Adapter::new(Standard::Ollama, "http://127.0.0.1:11434").encode(&sample_request());
        assert_eq!(call.url, "http://127.0.0.1:11434/api/chat");
        assert_eq!(call.body["options"]["num_predict"], 256);
        assert_eq!(call.body["options"]["top_k"], 40);
        assert_eq!(call.body["think"], true);
    }

    #[test]
    fn local_openai_compatible_keeps_full_sampling() {
        let call = Adapter::new(Standard::Vllm, "http://127.0.0.1:8000").encode(&sample_request());
        assert_eq!(call.body["top_k"], 40);
        let min_p = call.body["min_p"].as_f64().unwrap();
        assert!((min_p - 0.05).abs() < 1e-6);
    }

    #[test]
    fn openai_stream_normalises_reasoning_usage_and_done() {
        let adapter = Adapter::new(Standard::DeepSeek, "https://api.deepseek.com");
        let events = adapter.decode_chunk(
            r#"data: {"choices":[{"delta":{"content":"hi","reasoning_content":"think"},"finish_reason":null}]}"#,
        );
        assert_eq!(
            events,
            vec![
                StreamEvent::Delta { text: "hi".into() },
                StreamEvent::Reasoning { text: "think".into() }
            ]
        );
        let usage = adapter.decode_chunk(
            r#"data: {"choices":[],"usage":{"prompt_tokens":1000,"completion_tokens":200,"completion_tokens_details":{"reasoning_tokens":50},"prompt_tokens_details":{"cached_tokens":400}}}"#,
        );
        assert_eq!(
            usage,
            vec![StreamEvent::Usage(Usage {
                input: 600,
                output: 150,
                reasoning: 50,
                cache_read: 400,
                cache_write: 0,
            })]
        );
        assert_eq!(
            adapter.decode_chunk(r#"data: {"choices":[{"delta":{},"finish_reason":"stop"}]}"#),
            vec![StreamEvent::Done { reason: "stop".into() }]
        );
        assert!(adapter.decode_chunk("data: [DONE]").is_empty());
    }

    #[test]
    fn anthropic_stream_normalises_text_thinking_and_cache() {
        let adapter = Adapter::new(Standard::AnthropicMessages, "https://api.anthropic.com");
        assert_eq!(
            adapter.decode_chunk(
                r#"data: {"type":"content_block_delta","delta":{"type":"thinking_delta","thinking":"hmm"}}"#
            ),
            vec![StreamEvent::Reasoning { text: "hmm".into() }]
        );
        assert_eq!(
            adapter.decode_chunk(
                r#"data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"ok"}}"#
            ),
            vec![StreamEvent::Delta { text: "ok".into() }]
        );
        assert_eq!(
            adapter.decode_chunk(
                r#"data: {"type":"message_start","message":{"usage":{"input_tokens":12,"output_tokens":1,"cache_read_input_tokens":5,"cache_creation_input_tokens":2}}}"#
            ),
            vec![StreamEvent::Usage(Usage {
                input: 12,
                output: 1,
                reasoning: 0,
                cache_read: 5,
                cache_write: 2,
            })]
        );
        assert_eq!(
            adapter.decode_chunk(r#"data: {"type":"message_stop"}"#),
            vec![StreamEvent::Done { reason: "stop".into() }]
        );
    }

    #[test]
    fn gemini_stream_marks_thought_parts() {
        let adapter = Adapter::new(Standard::Gemini, "");
        let events = adapter.decode_chunk(
            r#"data: {"candidates":[{"content":{"parts":[{"text":"plan","thought":true},{"text":"answer"}]},"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":10,"cachedContentTokenCount":4,"candidatesTokenCount":3,"thoughtsTokenCount":2}}"#,
        );
        assert_eq!(
            events,
            vec![
                StreamEvent::Reasoning { text: "plan".into() },
                StreamEvent::Delta { text: "answer".into() },
                StreamEvent::Done { reason: "stop".into() },
                StreamEvent::Usage(Usage {
                    input: 6,
                    output: 3,
                    reasoning: 2,
                    cache_read: 4,
                    cache_write: 0,
                }),
            ]
        );
    }

    #[test]
    fn ollama_ndjson_and_responses_events() {
        let ollama = Adapter::new(Standard::Ollama, "");
        assert_eq!(
            ollama.decode_chunk(r#"{"message":{"content":"yo","thinking":"hm"},"done":false}"#),
            vec![
                StreamEvent::Delta { text: "yo".into() },
                StreamEvent::Reasoning { text: "hm".into() }
            ]
        );
        assert_eq!(
            ollama.decode_chunk(
                r#"{"message":{"content":""},"done":true,"done_reason":"stop","prompt_eval_count":8,"eval_count":4}"#
            ),
            vec![
                StreamEvent::Usage(Usage { input: 8, output: 4, ..Usage::default() }),
                StreamEvent::Done { reason: "stop".into() }
            ]
        );
        let responses = Adapter::new(Standard::OpenAiResponses, "");
        assert_eq!(
            responses.decode_chunk(r#"{"type":"response.output_text.delta","delta":"x"}"#),
            vec![StreamEvent::Delta { text: "x".into() }]
        );
        assert_eq!(
            responses.decode_chunk(
                r#"{"type":"response.reasoning_summary_text.delta","delta":"why"}"#
            ),
            vec![StreamEvent::Reasoning { text: "why".into() }]
        );
    }
}
