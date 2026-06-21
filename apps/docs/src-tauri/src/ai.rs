use serde::{Deserialize, Serialize};
use tench_engine_core::{office_engine_status, run_office_chat, OFFICE_CHAT_MODEL};
use tench_shared_types::{ChatCompletionParams, ChatMessage};

/// Docs-specific system prompts for different AI tasks
pub mod prompts {
    const SYSTEM_BASE: &str =
        "You are Tench Docs AI, a writing assistant integrated into the Tench Docs editor. \
        You help users write, edit, and improve their documents. \
        Respond concisely and directly. When suggesting edits, provide the specific text changes.";

    pub fn writing_assistant() -> String {
        SYSTEM_BASE.to_string()
    }

    pub fn grammar_check() -> String {
        format!("{SYSTEM_BASE} Focus on grammar, spelling, and punctuation errors. List each issue and suggest corrections.")
    }

    pub fn style_improvement() -> String {
        format!("{SYSTEM_BASE} Focus on improving writing style: clarity, conciseness, tone consistency, and readability.")
    }

    pub fn summarize() -> String {
        format!(
            "{SYSTEM_BASE} Provide a concise summary of the given text. Capture the key points."
        )
    }

    pub fn expand() -> String {
        format!("{SYSTEM_BASE} Expand the given text with more detail, examples, or explanations while maintaining the original tone.")
    }

    pub fn translate(target_lang: &str) -> String {
        format!("{SYSTEM_BASE} Translate the given text to {target_lang}. Maintain the original formatting and tone.")
    }
}

/// AI request types from the frontend
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

/// Engine status check
#[derive(Debug, Serialize, Deserialize)]
pub struct EngineStatusInfo {
    pub available: bool,
    pub provider: String,
    pub model: String,
}

/// Build chat completion params from a docs AI request
fn build_chat_params(request: &AiChatRequest) -> ChatCompletionParams {
    let system_prompt = match request.task_type.as_deref() {
        Some("grammar") => prompts::grammar_check(),
        Some("style") => prompts::style_improvement(),
        Some("summarize") => prompts::summarize(),
        Some("expand") => prompts::expand(),
        Some("translate_ko") => prompts::translate("Korean"),
        Some("translate_en") => prompts::translate("English"),
        _ => prompts::writing_assistant(),
    };

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

/// Handle AI chat request
pub fn handle_ai_chat(request: AiChatRequest) -> Result<AiChatResponse, String> {
    let params = build_chat_params(&request);
    let result = run_office_chat("docs_ai", params)?;

    Ok(AiChatResponse {
        message: AiChatMessage {
            role: "assistant".to_string(),
            content: result.content,
        },
        model: result.model,
    })
}

/// Handle AI suggestion request (single-shot)
pub fn handle_ai_suggestion(request: AiSuggestionRequest) -> Result<String, String> {
    let chat_request = AiChatRequest {
        messages: vec![AiChatMessage {
            role: "user".to_string(),
            content: request.text,
        }],
        task_type: Some(request.task_type),
    };

    let response = handle_ai_chat(chat_request)?;
    Ok(response.message.content)
}

/// Check engine status
pub fn get_engine_status() -> EngineStatusInfo {
    let status = office_engine_status("engine", OFFICE_CHAT_MODEL);

    EngineStatusInfo {
        available: status.available,
        provider: status.provider,
        model: status.model,
    }
}
