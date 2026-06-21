use std::str::FromStr;

use serde_json::{json, Value};
use tench_shared_types::{
    parse_model_id, ChatCompletionParams, EngineError, EngineErrorType, EngineEvent, EngineMethod,
    EngineRequest, EngineResponse, ModelNamespace, TaskState,
};

use crate::providers::MockProvider;
use crate::tasks::task_status_from_params;
use crate::traits::{EngineEventStream, EngineProvider};
use crate::util::parse_json_params;
use crate::wire::response_to_events;
use crate::LocalProvider;

#[derive(Clone, Debug)]
pub struct EngineRouter<P = MockProvider> {
    provider: P,
}

impl<P> EngineRouter<P>
where
    P: EngineProvider,
{
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn call(&self, request: EngineRequest) -> EngineResponse {
        match self.dispatch(&request.id, request.method, request.params) {
            Ok(result) => EngineResponse::success(request.id, result),
            Err(error) => EngineResponse::failure(request.id, error),
        }
    }

    pub fn call_method(&self, id: &str, method_name: &str, params: Value) -> EngineResponse {
        let Ok(method) = EngineMethod::from_str(method_name) else {
            return EngineResponse::failure(
                id,
                EngineError::new(
                    "invalid_method",
                    format!("Unknown Engine method: {method_name}"),
                    EngineErrorType::InvalidRequest,
                    id,
                    false,
                ),
            );
        };

        self.call(EngineRequest::new(id, method, params))
    }

    pub fn stream(&self, request: EngineRequest) -> EngineEventStream {
        match request.method {
            EngineMethod::ChatCompletionsCreate => {
                let params = parse_chat_params(&request.id, request.params);
                match params {
                    Ok(params) => self.provider.stream_chat_completion(&request.id, params),
                    Err(error) => vec![EngineEvent::Error { error }],
                }
            }
            _ => response_to_events(self.call(request)),
        }
    }

    pub fn cancel(&self, task_id: &str) -> EngineResponse {
        self.call(EngineRequest::new(
            format!("cancel_{task_id}"),
            EngineMethod::TasksCancel,
            json!({ "task_id": task_id }),
        ))
    }

    fn dispatch(
        &self,
        request_id: &str,
        method: EngineMethod,
        params: Value,
    ) -> Result<Value, EngineError> {
        match method {
            EngineMethod::ModelsList => Ok(json!({
                "object": "list",
                "data": self.provider.list_models(),
            })),
            EngineMethod::ProvidersList => Ok(json!({
                "object": "list",
                "data": [self.provider.provider()],
            })),
            EngineMethod::ChatCompletionsCreate => {
                let params = parse_chat_params(request_id, params)?;
                Ok(json!(self.provider.chat_completion(request_id, params)?))
            }
            EngineMethod::CompletionsCreate => self.provider.text_completion(request_id, params),
            EngineMethod::EmbeddingsCreate => self.provider.embeddings(request_id, params),
            EngineMethod::TasksGet => Ok(json!(task_status_from_params(
                request_id,
                params,
                TaskState::Completed,
            )?)),
            EngineMethod::TasksCancel => Ok(json!(task_status_from_params(
                request_id,
                params,
                TaskState::Cancelled,
            )?)),
            EngineMethod::UsageStats => Ok(self.provider.usage_stats()),
        }
    }
}

impl Default for EngineRouter<MockProvider> {
    fn default() -> Self {
        Self::new(MockProvider::default())
    }
}

/// Multi-provider router that dispatches requests to the correct provider
/// based on the model ID namespace.
#[derive(Clone, Debug)]
pub struct MultiProviderRouter {
    providers: Vec<LocalProvider>,
    fallback: LocalProvider,
}

