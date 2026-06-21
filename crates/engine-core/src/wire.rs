use std::io::BufRead;
use std::time::Duration;

use serde_json::{json, Value};
use tench_shared_types::{EngineError, EngineErrorType, EngineEvent, EngineResponse};

use crate::EngineEventStream;

pub(crate) fn transport_error(request_id: &str, transport: &str) -> EngineError {
    transport_error_with_message(
        request_id,
        transport,
        format!("{transport} transport is not connected"),
    )
}

pub(crate) fn transport_error_with_message(
    request_id: &str,
    transport: &str,
    message: impl Into<String>,
) -> EngineError {
    EngineError::new(
        "transport_unavailable",
        message.into(),
        EngineErrorType::RuntimeError,
        request_id,
        false,
    )
    .with_details(json!({ "transport": transport }))
}

pub(crate) fn transport_unavailable(request_id: &str, transport: &str) -> EngineResponse {
    EngineResponse::failure(request_id, transport_error(request_id, transport))
}

pub(crate) fn rpc_url(base_url: &str) -> String {
    format!("{}/rpc", base_url.trim_end_matches('/'))
}

pub(crate) fn endpoint_url(base_url: &str, path: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path)
}

pub(crate) fn http_get_json(base_url: &str, path: &str) -> Result<Value, String> {
    let request = ureq::get(&endpoint_url(base_url, path));
    let request = inject_common_headers(request);
    request
        .call()
        .map_err(|error| error.to_string())?
        .into_json::<Value>()
        .map_err(|error| error.to_string())
}

pub(crate) fn http_post_json(base_url: &str, path: &str, payload: Value) -> Result<Value, String> {
    let request = ureq::post(&endpoint_url(base_url, path));
    let request = inject_common_headers(request);
    request
        .send_json(payload)
        .map_err(|error| error.to_string())?
        .into_json::<Value>()
        .map_err(|error| error.to_string())
}

/// Extract a `Retry-After` header value (in seconds) from a ureq response.
#[allow(dead_code)]
pub(crate) fn extract_retry_after(response: &ureq::Response) -> Option<u64> {
    response
        .header("Retry-After")
        .and_then(|value| value.parse::<u64>().ok())
}

/// Build an `EngineError` from an HTTP error response, extracting the
/// `Retry-After` header when present.
#[allow(dead_code)]
pub(crate) fn http_error_from_response(
    request_id: &str,
    status: u16,
    response: ureq::Response,
) -> EngineError {
    let retry_after = extract_retry_after(&response);
    let body: Option<Value> = response.into_json::<Value>().ok();

    let (error_type, retryable) = match status {
        400 => (EngineErrorType::InvalidRequest, false),
        401 => (EngineErrorType::AuthenticationError, false),
        403 => (EngineErrorType::PermissionError, false),
        429 => (EngineErrorType::RateLimitError, true),
        _ if status >= 500 => (EngineErrorType::ProviderError, true),
        _ => (EngineErrorType::RuntimeError, false),
    };

    let message = body
        .as_ref()
        .and_then(|b| b.get("error"))
        .and_then(|e| e.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("HTTP error")
        .to_string();

    let code = body
        .as_ref()
        .and_then(|b| b.get("error"))
        .and_then(|e| e.get("code"))
        .and_then(Value::as_str)
        .unwrap_or("http_error")
        .to_string();

    let mut details = json!({
        "http_status": status,
    });

    if let Some(secs) = retry_after {
        details["retry_after_seconds"] = json!(secs);
    }

    EngineError::new(code, message, error_type, request_id, retryable).with_details(details)
}

/// Inject common Tench headers into a ureq request.
fn inject_common_headers(request: ureq::Request) -> ureq::Request {
    let app_id = TENCH_APP_ID.with(|cell| cell.borrow().clone());
    let request = match app_id {
        Some(app_id) => request.set("X-Tench-App", &app_id),
        None => request,
    };
    let request_id = TENCH_REQUEST_ID.with(|cell| cell.borrow().clone());
    match request_id {
        Some(request_id) => request.set("X-Tench-Request-Id", &request_id),
        None => request,
    }
}

thread_local! {
    pub(crate) static TENCH_APP_ID: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    pub(crate) static TENCH_REQUEST_ID: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Set the thread-local `X-Tench-App` header value for outgoing HTTP requests.
pub fn set_tench_app_id(app_id: Option<String>) {
    TENCH_APP_ID.with(|cell| *cell.borrow_mut() = app_id);
}

/// Set the thread-local `X-Tench-Request-Id` header value for outgoing HTTP requests.
pub fn set_tench_request_id(request_id: Option<String>) {
    TENCH_REQUEST_ID.with(|cell| *cell.borrow_mut() = request_id);
}

pub(crate) fn response_to_events(response: EngineResponse) -> EngineEventStream {
    match response.error {
        Some(error) => vec![EngineEvent::Error { error }],
        None => vec![
            EngineEvent::Metadata {
                request_id: response.id.clone(),
                metadata: response.result.unwrap_or_else(|| json!({})),
            },
            EngineEvent::Done {
                request_id: response.id,
                usage: None,
            },
        ],
    }
}

pub(crate) fn parse_sse_reader<R: BufRead>(
    reader: R,
    fallback_request_id: &str,
) -> EngineEventStream {
    let mut events = Vec::new();
    let mut current_event = String::new();
    let mut current_data = String::new();

    for line in reader.lines() {
        let Ok(line) = line else {
            events.push(EngineEvent::Error {
                error: transport_error_with_message(
                    fallback_request_id,
                    "sse",
                    "Failed to read SSE stream",
                ),
            });
            return events;
        };

        if line.is_empty() {
            push_sse_event(
                &mut events,
                &current_event,
                &current_data,
                fallback_request_id,
            );
            current_event.clear();
            current_data.clear();
            continue;
        }

        if let Some(event) = line.strip_prefix("event:") {
            current_event = event.trim().to_string();
        } else if let Some(data) = line.strip_prefix("data:") {
            if !current_data.is_empty() {
                current_data.push('\n');
            }
            current_data.push_str(data.trim());
        }
    }

    push_sse_event(
        &mut events,
        &current_event,
        &current_data,
        fallback_request_id,
    );
    events
}

pub(crate) fn ureq_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .build()
}

/// Build a ureq agent configured for cloud provider requests.
#[allow(dead_code)]
pub(crate) fn ureq_agent_with_auth(_api_key: &str, _organization: Option<&str>) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(60))
        .build()
}

