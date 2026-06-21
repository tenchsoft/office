use super::*;

/// Export OfficeContent to ODS bytes.
pub fn export_ods_bytes(content: &OfficeContent) -> Result<Vec<u8>, String> {
    let workbook = match content {
        OfficeContent::Sheets(wb) => wb,
        _ => return Err("Expected Sheets content for ODS export.".to_string()),
    };

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    // mimetype (must be first, uncompressed)
    files.push((
        "mimetype".to_string(),
        b"application/vnd.oasis.opendocument.spreadsheet".to_vec(),
    ));

    // META-INF/manifest.xml
    files.push((
        "META-INF/manifest.xml".to_string(),
        br#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
  <manifest:file-entry manifest:media-type="application/vnd.oasis.opendocument.spreadsheet" manifest:full-path="/"/>
  <manifest:file-entry manifest:media-type="text/xml" manifest:full-path="content.xml"/>
  <manifest:file-entry manifest:media-type="text/xml" manifest:full-path="styles.xml"/>
</manifest:manifest>"#.to_vec(),
    ));

    // Build automatic styles from cell styles
    let auto_styles = build_ods_auto_styles(workbook);
    let styles_xml = build_ods_styles_xml();
    files.push(("styles.xml".to_string(), styles_xml));

    // content.xml
    let mut content_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
  xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
  xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"
  xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">
<office:automatic-styles>"#
        .to_vec();

    // Write automatic styles for column widths, row heights, and cell styles
    let auto_style_xml = build_ods_content_auto_styles(workbook, &auto_styles);
    content_xml.extend_from_slice(&auto_style_xml);

    content_xml.extend_from_slice(b"</office:automatic-styles><office:body><office:spreadsheet>");

    for sheet in &workbook.sheets {
        write!(
            content_xml,
            r#"<table:table table:name="{}">"#,
            xml_escape(&sheet.name)
        )
        .unwrap();

        // Find max row/col
        let max_row = sheet
            .cells
            .iter()
            .filter_map(|c| format::row_from_address(&c.address))
            .max()
            .unwrap_or(0);
        let max_col = sheet
            .cells
            .iter()
            .filter_map(|c| format::col_from_address(&c.address))
            .max()
            .unwrap_or(0);

        // Column declarations with widths
        let col_widths = extract_col_widths_ods(sheet);
        for col in 1..=max_col {
            if col_widths.contains_key(&col) {
                write!(
                    content_xml,
                    r#"<table:table-column table:style-name="co{}"/>"#,
                    col
                )
                .unwrap();
            } else {
                content_xml.extend_from_slice(b"<table:table-column/>");
            }
        }

        // Collect row heights
        let row_heights = extract_row_heights_ods(sheet);

        for row in 1..=max_row {
            // Row with optional height style
            if row_heights.contains_key(&row) {
                write!(
                    content_xml,
                    r#"<table:table-row table:style-name="ro{}">"#,
                    row
                )
                .unwrap();
            } else {
                content_xml.extend_from_slice(b"<table:table-row>");
            }

            for col in 1..=max_col {
                let address = format::address_from_row_col(row, col);
                if let Some(cell) = sheet.cells.iter().find(|c| c.address == address) {
                    let style_attr = if has_non_empty_style(&cell.style) {
                        let style_name = get_ods_cell_style_name(&cell.style, &auto_styles);
                        format!(r#" table:style-name="{}""#, style_name)
                    } else {
                        String::new()
                    };

                    // Handle merged cells
                    let merge_attr = build_ods_merge_attrs(&cell.style);

                    // Handle formula
                    let formula_attr = if let Some(ref formula) = cell.formula {
                        format!(
                            r#" table:formula="of:={}""#,
                            xml_escape(&convert_formula_to_ods(formula))
                        )
                    } else {
                        String::new()
                    };

                    match &cell.value {
                        CellValue::String(s) => {
                            write!(
                                content_xml,
                                r#"<table:table-cell office:value-type="string"{}{}{}><text:p>{}</text:p></table:table-cell>"#,
                                style_attr, merge_attr, formula_attr,
                                xml_escape(s)
                            )
                            .unwrap();
                        }
                        CellValue::Number(n) => {
                            write!(
                                content_xml,
                                r#"<table:table-cell office:value-type="float" office:value="{}"{}{}{}><text:p>{}</text:p></table:table-cell>"#,
                                n, style_attr, merge_attr, formula_attr, n
                            )
                            .unwrap();
                        }
                        CellValue::Boolean(b) => {
                            write!(
                                content_xml,
                                r#"<table:table-cell office:value-type="boolean" office:boolean-value="{}"{}{}{}><text:p>{}</text:p></table:table-cell>"#,
                                if *b { "true" } else { "false" },
                                style_attr, merge_attr, formula_attr,
                                if *b { "TRUE" } else { "FALSE" }
                            )
                            .unwrap();
                        }
                        CellValue::Empty => {
                            if style_attr.is_empty()
                                && merge_attr.is_empty()
                                && formula_attr.is_empty()
                            {
                                content_xml.extend_from_slice(
                                    b"<table:table-cell office:value-type=\"string\"/>",
                                );
                            } else {
                                write!(
                                    content_xml,
                                    r#"<table:table-cell office:value-type="string"{}{}{}/>"#,
                                    style_attr, merge_attr, formula_attr
                                )
                                .unwrap();
                            }
                        }
                    }
                } else {
                    content_xml.extend_from_slice(b"<table:table-cell/>");
                }
            }
            content_xml.extend_from_slice(b"</table:table-row>");
        }

        content_xml.extend_from_slice(b"</table:table>");
    }

    content_xml.extend_from_slice(b"</office:spreadsheet></office:body></office:document-content>");
    files.push(("content.xml".to_string(), content_xml));

    write_zip_bytes(&files)
}

// ---------------------------------------------------------------------------
// Export style helpers
// ---------------------------------------------------------------------------

struct OdsAutoStyle {
    name: String,
    bold: bool,
    italic: bool,
    font_size: Option<f32>,
    font_family: Option<String>,
    text_color: Option<String>,
    background_color: Option<String>,
    alignment_h: Option<String>,
    alignment_v: Option<String>,
    wrap_text: bool,
}

fn build_ods_auto_styles(workbook: &WorkbookContent) -> Vec<OdsAutoStyle> {
    let mut style_map: HashMap<String, usize> = HashMap::new();
    let mut styles = Vec::new();

    for sheet in &workbook.sheets {
        for cell in &sheet.cells {
            let key = ods_cell_style_key(&cell.style);
            if key == "default" {
                continue;
            }
            if !style_map.contains_key(&key) {
                style_map.insert(key.clone(), styles.len());
                styles.push(OdsAutoStyle {
                    name: format!("ce{}", styles.len()),
                    bold: cell
                        .style
                        .get("bold")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    italic: cell
                        .style
                        .get("italic")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    font_size: cell
                        .style
                        .get("font_size")
                        .and_then(|v| v.as_f64())
                        .map(|v| v as f32),
                    font_family: cell
                        .style
                        .get("font_family")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    text_color: cell
                        .style
                        .get("text_color")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    background_color: cell
                        .style
                        .get("background_color")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    alignment_h: cell
                        .style
                        .get("alignment_h")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    alignment_v: cell
                        .style
                        .get("alignment_v")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    wrap_text: cell
                        .style
                        .get("wrap_text")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                });
            }
        }
    }
    styles
}

fn ods_cell_style_key(style: &Value) -> String {
    if let Some(obj) = style.as_object() {
        let mut parts: Vec<String> = Vec::new();
        if obj.get("bold").and_then(|v| v.as_bool()).unwrap_or(false) {
            parts.push("b".to_string());
        }
        if obj.get("italic").and_then(|v| v.as_bool()).unwrap_or(false) {
            parts.push("i".to_string());
        }
        if let Some(v) = obj.get("font_size").and_then(|v| v.as_f64()) {
            parts.push(format!("fs{}", v));
        }
        if let Some(v) = obj.get("font_family").and_then(|v| v.as_str()) {
            parts.push(v.to_string());
        }
        if let Some(v) = obj.get("text_color").and_then(|v| v.as_str()) {
            parts.push(v.to_string());
        }
        if let Some(v) = obj.get("background_color").and_then(|v| v.as_str()) {
            parts.push(v.to_string());
        }
        if let Some(v) = obj.get("alignment_h").and_then(|v| v.as_str()) {
            parts.push(v.to_string());
        }
        if let Some(v) = obj.get("alignment_v").and_then(|v| v.as_str()) {
            parts.push(v.to_string());
        }
        if obj
            .get("wrap_text")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            parts.push("wrap".to_string());
        }
        if parts.is_empty() {
            "default".to_string()
        } else {
            parts.join("|")
        }
    } else {
        "default".to_string()
    }
}

