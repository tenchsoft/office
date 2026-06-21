use std::path::Path;

use tench_document_core::{
    DiagnosticSeverity, ImportExportDiagnostic, OfficeContent, OfficeFileFormat, TenchDocument,
};
use tench_office_io::diagnostic;
use tench_office_io::docs::{docx, epub, format as docs_format, odt, pdf as docs_pdf, rtf};

pub fn preview_text(content: &OfficeContent) -> String {
    docs_format::docs_content_to_plain_text(content)
}

pub fn import_text(
    raw: &str,
    format: OfficeFileFormat,
    docs_error_mentions_rtf: bool,
) -> Result<(OfficeContent, Vec<ImportExportDiagnostic>), String> {
    match format {
        OfficeFileFormat::Txt => Ok((docs_format::plain_text_to_docs_content(raw), Vec::new())),
        OfficeFileFormat::Md => Ok((docs_format::markdown_to_docs_content(raw), Vec::new())),
        OfficeFileFormat::Html => Ok((
            docs_format::html_to_docs_content(raw),
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Info,
                "html_basic_import",
                "HTML import keeps text structure and removes script/style content.",
                true,
            )],
        )),
        OfficeFileFormat::Rtf => {
            Err("RTF import requires a file path. Use import_binary instead.".to_string())
        }
        _ if docs_error_mentions_rtf => Err(format!(
            "Text-based import only supports TXT, Markdown, HTML, and RTF, got {}.",
            format.extension()
        )),
        _ => Err(format!(
            "Text-based import only supports TXT, Markdown, HTML. Got {}.",
            format.extension()
        )),
    }
}

pub fn import_binary(
    path: &Path,
    format: OfficeFileFormat,
    include_hwp: bool,
) -> Result<(OfficeContent, Vec<ImportExportDiagnostic>), String> {
    match format {
        OfficeFileFormat::Hwp if include_hwp => {
            let tdm = tench_hwp_io::read_hwp(path).map_err(|e| format!("HWP import error: {e}"))?;
            Ok((
                docs_format::tdm_to_docs_content(&tdm),
                vec![diagnostic::diagnostic(
                    DiagnosticSeverity::Warning,
                    "hwp_basic_import",
                    "HWP import preserves basic paragraphs, headings, tables, and text formatting. Advanced features (shapes, equations, macros) are not yet supported.",
                    true,
                )],
            ))
        }
        OfficeFileFormat::Hwpx if include_hwp => {
            let tdm =
                tench_hwp_io::read_hwpx(path).map_err(|e| format!("HWPX import error: {e}"))?;
            Ok((
                docs_format::tdm_to_docs_content(&tdm),
                vec![diagnostic::diagnostic(
                    DiagnosticSeverity::Warning,
                    "hwpx_basic_import",
                    "HWPX import preserves basic paragraphs, headings, tables, and text formatting.",
                    true,
                )],
            ))
        }
        OfficeFileFormat::Hwt if include_hwp => {
            let tdm = tench_hwp_io::read_hwp(path).map_err(|e| format!("HWT import error: {e}"))?;
            Ok((
                docs_format::tdm_to_docs_content(&tdm),
                vec![diagnostic::diagnostic(
                    DiagnosticSeverity::Info,
                    "hwt_template_import",
                    "HWT template imported. Template metadata is preserved.",
                    true,
                )],
            ))
        }
        OfficeFileFormat::Docx => Ok((
            docx::import_docx_as_content(path)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Warning,
                "docx_basic_import",
                "DOCX import currently preserves basic paragraphs, headings, and text marks.",
                true,
            )],
        )),
        OfficeFileFormat::Odt => Ok((
            odt::import_odt_as_content(path)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Warning,
                "odt_basic_import",
                "ODT import currently preserves basic paragraphs and headings.",
                true,
            )],
        )),
        OfficeFileFormat::Rtf => Ok((
            rtf::import_rtf_as_content(path)?,
            vec![diagnostic::diagnostic(
                DiagnosticSeverity::Info,
                "rtf_import",
                "RTF import preserves basic formatting: bold, italic, underline, strike, tables.",
                true,
            )],
        )),
        _ if include_hwp => Err(format!(
            "Binary import only supports HWP, HWPX, DOCX, ODT, RTF. Got {}.",
            format.extension()
        )),
        _ => Err(format!(
            "Binary import only supports DOCX, ODT, and RTF, got {}.",
            format.extension()
        )),
    }
}