/// HTTP POST with Bearer auth for cloud providers.
#[allow(dead_code)]
pub(crate) fn http_post_json_with_auth(
    base_url: &str,
    path: &str,
    payload: Value,
    api_key: &str,
    organization: Option<&str>,
) -> Result<Value, String> {
    let url = endpoint_url(base_url, path);
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(60))
        .build();
    let mut request = agent
        .post(&url)
        .set("Authorization", &format!("Bearer {api_key}"));
    if let Some(org) = organization {
        request = request.set("OpenAI-Organization", org);
    }
    request
        .send_json(payload)
        .map_err(|e| e.to_string())?
        .into_json::<Value>()
        .map_err(|e| e.to_string())
}

fn push_sse_event(
    events: &mut EngineEventStream,
    event_name: &str,
    data: &str,
    fallback_request_id: &str,
) {
    if event_name.is_empty() || data.is_empty() {
        return;
    }

    let parsed = serde_json::from_str::<Value>(data)
        .ok()
        .and_then(|value| sse_value_to_engine_event(event_name, value));

    match parsed {
        Some(event) => events.push(event),
        None => events.push(EngineEvent::Warning {
            request_id: fallback_request_id.to_string(),
            message: format!("Ignored malformed SSE event: {event_name}"),
        }),
    }
}

fn sse_value_to_engine_event(event_name: &str, value: Value) -> Option<EngineEvent> {
    match event_name {
        "token" => Some(EngineEvent::Token {
            request_id: value.get("request_id")?.as_str()?.to_string(),
            delta: value.get("delta")?.as_str()?.to_string(),
        }),
        "thinking" => Some(EngineEvent::Thinking {
            request_id: value.get("request_id")?.as_str()?.to_string(),
            delta: value.get("delta")?.as_str()?.to_string(),
        }),
        "tool_call" => Some(EngineEvent::ToolCall {
            request_id: value.get("request_id")?.as_str()?.to_string(),
            call: value.get("call")?.clone(),
        }),
        "progress" => Some(EngineEvent::Progress {
            task_id: value.get("task_id")?.as_str()?.to_string(),
            request_id: value.get("request_id")?.as_str()?.to_string(),
            stage: value.get("stage")?.as_str()?.to_string(),
            progress: value.get("progress")?.as_f64()? as f32,
            message: value.get("message")?.as_str()?.to_string(),
            eta_ms: value.get("eta_ms").and_then(Value::as_u64),
        }),
        "metadata" => Some(EngineEvent::Metadata {
            request_id: value.get("request_id")?.as_str()?.to_string(),
            metadata: value.get("metadata")?.clone(),
        }),
        "warning" => Some(EngineEvent::Warning {
            request_id: value.get("request_id")?.as_str()?.to_string(),
            message: value.get("message")?.as_str()?.to_string(),
        }),
        "error" => Some(EngineEvent::Error {
            error: serde_json::from_value(value.get("error")?.clone()).ok()?,
        }),
        "done" => Some(EngineEvent::Done {
            request_id: value.get("request_id")?.as_str()?.to_string(),
            usage: value
                .get("usage")
                .filter(|usage| !usage.is_null())
                .and_then(|usage| serde_json::from_value(usage.clone()).ok()),
        }),
        _ => None,
    }
}
