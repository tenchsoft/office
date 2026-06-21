use tench_document_core::{
    OfficeArtifact, OfficeContent, OfficeExportResponse, OfficeFileFormat, OfficeOpenResponse,
    OfficeRecentFile, OfficeRecoveryMetadata, OfficeSaveResponse,
};
use tench_office_io::watcher::OfficeFileWatcher;
use tench_office_runtime::{OfficeRuntimeProduct, SLIDES_RUNTIME};

use crate::ai;
use crate::config::SlidesConfig;
use crate::presentation_service;

// ── Presentation CRUD ───────────────────────────────────────────

#[tauri::command]
pub fn runtime_capabilities() -> OfficeRuntimeProduct {
    SLIDES_RUNTIME.clone()
}

#[tauri::command]
pub fn create_presentation(title: Option<String>) -> Result<OfficeOpenResponse, String> {
    Ok(presentation_service::create_presentation(title))
}

#[tauri::command]
pub fn open_presentation(path: String) -> Result<OfficeOpenResponse, String> {
    presentation_service::open_presentation(path)
}

#[tauri::command]
pub fn save_presentation(
    artifact: OfficeArtifact,
    content: OfficeContent,
    target_path: Option<String>,
    format: Option<OfficeFileFormat>,
) -> Result<OfficeSaveResponse, String> {
    presentation_service::save_presentation(artifact, content, target_path, format)
}

#[tauri::command]
pub fn save_presentation_as(
    artifact: OfficeArtifact,
    content: OfficeContent,
    target_path: String,
    format: Option<OfficeFileFormat>,
) -> Result<OfficeSaveResponse, String> {
    presentation_service::save_presentation(artifact, content, Some(target_path), format)
}

#[tauri::command]
pub fn import_presentation(
    source_path: String,
    source_format: Option<OfficeFileFormat>,
) -> Result<OfficeOpenResponse, String> {
    presentation_service::import_presentation(source_path, source_format)
}

#[tauri::command]
pub fn export_presentation(
    artifact_id: String,
    content: OfficeContent,
    target_format: OfficeFileFormat,
    output_path: String,
) -> Result<OfficeExportResponse, String> {
    presentation_service::export_presentation(artifact_id, content, target_format, output_path)
}

// ── Recent files ────────────────────────────────────────────────

#[tauri::command]
pub fn get_recent_presentations() -> Result<Vec<OfficeRecentFile>, String> {
    presentation_service::get_recent_presentations()
}

// ── Recovery ────────────────────────────────────────────────────

#[tauri::command]
pub fn save_recovery_snapshot(
    artifact: OfficeArtifact,
    content: OfficeContent,
) -> Result<OfficeRecoveryMetadata, String> {
    presentation_service::save_recovery_snapshot(artifact, content)
}

#[tauri::command]
pub fn get_recovery_presentations() -> Result<Vec<OfficeRecoveryMetadata>, String> {
    presentation_service::get_recovery_presentations()
}

#[tauri::command]
pub fn open_recovery_presentation(recovery_path: String) -> Result<OfficeOpenResponse, String> {
    presentation_service::open_recovery_presentation(recovery_path)
}

#[tauri::command]
pub fn delete_recovery_presentation(recovery_path: String) -> Result<(), String> {
    presentation_service::delete_recovery_presentation(recovery_path)
}

#[tauri::command]
pub fn clear_recovery_snapshots(artifact_id: String) -> Result<(), String> {
    presentation_service::clear_recovery_snapshots(artifact_id)
}

// ── Config ──────────────────────────────────────────────────────

#[tauri::command]
pub fn load_config() -> Result<SlidesConfig, String> {
    crate::config::load_config()
}

#[tauri::command]
pub fn save_config(config: SlidesConfig) -> Result<(), String> {
    crate::config::save_config(&config)
}

// ── AI ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn ai_chat(request: ai::AiChatRequest) -> Result<ai::AiChatResponse, String> {
    ai::chat(request)
}

#[tauri::command]
pub fn ai_suggestion(request: ai::AiSuggestionRequest) -> Result<String, String> {
    ai::suggestion(request)
}

#[tauri::command]
pub fn get_engine_status() -> Result<ai::EngineStatusInfo, String> {
    ai::get_status()
}

// ── File watcher ────────────────────────────────────────────────

#[tauri::command]
pub fn watch_file(
    path: String,
    watcher: tauri::State<'_, std::sync::Mutex<OfficeFileWatcher>>,
) -> Result<(), String> {
    watcher.lock().map_err(|e| e.to_string())?.watch(&path);
    Ok(())
}

#[tauri::command]
pub fn unwatch_file(
    path: String,
    watcher: tauri::State<'_, std::sync::Mutex<OfficeFileWatcher>>,
) -> Result<(), String> {
    watcher.lock().map_err(|e| e.to_string())?.unwatch(&path);
    Ok(())
}

#[tauri::command]
pub fn check_file_changes(
    watcher: tauri::State<'_, std::sync::Mutex<OfficeFileWatcher>>,
) -> Result<Vec<tench_fs_core::OfficeFileWatchEvent>, String> {
    Ok(watcher.lock().map_err(|e| e.to_string())?.check_changes())
}

#[tauri::command]
pub fn mark_file_self_saved(
    path: String,
    watcher: tauri::State<'_, std::sync::Mutex<OfficeFileWatcher>>,
) -> Result<(), String> {
    watcher
        .lock()
        .map_err(|e| e.to_string())?
        .mark_self_saved(&path);
    Ok(())
}
