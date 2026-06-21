use std::io::BufReader;

use serde_json::{json, Value};
use tench_shared_types::{
    ChatCompletionParams, ChatCompletionResult, EngineError, EngineErrorType, EngineEvent,
    LocalRuntimeDescriptor, LocalRuntimeKind, LocalRuntimeStatus, ModelDescriptor, ModelLocation,
    ProviderDescriptor, ProviderStatus,
};

use crate::providers::openai_compat::{
    parse_llama_cpp_models, parse_openai_chat_response, parse_openai_chat_sse_reader,
};
use crate::util::provider_error;
use crate::wire::{endpoint_url, http_get_json, http_post_json, ureq_agent};
use crate::{detect_hardware_profile, EngineEventStream, EngineProvider};

#[derive(Clone, Debug)]
pub struct LlamaCppProvider {
    base_url: String,
}

impl Default for LlamaCppProvider {
    fn default() -> Self {
        Self::new("http://127.0.0.1:8080")
    }
}

impl LlamaCppProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub fn is_available(&self) -> bool {
        http_get_json(&self.base_url, "/v1/health").is_ok()
            || http_get_json(&self.base_url, "/health").is_ok()
            || http_get_json(&self.base_url, "/v1/models").is_ok()
    }

    pub fn runtime(&self) -> LocalRuntimeDescriptor {
        LocalRuntimeDescriptor {
            kind: LocalRuntimeKind::LlamaCpp,
            status: if self.is_available() {
                LocalRuntimeStatus::Available
            } else {
                LocalRuntimeStatus::Unavailable
            },
            endpoint: Some(self.base_url.clone()),
            version: None,
            hardware: Some(detect_hardware_profile()),
        }
    }

    fn llama_cpp_model_name(
        &self,
        request_id: &str,
        requested_model: &str,
    ) -> Result<String, EngineError> {
        if let Some(model_name) = requested_model.strip_prefix("runtime/llamacpp/") {
            return Ok(model_name.to_string());
        }
        if let Some(model_name) = requested_model.strip_prefix("local/llamacpp/") {
            return Ok(model_name.to_string());
        }

        if requested_model.starts_with("tench/") {
            return self
                .list_models()
                .first()
                .and_then(|model| {
                    model
                        .id
                        .strip_prefix("runtime/llamacpp/")
                        .map(str::to_string)
                })
                .ok_or_else(|| {
                    EngineError::new(
                        "model_not_found",
                        "No llama.cpp models are exposed by the configured server",
                        EngineErrorType::InvalidRequest,
                        request_id,
                        false,
                    )
                });
        }

        Ok(requested_model.to_string())
    }
}

impl EngineProvider for LlamaCppProvider {
    fn provider(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: "llamacpp".to_string(),
            display_name: "llama.cpp".to_string(),
            status: if self.is_available() {
                ProviderStatus::Available
            } else {
                ProviderStatus::Unavailable
            },
            location: ModelLocation::Local,
        }
    }

    fn list_models(&self) -> Vec<ModelDescriptor> {
        http_get_json(&self.base_url, "/v1/models")
            .ok()
            .map(parse_llama_cpp_models)
            .unwrap_or_default()
    }

    fn chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> Result<ChatCompletionResult, EngineError> {
        let model = self.llama_cpp_model_name(request_id, &params.model)?;
        let payload = json!({
            "model": model,
            "messages": params.messages.clone(),
            "stream": false,
        });
        let value =
            http_post_json(&self.base_url, "/v1/chat/completions", payload).map_err(|error| {
                provider_error(
                    request_id,
                    format!("llama.cpp chat request failed: {error}"),
                )
            })?;

        parse_openai_chat_response(request_id, &params.model, value)
    }

    fn stream_chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> EngineEventStream {
        let model = match self.llama_cpp_model_name(request_id, &params.model) {
            Ok(model) => model,
            Err(error) => return vec![EngineEvent::Error { error }],
        };
        let payload = json!({
            "model": model,
            "messages": params.messages.clone(),
            "stream": true,
            "stream_options": { "include_usage": true },
        });

        match ureq_agent()
            .post(&endpoint_url(&self.base_url, "/v1/chat/completions"))
            .send_json(payload)
        {
            Ok(response) => parse_openai_chat_sse_reader(
                BufReader::new(response.into_reader()),
                request_id,
                &params.model,
                "llamacpp",
            ),
            Err(error) => vec![EngineEvent::Error {
                error: provider_error(request_id, format!("llama.cpp stream failed: {error}")),
            }],
        }
    }

    fn text_completion(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        let requested_model = params
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("tench/chat");
        let model = self.llama_cpp_model_name(request_id, requested_model)?;
        let prompt = params.get("prompt").and_then(Value::as_str).unwrap_or("");
        let payload = json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });

        http_post_json(&self.base_url, "/v1/completions", payload).map_err(|error| {
            provider_error(request_id, format!("llama.cpp completion failed: {error}"))
        })
    }

    fn embeddings(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        let requested_model = params
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("tench/chat");
        let model = self.llama_cpp_model_name(request_id, requested_model)?;
        let input = params.get("input").cloned().unwrap_or_else(|| json!(""));
        let payload = json!({
            "model": model,
            "input": input,
        });

        http_post_json(&self.base_url, "/v1/embeddings", payload).map_err(|error| {
            provider_error(request_id, format!("llama.cpp embeddings failed: {error}"))
        })
    }

    fn usage_stats(&self) -> Value {
        json!({
            "provider": "llamacpp",
            "endpoint": self.base_url,
            "runtime": self.runtime(),
        })
    }
}
