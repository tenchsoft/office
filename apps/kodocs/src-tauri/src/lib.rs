use std::sync::{mpsc, Mutex, OnceLock};

use tench_office_io::watcher::OfficeFileWatcher;

mod ai;
mod commands;
mod config;
mod document_service;
mod session;
pub mod ui;

/// Result from a file dialog.
pub enum DialogResult {
    /// User selected a file to open.
    OpenFile(String),
    /// User picked a save location.
    SaveAs(String),
    /// User picked an image file to insert.
    InsertImage(String),
}

/// Global sender for dialog results.
static DIALOG_TX: OnceLock<mpsc::Sender<DialogResult>> = OnceLock::new();

/// Set the global dialog result sender (called once during init).
pub fn set_dialog_sender(tx: mpsc::Sender<DialogResult>) {
    let _ = DIALOG_TX.set(tx);
}

/// Get the global dialog result sender.
pub fn dialog_sender() -> Option<&'static mpsc::Sender<DialogResult>> {
    DIALOG_TX.get()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(OfficeFileWatcher::new("tench-kodocs")))
        .manage(Mutex::new(session::DocumentSessionManager::new()))
        .invoke_handler(tauri::generate_handler![
            commands::runtime_capabilities,
            commands::create_document,
            commands::open_document,
            commands::save_document,
            commands::save_document_as,
            commands::import_document,
            commands::export_document,
            commands::get_recent_documents,
            commands::save_recovery_snapshot,
            commands::get_recovery_documents,
            commands::open_recovery_document,
            commands::delete_recovery_document,
            commands::clear_recovery_snapshots,
            commands::load_config,
            commands::save_config,
            commands::ai_chat,
            commands::ai_suggestion,
            commands::get_engine_status,
            commands::watch_file,
            commands::unwatch_file,
            commands::check_file_changes,
            commands::mark_file_self_saved,
            commands::edit_insert_text,
            commands::edit_backspace,
            commands::edit_toggle_mark,
            commands::edit_set_block_type,
            commands::edit_set_alignment,
            commands::edit_indent,
            commands::edit_outdent,
            commands::edit_set_font_size,
            commands::edit_set_text_color,
            commands::edit_set_background_color,
            commands::edit_insert_link,
            commands::edit_insert_image,
            commands::edit_insert_table,
            commands::edit_insert_horizontal_rule,
            commands::edit_undo,
            commands::edit_redo,
            commands::edit_move_cursor,
            commands::edit_select_all,
            commands::edit_get_state,
            // Search / Replace
            commands::edit_find,
            commands::edit_find_next,
            commands::edit_find_prev,
            commands::edit_replace_next,
            commands::edit_replace_all,
            commands::edit_clear_search,
            // Clipboard
            commands::edit_cut,
            commands::edit_copy,
            commands::edit_paste,
            // Track Changes
            commands::edit_toggle_track_changes,
            commands::edit_get_tracked_changes,
            commands::edit_accept_change,
            commands::edit_reject_change,
            commands::edit_accept_all_changes,
            commands::edit_reject_all_changes,
            // Comments
            commands::edit_add_comment,
            commands::edit_edit_comment,
            commands::edit_delete_comment,
            commands::edit_resolve_comment,
            commands::edit_get_comments,
            commands::edit_mark_saved,
        ])
        .setup(|app| {
            crate::init_tenchi_ui(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Tench Kodocs");
}

pub type BackendState = tench_ui::platform::TauriBackendState;

/// Initialize tench-ui rendering on a Tauri window.
pub fn init_tenchi_ui(app: &mut tauri::App) {
    let (dialog_tx, dialog_rx) = mpsc::channel();
    set_dialog_sender(dialog_tx);

    let app_handle = app.handle().clone();

    tench_ui::platform::init_tauri_ui(
        app,
        tench_ui::platform::TauriUiOptions::default(),
        |backend, _app| {
            let mut kodocs_app = ui::KodocsApp::new();
            kodocs_app.set_app_handle(app_handle.clone());
            kodocs_app.set_dialog_receiver(dialog_rx);
            backend.set_root(kodocs_app);
        },
    );
}

#[cfg(test)]
mod test_util {
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    pub fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn set_test_data_home(path: &Path) {
        std::env::set_var("XDG_DATA_HOME", path);
        std::env::set_var("APPDATA", path);
    }
}
