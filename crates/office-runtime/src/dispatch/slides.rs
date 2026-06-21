use std::path::Path;

use tench_document_core::{
    DiagnosticSeverity, ImportExportDiagnostic, OfficeContent, OfficeFileFormat,
};
use tench_office_io::diagnostic;
use tench_office_io::slides::{format as slides_format, odp, pptx};

pub fn preview_text(content: &OfficeContent) -> String {
    slides_format::presentation_to_plain_text(content)
}

pub fn import_binary(
    path: &Path,
    format: OfficeFileFormat,
) -> Result<(OfficeContent, Vec<ImportExportDiagnostic>), String> {
    match format {
        OfficeFileFormat::Pptx => Ok((
            pptx::import_pptx(path)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Warning,
                "pptx_basic_import",
                "PPTX import preserves slide structure and text content.",
                true,
            )],
        )),
        OfficeFileFormat::Odp => Ok((
            odp::import_odp(path)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Warning,
                "odp_basic_import",
                "ODP import preserves slide structure and text content.",
                true,
            )],
        )),
        _ => Err(format!(
            "Binary import only supports PPTX and ODP, got {}.",
            format.extension()
        )),
    }
}

pub fn export_content(
    content: &OfficeContent,
    format: OfficeFileFormat,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    match format {
        OfficeFileFormat::Pptx => Ok((
            pptx::export_pptx_bytes(content)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Warning,
                "pptx_basic_export",
                "PPTX export writes slide structure and text content.",
                true,
            )],
        )),
        OfficeFileFormat::Odp => Ok((
            odp::export_odp_bytes(content)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Warning,
                "odp_basic_export",
                "ODP export writes slide structure and text content.",
                true,
            )],
        )),
        OfficeFileFormat::Pdf => Err(format!(
            "{} export is planned after the canvas engine is stable.",
            format.extension()
        )),
        _ => Err(format!(
            "{} is not a Tench Slides export format.",
            format.extension()
        )),
    }
}
