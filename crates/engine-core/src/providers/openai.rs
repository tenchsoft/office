use std::io::BufReader;

use serde_json::{json, Value};
use tench_shared_types::{
    ChatCompletionParams, ChatCompletionResult, EngineError, EngineEvent, ModelDescriptor,
    ModelLocation, ProviderDescriptor, ProviderStatus,
};

use crate::providers::openai_compat::{parse_openai_chat_response, parse_openai_chat_sse_reader};
use crate::util::provider_error;
use crate::wire::{endpoint_url, http_post_json_with_auth, ureq_agent_with_auth};
use crate::{EngineEventStream, EngineProvider};

/// OpenAI API adapter (GPT-4, GPT-4o, GPT-3.5-turbo, etc.)
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    organization: Option<String>,
}

#[allow(dead_code)]
impl OpenAIProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".to_string(),
            organization: None,
        }
    }

    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    fn build_messages(&self, params: &ChatCompletionParams) -> Value {
        params
            .messages
            .iter()
            .map(|msg| {
                json!({
                    "role": msg.role,
                    "content": msg.content,
                })
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl EngineProvider for OpenAIProvider {
    fn provider(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            status: ProviderStatus::Available,
            location: ModelLocation::Cloud,
        }
    }

    fn list_models(&self) -> Vec<ModelDescriptor> {
        vec![
            ModelDescriptor {
                id: "cloud/openai/gpt-4o".to_string(),
                display_name: "GPT-4o".to_string(),
                provider: "openai".to_string(),
                capability: "chat".to_string(),
                location: ModelLocation::Cloud,
            },
            ModelDescriptor {
                id: "cloud/openai/gpt-4o-mini".to_string(),
                display_name: "GPT-4o Mini".to_string(),
                provider: "openai".to_string(),
                capability: "chat".to_string(),
                location: ModelLocation::Cloud,
            },
            ModelDescriptor {
                id: "cloud/openai/gpt-4-turbo".to_string(),
                display_name: "GPT-4 Turbo".to_string(),
                provider: "openai".to_string(),
                capability: "chat".to_string(),
                location: ModelLocation::Cloud,
            },
        ]
    }

    fn chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> Result<ChatCompletionResult, EngineError> {
        let model = params
            .model
            .strip_prefix("cloud/openai/")
            .unwrap_or(&params.model);
        let payload = json!({
            "model": model,
            "messages": self.build_messages(&params),
        });

        let value = http_post_json_with_auth(
            &self.base_url,
            "/chat/completions",
            payload,
            &self.api_key,
            self.organization.as_deref(),
        )
        .map_err(|e| provider_error(request_id, format!("OpenAI request failed: {e}")))?;

        parse_openai_chat_response(request_id, &params.model, value)
    }

    fn stream_chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> EngineEventStream {
        let model = params
            .model
            .strip_prefix("cloud/openai/")
            .unwrap_or(&params.model);
        let payload = json!({
            "model": model,
            "messages": self.build_messages(&params),
            "stream": true,
        });

        match ureq_agent_with_auth(&self.api_key, self.organization.as_deref())
            .post(&endpoint_url(&self.base_url, "/chat/completions"))
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .send_json(payload)
        {
            Ok(response) => parse_openai_chat_sse_reader(
                BufReader::new(response.into_reader()),
                request_id,
                &params.model,
                "openai",
            ),
            Err(e) => vec![EngineEvent::Error {
                error: provider_error(request_id, format!("OpenAI stream failed: {e}")),
            }],
        }
    }

    fn text_completion(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        http_post_json_with_auth(
            &self.base_url,
            "/completions",
            params,
            &self.api_key,
            self.organization.as_deref(),
        )
        .map_err(|e| provider_error(request_id, format!("OpenAI completion failed: {e}")))
    }

    fn embeddings(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        http_post_json_with_auth(
            &self.base_url,
            "/embeddings",
            params,
            &self.api_key,
            self.organization.as_deref(),
        )
        .map_err(|e| provider_error(request_id, format!("OpenAI embeddings failed: {e}")))
    }

    fn usage_stats(&self) -> Value {
        json!({ "provider": "openai", "base_url": self.base_url })
    }
}
