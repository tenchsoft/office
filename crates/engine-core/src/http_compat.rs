use serde::Serialize;
use serde_json::{json, Value};
use tench_shared_types::{
    is_retryable_error_type, EngineError, EngineErrorType, EngineMethod, EngineRequest,
    EngineResponse,
};

use crate::{EngineProvider, EngineRouter};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HttpCompatResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u64>,
}

pub fn map_http_compat<P>(
    router: &EngineRouter<P>,
    http_method: &str,
    path: &str,
    body: Value,
) -> HttpCompatResponse
where
    P: EngineProvider,
{
    let mapped = match (http_method, path) {
        ("GET", "/v1/models") => Some(EngineRequest::new(
            "http_models_list",
            EngineMethod::ModelsList,
            json!({}),
        )),
        ("POST", "/v1/chat/completions") => Some(EngineRequest::new(
            request_id_from_body(&body, "http_chat_completion"),
            EngineMethod::ChatCompletionsCreate,
            body,
        )),
        ("POST", "/v1/completions") => Some(EngineRequest::new(
            request_id_from_body(&body, "http_completion"),
            EngineMethod::CompletionsCreate,
            body,
        )),
        ("POST", "/v1/embeddings") => Some(EngineRequest::new(
            request_id_from_body(&body, "http_embeddings"),
            EngineMethod::EmbeddingsCreate,
            body,
        )),
        _ => map_task_request(http_method, path),
    };

    let response = match mapped {
        Some(request) => router.call(request),
        None => EngineResponse::failure(
            "http_route",
            EngineError::new(
                "invalid_request",
                format!("Unsupported HTTP compatibility route: {http_method} {path}"),
                EngineErrorType::InvalidRequest,
                "http_route",
                false,
            ),
        ),
    };

    http_response_from_engine(response)
}

pub(crate) fn http_response_from_engine(response: EngineResponse) -> HttpCompatResponse {
    match response.error {
        Some(error) => {
            let retry_after_seconds = if is_retryable_error_type(&error.error_type) {
                default_retry_after_for_error(&error)
            } else {
                None
            };
            HttpCompatResponse {
                status: status_for_error(&error),
                content_type: "application/json",
                body: json!({ "error": error }),
                retry_after_seconds,
            }
        }
        None => HttpCompatResponse {
            status: 200,
            content_type: "application/json",
            body: response.result.unwrap_or_else(|| json!({})),
            retry_after_seconds: None,
        },
    }
}

pub(crate) fn status_for_error(error: &EngineError) -> u16 {
    match error.error_type {
        EngineErrorType::InvalidRequest => 400,
        EngineErrorType::AuthenticationError => 401,
        EngineErrorType::PermissionError => 403,
        EngineErrorType::RateLimitError => 429,
        EngineErrorType::Cancelled => 499,
        EngineErrorType::ProviderError
        | EngineErrorType::RuntimeError
        | EngineErrorType::ResourceError
        | EngineErrorType::InternalError => 500,
    }
}

fn request_id_from_body(body: &Value, fallback: &str) -> String {
    body.get("id")
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_string()
}

/// Returns a reasonable default `Retry-After` value (in seconds) for retryable errors.
fn default_retry_after_for_error(error: &EngineError) -> Option<u64> {
    match error.error_type {
        EngineErrorType::RateLimitError => Some(1),
        EngineErrorType::ProviderError | EngineErrorType::InternalError => Some(5),
        _ => None,
    }
}

fn map_task_request(http_method: &str, path: &str) -> Option<EngineRequest> {
    let task_id = path
        .strip_prefix("/api/v1/tasks/")?
        .strip_suffix("/cancel")
        .or_else(|| path.strip_prefix("/api/v1/tasks/"))?;

    match http_method {
        "GET" if !path.ends_with("/cancel") => Some(EngineRequest::new(
            format!("http_task_get_{task_id}"),
            EngineMethod::TasksGet,
            json!({ "task_id": task_id }),
        )),
        "POST" if path.ends_with("/cancel") => Some(EngineRequest::new(
            format!("http_task_cancel_{task_id}"),
            EngineMethod::TasksCancel,
            json!({ "task_id": task_id }),
        )),
        _ => None,
    }
}
