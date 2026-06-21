use std::sync::{Arc, Mutex};
use tench_office_runtime::{OfficeRuntimeProduct, SHEETS_RUNTIME};

use tench_document_core::{
    OfficeArtifact, OfficeContent, OfficeExportResponse, OfficeFileFormat, OfficeOpenResponse,
    OfficeRecentFile, OfficeRecoveryMetadata, OfficeSaveResponse,
};
use tench_fs_core::OfficeFileWatchEvent;
use tench_license_store::{LicenseState, LicenseStatus, LicenseStore};

use crate::ai;
use crate::ai::{AiChatRequest, AiSuggestionRequest};
use crate::config;
use crate::config::SheetsConfig;
use crate::ui::state::{CellRange, Margins, Orientation, PageSetup, PaperSize, Scaling};
use crate::workbook_service;
use tench_office_io::watcher::OfficeFileWatcher;

#[tauri::command]
pub fn runtime_capabilities() -> OfficeRuntimeProduct {
    SHEETS_RUNTIME.clone()
}

#[tauri::command]
pub fn create_workbook(title: Option<String>) -> Result<OfficeOpenResponse, String> {
    Ok(workbook_service::create_workbook(title))
}

#[tauri::command]
pub fn open_workbook(path: String) -> Result<OfficeOpenResponse, String> {
    workbook_service::open_workbook(path)
}

#[tauri::command]
pub fn save_workbook(
    artifact: OfficeArtifact,
    content: OfficeContent,
    target_path: Option<String>,
    format: Option<OfficeFileFormat>,
) -> Result<OfficeSaveResponse, String> {
    workbook_service::save_workbook(artifact, content, target_path, format)
}

#[tauri::command]
pub fn save_workbook_as(
    artifact: OfficeArtifact,
    content: OfficeContent,
    target_path: String,
    format: Option<OfficeFileFormat>,
) -> Result<OfficeSaveResponse, String> {
    workbook_service::save_workbook(artifact, content, Some(target_path), format)
}

#[tauri::command]
pub fn import_workbook(
    source_path: String,
    source_format: Option<OfficeFileFormat>,
) -> Result<OfficeOpenResponse, String> {
    workbook_service::import_workbook(source_path, source_format)
}

#[tauri::command]
pub fn export_workbook(
    artifact_id: String,
    content: OfficeContent,
    target_format: OfficeFileFormat,
    output_path: String,
) -> Result<OfficeExportResponse, String> {
    workbook_service::export_workbook(artifact_id, content, target_format, output_path)
}

#[tauri::command]
pub fn get_recent_workbooks() -> Result<Vec<OfficeRecentFile>, String> {
    workbook_service::get_recent_workbooks()
}

#[tauri::command]
pub fn save_recovery_snapshot(
    artifact: OfficeArtifact,
    content: OfficeContent,
) -> Result<OfficeRecoveryMetadata, String> {
    workbook_service::save_recovery_snapshot(artifact, content)
}

#[tauri::command]
pub fn get_recovery_workbooks() -> Result<Vec<OfficeRecoveryMetadata>, String> {
    workbook_service::get_recovery_workbooks()
}

#[tauri::command]
pub fn open_recovery_workbook(recovery_path: String) -> Result<OfficeOpenResponse, String> {
    workbook_service::open_recovery_workbook(recovery_path)
}

#[tauri::command]
pub fn delete_recovery_workbook(recovery_path: String) -> Result<(), String> {
    workbook_service::delete_recovery_workbook(recovery_path)
}

#[tauri::command]
pub fn clear_recovery_snapshots(artifact_id: String) -> Result<(), String> {
    workbook_service::clear_recovery_snapshots(artifact_id)
}

#[tauri::command]
pub fn load_config() -> Result<SheetsConfig, String> {
    config::load_config()
}

#[tauri::command]
pub fn save_config(config: SheetsConfig) -> Result<(), String> {
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
// 10.1 Page Setup commands
// ---------------------------------------------------------------------------

/// Serializable page setup response for Tauri IPC.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PageSetupResponse {
    pub paper_size: String,
    pub orientation: String,
    pub margin_top: f64,
    pub margin_bottom: f64,
    pub margin_left: f64,
    pub margin_right: f64,
    pub margin_header: f64,
    pub margin_footer: f64,
    pub scaling_percentage: f64,
    pub fit_width: Option<usize>,
    pub fit_height: Option<usize>,
    pub gridlines_print: bool,
    pub row_col_headers_print: bool,
    pub center_horizontally: bool,
    pub center_vertically: bool,
}

