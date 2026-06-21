use std::path::Path;

use tench_document_core::{
    DiagnosticSeverity, ImportExportDiagnostic, OfficeContent, OfficeFileFormat,
};
use tench_office_io::diagnostic;
use tench_office_io::sheets::{
    csv, format as sheets_format, html as sheets_html, ods, pdf as sheets_pdf, xlsx,
};

pub fn preview_text(content: &OfficeContent) -> String {
    sheets_format::workbook_to_plain_text(content)
}

pub fn import_binary(
    path: &Path,
    format: OfficeFileFormat,
) -> Result<(OfficeContent, Vec<ImportExportDiagnostic>), String> {
    match format {
        OfficeFileFormat::Xlsx => Ok((
            xlsx::import_xlsx(path)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Warning,
                "xlsx_basic_import",
                "XLSX import preserves cell values, formulas are stored but not evaluated.",
                true,
            )],
        )),
        OfficeFileFormat::Csv => Ok((
            csv::import_csv(path)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Info,
                "csv_import",
                "CSV import creates a single-sheet workbook.",
                true,
            )],
        )),
        OfficeFileFormat::Tsv => Ok((
            csv::import_tsv(path)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Info,
                "tsv_import",
                "TSV import creates a single-sheet workbook.",
                true,
            )],
        )),
        OfficeFileFormat::Ods => Ok((
            ods::import_ods(path)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Warning,
                "ods_basic_import",
                "ODS import preserves basic cell values.",
                true,
            )],
        )),
        _ => Err(format!(
            "Binary import only supports XLSX, CSV, TSV, and ODS, got {}.",
            format.extension()
        )),
    }
}

pub fn export_content(
    content: &OfficeContent,
    format: OfficeFileFormat,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    match format {
        OfficeFileFormat::Xlsx => Ok((
            xlsx::export_xlsx_bytes(content)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Warning,
                "xlsx_basic_export",
                "XLSX export writes cell values and basic structure.",
                true,
            )],
        )),
        OfficeFileFormat::Csv => Ok((csv::export_csv_bytes(content)?, Vec::new())),
        OfficeFileFormat::Tsv => Ok((csv::export_tsv_bytes(content)?, Vec::new())),
        OfficeFileFormat::Ods => Ok((
            ods::export_ods_bytes(content)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Warning,
                "ods_basic_export",
                "ODS export writes basic cell values.",
                true,
            )],
        )),
        OfficeFileFormat::Pdf => {
            let config = sheets_pdf::PdfExportConfig::default();
            Ok((
                sheets_pdf::export_pdf_bytes(content, &config)?,
                vec![diagnostic::diagnostic(
                    DiagnosticSeverity::Info,
                    "pdf_export",
                    "PDF export renders sheets with grid lines and basic formatting.",
                    true,
                )],
            ))
        }
        OfficeFileFormat::Html => Ok((sheets_html::export_html_bytes(content)?, Vec::new())),
        _ => Err(format!(
            "{} is not a Tench Sheets export format.",
            format.extension()
        )),
    }
}
