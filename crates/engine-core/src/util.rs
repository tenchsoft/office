use serde_json::Value;
use tench_shared_types::{ChatMessage, EngineError, EngineErrorType};

pub(crate) fn parse_json_params<T>(
    request_id: &str,
    params: Value,
    label: &str,
) -> Result<T, EngineError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(params).map_err(|error| {
        EngineError::new(
            "invalid_request",
            format!("Invalid {label} params: {error}"),
            EngineErrorType::InvalidRequest,
            request_id,
            false,
        )
    })
}

pub(crate) fn last_user_message(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(|message| message.content.clone())
        .unwrap_or_default()
}

pub(crate) fn token_estimate(text: &str) -> u64 {
    text.split_whitespace().count().max(1) as u64
}

pub(crate) fn provider_error(request_id: &str, message: impl Into<String>) -> EngineError {
    EngineError::new(
        "provider_unavailable",
        message.into(),
        EngineErrorType::ProviderError,
        request_id,
        true,
    )
}
