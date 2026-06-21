use std::sync::Mutex;
use tench_office_runtime::{OfficeRuntimeProduct, DOCS_RUNTIME};

use tench_document_core::{
    Alignment, BlockType, ClipboardContent, CommentRange, EditResult, ImageSource, MarkType,
    MoveDirection, OfficeArtifact, OfficeContent, OfficeExportResponse, OfficeFileFormat,
    OfficeOpenResponse, OfficeRecentFile, OfficeRecoveryMetadata, OfficeSaveResponse, SearchMatch,
};
use tench_fs_core::OfficeFileWatchEvent;

use crate::ai;
use crate::ai::{AiChatRequest, AiSuggestionRequest};
use crate::config;
use crate::config::DocsConfig;
use crate::document_service;
use crate::session::DocumentSessionManager;
use tench_office_io::watcher::OfficeFileWatcher;

#[tauri::command]
pub fn runtime_capabilities() -> OfficeRuntimeProduct {
    DOCS_RUNTIME.clone()
}

#[tauri::command]
pub fn create_document(title: Option<String>) -> Result<OfficeOpenResponse, String> {
    Ok(document_service::create_document(title))
}

#[tauri::command]
pub fn open_document(path: String) -> Result<OfficeOpenResponse, String> {
    document_service::open_document(path)
}

#[tauri::command]
pub fn save_document(
    artifact: OfficeArtifact,
    content: OfficeContent,
    target_path: Option<String>,
    format: Option<OfficeFileFormat>,
) -> Result<OfficeSaveResponse, String> {
    document_service::save_document(artifact, content, target_path, format)
}

#[tauri::command]
pub fn save_document_as(
    artifact: OfficeArtifact,
    content: OfficeContent,
    target_path: String,
    format: Option<OfficeFileFormat>,
) -> Result<OfficeSaveResponse, String> {
    document_service::save_document(artifact, content, Some(target_path), format)
}

#[tauri::command]
pub fn import_document(
    source_path: String,
    source_format: Option<OfficeFileFormat>,
) -> Result<OfficeOpenResponse, String> {
    document_service::import_document(source_path, source_format)
}

#[tauri::command]
pub fn export_document(
    artifact_id: String,
    content: OfficeContent,
    target_format: OfficeFileFormat,
    output_path: String,
) -> Result<OfficeExportResponse, String> {
    document_service::export_document(artifact_id, content, target_format, output_path)
}

#[tauri::command]
pub fn get_recent_documents() -> Result<Vec<OfficeRecentFile>, String> {
    document_service::get_recent_documents()
}

#[tauri::command]
pub fn save_recovery_snapshot(
    artifact: OfficeArtifact,
    content: OfficeContent,
) -> Result<OfficeRecoveryMetadata, String> {
    document_service::save_recovery_snapshot(artifact, content)
}

#[tauri::command]
pub fn get_recovery_documents() -> Result<Vec<OfficeRecoveryMetadata>, String> {
    document_service::get_recovery_documents()
}

#[tauri::command]
pub fn open_recovery_document(recovery_path: String) -> Result<OfficeOpenResponse, String> {
    document_service::open_recovery_document(recovery_path)
}

#[tauri::command]
pub fn delete_recovery_document(recovery_path: String) -> Result<(), String> {
    document_service::delete_recovery_document(recovery_path)
}

#[tauri::command]
pub fn clear_recovery_snapshots(artifact_id: String) -> Result<(), String> {
    document_service::clear_recovery_snapshots(artifact_id)
}

#[tauri::command]
pub fn load_config() -> Result<DocsConfig, String> {
    config::load_config()
}

#[tauri::command]
pub fn save_config(config: DocsConfig) -> Result<(), String> {
    config::save_config(&config)
}

#[tauri::command]
pub fn ai_chat(request: AiChatRequest) -> Result<ai::AiChatResponse, String> {
    ai::handle_ai_chat(request)
}

#[tauri::command]
pub fn ai_suggestion(request: AiSuggestionRequest) -> Result<String, String> {
    ai::handle_ai_suggestion(request)
}

