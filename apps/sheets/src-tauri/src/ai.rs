use serde::{Deserialize, Serialize};
use tench_engine_core::{office_engine_status, run_office_chat, OFFICE_CHAT_MODEL};
use tench_shared_types::{ChatCompletionParams, ChatMessage};

/// Sheets-specific system prompts for different AI tasks
pub mod prompts {
    const SYSTEM_BASE: &str =
        "You are Tench Sheets AI, a data assistant integrated into the Tench Sheets spreadsheet editor. \
        You help users analyze data, generate formulas, create charts, and work with spreadsheets. \
        Respond concisely and directly. When suggesting formulas, provide the exact formula syntax.";

    pub fn data_assistant() -> String {
        SYSTEM_BASE.to_string()
    }

    pub fn formula_generation() -> String {
        format!("{SYSTEM_BASE} Focus on generating correct spreadsheet formulas. Provide the formula and explain what it does.")
    }

    pub fn data_analysis() -> String {
        format!("{SYSTEM_BASE} Analyze the provided data. Identify patterns, trends, outliers, and provide insights.")
    }

    pub fn chart_recommendation() -> String {
        format!("{SYSTEM_BASE} Recommend the best chart type for the given data. Explain why and describe how to set it up.")
    }

    pub fn data_cleaning() -> String {
        format!("{SYSTEM_BASE} Suggest ways to clean and normalize the data. Provide specific transformations.")
    }

    pub fn translate(target_lang: &str) -> String {
        format!("{SYSTEM_BASE} Translate the given spreadsheet data or formulas to {target_lang}.")
    }
}

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

fn build_chat_params(request: &AiChatRequest) -> ChatCompletionParams {
    let system_prompt = match request.task_type.as_deref() {
        Some("formula") => prompts::formula_generation(),
        Some("analysis") => prompts::data_analysis(),
        Some("chart") => prompts::chart_recommendation(),
        Some("cleaning") => prompts::data_cleaning(),
        Some("translate_ko") => prompts::translate("Korean"),
        Some("translate_en") => prompts::translate("English"),
        _ => prompts::data_assistant(),
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

pub fn handle_ai_chat(request: AiChatRequest) -> Result<AiChatResponse, String> {
    let params = build_chat_params(&request);
    let result = run_office_chat("sheets_ai", params)?;

    Ok(AiChatResponse {
        message: AiChatMessage {
            role: "assistant".to_string(),
            content: result.content,
        },
        model: result.model,
    })
}

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

pub fn get_engine_status() -> EngineStatusInfo {
    let status = office_engine_status("mock", OFFICE_CHAT_MODEL);

    EngineStatusInfo {
        available: status.available,
        provider: status.provider,
        model: status.model,
    }
}
