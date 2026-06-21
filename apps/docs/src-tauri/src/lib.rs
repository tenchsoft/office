use std::sync::{mpsc, Mutex, OnceLock};

use tench_office_io::watcher::OfficeFileWatcher;
use tench_license_store::LicenseStore;

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

pub type BackendState = tench_ui::platform::TauriBackendState;

/// Interval between update checks. Per the licensing contract, checks happen
/// weekly (the device_token TTL is 10 days so there is a 3-day grace window).
const UPDATE_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(7 * 24 * 60 * 60);

/// Spawns a background thread that periodically checks for updates via
/// tauri-plugin-updater. The updater endpoint is the per-app
/// `/api/device/releases` URL configured in `tauri.conf.json`; we inject the
/// device_token at runtime via the `Authorization: Tench-Device` header so the
/// server can reject expired licenses.
///
/// Behaviour:
/// - License unactivated/expired → check is skipped (UI shows the
///   reactivation label).
/// - License active but token missing → skipped (caller will re-issue on
///   next user-initiated check).
/// - License active with token → call Tauri updater. On success, Tauri
///   itself handles download + install + restart.
///
/// Errors are logged and swallowed — the scheduler must never crash the app.
pub(crate) fn spawn_update_scheduler(
    app_handle: tauri::AppHandle,
    license_store: std::sync::Arc<tench_license_store::LicenseStore>,
    _product: &str,
) {
    let product = _product.to_string();
    std::thread::spawn(move || {
        // Small initial delay so the app window finishes opening before we
        // hit the network for the first check.
        std::thread::sleep(std::time::Duration::from_secs(30));
        loop {
            run_one_update_check(&app_handle, &license_store, &product);
            std::thread::sleep(UPDATE_CHECK_INTERVAL);
        }
    });
}

fn run_one_update_check(
    app_handle: &tauri::AppHandle,
    license_store: &tench_license_store::LicenseStore,
    _product: &str,
) {
    use tench_license_store::LicenseStatus;
    let state = license_store.state();
    if state.status() != LicenseStatus::Active {
        // Either unactivated or expired — surface via UI notification label
        // (handled by the UI layer); the scheduler just skips.
        return;
    }
    let Some(token) = state.device_token.clone() else {
        return;
    };

    let auth = format!("Tench-Device {token}");
    let result = tauri::async_runtime::block_on(async move {
        use tauri_plugin_updater::UpdaterExt;
        let updater = match app_handle
            .updater_builder()
            .header("Authorization", auth)?
            .build()
        {
            Ok(u) => u,
            Err(e) => {
                eprintln!("update scheduler: failed to build updater: {e}");
                return Ok::<(), tauri_plugin_updater::Error>(());
            }
        };
        match updater.check().await {
            Ok(Some(update)) => {
                eprintln!(
                    "update scheduler: new version {} available (current {}), installing",
                    update.version,
                    update.current_version
                );
                if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
                    eprintln!("update scheduler: download/install failed: {e}");
                }
            }
            Ok(None) => {
                eprintln!("update scheduler: no update available");
            }
            Err(e) => {
                eprintln!("update scheduler: check failed: {e}");
            }
        }
        Ok(())
    });
    let _ = result;
}

/// Initialize tench-ui rendering on a Tauri window.
pub fn init_tenchi_ui(app: &mut tauri::App, license_store: std::sync::Arc<tench_license_store::LicenseStore>) {
    let (dialog_tx, dialog_rx) = mpsc::channel();
    set_dialog_sender(dialog_tx);

    let app_handle = app.handle().clone();

    tench_ui::platform::init_tauri_ui(
        app,
        tench_ui::platform::TauriUiOptions::default(),
        move |backend, _app| {
            let mut docs_app = ui::DocsApp::new();
            docs_app.set_app_handle(app_handle.clone());
            docs_app.set_dialog_receiver(dialog_rx);
            docs_app.set_license_store(license_store.clone());
            backend.set_root(docs_app);
        },
    );
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load the local license credential store before starting Tauri so the
    // updater plugin and UI can both read the device token from a single
    // authoritative source. A failure here is non-fatal — we fall back to an
    // ephemeral in-memory store so the app still runs (the user can activate
    // from the License tab later).
    let license_store = LicenseStore::load_or_init("docs").unwrap_or_else(|e| {
        eprintln!("license store init failed, falling back to ephemeral: {e}");
        LicenseStore::ephemeral()
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Mutex::new(OfficeFileWatcher::new("tench-docs")))
        .manage(Mutex::new(session::DocumentSessionManager::new()))
        .manage(std::sync::Arc::clone(&license_store))
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
            // License
            commands::license_state,
            commands::license_activate,
            commands::license_release,
        ])
        .setup(move |app| {
            crate::init_tenchi_ui(app, license_store.clone());
            crate::spawn_update_scheduler(app.handle().clone(), license_store.clone(), "docs");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Tench Docs");
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
