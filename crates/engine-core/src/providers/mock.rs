use serde_json::{json, Value};
use tench_shared_types::{
    ChatChoice, ChatCompletionParams, ChatCompletionResult, ChatMessage, EngineError,
    EngineErrorType, EngineEvent, ModelDescriptor, ModelLocation, ProviderDescriptor,
    ProviderStatus, UsageStats,
};

use crate::util::{last_user_message, token_estimate};
use crate::{EngineEventStream, EngineProvider};

#[derive(Clone, Debug)]
pub struct MockProvider {
    provider: ProviderDescriptor,
    models: Vec<ModelDescriptor>,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self {
            provider: ProviderDescriptor {
                id: "mock".to_string(),
                display_name: "Mock Engine Provider".to_string(),
                status: ProviderStatus::Available,
                location: ModelLocation::Mock,
            },
            models: vec![
                ModelDescriptor {
                    id: "tench/chat".to_string(),
                    display_name: "Tench Mock Chat".to_string(),
                    provider: "mock".to_string(),
                    capability: "chat".to_string(),
                    location: ModelLocation::Mock,
                },
                ModelDescriptor {
                    id: "tench/code".to_string(),
                    display_name: "Tench Mock Code".to_string(),
                    provider: "mock".to_string(),
                    capability: "code".to_string(),
                    location: ModelLocation::Mock,
                },
                ModelDescriptor {
                    id: "tench/vision".to_string(),
                    display_name: "Tench Mock Vision".to_string(),
                    provider: "mock".to_string(),
                    capability: "vision".to_string(),
                    location: ModelLocation::Mock,
                },
            ],
        }
    }
}

impl EngineProvider for MockProvider {
    fn provider(&self) -> ProviderDescriptor {
        self.provider.clone()
    }

    fn list_models(&self) -> Vec<ModelDescriptor> {
        self.models.clone()
    }

    fn chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> Result<ChatCompletionResult, EngineError> {
        self.ensure_model(request_id, &params.model)?;
        let prompt = last_user_message(&params.messages);
        let content = if prompt.is_empty() {
            "mock response".to_string()
        } else {
            format!("mock response: {prompt}")
        };
        let usage = UsageStats::new(token_estimate(&prompt), token_estimate(&content));

        Ok(ChatCompletionResult {
            id: request_id.to_string(),
            model: params.model,
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage::assistant(content),
                finish_reason: "stop".to_string(),
            }],
            usage,
        })
    }

    fn stream_chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> EngineEventStream {
        match self.chat_completion(request_id, params) {
            Ok(result) => {
                let content = &result.choices[0].message.content;
                let mut events: Vec<EngineEvent> = content
                    .split_whitespace()
                    .map(|word| EngineEvent::Token {
                        request_id: request_id.to_string(),
                        delta: format!("{word} "),
                    })
                    .collect();

                events.push(EngineEvent::Metadata {
                    request_id: request_id.to_string(),
                    metadata: json!({
                        "model": result.model,
                        "provider": "mock",
                    }),
                });
                events.push(EngineEvent::Done {
                    request_id: request_id.to_string(),
                    usage: Some(result.usage),
                });
                events
            }
            Err(error) => vec![EngineEvent::Error { error }],
        }
    }

    fn text_completion(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        let model = params
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("tench/chat");
        self.ensure_model(request_id, model)?;

        let prompt = params.get("prompt").and_then(Value::as_str).unwrap_or("");
        let text = if prompt.is_empty() {
            "mock completion".to_string()
        } else {
            format!("mock completion: {prompt}")
        };

        Ok(json!({
            "id": request_id,
            "model": model,
            "choices": [{ "index": 0, "text": text, "finish_reason": "stop" }],
            "usage": UsageStats::new(token_estimate(prompt), token_estimate(&text)),
        }))
    }

    fn embeddings(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        let model = params
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("tench/chat");
        self.ensure_model(request_id, model)?;

        Ok(json!({
            "object": "list",
            "model": model,
            "data": [{
                "object": "embedding",
                "index": 0,
                "embedding": [0.1, 0.2, 0.3, 0.4]
            }],
            "usage": UsageStats::new(4, 0),
        }))
    }

    fn usage_stats(&self) -> Value {
        json!({
            "provider": "mock",
            "requests": 0,
            "input_tokens": 0,
            "output_tokens": 0,
            "cost_estimate_usd": 0.0,
        })
    }
}

impl MockProvider {
    fn ensure_model(&self, request_id: &str, model: &str) -> Result<(), EngineError> {
        if self.models.iter().any(|candidate| candidate.id == model) {
            return Ok(());
        }

        Err(EngineError::new(
            "model_not_found",
            format!("Model not found: {model}"),
            EngineErrorType::InvalidRequest,
            request_id,
            false,
        ))
    }
}
