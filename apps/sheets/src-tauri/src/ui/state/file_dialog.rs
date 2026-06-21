// ---------------------------------------------------------------------------
// Phase 4: File dialog types
// ---------------------------------------------------------------------------

/// Mode for the file path input dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDialogMode {
    Open,
    SaveAs,
    ImportCsv,
    ImportTsv,
    ImportOds,
    ExportXlsx,
    ExportPdf,
    ExportHtml,
    ExportCsv,
}

impl FileDialogMode {
    /// Human-readable label for the dialog title.
    pub fn label(&self) -> &str {
        match self {
            FileDialogMode::Open => "Open File",
            FileDialogMode::SaveAs => "Save As",
            FileDialogMode::ImportCsv => "Import CSV",
            FileDialogMode::ImportTsv => "Import TSV",
            FileDialogMode::ImportOds => "Import ODS",
            FileDialogMode::ExportXlsx => "Export XLSX",
            FileDialogMode::ExportPdf => "Export PDF",
            FileDialogMode::ExportHtml => "Export HTML",
            FileDialogMode::ExportCsv => "Export CSV",
        }
    }

    /// Default file extension hint.
    pub fn default_extension(&self) -> &str {
        match self {
            FileDialogMode::Open => ".xlsx",
            FileDialogMode::SaveAs => ".tenchsheet",
            FileDialogMode::ImportCsv => ".csv",
            FileDialogMode::ImportTsv => ".tsv",
            FileDialogMode::ImportOds => ".ods",
            FileDialogMode::ExportXlsx => ".xlsx",
            FileDialogMode::ExportPdf => ".pdf",
            FileDialogMode::ExportHtml => ".html",
            FileDialogMode::ExportCsv => ".csv",
        }
    }
}
