use std::io::BufReader;

use serde_json::{json, Value};
use tench_shared_types::{
    ChatCompletionParams, ChatCompletionResult, EngineError, EngineErrorType, EngineEvent,
    ModelDescriptor, ModelLocation, ProviderDescriptor, ProviderStatus,
};

use crate::providers::openai_compat::{parse_openai_chat_response, parse_openai_chat_sse_reader};
use crate::util::provider_error;
use crate::wire::{endpoint_url, http_post_json_with_auth, ureq_agent_with_auth};
use crate::{EngineEventStream, EngineProvider};

// ---------------------------------------------------------------------------
// Anthropic (Claude) — uses its own Messages API format
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
}

#[allow(dead_code)]
impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }
}

impl EngineProvider for AnthropicProvider {
    fn provider(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: "anthropic".to_string(),
            display_name: "Anthropic".to_string(),
            status: ProviderStatus::Available,
            location: ModelLocation::Cloud,
        }
    }

    fn list_models(&self) -> Vec<ModelDescriptor> {
        vec![
            ModelDescriptor {
                id: "cloud/anthropic/claude-sonnet-4-20250514".into(),
                display_name: "Claude Sonnet 4".into(),
                provider: "anthropic".into(),
                capability: "chat".into(),
                location: ModelLocation::Cloud,
            },
            ModelDescriptor {
                id: "cloud/anthropic/claude-3-5-sonnet-20241022".into(),
                display_name: "Claude 3.5 Sonnet".into(),
                provider: "anthropic".into(),
                capability: "chat".into(),
                location: ModelLocation::Cloud,
            },
            ModelDescriptor {
                id: "cloud/anthropic/claude-3-haiku-20240307".into(),
                display_name: "Claude 3 Haiku".into(),
                provider: "anthropic".into(),
                capability: "chat".into(),
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
            .strip_prefix("cloud/anthropic/")
            .unwrap_or(&params.model);
        let system_msg = params
            .messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone())
            .unwrap_or_default();
        let messages: Vec<Value> = params
            .messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|msg| json!({"role": msg.role, "content": msg.content}))
            .collect();

        let payload = json!({ "model": model, "max_tokens": 4096, "system": system_msg, "messages": messages });
        let url = endpoint_url(&self.base_url, "/messages");
        let value: Value = ureq::AgentBuilder::new()
            .build()
            .post(&url)
            .set("x-api-key", &self.api_key)
            .set("anthropic-version", "2023-06-01")
            .set("content-type", "application/json")
            .send_json(payload)
            .map_err(|e| provider_error(request_id, format!("Anthropic request failed: {e}")))?
            .into_json()
            .map_err(|e| provider_error(request_id, format!("Anthropic parse failed: {e}")))?;

        let content = value
            .get("content")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(|c| c.get("text"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let input_tokens = value
            .get("usage")
            .and_then(|u| u.get("input_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let output_tokens = value
            .get("usage")
            .and_then(|u| u.get("output_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0);

        Ok(ChatCompletionResult {
            id: request_id.to_string(),
            model: model.to_string(),
            choices: vec![tench_shared_types::ChatChoice {
                index: 0,
                message: tench_shared_types::ChatMessage::assistant(content.to_string()),
                finish_reason: "stop".into(),
            }],
            usage: tench_shared_types::UsageStats::new(input_tokens, output_tokens),
        })
    }

    fn stream_chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> EngineEventStream {
        match self.chat_completion(request_id, params) {
            Ok(result) => {
                let content = result
                    .choices
                    .first()
                    .map(|c| c.message.content.clone())
                    .unwrap_or_default();
                vec![
                    EngineEvent::Token {
                        request_id: request_id.to_string(),
                        delta: content,
                    },
                    EngineEvent::Done {
                        request_id: request_id.to_string(),
                        usage: Some(result.usage),
                    },
                ]
            }
            Err(e) => vec![EngineEvent::Error { error: e }],
        }
    }

    fn text_completion(&self, request_id: &str, _params: Value) -> Result<Value, EngineError> {
        Err(EngineError::new(
            "not_supported",
            "Anthropic does not support text completions",
            EngineErrorType::InvalidRequest,
            request_id,
            false,
        ))
    }

    fn embeddings(&self, request_id: &str, _params: Value) -> Result<Value, EngineError> {
        Err(EngineError::new(
            "not_supported",
            "Anthropic does not support embeddings",
            EngineErrorType::InvalidRequest,
            request_id,
            false,
        ))
    }

    fn usage_stats(&self) -> Value {
        json!({"provider": "anthropic"})
    }
}

// ---------------------------------------------------------------------------
// Google Gemini — uses generateContent API
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct GeminiProvider {
    api_key: String,
}

#[allow(dead_code)]
impl GeminiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }
}

impl EngineProvider for GeminiProvider {
    fn provider(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: "gemini".into(),
            display_name: "Google Gemini".into(),
            status: ProviderStatus::Available,
            location: ModelLocation::Cloud,
        }
    }

    fn list_models(&self) -> Vec<ModelDescriptor> {
        vec![
            ModelDescriptor {
                id: "cloud/gemini/gemini-2.0-flash".into(),
                display_name: "Gemini 2.0 Flash".into(),
                provider: "gemini".into(),
                capability: "chat".into(),
                location: ModelLocation::Cloud,
            },
            ModelDescriptor {
                id: "cloud/gemini/gemini-1.5-pro".into(),
                display_name: "Gemini 1.5 Pro".into(),
                provider: "gemini".into(),
                capability: "chat".into(),
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
            .strip_prefix("cloud/gemini/")
            .unwrap_or(&params.model);
        let contents: Vec<Value> = params
            .messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| {
                let role = if m.role == "assistant" {
                    "model"
                } else {
                    "user"
                };
                json!({"role": role, "parts": [{"text": m.content}]})
            })
            .collect();
        let mut payload = json!({"contents": contents});
        if let Some(sys) = params.messages.iter().find(|m| m.role == "system") {
            payload["systemInstruction"] = json!({"parts": [{"text": sys.content}]});
        }
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={}", self.api_key);
        let value: Value = ureq::agent()
            .post(&url)
            .send_json(payload)
            .map_err(|e| provider_error(request_id, format!("Gemini request failed: {e}")))?
            .into_json()
            .map_err(|e| provider_error(request_id, format!("Gemini parse failed: {e}")))?;
        let content = value
            .get("candidates")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(Value::as_array)
            .and_then(|p| p.first())
            .and_then(|p| p.get("text"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        Ok(ChatCompletionResult {
            id: request_id.to_string(),
            model: model.to_string(),
            choices: vec![tench_shared_types::ChatChoice {
                index: 0,
                message: tench_shared_types::ChatMessage::assistant(content.to_string()),
                finish_reason: "stop".into(),
            }],
            usage: tench_shared_types::UsageStats::new(0, 0),
        })
    }

    fn stream_chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> EngineEventStream {
        match self.chat_completion(request_id, params) {
            Ok(result) => {
                let c = result
                    .choices
                    .first()
                    .map(|c| c.message.content.clone())
                    .unwrap_or_default();
                vec![
                    EngineEvent::Token {
                        request_id: request_id.to_string(),
                        delta: c,
                    },
                    EngineEvent::Done {
                        request_id: request_id.to_string(),
                        usage: Some(result.usage),
                    },
                ]
            }
            Err(e) => vec![EngineEvent::Error { error: e }],
        }
    }

    fn text_completion(&self, request_id: &str, _params: Value) -> Result<Value, EngineError> {
        Err(EngineError::new(
            "not_supported",
            "Use chat_completion for Gemini",
            EngineErrorType::InvalidRequest,
            request_id,
            false,
        ))
    }

    fn embeddings(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        let model = params
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("text-embedding-004");
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:embedContent?key={}",
            self.api_key
        );
        ureq::agent()
            .post(&url)
            .send_json(params)
            .map_err(|e| provider_error(request_id, format!("Gemini embed failed: {e}")))?
            .into_json()
            .map_err(|e| provider_error(request_id, format!("Gemini embed parse: {e}")))
    }

    fn usage_stats(&self) -> Value {
        json!({"provider": "gemini"})
    }
}

// ---------------------------------------------------------------------------
// OpenAI-compatible cloud providers (macro-generated)
// ---------------------------------------------------------------------------

macro_rules! openai_compat_provider {
    ($name:ident, $id:literal, $display:literal, $base:literal, $models:expr) => {
        #[derive(Clone, Debug)]
        #[allow(dead_code)]
        pub struct $name { api_key: String }

        #[allow(dead_code)]
        impl $name {
            pub fn new(api_key: impl Into<String>) -> Self { Self { api_key: api_key.into() } }
        }

        impl EngineProvider for $name {
            fn provider(&self) -> ProviderDescriptor {
                ProviderDescriptor { id: $id.into(), display_name: $display.into(), status: ProviderStatus::Available, location: ModelLocation::Cloud }
            }

            fn list_models(&self) -> Vec<ModelDescriptor> { $models }

            fn chat_completion(&self, request_id: &str, params: ChatCompletionParams) -> Result<ChatCompletionResult, EngineError> {
                let model = params.model.strip_prefix(&format!("cloud/{}/", $id)).unwrap_or(&params.model);
                let messages: Vec<Value> = params.messages.iter().map(|m| json!({"role": m.role, "content": m.content})).collect();
                let payload = json!({"model": model, "messages": messages});
                let value = http_post_json_with_auth($base, "/chat/completions", payload, &self.api_key, None)
                    .map_err(|e| provider_error(request_id, format!("{} request failed: {e}", $display)))?;
                parse_openai_chat_response(request_id, &params.model, value)
            }

            fn stream_chat_completion(&self, request_id: &str, params: ChatCompletionParams) -> EngineEventStream {
                let model = params.model.strip_prefix(&format!("cloud/{}/", $id)).unwrap_or(&params.model);
                let messages: Vec<Value> = params.messages.iter().map(|m| json!({"role": m.role, "content": m.content})).collect();
                let payload = json!({"model": model, "messages": messages, "stream": true});
                match ureq_agent_with_auth(&self.api_key, None)
                    .post(&endpoint_url($base, "/chat/completions"))
                    .set("Authorization", &format!("Bearer {}", self.api_key))
                    .send_json(payload)
                {
                    Ok(response) => parse_openai_chat_sse_reader(BufReader::new(response.into_reader()), request_id, &params.model, $id),
                    Err(e) => vec![EngineEvent::Error { error: provider_error(request_id, format!("{} stream failed: {e}", $display)) }],
                }
            }

            fn text_completion(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
                http_post_json_with_auth($base, "/completions", params, &self.api_key, None)
                    .map_err(|e| provider_error(request_id, format!("{} completion failed: {e}", $display)))
            }

            fn embeddings(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
                http_post_json_with_auth($base, "/embeddings", params, &self.api_key, None)
                    .map_err(|e| provider_error(request_id, format!("{} embeddings failed: {e}", $display)))
            }

            fn usage_stats(&self) -> Value { json!({"provider": $id, "base_url": $base}) }
        }
    };
}

openai_compat_provider!(
    MistralProvider,
    "mistral",
    "Mistral AI",
    "https://api.mistral.ai/v1",
    vec![
        ModelDescriptor {
            id: "cloud/mistral/mistral-large-latest".into(),
            display_name: "Mistral Large".into(),
            provider: "mistral".into(),
            capability: "chat".into(),
            location: ModelLocation::Cloud
        },
        ModelDescriptor {
            id: "cloud/mistral/mistral-small-latest".into(),
            display_name: "Mistral Small".into(),
            provider: "mistral".into(),
            capability: "chat".into(),
            location: ModelLocation::Cloud
        },
    ]
);

openai_compat_provider!(
    CohereProvider,
    "cohere",
    "Cohere",
    "https://api.cohere.ai/v2",
    vec![ModelDescriptor {
        id: "cloud/cohere/command-r-plus".into(),
        display_name: "Command R+".into(),
        provider: "cohere".into(),
        capability: "chat".into(),
        location: ModelLocation::Cloud
    },]
);

openai_compat_provider!(
    TogetherProvider,
    "together",
    "Together AI",
    "https://api.together.xyz/v1",
    vec![ModelDescriptor {
        id: "cloud/together/meta-llama/Llama-3-70b-chat-hf".into(),
        display_name: "Llama 3 70B".into(),
        provider: "together".into(),
        capability: "chat".into(),
        location: ModelLocation::Cloud
    },]
);

openai_compat_provider!(
    FireworksProvider,
    "fireworks",
    "Fireworks AI",
    "https://api.fireworks.ai/inference/v1",
    vec![ModelDescriptor {
        id: "cloud/fireworks/llama-v3-70b-instruct".into(),
        display_name: "Llama 3 70B".into(),
        provider: "fireworks".into(),
        capability: "chat".into(),
        location: ModelLocation::Cloud
    },]
);

openai_compat_provider!(
    GroqProvider,
    "groq",
    "Groq",
    "https://api.groq.com/openai/v1",
    vec![
        ModelDescriptor {
            id: "cloud/groq/llama-3.1-70b-versatile".into(),
            display_name: "Llama 3.1 70B".into(),
            provider: "groq".into(),
            capability: "chat".into(),
            location: ModelLocation::Cloud
        },
        ModelDescriptor {
            id: "cloud/groq/mixtral-8x7b-32768".into(),
            display_name: "Mixtral 8x7B".into(),
            provider: "groq".into(),
            capability: "chat".into(),
            location: ModelLocation::Cloud
        },
    ]
);

openai_compat_provider!(
    PerplexityProvider,
    "perplexity",
    "Perplexity",
    "https://api.perplexity.ai",
    vec![ModelDescriptor {
        id: "cloud/perplexity/llama-3.1-sonar-large-128k-online".into(),
        display_name: "Sonar Large".into(),
        provider: "perplexity".into(),
        capability: "chat".into(),
        location: ModelLocation::Cloud
    },]
);

openai_compat_provider!(
    DeepSeekProvider,
    "deepseek",
    "DeepSeek",
    "https://api.deepseek.com/v1",
    vec![
        ModelDescriptor {
            id: "cloud/deepseek/deepseek-chat".into(),
            display_name: "DeepSeek Chat".into(),
            provider: "deepseek".into(),
            capability: "chat".into(),
            location: ModelLocation::Cloud
        },
        ModelDescriptor {
            id: "cloud/deepseek/deepseek-coder".into(),
            display_name: "DeepSeek Coder".into(),
            provider: "deepseek".into(),
            capability: "chat".into(),
            location: ModelLocation::Cloud
        },
    ]
);