fn get_ods_cell_style_name(style: &Value, auto_styles: &[OdsAutoStyle]) -> String {
    let key = ods_cell_style_key(style);
    if key == "default" {
        return "Default".to_string();
    }
    // Find matching auto style
    for s in auto_styles {
        if ods_style_matches_key(s, &key) {
            return s.name.clone();
        }
    }
    "Default".to_string()
}

fn ods_style_matches_key(s: &OdsAutoStyle, key: &str) -> bool {
    let mut parts: Vec<String> = Vec::new();
    if s.bold {
        parts.push("b".to_string());
    }
    if s.italic {
        parts.push("i".to_string());
    }
    if let Some(fs) = s.font_size {
        parts.push(format!("fs{}", fs));
    }
    if let Some(ref ff) = s.font_family {
        parts.push(ff.clone());
    }
    if let Some(ref tc) = s.text_color {
        parts.push(tc.clone());
    }
    if let Some(ref bc) = s.background_color {
        parts.push(bc.clone());
    }
    if let Some(ref ah) = s.alignment_h {
        parts.push(ah.clone());
    }
    if let Some(ref av) = s.alignment_v {
        parts.push(av.clone());
    }
    if s.wrap_text {
        parts.push("wrap".to_string());
    }
    parts.join("|") == key
}

