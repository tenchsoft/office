use std::sync::Mutex;

use tench_office_io::watcher::OfficeFileWatcher;
use tench_license_store::LicenseStore;

mod ai;
mod commands;
mod config;
pub mod ui;
mod workbook_service;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load the local license credential store before starting Tauri so the
    // updater plugin and UI can both read the device token from a single
    // authoritative source. A failure here is non-fatal — we fall back to an
    // ephemeral in-memory store so the app still runs (the user can activate
    // from the License tab later).
    let license_store = LicenseStore::load_or_init("sheets").unwrap_or_else(|e| {
        eprintln!("license store init failed, falling back to ephemeral: {e}");
        LicenseStore::ephemeral()
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Mutex::new(OfficeFileWatcher::new("tench-sheets")))
        .manage(std::sync::Arc::clone(&license_store))
        .invoke_handler(tauri::generate_handler![
            commands::runtime_capabilities,
            commands::create_workbook,
            commands::open_workbook,
            commands::save_workbook,
            commands::save_workbook_as,
            commands::import_workbook,
            commands::export_workbook,
            commands::get_recent_workbooks,
            commands::save_recovery_snapshot,
            commands::get_recovery_workbooks,
            commands::open_recovery_workbook,
            commands::delete_recovery_workbook,
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
            commands::get_page_setup,
            commands::set_page_setup_cmd,
            commands::set_print_area,
            commands::clear_print_area,
            commands::print_document,
            commands::open_print_preview,
            commands::license_state,
            commands::license_activate,
            commands::license_release,
        ])
        .setup(move |app| {
            crate::init_tenchi_ui(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Tench Sheets");
}

pub type BackendState = tench_ui::platform::TauriBackendState;

/// Initialize tench-ui rendering on a Tauri window.
pub fn init_tenchi_ui(app: &mut tauri::App) {
    tench_ui::platform::init_tauri_ui(
        app,
        tench_ui::platform::TauriUiOptions::default(),
        |backend, _app| {
            backend.set_root(ui::SheetsApp::new());
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
