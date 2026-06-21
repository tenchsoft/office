use serde::{Deserialize, Serialize};
use tench_engine_core::{office_engine_status, run_office_chat, OFFICE_CHAT_MODEL};
use tench_shared_types::{ChatCompletionParams, ChatMessage};
// ── Types ───────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AiChatRequest {
    pub messages: Vec<AiChatMessage>,
    pub task_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiSuggestionRequest {
    pub text: String,
    pub task_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiChatResponse {
    pub message: AiChatMessage,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineStatusInfo {
    pub available: bool,
    pub provider: String,
    pub model: String,
}

// ── Prompt builder ──────────────────────────────────────────────

fn build_system_prompt(task: &str) -> String {
    format!(
        "You are a presentation design assistant for Tench Slides. \
         Help with slide content, layout suggestions, and presentation structure. \
         Task: {task}"
    )
}

// ── AI functions ────────────────────────────────────────────────

fn build_chat_params(request: &AiChatRequest) -> ChatCompletionParams {
    let system_prompt = build_system_prompt(request.task_type.as_deref().unwrap_or("general_chat"));

    let mut messages = vec![ChatMessage::user(system_prompt)];
    for msg in &request.messages {
        if msg.role == "user" {
            messages.push(ChatMessage::user(&msg.content));
        } else {
            messages.push(ChatMessage::assistant(&msg.content));
        }
    }

    ChatCompletionParams {
        model: OFFICE_CHAT_MODEL.to_string(),
        messages,
        stream: false,
    }
}

pub fn chat(request: AiChatRequest) -> Result<AiChatResponse, String> {
    let params = build_chat_params(&request);

    let result = run_office_chat("slides_ai", params)?;

    Ok(AiChatResponse {
        message: AiChatMessage {
            role: "assistant".to_string(),
            content: result.content,
        },
        model: result.model,
    })
}

pub fn suggestion(request: AiSuggestionRequest) -> Result<String, String> {
    let chat_request = AiChatRequest {
        messages: vec![AiChatMessage {
            role: "user".to_string(),
            content: request.text,
        }],
        task_type: Some(request.task_type),
    };

    let response = chat(chat_request)?;
    Ok(response.message.content)
}

pub fn get_status() -> Result<EngineStatusInfo, String> {
    let status = office_engine_status("mock", OFFICE_CHAT_MODEL);

    Ok(EngineStatusInfo {
        available: status.available,
        provider: status.provider,
        model: status.model,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_returns_common_message_response_shape() {
        let response = chat(AiChatRequest {
            messages: vec![AiChatMessage {
                role: "user".to_string(),
                content: "Draft an intro slide".to_string(),
            }],
            task_type: Some("slide_generation".to_string()),
        })
        .expect("chat");

        assert_eq!(response.message.role, "assistant");
        assert!(!response.message.content.is_empty());
        assert_eq!(response.model, OFFICE_CHAT_MODEL);
    }

    #[test]
    fn suggestion_uses_common_request_shape() {
        let response = suggestion(AiSuggestionRequest {
            text: "Make this slide clearer".to_string(),
            task_type: "design_recommendation".to_string(),
        })
        .expect("suggestion");

        assert!(!response.is_empty());
    }

    #[test]
    fn status_includes_provider() {
        let status = get_status().expect("status");

        assert!(status.available);
        assert_eq!(status.provider, "mock");
        assert_eq!(status.model, OFFICE_CHAT_MODEL);
    }
}