#[tauri::command]
pub fn get_engine_status() -> Result<ai::EngineStatusInfo, String> {
    Ok(ai::get_engine_status())
}

#[tauri::command]
pub fn watch_file(
    path: String,
    watcher: tauri::State<'_, Mutex<OfficeFileWatcher>>,
) -> Result<(), String> {
    watcher.lock().map_err(|e| e.to_string())?.watch(&path);
    Ok(())
}

#[tauri::command]
pub fn unwatch_file(
    path: String,
    watcher: tauri::State<'_, Mutex<OfficeFileWatcher>>,
) -> Result<(), String> {
    watcher.lock().map_err(|e| e.to_string())?.unwatch(&path);
    Ok(())
}

#[tauri::command]
pub fn check_file_changes(
    watcher: tauri::State<'_, Mutex<OfficeFileWatcher>>,
) -> Result<Vec<OfficeFileWatchEvent>, String> {
    Ok(watcher.lock().map_err(|e| e.to_string())?.check_changes())
}

#[tauri::command]
pub fn mark_file_self_saved(
    path: String,
    watcher: tauri::State<'_, Mutex<OfficeFileWatcher>>,
) -> Result<(), String> {
    watcher
        .lock()
        .map_err(|e| e.to_string())?
        .mark_self_saved(&path);
    Ok(())
}

// ---------------------------------------------------------------------------
// Edit commands — delegate to DocumentEngine via DocumentSessionManager
// ---------------------------------------------------------------------------

/// Helper: acquire the session manager lock and run an engine operation.
fn with_engine<F>(
    sessions: &tauri::State<'_, Mutex<DocumentSessionManager>>,
    doc_id: &str,
    op: F,
) -> Result<EditResult, String>
where
    F: FnOnce(&mut tench_document_core::DocumentEngine) -> EditResult,
{
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    Ok(op(engine))
}

#[tauri::command]
pub fn edit_insert_text(
    doc_id: String,
    text: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.insert_text(&text))
}

#[tauri::command]
pub fn edit_backspace(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.backspace())
}

#[tauri::command]
pub fn edit_toggle_mark(
    doc_id: String,
    mark: MarkType,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.toggle_mark(mark))
}

#[tauri::command]
pub fn edit_set_block_type(
    doc_id: String,
    block_type: BlockType,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.set_block_type(block_type))
}

#[tauri::command]
pub fn edit_set_alignment(
    doc_id: String,
    alignment: Alignment,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.set_alignment(alignment))
}

#[tauri::command]
pub fn edit_indent(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.indent())
}

#[tauri::command]
pub fn edit_outdent(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.outdent())
}

#[tauri::command]
pub fn edit_set_font_size(
    doc_id: String,
    size: f32,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.set_font_size(size))
}

#[tauri::command]
pub fn edit_set_text_color(
    doc_id: String,
    color: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.set_text_color(color))
}

#[tauri::command]
pub fn edit_set_background_color(
    doc_id: String,
    color: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.set_background_color(color))
}

#[tauri::command]
pub fn edit_insert_link(
    doc_id: String,
    href: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.insert_link(&href))
}

#[tauri::command]
pub fn edit_insert_image(
    doc_id: String,
    src: String,
    width: f64,
    height: f64,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    let source = ImageSource::Referenced { path: src };
    with_engine(&sessions, &doc_id, |e| {
        e.insert_image(source, width, height)
    })
}

#[tauri::command]
pub fn edit_insert_table(
    doc_id: String,
    rows: usize,
    cols: usize,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.insert_table(rows, cols))
}

#[tauri::command]
pub fn edit_insert_horizontal_rule(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.insert_horizontal_rule())
}

#[tauri::command]
pub fn edit_undo(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.undo())
}

#[tauri::command]
pub fn edit_redo(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.redo())
}

#[tauri::command]
pub fn edit_move_cursor(
    doc_id: String,
    direction: MoveDirection,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.move_cursor(direction))
}