impl From<&PageSetup> for PageSetupResponse {
    fn from(setup: &PageSetup) -> Self {
        let (scaling_percentage, fit_width, fit_height) = match setup.scaling {
            Scaling::Percentage(p) => (p, None, None),
            Scaling::FitToPages { width, height } => (100.0, width, height),
        };
        Self {
            paper_size: setup.paper_size.label().to_string(),
            orientation: setup.orientation.label().to_string(),
            margin_top: setup.margins.top,
            margin_bottom: setup.margins.bottom,
            margin_left: setup.margins.left,
            margin_right: setup.margins.right,
            margin_header: setup.margins.header,
            margin_footer: setup.margins.footer,
            scaling_percentage,
            fit_width,
            fit_height,
            gridlines_print: setup.gridlines_print,
            row_col_headers_print: setup.row_col_headers_print,
            center_horizontally: setup.center_horizontally,
            center_vertically: setup.center_vertically,
        }
    }
}

/// Serializable print preview response for Tauri IPC.
#[derive(serde::Serialize)]
pub struct PrintPreviewResponse {
    pub total_pages: usize,
    pub current_page: usize,
    pub zoom: f64,
}

#[tauri::command]
pub fn get_page_setup(
    backend: tauri::State<'_, crate::BackendState>,
) -> Result<PageSetupResponse, String> {
    let mut b = backend.backend.lock().map_err(|e| e.to_string())?;
    let app = b
        .root_widget::<crate::ui::SheetsApp>()
        .ok_or("No root widget")?;
    let setup = app.state().get_page_setup();
    Ok(PageSetupResponse::from(setup))
}

// Too many parameters required by the Tauri command signature.
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn set_page_setup_cmd(
    paper_size: String,
    orientation: String,
    margin_top: f64,
    margin_bottom: f64,
    margin_left: f64,
    margin_right: f64,
    margin_header: f64,
    margin_footer: f64,
    scaling_percentage: f64,
    fit_width: Option<usize>,
    fit_height: Option<usize>,
    gridlines_print: bool,
    row_col_headers_print: bool,
    center_horizontally: bool,
    center_vertically: bool,
    backend: tauri::State<'_, crate::BackendState>,
) -> Result<(), String> {
    let mut b = backend.backend.lock().map_err(|e| e.to_string())?;
    let app = b
        .root_widget::<crate::ui::SheetsApp>()
        .ok_or("No root widget")?;

    let paper = match paper_size.as_str() {
        "A4" => PaperSize::A4,
        "Letter" => PaperSize::Letter,
        "Legal" => PaperSize::Legal,
        "Tabloid" => PaperSize::Tabloid,
        "A3" => PaperSize::A3,
        "A5" => PaperSize::A5,
        "B5" => PaperSize::B5,
        "Envelope10" => PaperSize::Envelope10,
        "EnvelopeDL" => PaperSize::EnvelopeDL,
        other => {
            // Try to parse "Custom WxH" format
            let parts: Vec<&str> = other.split('x').collect();
            if parts.len() == 2 {
                let w = parts[0].trim().parse::<f64>().unwrap_or(210.0);
                let h = parts[1].trim().parse::<f64>().unwrap_or(297.0);
                PaperSize::Custom(w, h)
            } else {
                PaperSize::A4
            }
        }
    };

    let orient = if orientation == "Landscape" {
        Orientation::Landscape
    } else {
        Orientation::Portrait
    };

    let scaling = if fit_width.is_some() || fit_height.is_some() {
        Scaling::FitToPages {
            width: fit_width,
            height: fit_height,
        }
    } else {
        Scaling::Percentage(scaling_percentage)
    };

    let setup = PageSetup {
        paper_size: paper,
        orientation: orient,
        margins: Margins {
            top: margin_top,
            bottom: margin_bottom,
            left: margin_left,
            right: margin_right,
            header: margin_header,
            footer: margin_footer,
        },
        scaling,
        print_area: app.state().page_setup.print_area.clone(),
        print_titles_rows: app.state().page_setup.print_titles_rows,
        print_titles_cols: app.state().page_setup.print_titles_cols,
        repeat_header: app.state().page_setup.repeat_header,
        gridlines_print,
        row_col_headers_print,
        center_horizontally,
        center_vertically,
        header_left: app.state().page_setup.header_left.clone(),
        header_center: app.state().page_setup.header_center.clone(),
        header_right: app.state().page_setup.header_right.clone(),
        footer_left: app.state().page_setup.footer_left.clone(),
        footer_center: app.state().page_setup.footer_center.clone(),
        footer_right: app.state().page_setup.footer_right.clone(),
    };

    app.state_mut().set_page_setup(setup);
    Ok(())
}

