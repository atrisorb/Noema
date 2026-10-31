//! Embedding pipeline: provider-specific encode/decode plus the cosine helper
//! used by clustering. Vectors stay f32, exactly as stored in SQLite BLOBs.

use serde_json::{Value, json};

use crate::{HttpCall, Standard};

/// Batch embedding call. OpenAI-compatible servers share one shape; Gemini and
/// Ollama keep their native payload.
pub fn encode(
    standard: Standard,
    base_url: &str,
    api_key: Option<&str>,
    model: &str,
    inputs: &[String],
) -> HttpCall {
    let (path, body) = match standard {
        Standard::Gemini => (
            format!("/v1beta/models/{model}:batchEmbedContents"),
            json!({
                "requests": inputs.iter().map(|text| json!({
                    "model": format!("models/{model}"),
                    "content": { "parts": [{ "text": text }] }
                })).collect::<Vec<_>>()
            }),
        ),
        Standard::Ollama => (
            "/api/embed".to_string(),
            json!({ "model": model, "input": inputs }),
        ),
        _ => (
            "/v1/embeddings".to_string(),
            json!({ "model": model, "input": inputs }),
        ),
    };
    let mut headers = vec![("content-type".to_string(), "application/json".to_string())];
    if let Some(key) = api_key {
        match standard {
            Standard::Gemini => headers.push(("x-goog-api-key".into(), key.to_string())),
            _ => headers.push(("authorization".into(), format!("Bearer {key}"))),
        }
    }
    HttpCall {
        method: "POST",
        url: format!("{}{}", base_url.trim_end_matches('/'), path),
        headers,
        body,
    }
}

/// Decode one provider response into vectors, preserving input order.
pub fn decode(standard: Standard, body: &Value) -> Vec<Vec<f32>> {
    match standard {
        Standard::Gemini => body["embeddings"]
            .as_array()
            .map(|items| items.iter().map(|e| floats(&e["values"])).collect())
            .unwrap_or_default(),
        Standard::Ollama => body["embeddings"]
            .as_array()
            .map(|items| items.iter().map(floats).collect())
            .unwrap_or_default(),
        _ => {
            let mut items: Vec<(u64, Vec<f32>)> = body["data"]
                .as_array()
                .map(|data| {
                    data.iter()
                        .map(|d| (d["index"].as_u64().unwrap_or(0), floats(&d["embedding"])))
                        .collect()
                })
                .unwrap_or_default();
            items.sort_by_key(|(i, _)| *i);
            items.into_iter().map(|(_, v)| v).collect()
        }
    }
}

fn floats(v: &Value) -> Vec<f32> {
    v.as_array()
        .map(|a| a.iter().filter_map(|n| n.as_f64().map(|f| f as f32)).collect())
        .unwrap_or_default()
}

pub use noema_core::vector::cosine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_vectors_keep_input_order() {
        let body = json!({ "data": [
            { "index": 1, "embedding": [3.0, 4.0] },
            { "index": 0, "embedding": [0.6, 0.8] }
        ]});
        let vectors = decode(Standard::OpenAiChat, &body);
        assert_eq!(vectors, vec![vec![0.6, 0.8], vec![3.0, 4.0]]);
        assert!((cosine(&vectors[0], &vectors[0]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gemini_and_ollama_shapes() {
        let gemini = json!({ "embeddings": [{ "values": [1.0, 0.0] }] });
        assert_eq!(decode(Standard::Gemini, &gemini), vec![vec![1.0, 0.0]]);
        let ollama = json!({ "embeddings": [[0.0, 1.0]] });
        assert_eq!(decode(Standard::Ollama, &ollama), vec![vec![0.0, 1.0]]);
    }
}
