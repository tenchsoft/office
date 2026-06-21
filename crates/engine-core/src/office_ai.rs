use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tench_shared_types::{ChatCompletionParams, EngineMethod, EngineRequest};

use crate::EngineRouter;

pub const OFFICE_CHAT_MODEL: &str = "tench/chat";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfficeChatResult {
    pub content: String,
    pub model: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfficeEngineStatus {
    pub available: bool,
    pub provider: String,
    pub model: String,
}

pub fn run_office_chat(
    request_prefix: &str,
    params: ChatCompletionParams,
) -> Result<OfficeChatResult, String> {
    let router = EngineRouter::default();
    let response = router.call(EngineRequest::new(
        timestamped_request_id(request_prefix),
        EngineMethod::ChatCompletionsCreate,
        serde_json::to_value(&params).unwrap_or_default(),
    ));

    if let Some(error) = response.error {
        return Err(error.message);
    }

    let result = response.result.unwrap_or_default();
    let content = result
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or("No response generated.")
        .to_string();
    let model = result
        .get("model")
        .and_then(|model| model.as_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(OfficeChatResult { content, model })
}

pub fn office_engine_status(provider: &str, model: &str) -> OfficeEngineStatus {
    let router = EngineRouter::default();
    let response = router.call(EngineRequest::new(
        "status_check".to_string(),
        EngineMethod::ModelsList,
        json!({}),
    ));
    let model_available = response
        .result
        .as_ref()
        .and_then(|result| result.get("data"))
        .and_then(Value::as_array)
        .map(|models| {
            models.iter().any(|candidate| {
                candidate.get("id").and_then(Value::as_str) == Some(model)
                    && candidate.get("provider").and_then(Value::as_str) == Some(provider)
            })
        })
        .unwrap_or(false);

    OfficeEngineStatus {
        available: response.error.is_none() && model_available,
        provider: provider.to_string(),
        model: model.to_string(),
    }
}

fn timestamped_request_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{prefix}_{millis}")
}

#[cfg(test)]
mod tests {
    use tench_shared_types::ChatMessage;

    use super::*;

    #[test]
    fn office_chat_extracts_assistant_content() {
        let result = run_office_chat(
            "test_ai",
            ChatCompletionParams {
                model: OFFICE_CHAT_MODEL.to_string(),
                messages: vec![ChatMessage::user("hello")],
                stream: false,
            },
        )
        .expect("chat");

        assert_eq!(result.model, OFFICE_CHAT_MODEL);
        assert!(!result.content.is_empty());
    }

    #[test]
    fn office_engine_status_reports_mock_available() {
        let status = office_engine_status("mock", OFFICE_CHAT_MODEL);

        assert!(status.available);
        assert_eq!(status.provider, "mock");
        assert_eq!(status.model, OFFICE_CHAT_MODEL);
    }
}
