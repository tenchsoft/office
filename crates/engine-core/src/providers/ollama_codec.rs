use std::io::BufRead;

use serde_json::{json, Value};
use tench_shared_types::{
    ChatChoice, ChatCompletionResult, ChatMessage, EngineError, EngineEvent, ModelDescriptor,
    ModelLocation, UsageStats,
};

use crate::util::{provider_error, token_estimate};
use crate::EngineEventStream;

pub(crate) fn parse_ollama_tags(value: Value) -> Vec<ModelDescriptor> {
    value
        .get("models")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|model| {
            let name = model
                .get("name")
                .or_else(|| model.get("model"))
                .and_then(Value::as_str)?;
            Some(ModelDescriptor {
                id: format!("runtime/ollama/{name}"),
                display_name: name.to_string(),
                provider: "ollama".to_string(),
                capability: "chat".to_string(),
                location: ModelLocation::Local,
            })
        })
        .collect()
}

pub(crate) fn parse_ollama_chat_response(
    request_id: &str,
    model: &str,
    value: Value,
) -> Result<ChatCompletionResult, EngineError> {
    let content = value
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let input_tokens = value
        .get("prompt_eval_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let output_tokens = value
        .get("eval_count")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| token_estimate(&content));

    Ok(ChatCompletionResult {
        id: request_id.to_string(),
        model: model.to_string(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage::assistant(content),
            finish_reason: "stop".to_string(),
        }],
        usage: UsageStats::new(input_tokens, output_tokens),
    })
}

pub(crate) fn parse_ollama_stream_reader<R: BufRead>(
    reader: R,
    request_id: &str,
    model: &str,
) -> EngineEventStream {
    let mut events = Vec::new();
    let mut output_tokens = 0;

    for line in reader.lines() {
        let Ok(line) = line else {
            events.push(EngineEvent::Error {
                error: provider_error(request_id, "Failed to read Ollama stream"),
            });
            return events;
        };
        if line.trim().is_empty() {
            continue;
        }

        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            events.push(EngineEvent::Warning {
                request_id: request_id.to_string(),
                message: "Ignored malformed Ollama stream line".to_string(),
            });
            continue;
        };

        if let Some(delta) = value
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
            .filter(|delta| !delta.is_empty())
        {
            output_tokens += token_estimate(delta);
            events.push(EngineEvent::Token {
                request_id: request_id.to_string(),
                delta: delta.to_string(),
            });
        }

        if value.get("done").and_then(Value::as_bool).unwrap_or(false) {
            let input_tokens = value
                .get("prompt_eval_count")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let eval_tokens = value
                .get("eval_count")
                .and_then(Value::as_u64)
                .unwrap_or(output_tokens);
            events.push(EngineEvent::Metadata {
                request_id: request_id.to_string(),
                metadata: json!({
                    "model": model,
                    "provider": "ollama",
                }),
            });
            events.push(EngineEvent::Done {
                request_id: request_id.to_string(),
                usage: Some(UsageStats::new(input_tokens, eval_tokens)),
            });
        }
    }

    events
}