pub fn export_content(
    content: &OfficeContent,
    format: OfficeFileFormat,
    include_hwp: bool,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    match format {
        OfficeFileFormat::Txt => Ok((
            docs_format::docs_content_to_plain_text(content).into_bytes(),
            Vec::new(),
        )),
        OfficeFileFormat::Md => Ok((
            docs_format::docs_content_to_markdown(content).into_bytes(),
            Vec::new(),
        )),
        OfficeFileFormat::Html => Ok((
            docs_format::docs_content_to_html(content).into_bytes(),
            Vec::new(),
        )),
        OfficeFileFormat::Docx => docx_export(content),
        OfficeFileFormat::Odt => odt_export(content),
        OfficeFileFormat::Pdf => pdf_export(content),
        OfficeFileFormat::Rtf => rtf_export(content),
        OfficeFileFormat::Epub => epub_export(content),
        OfficeFileFormat::Hwp if include_hwp => hwp_export(content),
        OfficeFileFormat::Hwpx if include_hwp => hwpx_export(content),
        _ if include_hwp => Err(format!(
            "{} is not a Tench Kodocs export format.",
            format.extension()
        )),
        _ => Err(format!(
            "{} is not a Tench Docs export format.",
            format.extension()
        )),
    }
}

fn docx_export(content: &OfficeContent) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    Ok((
        docx::export_docx_bytes_from_content(content)?,
        vec![diagnostic::diagnostic(
            DiagnosticSeverity::Info,
            "docx_export",
            "DOCX export writes paragraphs, headings, lists, tables, images, headers/footers, and page setup.",
            true,
        )],
    ))
}

fn odt_export(content: &OfficeContent) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    Ok((
        odt::export_odt_bytes_from_content(content)?,
        vec![diagnostic::diagnostic(
            DiagnosticSeverity::Info,
            "odt_export",
            "ODT export writes paragraphs, headings, lists, tables, images, and inline formatting.",
            true,
        )],
    ))
}

fn pdf_export(content: &OfficeContent) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    Ok((
        docs_pdf::export_pdf_bytes_from_content(content)?,
        vec![diagnostic::diagnostic(
            DiagnosticSeverity::Info,
            "pdf_export",
            "PDF export produces a basic PDF with text, bold formatting, and page layout.",
            true,
        )],
    ))
}

fn rtf_export(content: &OfficeContent) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    Ok((
        rtf::export_rtf_bytes_from_content(content)?,
        vec![diagnostic::diagnostic(
            DiagnosticSeverity::Info,
            "rtf_export",
            "RTF export writes basic formatting: bold, italic, underline, strike, tables, alignment.",
            true,
        )],
    ))
}

fn epub_export(content: &OfficeContent) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    Ok((
        epub::export_epub_bytes_from_content(content)?,
        vec![diagnostic::diagnostic(
            DiagnosticSeverity::Info,
            "epub_export",
            "EPUB export produces a valid EPUB 3.0 archive with chapter HTML and TOC.",
            true,
        )],
    ))
}

fn hwp_export(content: &OfficeContent) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    let tdm = extract_tdm(content);
    let bytes =
        tench_hwp_io::write_hwp_bytes(&tdm).map_err(|e| format!("HWP export error: {e}"))?;
    Ok((
        bytes,
        vec![diagnostic::diagnostic(
            DiagnosticSeverity::Info,
            "hwp_export",
            "HWP export writes basic paragraphs, headings, tables, and text formatting.",
            true,
        )],
    ))
}

fn hwpx_export(content: &OfficeContent) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    let tdm = extract_tdm(content);
    let bytes =
        tench_hwp_io::write_hwpx_bytes(&tdm).map_err(|e| format!("HWPX export error: {e}"))?;
    Ok((
        bytes,
        vec![diagnostic::diagnostic(
            DiagnosticSeverity::Info,
            "hwpx_export",
            "HWPX export writes basic paragraphs, headings, tables, and text formatting.",
            true,
        )],
    ))
}

fn extract_tdm(content: &OfficeContent) -> TenchDocument {
    match content {
        OfficeContent::Docs(rich) => rich
            .document
            .clone()
            .unwrap_or_else(|| TenchDocument::new("")),
        _ => TenchDocument::new(""),
    }
}
