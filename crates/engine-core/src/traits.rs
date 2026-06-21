use serde_json::Value;
use tench_shared_types::{
    ChatCompletionParams, ChatCompletionResult, ConnectionProfile, EngineError, EngineEvent,
    EngineRequest, EngineResponse, ModelDescriptor, ProviderDescriptor,
};

pub type EngineEventStream = Vec<EngineEvent>;

pub trait EngineClient {
    fn call(&self, request: EngineRequest) -> EngineResponse;
    fn stream(&self, request: EngineRequest) -> EngineEventStream;
    fn cancel(&self, task_id: &str) -> EngineResponse;
}

pub trait EngineTransport: EngineClient {
    fn profile(&self) -> ConnectionProfile;
}

pub trait EngineProvider {
    fn provider(&self) -> ProviderDescriptor;
    fn list_models(&self) -> Vec<ModelDescriptor>;
    fn chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> Result<ChatCompletionResult, EngineError>;
    fn stream_chat_completion(
        &self,
        request_id: &str,
        params: ChatCompletionParams,
    ) -> EngineEventStream;
    fn text_completion(&self, request_id: &str, params: Value) -> Result<Value, EngineError>;
    fn embeddings(&self, request_id: &str, params: Value) -> Result<Value, EngineError>;
    fn usage_stats(&self) -> Value;
}