#[tauri::command]
pub fn set_print_area(
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
    backend: tauri::State<'_, crate::BackendState>,
) -> Result<(), String> {
    let mut b = backend.backend.lock().map_err(|e| e.to_string())?;
    let app = b
        .root_widget::<crate::ui::SheetsApp>()
        .ok_or("No root widget")?;
    app.state_mut().set_print_area(CellRange {
        start_row,
        start_col,
        end_row,
        end_col,
    });
    Ok(())
}

#[tauri::command]
pub fn clear_print_area(backend: tauri::State<'_, crate::BackendState>) -> Result<(), String> {
    let mut b = backend.backend.lock().map_err(|e| e.to_string())?;
    let app = b
        .root_widget::<crate::ui::SheetsApp>()
        .ok_or("No root widget")?;
    app.state_mut().clear_print_area();
    Ok(())
}

#[tauri::command]
pub fn print_document(backend: tauri::State<'_, crate::BackendState>) -> Result<String, String> {
    let mut b = backend.backend.lock().map_err(|e| e.to_string())?;
    let app = b
        .root_widget::<crate::ui::SheetsApp>()
        .ok_or("No root widget")?;
    app.state_mut().compute_print_pages();
    let total = app.state().print_preview.pages.len();
    // In a real implementation, this would invoke the system print dialog.
    // For now, return a summary of what would be printed.
    Ok(format!("Prepared {} page(s) for printing", total))
}

#[tauri::command]
pub fn open_print_preview(
    backend: tauri::State<'_, crate::BackendState>,
) -> Result<PrintPreviewResponse, String> {
    let mut b = backend.backend.lock().map_err(|e| e.to_string())?;
    let app = b
        .root_widget::<crate::ui::SheetsApp>()
        .ok_or("No root widget")?;
    app.state_mut().compute_print_pages();
    app.state_mut().print_preview.visible = true;
    Ok(PrintPreviewResponse {
        total_pages: app.state().print_preview.pages.len(),
        current_page: app.state().print_preview.current_page,
        zoom: app.state().print_preview.zoom,
    })
}

// ---------------------------------------------------------------------------
// License commands
// ---------------------------------------------------------------------------

/// Snapshot of the local license state, returned to the UI.
#[derive(serde::Serialize)]
pub struct LicenseStateResponse {
    pub device_id: String,
    pub license_key: Option<String>,
    pub status: &'static str,
    pub token_expires_at: Option<String>,
    pub ephemeral: bool,
}

impl From<LicenseState> for LicenseStateResponse {
    fn from(state: LicenseState) -> Self {
        let status = match state.status() {
            LicenseStatus::Unactivated => "unactivated",
            LicenseStatus::Active => "active",
            LicenseStatus::Expired => "expired",
        };
        Self {
            device_id: state.device_id,
            license_key: state.license_key,
            status,
            token_expires_at: state.token_expires_at,
            ephemeral: false,
        }
    }
}

#[tauri::command]
pub fn license_state(
    license_store: tauri::State<'_, Arc<LicenseStore>>,
) -> Result<LicenseStateResponse, String> {
    let state = license_store.state();
    let mut resp = LicenseStateResponse::from(state);
    resp.ephemeral = license_store.is_ephemeral();
    Ok(resp)
}

#[tauri::command]
pub fn license_activate(
    license_store: tauri::State<'_, Arc<LicenseStore>>,
    license_key: String,
) -> Result<LicenseStateResponse, String> {
    tench_update_client::activate_license(
        &license_store,
        None,
        &license_key,
        "sheets",
        env!("CARGO_PKG_VERSION"),
    )
    .map_err(|e| e.to_string())?;
    let state = license_store.state();
    let mut resp = LicenseStateResponse::from(state);
    resp.ephemeral = license_store.is_ephemeral();
    Ok(resp)
}

#[tauri::command]
pub fn license_release(
    license_store: tauri::State<'_, Arc<LicenseStore>>,
) -> Result<LicenseStateResponse, String> {
    tench_update_client::release_license(&license_store).map_err(|e| e.to_string())?;
    let state = license_store.state();
    let mut resp = LicenseStateResponse::from(state);
    resp.ephemeral = license_store.is_ephemeral();
    Ok(resp)
}
