use std::sync::{mpsc, Mutex, OnceLock};

use tench_office_io::watcher::OfficeFileWatcher;

mod ai;
mod commands;
mod config;
mod presentation_service;
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
            let mut slides_app = ui::SlidesApp::new();
            slides_app.set_app_handle(app_handle.clone());
            slides_app.set_dialog_receiver(dialog_rx);
            backend.set_root(slides_app);
        },
    );
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(OfficeFileWatcher::new("tench-slides")))
        .invoke_handler(tauri::generate_handler![
            commands::runtime_capabilities,
            commands::create_presentation,
            commands::open_presentation,
            commands::save_presentation,
            commands::save_presentation_as,
            commands::import_presentation,
            commands::export_presentation,
            commands::get_recent_presentations,
            commands::save_recovery_snapshot,
            commands::get_recovery_presentations,
            commands::open_recovery_presentation,
            commands::delete_recovery_presentation,
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
        ])
        .setup(|app| {
            crate::init_tenchi_ui(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Tench Slides");
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
