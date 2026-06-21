use std::io::BufRead;

use serde_json::{json, Value};
use tench_shared_types::{
    ChatChoice, ChatCompletionResult, ChatMessage, EngineError, EngineEvent, ModelDescriptor,
    ModelLocation, UsageStats,
};

use crate::util::{provider_error, token_estimate};
use crate::EngineEventStream;

pub(crate) fn parse_llama_cpp_models(value: Value) -> Vec<ModelDescriptor> {
    value
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|model| {
            let id = model.get("id").and_then(Value::as_str)?;
            let capability = if model
                .get("meta")
                .and_then(|meta| meta.get("embedding"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                "embedding"
            } else {
                "chat"
            };

            Some(ModelDescriptor {
                id: format!("runtime/llamacpp/{id}"),
                display_name: id.to_string(),
                provider: "llamacpp".to_string(),
                capability: capability.to_string(),
                location: ModelLocation::Local,
            })
        })
        .collect()
}

pub(crate) fn parse_openai_chat_response(
    request_id: &str,
    requested_model: &str,
    value: Value,
) -> Result<ChatCompletionResult, EngineError> {
    let model = value
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(requested_model)
        .to_string();
    let content = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let finish_reason = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("finish_reason"))
        .and_then(Value::as_str)
        .unwrap_or("stop")
        .to_string();
    let input_tokens = value
        .get("usage")
        .and_then(|usage| usage.get("prompt_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let output_tokens = value
        .get("usage")
        .and_then(|usage| usage.get("completion_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or_else(|| token_estimate(&content));

    Ok(ChatCompletionResult {
        id: request_id.to_string(),
        model,
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage::assistant(content),
            finish_reason,
        }],
        usage: UsageStats::new(input_tokens, output_tokens),
    })
}

pub(crate) fn parse_openai_chat_sse_reader<R: BufRead>(
    reader: R,
    request_id: &str,
    model: &str,
    provider: &str,
) -> EngineEventStream {
    let mut events = Vec::new();
    let mut output_tokens = 0;
    let mut done = false;

    for line in reader.lines() {
        let Ok(line) = line else {
            events.push(EngineEvent::Error {
                error: provider_error(request_id, "Failed to read OpenAI-compatible stream"),
            });
            return events;
        };
        let line = line.trim();
        if line.is_empty() || line.starts_with("event:") {
            continue;
        }

        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data == "[DONE]" {
            if !done {
                push_done(&mut events, request_id, model, provider, 0, output_tokens);
                done = true;
            }
            continue;
        }

        let Ok(value) = serde_json::from_str::<Value>(data) else {
            events.push(EngineEvent::Warning {
                request_id: request_id.to_string(),
                message: "Ignored malformed OpenAI-compatible stream line".to_string(),
            });
            continue;
        };

        if let Some(delta) = value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("delta"))
            .and_then(|delta| delta.get("content"))
            .and_then(Value::as_str)
            .filter(|delta| !delta.is_empty())
        {
            output_tokens += token_estimate(delta);
            events.push(EngineEvent::Token {
                request_id: request_id.to_string(),
                delta: delta.to_string(),
            });
        }

        if let Some(usage) = value.get("usage").filter(|usage| !usage.is_null()) {
            let input_tokens = usage
                .get("prompt_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let completion_tokens = usage
                .get("completion_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(output_tokens);
            push_done(
                &mut events,
                request_id,
                model,
                provider,
                input_tokens,
                completion_tokens,
            );
            done = true;
        }
    }

    if !done {
        push_done(&mut events, request_id, model, provider, 0, output_tokens);
    }

    events
}

fn push_done(
    events: &mut EngineEventStream,
    request_id: &str,
    model: &str,
    provider: &str,
    input_tokens: u64,
    output_tokens: u64,
) {
    events.push(EngineEvent::Metadata {
        request_id: request_id.to_string(),
        metadata: json!({
            "model": model,
            "provider": provider,
        }),
    });
    events.push(EngineEvent::Done {
        request_id: request_id.to_string(),
        usage: Some(UsageStats::new(input_tokens, output_tokens)),
    });
}
