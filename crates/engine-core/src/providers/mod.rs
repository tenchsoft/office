pub(crate) mod cloud;
pub(crate) mod llama_cpp;
mod mock;
pub(crate) mod ollama;
pub(crate) mod ollama_codec;
pub(crate) mod openai;
pub(crate) mod openai_compat;

use serde_json::Value;
use tench_shared_types::{
    ChatCompletionParams, ChatCompletionResult, EngineError, LocalRuntimeDescriptor,
    LocalRuntimeKind, LocalRuntimeStatus,
};

#[allow(unused_imports)]
pub use cloud::{
    AnthropicProvider, CohereProvider, DeepSeekProvider, FireworksProvider, GeminiProvider,
    GroqProvider, MistralProvider, PerplexityProvider, TogetherProvider,
};
pub use llama_cpp::LlamaCppProvider;
pub use mock::MockProvider;
pub use ollama::OllamaProvider;
#[allow(unused_imports)]
pub use openai::OpenAIProvider;

use crate::{detect_hardware_profile, EngineEventStream, EngineProvider, NativeProvider};

#[derive(Clone, Debug)]
pub enum LocalProvider {
    Native(NativeProvider),
    Mock(MockProvider),
    Ollama(OllamaProvider),
    LlamaCpp(LlamaCppProvider),
}

impl LocalProvider {
    pub fn auto() -> Self {
        Self::Native(NativeProvider::default())
    }

    pub fn native() -> Self {
        Self::Native(NativeProvider::default())
    }

    pub fn mock() -> Self {
        Self::Mock(MockProvider::default())
    }

    pub fn ollama(base_url: impl Into<String>) -> Self {
        Self::Ollama(OllamaProvider::new(base_url))
    }

    pub fn llama_cpp(base_url: impl Into<String>) -> Self {
        Self::LlamaCpp(LlamaCppProvider::new(base_url))
    }

    /// Returns the provider ID string used for routing (e.g. "native", "ollama", "mock").
    pub fn provider_id(&self) -> &str {
        match self {
            Self::Native(_) => "native",
            Self::Mock(_) => "mock",
            Self::Ollama(_) => "ollama",
            Self::LlamaCpp(_) => "llamacpp",
        }
    }

    pub fn cancel(&self, task_id: &str) -> tench_shared_types::EngineResponse {
        crate::router::EngineRouter::new(self.clone()).cancel(task_id)
    }

    pub fn runtime(&self) -> LocalRuntimeDescriptor {
        match self {
            Self::Native(provider) => provider.runtime(),
            Self::Mock(_) => LocalRuntimeDescriptor {
                kind: LocalRuntimeKind::Mock,
                status: LocalRuntimeStatus::Available,
                endpoint: None,
                version: Some("mock".to_string()),
                hardware: Some(detect_hardware_profile()),
            },
            Self::Ollama(provider) => provider.runtime(),
            Self::LlamaCpp(provider) => provider.runtime(),
        }
    }
}

impl EngineProvider for LocalProvider {
    fn provider(&self) -> tench_shared_types::ProviderDescriptor {
        match self {
            Self::Native(provider) => provider.provider(),
            Self::Mock(provider) => provider.provider(),
            Self::Ollama(provider) => provider.provider(),
            Self::LlamaCpp(provider) => provider.provider(),
        }
    }

    fn list_models(&self) -> Vec<tench_shared_types::ModelDescriptor> {
        match self {
            Self::Native(provider) => provider.list_models(),
            Self::Mock(provider) => provider.list_models(),
            Self::Ollama(provider) => provider.list_models(),
            Self::LlamaCpp(provider) => provider.list_models(),
        }
    }

    fn chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> Result<ChatCompletionResult, EngineError> {
        match self {
            Self::Native(provider) => provider.chat_completion(request_id, params),
            Self::Mock(provider) => provider.chat_completion(request_id, params),
            Self::Ollama(provider) => provider.chat_completion(request_id, params),
            Self::LlamaCpp(provider) => provider.chat_completion(request_id, params),
        }
    }

    fn stream_chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> EngineEventStream {
        match self {
            Self::Native(provider) => provider.stream_chat_completion(request_id, params),
            Self::Mock(provider) => provider.stream_chat_completion(request_id, params),
            Self::Ollama(provider) => provider.stream_chat_completion(request_id, params),
            Self::LlamaCpp(provider) => provider.stream_chat_completion(request_id, params),
        }
    }

    fn text_completion(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        match self {
            Self::Native(provider) => provider.text_completion(request_id, params),
            Self::Mock(provider) => provider.text_completion(request_id, params),
            Self::Ollama(provider) => provider.text_completion(request_id, params),
            Self::LlamaCpp(provider) => provider.text_completion(request_id, params),
        }
    }

    fn embeddings(&self, request_id: &str, params: Value) -> Result<Value, EngineError> {
        match self {
            Self::Native(provider) => provider.embeddings(request_id, params),
            Self::Mock(provider) => provider.embeddings(request_id, params),
            Self::Ollama(provider) => provider.embeddings(request_id, params),
            Self::LlamaCpp(provider) => provider.embeddings(request_id, params),
        }
    }

    fn usage_stats(&self) -> Value {
        match self {
            Self::Native(provider) => provider.usage_stats(),
            Self::Mock(provider) => provider.usage_stats(),
            Self::Ollama(provider) => provider.usage_stats(),
            Self::LlamaCpp(provider) => provider.usage_stats(),
        }
    }
}
