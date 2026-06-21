use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use serde_json::Value;
use tench_document_core::{CellContent, CellValue, OfficeContent, SheetContent, WorkbookContent};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::format;

mod export;
mod import;

pub use export::export_ods_bytes;
pub use import::import_ods;

// ---------------------------------------------------------------------------
// ZIP utilities
// ---------------------------------------------------------------------------

fn write_zip_bytes(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for (path, data) in files {
        writer
            .start_file(path, options)
            .map_err(|e| format!("ZIP start_file {path}: {e}"))?;
        writer
            .write_all(data)
            .map_err(|e| format!("ZIP write {path}: {e}"))?;
    }

    let cursor = writer.finish().map_err(|e| format!("ZIP finish: {e}"))?;
    Ok(cursor.into_inner())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn xml_unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ods_export_produces_valid_zip() {
        let content = format::empty_workbook_content("Test");
        let bytes = export_ods_bytes(&content).expect("export");
        assert!(!bytes.is_empty());

        let reader = std::io::Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(reader).expect("valid zip");
        assert!(archive.file_names().any(|n| n == "content.xml"));
    }

    #[test]
    fn ods_export_with_formula() {
        let mut content = format::empty_workbook_content("FormulaTest");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets[0].cells.push(CellContent {
                address: "A1".to_string(),
                value: CellValue::Number(125.0),
                formula: Some("B2+B3".to_string()),
                style: serde_json::json!({}),
            });
        }

        let bytes = export_ods_bytes(&content).expect("export");
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).expect("valid zip");
        let mut content_file = archive.by_name("content.xml").expect("content.xml");
        let mut xml = String::new();
        content_file.read_to_string(&mut xml).unwrap();
        assert!(xml.contains("table:formula="));
    }

    #[test]
    fn ods_export_with_style() {
        let mut content = format::empty_workbook_content("StyleTest");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets[0].cells.push(CellContent {
                address: "A1".to_string(),
                value: CellValue::String("Bold".to_string()),
                formula: None,
                style: serde_json::json!({
                    "bold": true,
                    "background_color": "#FF0000"
                }),
            });
        }

        let bytes = export_ods_bytes(&content).expect("export");
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).expect("valid zip");
        let mut content_file = archive.by_name("content.xml").expect("content.xml");
        let mut xml = String::new();
        content_file.read_to_string(&mut xml).unwrap();
        assert!(xml.contains("fo:font-weight=\"bold\""));
        assert!(xml.contains("fo:background-color"));
    }

    #[test]
    fn ods_export_with_merge_cells() {
        let mut content = format::empty_workbook_content("MergeTest");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets[0].cells.push(CellContent {
                address: "A1".to_string(),
                value: CellValue::String("Merged".to_string()),
                formula: None,
                style: serde_json::json!({
                    "merged": true,
                    "merge_range": "A1:C3"
                }),
            });
        }

        let bytes = export_ods_bytes(&content).expect("export");
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).expect("valid zip");
        let mut content_file = archive.by_name("content.xml").expect("content.xml");
        let mut xml = String::new();
        content_file.read_to_string(&mut xml).unwrap();
        assert!(xml.contains("table:number-columns-spanned=\"3\""));
        assert!(xml.contains("table:number-rows-spanned=\"3\""));
    }

    #[test]
    fn ods_formula_conversion() {
        assert_eq!(
            export::convert_formula_to_ods("SUM(A1:B10)"),
            "SUM([.A1]:[.B10])"
        );
        assert_eq!(export::convert_formula_to_ods("B2+B3"), "[.B2]+[.B3]");
    }
}
