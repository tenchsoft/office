use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use serde_json::Value;
use tench_document_core::{CellContent, CellValue, OfficeContent, SheetContent, WorkbookContent};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use super::format;

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

mod export;
mod export_support;
mod import;
mod xml;
mod zip_io;

pub use export::export_xlsx_bytes;
pub use import::import_xlsx;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::zip_io::write_zip_bytes;
    use super::*;
    use tench_document_core::OfficeContent;

    #[test]
    fn xlsx_export_produces_valid_zip() {
        let content = format::empty_workbook_content("Test");
        let bytes = export_xlsx_bytes(&content).expect("export");
        assert!(!bytes.is_empty());

        // Verify it's a valid ZIP
        let reader = std::io::Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(reader).expect("valid zip");
        assert!(archive.file_names().any(|n| n == "xl/workbook.xml"));
    }

    #[test]
    fn xlsx_round_trip_preserves_cells() {
        let dir = std::env::temp_dir().join(format!("tench_xlsx_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.xlsx");

        let mut content = format::empty_workbook_content("RoundTrip");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets[0].cells.push(CellContent {
                address: "A1".to_string(),
                value: CellValue::String("Hello".to_string()),
                formula: None,
                style: serde_json::json!({}),
            });
            wb.sheets[0].cells.push(CellContent {
                address: "B1".to_string(),
                value: CellValue::Number(42.0),
                formula: None,
                style: serde_json::json!({}),
            });
        }

        let bytes = export_xlsx_bytes(&content).expect("export");
        std::fs::write(&path, &bytes).unwrap();

        let imported = import_xlsx(&path).expect("import");
        if let OfficeContent::Sheets(wb) = imported {
            assert_eq!(wb.sheets.len(), 1);
            assert!(wb.sheets[0].cells.iter().any(|c| c.address == "A1"));
            assert!(wb.sheets[0].cells.iter().any(|c| c.address == "B1"));
        } else {
            panic!("Expected Sheets content");
        }

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn xlsx_export_with_formula() {
        let mut content = format::empty_workbook_content("FormulaTest");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets[0].cells.push(CellContent {
                address: "A1".to_string(),
                value: CellValue::Number(125.0),
                formula: Some("B2+B3".to_string()),
                style: serde_json::json!({}),
            });
        }

        let bytes = export_xlsx_bytes(&content).expect("export");
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).expect("valid zip");
        let mut sheet_file = archive.by_name("xl/worksheets/sheet1.xml").expect("sheet1");
        let mut xml = String::new();
        sheet_file.read_to_string(&mut xml).unwrap();
        assert!(xml.contains("<f>B2+B3</f>"));
        assert!(xml.contains("<v>125</v>"));
    }

    #[test]
    fn xlsx_export_with_style() {
        let mut content = format::empty_workbook_content("StyleTest");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets[0].cells.push(CellContent {
                address: "A1".to_string(),
                value: CellValue::String("Bold".to_string()),
                formula: None,
                style: serde_json::json!({
                    "bold": true,
                    "font_family": "Arial",
                    "font_size": 14.0,
                    "background_color": "#FF0000",
                    "alignment_h": "center"
                }),
            });
        }

        let bytes = export_xlsx_bytes(&content).expect("export");
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).expect("valid zip");

        // Check sheet has style index
        {
            let mut sheet_file = archive.by_name("xl/worksheets/sheet1.xml").expect("sheet1");
            let mut sheet_xml = String::new();
            sheet_file.read_to_string(&mut sheet_xml).unwrap();
            assert!(sheet_xml.contains("s=\"1\""));
        }

        // Check styles has bold and font family
        {
            let mut styles_file = archive.by_name("xl/styles.xml").expect("styles");
            let mut styles_xml = String::new();
            styles_file.read_to_string(&mut styles_xml).unwrap();
            assert!(styles_xml.contains("<b/>"));
            assert!(styles_xml.contains("Arial"));
        }
    }

    #[test]
    fn xlsx_export_with_merge_cells() {
        let mut content = format::empty_workbook_content("MergeTest");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets[0].cells.push(CellContent {
                address: "A1".to_string(),
                value: CellValue::String("Merged".to_string()),
                formula: None,
                style: serde_json::json!({
                    "merged": true,
                    "merge_range": "A1:B3"
                }),
            });
        }

        let bytes = export_xlsx_bytes(&content).expect("export");
        let reader = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader).expect("valid zip");
        let mut sheet_file = archive.by_name("xl/worksheets/sheet1.xml").expect("sheet1");
        let mut xml = String::new();
        sheet_file.read_to_string(&mut xml).unwrap();
        assert!(xml.contains("<mergeCell ref=\"A1:B3\"/>"));
    }

    #[test]
    fn xlsx_import_reads_formula() {
        let dir =
            std::env::temp_dir().join(format!("tench_xlsx_formula_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("formula_test.xlsx");

        // Create an XLSX with formula manually
        let files: Vec<(String, Vec<u8>)> = vec![
            ("[Content_Types].xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#.to_vec(),
        ),
            ("_rels/.rels".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#.to_vec(),
        ),
            ("xl/_rels/workbook.xml.rels".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#.to_vec(),
        ),
            ("xl/workbook.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#.to_vec(),
        ),
            (
            "xl/styles.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="1"><font><sz val="11"/></font></fonts>
  <fills count="1"><fill><patternFill patternType="none"/></fill></fills>
  <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
  <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
  <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>
</styleSheet>"#
                .to_vec(),
        ),
            (
            "xl/worksheets/sheet1.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<sheetData>
<row r="1"><c r="A1"><f>SUM(B1:B10)</f><v>42</v></c></row>
</sheetData>
</worksheet>"#
                .to_vec(),
        )];

        let bytes = write_zip_bytes(&files).expect("write zip");
        std::fs::write(&path, &bytes).unwrap();

        let imported = import_xlsx(&path).expect("import");
        if let OfficeContent::Sheets(wb) = imported {
            let cell = wb.sheets[0]
                .cells
                .iter()
                .find(|c| c.address == "A1")
                .expect("cell A1");
            assert_eq!(cell.formula.as_deref(), Some("SUM(B1:B10)"));
            assert_eq!(cell.value, CellValue::Number(42.0));
        }

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn xlsx_import_reads_styles() {
        let dir =
            std::env::temp_dir().join(format!("tench_xlsx_style_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("style_test.xlsx");

        let files: Vec<(String, Vec<u8>)> = vec![
            ("[Content_Types].xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#.to_vec(),
        ),
            ("_rels/.rels".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#.to_vec(),
        ),
            ("xl/_rels/workbook.xml.rels".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#.to_vec(),
        ),
            ("xl/workbook.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#.to_vec(),
        ),
            ("xl/styles.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="2">
    <font><sz val="11"/><name val="Calibri"/></font>
    <font><b/><sz val="14"/><color rgb="FFFF0000"/><name val="Arial"/></font>
  </fonts>
  <fills count="2">
    <fill><patternFill patternType="none"/></fill>
    <fill><patternFill patternType="solid"><fgColor rgb="FF00FF00"/></patternFill></fill>
  </fills>
  <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
  <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
  <cellXfs count="2">
    <xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>
    <xf numFmtId="0" fontId="1" fillId="1" borderId="0" xfId="0" applyAlignment="1" horizontal="center"/>
  </cellXfs>
</styleSheet>"#.to_vec(),
        ),
            (
            "xl/worksheets/sheet1.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<sheetData>
<row r="1"><c r="A1" s="1"><v>42</v></c></row>
</sheetData>
</worksheet>"#
                .to_vec(),
        )];

        let bytes = write_zip_bytes(&files).expect("write zip");
        std::fs::write(&path, &bytes).unwrap();

        let imported = import_xlsx(&path).expect("import");
        if let OfficeContent::Sheets(wb) = imported {
            let cell = wb.sheets[0]
                .cells
                .iter()
                .find(|c| c.address == "A1")
                .expect("cell A1");
            let style = &cell.style;
            assert_eq!(style.get("bold").and_then(|v| v.as_bool()), Some(true));
            assert_eq!(
                style.get("font_family").and_then(|v| v.as_str()),
                Some("Arial")
            );
            assert_eq!(
                style.get("alignment_h").and_then(|v| v.as_str()),
                Some("center")
            );
        }

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn xlsx_import_reads_merge_cells() {
        let dir =
            std::env::temp_dir().join(format!("tench_xlsx_merge_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("merge_test.xlsx");

        let files: Vec<(String, Vec<u8>)> = vec![
            ("[Content_Types].xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#.to_vec(),
        ),
            ("_rels/.rels".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#.to_vec(),
        ),
            ("xl/_rels/workbook.xml.rels".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#.to_vec(),
        ),
            ("xl/workbook.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#.to_vec(),
        ),
            (
            "xl/styles.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="1"><font><sz val="11"/></font></fonts>
  <fills count="1"><fill><patternFill patternType="none"/></fill></fills>
  <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
  <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
  <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>
</styleSheet>"#
                .to_vec(),
        ),
            (
            "xl/worksheets/sheet1.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<sheetData>
<row r="1"><c r="A1"><v>Hello</v></c></row>
</sheetData>
<mergeCells count="1"><mergeCell ref="A1:B3"/></mergeCells>
</worksheet>"#
                .to_vec(),
        )];

        let bytes = write_zip_bytes(&files).expect("write zip");
        std::fs::write(&path, &bytes).unwrap();

        let imported = import_xlsx(&path).expect("import");
        if let OfficeContent::Sheets(wb) = imported {
            let cell = wb.sheets[0]
                .cells
                .iter()
                .find(|c| c.address == "A1")
                .expect("cell A1");
            assert_eq!(
                cell.style.get("merge_range").and_then(|v| v.as_str()),
                Some("A1:B3")
            );
        }

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn xlsx_import_reads_col_widths_and_row_heights() {
        let dir = std::env::temp_dir().join(format!("tench_xlsx_dims_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("dims_test.xlsx");

        let files: Vec<(String, Vec<u8>)> = vec![
            ("[Content_Types].xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#.to_vec(),
        ),
            ("_rels/.rels".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#.to_vec(),
        ),
            ("xl/_rels/workbook.xml.rels".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#.to_vec(),
        ),
            ("xl/workbook.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#.to_vec(),
        ),
            (
            "xl/styles.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="1"><font><sz val="11"/></font></fonts>
  <fills count="1"><fill><patternFill patternType="none"/></fill></fills>
  <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
  <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
  <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>
</styleSheet>"#
                .to_vec(),
        ),
            (
            "xl/worksheets/sheet1.xml".to_string(),
            br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<cols><col min="1" max="1" width="25.5" customWidth="1"/></cols>
<sheetData>
<row r="1" ht="40" customHeight="1"><c r="A1"><v>100</v></c></row>
</sheetData>
</worksheet>"#
                .to_vec(),
        )];

        let bytes = write_zip_bytes(&files).expect("write zip");
        std::fs::write(&path, &bytes).unwrap();

        let imported = import_xlsx(&path).expect("import");
        if let OfficeContent::Sheets(wb) = imported {
            let cell = wb.sheets[0]
                .cells
                .iter()
                .find(|c| c.address == "A1")
                .expect("A1");
            // Column width should be stored in the cell's style
            assert_eq!(
                cell.style.get("_col_width").and_then(|v| v.as_f64()),
                Some(25.5)
            );
            assert_eq!(
                cell.style.get("_row_height").and_then(|v| v.as_f64()),
                Some(40.0)
            );
        }

        let _ = std::fs::remove_dir_all(dir);
    }
}