impl MultiProviderRouter {
    pub fn new(providers: Vec<LocalProvider>, fallback: LocalProvider) -> Self {
        Self {
            providers,
            fallback,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(vec![], LocalProvider::mock())
    }

    pub fn call(&self, request: EngineRequest) -> EngineResponse {
        match self.dispatch(&request.id, request.method, request.params) {
            Ok(result) => EngineResponse::success(request.id, result),
            Err(error) => EngineResponse::failure(request.id, error),
        }
    }

    pub fn stream(&self, request: EngineRequest) -> EngineEventStream {
        match request.method {
            EngineMethod::ChatCompletionsCreate => {
                let params = parse_chat_params(&request.id, request.params);
                match params {
                    Ok(params) => {
                        let provider = self.resolve_provider_for_model(&params.model);
                        provider.stream_chat_completion(&request.id, params)
                    }
                    Err(error) => vec![EngineEvent::Error { error }],
                }
            }
            _ => response_to_events(self.call(request)),
        }
    }

    pub fn cancel(&self, task_id: &str) -> EngineResponse {
        self.fallback.cancel(task_id)
    }

    fn dispatch(
        &self,
        request_id: &str,
        method: EngineMethod,
        params: Value,
    ) -> Result<Value, EngineError> {
        match method {
            EngineMethod::ModelsList => {
                let mut all_models = Vec::new();
                for provider in &self.providers {
                    all_models.extend(provider.list_models());
                }
                all_models.extend(self.fallback.list_models());
                Ok(json!({
                    "object": "list",
                    "data": all_models,
                }))
            }
            EngineMethod::ProvidersList => {
                let mut all_providers = Vec::new();
                for provider in &self.providers {
                    all_providers.push(provider.provider());
                }
                all_providers.push(self.fallback.provider());
                Ok(json!({
                    "object": "list",
                    "data": all_providers,
                }))
            }
            EngineMethod::ChatCompletionsCreate => {
                let params = parse_chat_params(request_id, params)?;
                let provider = self.resolve_provider_for_model(&params.model);
                Ok(json!(provider.chat_completion(request_id, params)?))
            }
            EngineMethod::CompletionsCreate => {
                let model = params
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or("tench/chat");
                let provider = self.resolve_provider_for_model(model);
                provider.text_completion(request_id, params)
            }
            EngineMethod::EmbeddingsCreate => {
                let model = params
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or("tench/chat");
                let provider = self.resolve_provider_for_model(model);
                provider.embeddings(request_id, params)
            }
            EngineMethod::TasksGet => Ok(json!(task_status_from_params(
                request_id,
                params,
                TaskState::Completed,
            )?)),
            EngineMethod::TasksCancel => Ok(json!(task_status_from_params(
                request_id,
                params,
                TaskState::Cancelled,
            )?)),
            EngineMethod::UsageStats => Ok(self.fallback.usage_stats()),
        }
    }

    fn resolve_provider_for_model(&self, model_id: &str) -> &LocalProvider {
        match parse_model_id(model_id) {
            Some((namespace, _name)) => self.provider_for_namespace(namespace, model_id),
            None => &self.fallback,
        }
    }

    fn provider_for_namespace(&self, namespace: ModelNamespace, model_id: &str) -> &LocalProvider {
        match namespace {
            ModelNamespace::LocalNative => self.find_provider("native"),
            ModelNamespace::Runtime => {
                // Extract adapter name from runtime/{adapter}/{name}
                if let Some(rest) = model_id.strip_prefix("runtime/") {
                    if let Some(slash) = rest.find('/') {
                        let adapter = &rest[..slash];
                        return match adapter {
                            "ollama" => self.find_provider("ollama"),
                            "llamacpp" => self.find_provider("llamacpp"),
                            _ => &self.fallback,
                        };
                    }
                }
                &self.fallback
            }
            // Cloud not implemented yet; fall through to fallback which will
            // return model_not_found since no provider has cloud models.
            ModelNamespace::Cloud => &self.fallback,
            ModelNamespace::TenchAlias => &self.fallback,
        }
    }

    fn find_provider(&self, id: &str) -> &LocalProvider {
        self.providers
            .iter()
            .find(|p| p.provider_id() == id)
            .unwrap_or(&self.fallback)
    }
}

pub(crate) fn parse_chat_params(
    request_id: &str,
    params: Value,
) -> Result<ChatCompletionParams, EngineError> {
    parse_json_params(request_id, params, "chat completion")
}