fn has_non_empty_style(style: &Value) -> bool {
    if let Some(obj) = style.as_object() {
        // Ignore internal metadata keys (starting with _)
        obj.keys().any(|k| !k.starts_with('_'))
    } else {
        false
    }
}

fn build_ods_merge_attrs(style: &Value) -> String {
    let mut attrs = String::new();
    if let Some(obj) = style.as_object() {
        if obj.get("merged").and_then(|v| v.as_bool()).unwrap_or(false) {
            if let Some(range) = obj.get("merge_range").and_then(|v| v.as_str()) {
                // Parse "A1:B3" into col_span and row_span
                if let Some((start, end)) = range.split_once(':') {
                    let start_col = format::col_from_address(start).unwrap_or(1);
                    let start_row = format::row_from_address(start).unwrap_or(1);
                    let end_col = format::col_from_address(end).unwrap_or(start_col);
                    let end_row = format::row_from_address(end).unwrap_or(start_row);
                    let col_span = end_col - start_col + 1;
                    let row_span = end_row - start_row + 1;
                    if col_span > 1 {
                        attrs.push_str(&format!(r#" table:number-columns-spanned="{}""#, col_span));
                    }
                    if row_span > 1 {
                        attrs.push_str(&format!(r#" table:number-rows-spanned="{}""#, row_span));
                    }
                }
            }
        }
    }
    attrs
}

pub(super) fn convert_formula_to_ods(formula: &str) -> String {
    // Basic conversion from Excel-like formula to ODS OpenFormula
    // Replace cell references: A1 -> [.A1]
    let mut result = String::new();
    let chars: Vec<char> = formula.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_alphabetic() {
            // Start of potential cell reference
            let mut col_part = String::new();
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                col_part.push(chars[i]);
                i += 1;
            }
            if i < chars.len() && chars[i].is_ascii_digit() {
                let mut row_part = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    row_part.push(chars[i]);
                    i += 1;
                }
                // It's a cell reference
                result.push_str(&format!("[.{}{}]", col_part, row_part));
            } else {
                result.push_str(&col_part);
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

fn build_ods_styles_xml() -> Vec<u8> {
    let mut xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
  xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
  xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0">"#
        .to_vec();

    // Default cell style
    xml.extend_from_slice(
        br#"<office:styles><style:default-style style:family="table-cell"><style:text-properties fo:font-size="12pt"/></style:default-style></office:styles>"#,
    );

    xml.extend_from_slice(b"</office:document-styles>");
    xml
}

fn build_ods_content_auto_styles(
    workbook: &WorkbookContent,
    auto_styles: &[OdsAutoStyle],
) -> Vec<u8> {
    let mut xml = Vec::new();

    // Column width styles
    for sheet in &workbook.sheets {
        let col_widths = extract_col_widths_ods(sheet);
        for (col, width) in &col_widths {
            write!(
                xml,
                r#"<style:style style:name="co{}" style:family="table-column"><style:table-column-properties fo:break-before="auto" style:column-width="{:.2}cm"/></style:style>"#,
                col, width
            )
            .unwrap();
        }

        // Row height styles
        let row_heights = extract_row_heights_ods(sheet);
        for (row, height) in &row_heights {
            write!(
                xml,
                r#"<style:style style:name="ro{}" style:family="table-row"><style:table-row-properties style:row-height="{:.2}cm" fo:break-before="auto"/></style:style>"#,
                row, height
            )
            .unwrap();
        }
    }

    // Cell styles
    for s in auto_styles {
        write!(
            xml,
            r#"<style:style style:name="{}" style:family="table-cell">"#,
            s.name
        )
        .unwrap();

        // Text properties
        let mut text_props = String::new();
        if s.bold {
            text_props.push_str(r#" fo:font-weight="bold""#);
        }
        if s.italic {
            text_props.push_str(r#" fo:font-style="italic""#);
        }
        if let Some(fs) = s.font_size {
            text_props.push_str(&format!(r#" fo:font-size="{:.0}pt""#, fs));
        }
        if let Some(ref ff) = s.font_family {
            text_props.push_str(&format!(r#" fo:font-family="{}""#, xml_escape(ff)));
        }
        if let Some(ref tc) = s.text_color {
            text_props.push_str(&format!(r#" fo:color="{}""#, tc));
        }
        if !text_props.is_empty() {
            write!(xml, r#"<style:text-properties{}/>"#, text_props).unwrap();
        }

        // Cell properties
        let mut cell_props = String::new();
        if let Some(ref bc) = s.background_color {
            cell_props.push_str(&format!(r#" fo:background-color="{}""#, bc));
        }
        if let Some(ref ah) = s.alignment_h {
            cell_props.push_str(&format!(r#" fo:text-align="{}""#, ah));
        }
        if let Some(ref av) = s.alignment_v {
            cell_props.push_str(&format!(r#" style:vertical-align="{}""#, av));
        }
        if s.wrap_text {
            cell_props.push_str(r#" fo:wrap-option="wrap""#);
        }
        if !cell_props.is_empty() {
            write!(xml, r#"<style:table-cell-properties{}/>"#, cell_props).unwrap();
        }

        xml.extend_from_slice(b"</style:style>");
    }

    xml
}

fn extract_col_widths_ods(sheet: &SheetContent) -> HashMap<u32, f64> {
    let mut widths: HashMap<u32, f64> = HashMap::new();
    for cell in &sheet.cells {
        if let Some(obj) = cell.style.as_object() {
            if let Some(v) = obj.get("_col_width").and_then(|v| v.as_f64()) {
                if let Some(col) = format::col_from_address(&cell.address) {
                    // Convert from character width to cm (approximate: 1 char ~ 0.25cm)
                    widths.insert(col, v * 0.25);
                }
            }
        }
    }
    widths
}

fn extract_row_heights_ods(sheet: &SheetContent) -> HashMap<u32, f64> {
    let mut heights: HashMap<u32, f64> = HashMap::new();
    for cell in &sheet.cells {
        if let Some(obj) = cell.style.as_object() {
            if let Some(v) = obj.get("_row_height").and_then(|v| v.as_f64()) {
                if let Some(row) = format::row_from_address(&cell.address) {
                    // Convert from points to cm (1pt ~ 0.0353cm)
                    heights.insert(row, v * 0.0353);
                }
            }
        }
    }
    heights
}