#[tauri::command]
pub fn edit_select_all(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.select_all())
}

#[tauri::command]
pub fn edit_get_state(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    Ok(EditResult {
        document: engine.get_document().clone(),
        cursor: engine.get_cursor().clone(),
        selection: engine.get_selection().clone(),
        dirty: engine.is_dirty(),
    })
}

// ---------------------------------------------------------------------------
// Search / Replace commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn edit_find(
    doc_id: String,
    query: String,
    case_sensitive: bool,
    regex: bool,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<Vec<SearchMatch>, String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    Ok(engine.find(&query, case_sensitive, regex))
}

#[tauri::command]
pub fn edit_find_next(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.find_next())
}

#[tauri::command]
pub fn edit_find_prev(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.find_prev())
}

#[tauri::command]
pub fn edit_replace_next(
    doc_id: String,
    replacement: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.replace_next(&replacement))
}

#[tauri::command]
pub fn edit_replace_all(
    doc_id: String,
    query: String,
    replacement: String,
    case_sensitive: bool,
    regex: bool,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<usize, String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    Ok(engine.replace_all(&query, &replacement, case_sensitive, regex))
}

#[tauri::command]
pub fn edit_clear_search(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<(), String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    engine.clear_search();
    Ok(())
}

// ---------------------------------------------------------------------------
// Clipboard commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn edit_cut(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<ClipboardContent, String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    Ok(engine.cut())
}

#[tauri::command]
pub fn edit_copy(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<ClipboardContent, String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    Ok(engine.copy())
}

#[tauri::command]
pub fn edit_paste(
    doc_id: String,
    content: ClipboardContent,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.paste(content))
}

// ---------------------------------------------------------------------------
// Track Changes commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn edit_toggle_track_changes(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<bool, String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    engine.toggle_track_changes();
    Ok(engine.is_track_changes_enabled())
}

#[tauri::command]
pub fn edit_get_tracked_changes(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<Vec<tench_document_core::TrackedChange>, String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    Ok(engine.get_tracked_changes().to_vec())
}

#[tauri::command]
pub fn edit_accept_change(
    doc_id: String,
    change_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.accept_change(&change_id))
}

#[tauri::command]
pub fn edit_reject_change(
    doc_id: String,
    change_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.reject_change(&change_id))
}

#[tauri::command]
pub fn edit_accept_all_changes(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.accept_all_changes())
}

#[tauri::command]
pub fn edit_reject_all_changes(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<EditResult, String> {
    with_engine(&sessions, &doc_id, |e| e.reject_all_changes())
}

// ---------------------------------------------------------------------------
// Comment commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn edit_add_comment(
    doc_id: String,
    text: String,
    range: CommentRange,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<tench_document_core::Comment, String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    Ok(engine.add_comment(&text, range))
}

#[tauri::command]
pub fn edit_edit_comment(
    doc_id: String,
    comment_id: String,
    text: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<(), String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    engine.edit_comment(&comment_id, &text);
    Ok(())
}

#[tauri::command]
pub fn edit_delete_comment(
    doc_id: String,
    comment_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<(), String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    engine.delete_comment(&comment_id);
    Ok(())
}

#[tauri::command]
pub fn edit_resolve_comment(
    doc_id: String,
    comment_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<(), String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    engine.resolve_comment(&comment_id);
    Ok(())
}

#[tauri::command]
pub fn edit_get_comments(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<Vec<tench_document_core::Comment>, String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    Ok(engine.get_comments().to_vec())
}

#[tauri::command]
pub fn edit_mark_saved(
    doc_id: String,
    sessions: tauri::State<'_, Mutex<DocumentSessionManager>>,
) -> Result<(), String> {
    let mut mgr = sessions.lock().map_err(|e| e.to_string())?;
    let engine = mgr
        .get_engine(&doc_id)
        .ok_or_else(|| format!("No session for doc_id: {doc_id}"))?;
    engine.mark_saved();
    Ok(())
}
