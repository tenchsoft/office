use serde_json::{json, Value};
use tench_shared_types::{ConnectionProfile, EngineMethod, EngineRequest, EngineResponse};

use crate::wire::{
    http_get_json, http_post_json, transport_error_with_message, transport_unavailable,
};
use crate::{EngineClient, EngineEventStream, EngineTransport};

use super::stream::remote_sse_stream;

#[derive(Clone, Debug)]
pub struct HttpCompatTransport {
    base_url: String,
}

impl HttpCompatTransport {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

impl EngineClient for HttpCompatTransport {
    fn call(&self, request: EngineRequest) -> EngineResponse {
        let request_id = request.id.clone();
        let result = match request.method {
            EngineMethod::ModelsList => http_get_json(&self.base_url, "/v1/models"),
            EngineMethod::ChatCompletionsCreate => {
                http_post_json(&self.base_url, "/v1/chat/completions", request.params)
            }
            EngineMethod::CompletionsCreate => {
                http_post_json(&self.base_url, "/v1/completions", request.params)
            }
            EngineMethod::EmbeddingsCreate => {
                http_post_json(&self.base_url, "/v1/embeddings", request.params)
            }
            EngineMethod::TasksGet => {
                let task_id = request.params.get("task_id").and_then(Value::as_str);
                match task_id {
                    Some(task_id) => {
                        http_get_json(&self.base_url, &format!("/api/v1/tasks/{task_id}"))
                    }
                    None => Err("Missing task_id".to_string()),
                }
            }
            EngineMethod::TasksCancel => {
                let task_id = request.params.get("task_id").and_then(Value::as_str);
                match task_id {
                    Some(task_id) => http_post_json(
                        &self.base_url,
                        &format!("/api/v1/tasks/{task_id}/cancel"),
                        json!({}),
                    ),
                    None => Err("Missing task_id".to_string()),
                }
            }
            EngineMethod::ProvidersList => http_get_json(&self.base_url, "/api/v1/providers"),
            EngineMethod::UsageStats => http_get_json(&self.base_url, "/api/v1/usage/stats"),
        };

        match result {
            Ok(result) => EngineResponse::success(request_id, result),
            Err(error) => EngineResponse::failure(
                request_id.clone(),
                transport_error_with_message(&request_id, "http_compat", error),
            ),
        }
    }

    fn stream(&self, request: EngineRequest) -> EngineEventStream {
        remote_sse_stream(&self.base_url, request, "http_compat")
    }

    fn cancel(&self, task_id: &str) -> EngineResponse {
        transport_unavailable(&format!("cancel_{task_id}"), "http_compat")
    }
}

impl EngineTransport for HttpCompatTransport {
    fn profile(&self) -> ConnectionProfile {
        ConnectionProfile::HttpCompat {
            base_url: self.base_url.clone(),
        }
    }
}
