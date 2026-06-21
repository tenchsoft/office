use super::*;

impl SheetsState {
    pub fn show_file_dialog(&mut self, mode: FileDialogMode) {
        self.file_dialog_mode = mode;
        self.file_dialog_path.clear();
        self.show_file_dialog = true;
    }

    /// Confirm the file dialog and execute the appropriate action.
    pub fn confirm_file_dialog(&mut self) -> bool {
        let path = self.file_dialog_path.trim().to_string();
        if path.is_empty() {
            return false;
        }
        let mode = self.file_dialog_mode;
        self.show_file_dialog = false;

        match mode {
            FileDialogMode::Open => match workbook_service::open_workbook(path.clone()) {
                Ok(_response) => {
                    self.add_recent_file(&path);
                    self.toast = Some((format!("Opened: {path}"), Instant::now()));
                    true
                }
                Err(e) => {
                    self.toast = Some((format!("Open failed: {e}"), Instant::now()));
                    false
                }
            },
            FileDialogMode::SaveAs => {
                let artifact = self.current_artifact().clone();
                let content = self.current_content().clone();
                match workbook_service::save_workbook(artifact, content, Some(path.clone()), None) {
                    Ok(saved) => {
                        self.auto_save.mark_saved();
                        self.apply_saved_artifact(saved.artifact);
                        self.add_recent_file(&path);
                        true
                    }
                    Err(e) => {
                        self.toast = Some((format!("Save failed: {e}"), Instant::now()));
                        false
                    }
                }
            }
            FileDialogMode::ImportCsv | FileDialogMode::ImportTsv | FileDialogMode::ImportOds => {
                let format = match mode {
                    FileDialogMode::ImportCsv => Some(tench_document_core::OfficeFileFormat::Csv),
                    FileDialogMode::ImportTsv => Some(tench_document_core::OfficeFileFormat::Csv),
                    _ => None,
                };
                match workbook_service::import_workbook(path.clone(), format) {
                    Ok(_response) => {
                        self.add_recent_file(&path);
                        self.toast = Some((format!("Imported: {path}"), Instant::now()));
                        true
                    }
                    Err(e) => {
                        self.toast = Some((format!("Import failed: {e}"), Instant::now()));
                        false
                    }
                }
            }
            FileDialogMode::ExportXlsx
            | FileDialogMode::ExportPdf
            | FileDialogMode::ExportHtml
            | FileDialogMode::ExportCsv => {
                let format = match mode {
                    FileDialogMode::ExportXlsx => tench_document_core::OfficeFileFormat::Xlsx,
                    FileDialogMode::ExportCsv => tench_document_core::OfficeFileFormat::Csv,
                    _ => tench_document_core::OfficeFileFormat::Xlsx,
                };
                let artifact_id = self.current_artifact().id.clone();
                let content = self.current_content().clone();
                match workbook_service::export_workbook(artifact_id, content, format, path.clone())
                {
                    Ok(_) => {
                        self.toast = Some((format!("Exported: {path}"), Instant::now()));
                        true
                    }
                    Err(e) => {
                        self.toast = Some((format!("Export failed: {e}"), Instant::now()));
                        false
                    }
                }
            }
        }
    }

    /// Cancel the file dialog.
    pub fn cancel_file_dialog(&mut self) {
        self.show_file_dialog = false;
    }

    /// Add a file path to the recent files list.
    pub fn add_recent_file(&mut self, path: &str) {
        // Remove duplicates
        self.recent_files.retain(|p| p != path);
        self.recent_files.insert(0, path.to_string());
        // Keep only last 10
        self.recent_files.truncate(10);
    }
}
