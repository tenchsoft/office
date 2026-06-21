use super::export_support::{
    build_sheet_xml, build_style_registry, build_styles_xml, get_sheet_tab_color,
};
use super::xml::xml_escape;
use super::zip_io::write_zip_bytes;
use super::*;

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

/// Export OfficeContent to XLSX bytes.
pub fn export_xlsx_bytes(content: &OfficeContent) -> Result<Vec<u8>, String> {
    let workbook = match content {
        OfficeContent::Sheets(wb) => wb,
        _ => return Err("Expected Sheets content for XLSX export.".to_string()),
    };

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    // [Content_Types].xml
    files.push(("[Content_Types].xml".to_string(),
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
  <Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
</Types>"#.to_vec(),
    ));

    // _rels/.rels
    files.push(("_rels/.rels".to_string(),
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#.to_vec(),
    ));

    // Collect all string values for shared strings table
    let mut shared_strings: Vec<String> = Vec::new();
    for sheet in &workbook.sheets {
        for cell in &sheet.cells {
            if let CellValue::String(s) = &cell.value {
                if !shared_strings.contains(s) {
                    shared_strings.push(s.clone());
                }
            }
        }
    }

    // xl/_rels/workbook.xml.rels
    let mut rels = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#
        .to_vec();
    for (idx, _sheet) in workbook.sheets.iter().enumerate() {
        write!(
            rels,
            r#"  <Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{}.xml"/>"#,
            idx + 1,
            idx + 1
        )
        .unwrap();
    }
    writeln!(
        rels,
        r#"  <Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>"#,
        workbook.sheets.len() + 1
    )
    .unwrap();
    rels.extend_from_slice(b"</Relationships>");
    files.push(("xl/_rels/workbook.xml.rels".to_string(), rels));

    // Build style registry
    let mut style_registry = build_style_registry(workbook);
    let styles_xml = build_styles_xml(&style_registry);
    files.push(("xl/styles.xml".to_string(), styles_xml));

    // xl/workbook.xml
    let mut wb_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets>"#
        .to_vec();
    for (idx, sheet) in workbook.sheets.iter().enumerate() {
        // Check for tab color in sheet cells metadata
        let tab_color_attr = get_sheet_tab_color(sheet);
        write!(
            wb_xml,
            r#"<sheet name="{}" sheetId="{}" r:id="rId{}"{}>"#,
            xml_escape(&sheet.name),
            idx + 1,
            idx + 1,
            tab_color_attr
        )
        .unwrap();
        wb_xml.extend_from_slice(b"</sheet>");
    }
    wb_xml.extend_from_slice(b"</sheets></workbook>");
    files.push(("xl/workbook.xml".to_string(), wb_xml));

    // xl/sharedStrings.xml
    let mut ss_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count=""#
        .to_vec();
    write!(
        ss_xml,
        r#"{}" uniqueCount="{}">"#,
        shared_strings.len(),
        shared_strings.len()
    )
    .unwrap();
    for s in &shared_strings {
        write!(ss_xml, "<si><t>{}</t></si>", xml_escape(s)).unwrap();
    }
    ss_xml.extend_from_slice(b"</sst>");
    files.push(("xl/sharedStrings.xml".to_string(), ss_xml));

    // xl/worksheets/sheetN.xml
    for (idx, sheet) in workbook.sheets.iter().enumerate() {
        let sheet_xml = build_sheet_xml(sheet, &shared_strings, &mut style_registry);
        let sheet_path = format!("xl/worksheets/sheet{}.xml", idx + 1);
        files.push((sheet_path, sheet_xml));
    }

    write_zip_bytes(&files)
}
