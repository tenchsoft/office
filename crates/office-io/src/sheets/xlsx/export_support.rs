use super::xml::xml_escape;
use super::*;

// ---------------------------------------------------------------------------
// Style registry for export
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
struct StyleKey {
    font_family: String,
    font_size: String,
    bold: bool,
    italic: bool,
    text_color: String,
    background_color: String,
    border_top_style: String,
    border_top_color: String,
    border_bottom_style: String,
    border_bottom_color: String,
    border_left_style: String,
    border_left_color: String,
    border_right_style: String,
    border_right_color: String,
    alignment_h: String,
    alignment_v: String,
    wrap_text: bool,
    number_format: String,
}

pub(super) struct StyleRegistry {
    entries: Vec<(StyleKey, u32)>, // (key, numFmtId if custom)
    next_custom_numfmt: u32,
}

impl StyleRegistry {
    fn new() -> Self {
        StyleRegistry {
            entries: Vec::new(),
            next_custom_numfmt: 164, // Custom number formats start at 164
        }
    }

    fn get_or_insert(&mut self, key: &StyleKey) -> usize {
        if let Some(pos) = self.entries.iter().position(|(k, _)| k == key) {
            pos
        } else {
            let numfmt_id = if key.number_format.is_empty() {
                0
            } else {
                let id = self.next_custom_numfmt;
                self.next_custom_numfmt += 1;
                id
            };
            self.entries.push((key.clone(), numfmt_id));
            self.entries.len() - 1
        }
    }
}

fn style_key_from_json(style: &Value) -> StyleKey {
    let mut key = StyleKey::default();
    if let Some(obj) = style.as_object() {
        if let Some(v) = obj.get("font_family").and_then(|v| v.as_str()) {
            key.font_family = v.to_string();
        }
        if let Some(v) = obj.get("font_size").and_then(|v| v.as_f64()) {
            key.font_size = format!("{}", v);
        }
        if let Some(v) = obj.get("bold").and_then(|v| v.as_bool()) {
            key.bold = v;
        }
        if let Some(v) = obj.get("italic").and_then(|v| v.as_bool()) {
            key.italic = v;
        }
        if let Some(v) = obj.get("text_color").and_then(|v| v.as_str()) {
            key.text_color = v.to_string();
        }
        if let Some(v) = obj.get("background_color").and_then(|v| v.as_str()) {
            key.background_color = v.to_string();
        }
        if let Some(v) = obj.get("border_top").and_then(|v| v.as_str()) {
            key.border_top_style = v.to_string();
        }
        if let Some(v) = obj.get("border_top_color").and_then(|v| v.as_str()) {
            key.border_top_color = v.to_string();
        }
        if let Some(v) = obj.get("border_bottom").and_then(|v| v.as_str()) {
            key.border_bottom_style = v.to_string();
        }
        if let Some(v) = obj.get("border_bottom_color").and_then(|v| v.as_str()) {
            key.border_bottom_color = v.to_string();
        }
        if let Some(v) = obj.get("border_left").and_then(|v| v.as_str()) {
            key.border_left_style = v.to_string();
        }
        if let Some(v) = obj.get("border_left_color").and_then(|v| v.as_str()) {
            key.border_left_color = v.to_string();
        }
        if let Some(v) = obj.get("border_right").and_then(|v| v.as_str()) {
            key.border_right_style = v.to_string();
        }
        if let Some(v) = obj.get("border_right_color").and_then(|v| v.as_str()) {
            key.border_right_color = v.to_string();
        }
        if let Some(v) = obj.get("alignment_h").and_then(|v| v.as_str()) {
            key.alignment_h = v.to_string();
        }
        if let Some(v) = obj.get("alignment_v").and_then(|v| v.as_str()) {
            key.alignment_v = v.to_string();
        }
        if let Some(v) = obj.get("wrap_text").and_then(|v| v.as_bool()) {
            key.wrap_text = v;
        }
        if let Some(v) = obj.get("number_format").and_then(|v| v.as_str()) {
            key.number_format = v.to_string();
        }
    }
    key
}

pub(super) fn build_style_registry(workbook: &WorkbookContent) -> StyleRegistry {
    let mut registry = StyleRegistry::new();
    // Always add default style at index 0
    registry.get_or_insert(&StyleKey::default());
    for sheet in &workbook.sheets {
        for cell in &sheet.cells {
            let key = style_key_from_json(&cell.style);
            if key != StyleKey::default() {
                registry.get_or_insert(&key);
            }
        }
    }
    registry
}

pub(super) fn build_styles_xml(registry: &StyleRegistry) -> Vec<u8> {
    let mut xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">"#
        .to_vec();

    // Custom number formats
    let custom_fmts: Vec<(u32, &str)> = registry
        .entries
        .iter()
        .filter(|(_, numfmt_id)| *numfmt_id > 0)
        .map(|(key, numfmt_id)| (*numfmt_id, key.number_format.as_str()))
        .collect();

    if !custom_fmts.is_empty() {
        write!(xml, r#"<numFmts count="{}">"#, custom_fmts.len()).unwrap();
        for (id, fmt) in &custom_fmts {
            write!(
                xml,
                r#"<numFmt numFmtId="{}" formatCode="{}"/>"#,
                id,
                xml_escape(fmt)
            )
            .unwrap();
        }
        xml.extend_from_slice(b"</numFmts>");
    }

    // Fonts
    write!(xml, r#"<fonts count="{}">"#, registry.entries.len().max(1)).unwrap();
    for (key, _) in &registry.entries {
        xml.extend_from_slice(b"<font>");
        let sz = if key.font_size.is_empty() {
            "11"
        } else {
            &key.font_size
        };
        write!(xml, r#"<sz val="{}"/>"#, sz).unwrap();
        if key.bold {
            xml.extend_from_slice(b"<b/>");
        }
        if key.italic {
            xml.extend_from_slice(b"<i/>");
        }
        let name = if key.font_family.is_empty() {
            "Calibri"
        } else {
            &key.font_family
        };
        write!(xml, r#"<name val="{}"/>"#, xml_escape(name)).unwrap();
        if !key.text_color.is_empty() {
            write!(
                xml,
                r#"<color rgb="FF{}"/>"#,
                key.text_color.trim_start_matches('#')
            )
            .unwrap();
        }
        xml.extend_from_slice(b"</font>");
    }
    xml.extend_from_slice(b"</fonts>");

    // Fills
    write!(xml, r#"<fills count="{}">"#, registry.entries.len().max(1)).unwrap();
    // Default fill (none)
    xml.extend_from_slice(b"<fill><patternFill patternType=\"none\"/></fill>");
    for (key, _) in &registry.entries {
        if key.background_color.is_empty() {
            xml.extend_from_slice(b"<fill><patternFill patternType=\"none\"/></fill>");
        } else {
            write!(
                xml,
                r#"<fill><patternFill patternType="solid"><fgColor rgb="FF{}"/></patternFill></fill>"#,
                key.background_color.trim_start_matches('#')
            )
            .unwrap();
        }
    }
    xml.extend_from_slice(b"</fills>");

    // Borders
    write!(
        xml,
        r#"<borders count="{}">"#,
        registry.entries.len().max(1)
    )
    .unwrap();
    for (key, _) in &registry.entries {
        xml.extend_from_slice(b"<border>");
        write_border_edge(
            &mut xml,
            "left",
            &key.border_left_style,
            &key.border_left_color,
        );
        write_border_edge(
            &mut xml,
            "right",
            &key.border_right_style,
            &key.border_right_color,
        );
        write_border_edge(
            &mut xml,
            "top",
            &key.border_top_style,
            &key.border_top_color,
        );
        write_border_edge(
            &mut xml,
            "bottom",
            &key.border_bottom_style,
            &key.border_bottom_color,
        );
        xml.extend_from_slice(b"<diagonal/></border>");
    }
    xml.extend_from_slice(b"</borders>");

    // cellStyleXfs
    xml.extend_from_slice(
        br#"<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>"#,
    );

    // cellXfs
    write!(xml, r#"<cellXfs count="{}">"#, registry.entries.len()).unwrap();
    for (idx, (key, numfmt_id)) in registry.entries.iter().enumerate() {
        let numfmt = if *numfmt_id > 0 {
            numfmt_id.to_string()
        } else {
            "0".to_string()
        };
        let xf_attrs = format!(
            r#"numFmtId="{}" fontId="{}" fillId="{}" borderId="{}" xfId="0""#,
            numfmt,
            idx,
            if key.background_color.is_empty() {
                0
            } else {
                idx + 1
            },
            idx
        );

        let mut apply_attrs = String::new();
        if !key.alignment_h.is_empty() || !key.alignment_v.is_empty() || key.wrap_text {
            apply_attrs.push_str(" applyAlignment=\"1\"");
        }
        if *numfmt_id > 0 {
            apply_attrs.push_str(" applyNumberFormat=\"1\"");
        }

        let alignment_xml = build_alignment_xml(key);
        write!(
            xml,
            r#"<xf {}{}>{}</xf>"#,
            xf_attrs, apply_attrs, alignment_xml
        )
        .unwrap();
    }
    xml.extend_from_slice(b"</cellXfs>");

    xml.extend_from_slice(b"</styleSheet>");
    xml
}

fn write_border_edge(xml: &mut Vec<u8>, side: &str, style: &str, color: &str) {
    if style.is_empty() {
        write!(xml, "<{side}/>").unwrap();
    } else {
        if color.is_empty() {
            write!(xml, r#"<{side} style="{}"/>"#, style).unwrap();
        } else {
            write!(
                xml,
                r#"<{side} style="{}"><color rgb="FF{}"/></{side}>"#,
                style,
                color.trim_start_matches('#')
            )
            .unwrap();
        }
    }
}

fn build_alignment_xml(key: &StyleKey) -> String {
    let mut attrs = String::new();
    if !key.alignment_h.is_empty() {
        attrs.push_str(&format!(" horizontal=\"{}\"", key.alignment_h));
    }
    if !key.alignment_v.is_empty() {
        attrs.push_str(&format!(" vertical=\"{}\"", key.alignment_v));
    }
    if key.wrap_text {
        attrs.push_str(" wrapText=\"1\"");
    }
    if attrs.is_empty() {
        String::new()
    } else {
        format!("<alignment{}/>", attrs)
    }
}

pub(super) fn build_sheet_xml(
    sheet: &SheetContent,
    shared_strings: &[String],
    style_registry: &mut StyleRegistry,
) -> Vec<u8> {
    let mut xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">"#
        .to_vec();

    // Column widths from cell styles
    let col_widths = extract_col_widths(sheet);
    if !col_widths.is_empty() {
        xml.extend_from_slice(b"<cols>");
        for (col, width) in &col_widths {
            write!(
                xml,
                r#"<col min="{}" max="{}" width="{}" customWidth="1"/>"#,
                col, col, width
            )
            .unwrap();
        }
        xml.extend_from_slice(b"</cols>");
    }

    xml.extend_from_slice(b"<sheetData>");

    // Group cells by row for proper row element output
    let mut row_map: HashMap<u32, Vec<&CellContent>> = HashMap::new();
    for cell in &sheet.cells {
        if let Some(row) = format::row_from_address(&cell.address) {
            row_map.entry(row).or_default().push(cell);
        }
    }
    let mut rows: Vec<u32> = row_map.keys().copied().collect();
    rows.sort();

    // Row heights
    let row_heights = extract_row_heights(sheet);

    for row_num in &rows {
        let ht_attr = if let Some(ht) = row_heights.get(row_num) {
            format!(r#" ht="{}" customHeight="1""#, ht)
        } else {
            String::new()
        };
        write!(xml, r#"<row r="{}"{}>"#, row_num, ht_attr).unwrap();

        if let Some(cells) = row_map.get(row_num) {
            for cell in cells {
                let cell_ref = &cell.address;
                let style_idx = {
                    let key = style_key_from_json(&cell.style);
                    if key == StyleKey::default() {
                        None
                    } else {
                        Some(style_registry.get_or_insert(&key))
                    }
                };

                let s_attr = style_idx
                    .map(|idx| format!(r#" s="{}""#, idx))
                    .unwrap_or_default();

                match &cell.value {
                    CellValue::Number(n) => {
                        if let Some(ref formula) = cell.formula {
                            write!(
                                xml,
                                r#"<c r="{}"{}><f>{}</f><v>{}</v></c>"#,
                                cell_ref,
                                s_attr,
                                xml_escape(formula),
                                n
                            )
                            .unwrap();
                        } else {
                            write!(xml, r#"<c r="{}"{}><v>{}</v></c>"#, cell_ref, s_attr, n)
                                .unwrap();
                        }
                    }
                    CellValue::Boolean(b) => {
                        if let Some(ref formula) = cell.formula {
                            write!(
                                xml,
                                r#"<c r="{}" t="b"{}><f>{}</f><v>{}</v></c>"#,
                                cell_ref,
                                s_attr,
                                xml_escape(formula),
                                if *b { 1 } else { 0 }
                            )
                            .unwrap();
                        } else {
                            write!(
                                xml,
                                r#"<c r="{}" t="b"{}><v>{}</v></c>"#,
                                cell_ref,
                                s_attr,
                                if *b { 1 } else { 0 }
                            )
                            .unwrap();
                        }
                    }
                    CellValue::String(s) => {
                        if let Some(si) = shared_strings.iter().position(|ss| ss == s) {
                            if let Some(ref formula) = cell.formula {
                                write!(
                                    xml,
                                    r#"<c r="{}" t="s"{}><f>{}</f><v>{}</v></c>"#,
                                    cell_ref,
                                    s_attr,
                                    xml_escape(formula),
                                    si
                                )
                                .unwrap();
                            } else {
                                write!(
                                    xml,
                                    r#"<c r="{}" t="s"{}><v>{}</v></c>"#,
                                    cell_ref, s_attr, si
                                )
                                .unwrap();
                            }
                        }
                    }
                    CellValue::Empty => {
                        if cell.formula.is_some()
                            || !cell.style.as_object().is_none_or(|o| o.is_empty())
                        {
                            write!(xml, r#"<c r="{}"{}>"#, cell_ref, s_attr).unwrap();
                            if let Some(ref formula) = cell.formula {
                                write!(xml, "<f>{}</f>", xml_escape(formula)).unwrap();
                            }
                            xml.extend_from_slice(b"</c>");
                        }
                    }
                }
            }
        }
        xml.extend_from_slice(b"</row>");
    }
    xml.extend_from_slice(b"</sheetData>");

    // Merge cells
    let merge_ranges = extract_merge_ranges(sheet);
    if !merge_ranges.is_empty() {
        write!(xml, r#"<mergeCells count="{}">"#, merge_ranges.len()).unwrap();
        for range in &merge_ranges {
            write!(xml, r#"<mergeCell ref="{}"/>"#, range).unwrap();
        }
        xml.extend_from_slice(b"</mergeCells>");
    }

    // Conditional formatting
    let cond_fmts = extract_conditional_formatting(sheet);
    for (range, rules) in &cond_fmts {
        write!(xml, r#"<conditionalFormatting sqref="{}">"#, range).unwrap();
        for (priority, formula, bg_color) in rules {
            write!(
                xml,
                r#"<cfRule type="expression" dxfId="0" priority="{}"><formula>{}</formula>"#,
                priority,
                xml_escape(formula)
            )
            .unwrap();
            if !bg_color.is_empty() {
                write!(
                    xml,
                    r#"<dxf><font><b val="1"/></font><fill><patternFill><bgColor rgb="FF{}"/></patternFill></fill></dxf>"#,
                    bg_color.trim_start_matches('#')
                )
                .unwrap();
            }
            xml.extend_from_slice(b"</cfRule>");
        }
        xml.extend_from_slice(b"</conditionalFormatting>");
    }

    // Page setup
    let page_setup = extract_page_setup(sheet);
    if let Some(ps) = &page_setup {
        write!(xml, r#"<pageSetup {}"/>"#, ps).unwrap();
    }

    xml.extend_from_slice(b"</worksheet>");
    xml
}

// ---------------------------------------------------------------------------
// Helper: extract metadata from sheet cells
// ---------------------------------------------------------------------------

fn extract_col_widths(sheet: &SheetContent) -> Vec<(u32, f64)> {
    let mut widths: HashMap<u32, f64> = HashMap::new();
    for cell in &sheet.cells {
        if let Some(obj) = cell.style.as_object() {
            if let Some(v) = obj.get("_col_width").and_then(|v| v.as_f64()) {
                if let Some(col) = format::col_from_address(&cell.address) {
                    widths.insert(col, v);
                }
            }
        }
    }
    let mut result: Vec<(u32, f64)> = widths.into_iter().collect();
    result.sort_by_key(|(c, _)| *c);
    result
}

fn extract_row_heights(sheet: &SheetContent) -> HashMap<u32, f64> {
    let mut heights: HashMap<u32, f64> = HashMap::new();
    for cell in &sheet.cells {
        if let Some(obj) = cell.style.as_object() {
            if let Some(v) = obj.get("_row_height").and_then(|v| v.as_f64()) {
                if let Some(row) = format::row_from_address(&cell.address) {
                    heights.insert(row, v);
                }
            }
        }
    }
    heights
}

fn extract_merge_ranges(sheet: &SheetContent) -> Vec<String> {
    let mut ranges: Vec<String> = Vec::new();
    for cell in &sheet.cells {
        if let Some(obj) = cell.style.as_object() {
            if let Some(v) = obj.get("merge_range").and_then(|v| v.as_str()) {
                ranges.push(v.to_string());
            }
        }
    }
    ranges.dedup();
    ranges
}

// type_complexity: conditional formatting rules use nested tuples
#[allow(clippy::type_complexity)]
fn extract_conditional_formatting(
    sheet: &SheetContent,
) -> Vec<(String, Vec<(u32, String, String)>)> {
    // type_complexity: same nested tuple type
    #[allow(clippy::type_complexity)]
    let mut result: Vec<(String, Vec<(u32, String, String)>)> = Vec::new();
    for cell in &sheet.cells {
        if let Some(obj) = cell.style.as_object() {
            if let Some(arr) = obj.get("_conditional_formats").and_then(|v| v.as_array()) {
                for item in arr {
                    if let (Some(range), Some(formula), Some(priority)) = (
                        item.get("range").and_then(|v| v.as_str()),
                        item.get("formula").and_then(|v| v.as_str()),
                        item.get("priority").and_then(|v| v.as_u64()),
                    ) {
                        let bg_color = item.get("bg_color").and_then(|v| v.as_str()).unwrap_or("");
                        if let Some(entry) = result.iter_mut().find(|(r, _)| r == range) {
                            entry.1.push((
                                priority as u32,
                                formula.to_string(),
                                bg_color.to_string(),
                            ));
                        } else {
                            result.push((
                                range.to_string(),
                                vec![(priority as u32, formula.to_string(), bg_color.to_string())],
                            ));
                        }
                    }
                }
            }
        }
    }
    result
}

fn extract_page_setup(sheet: &SheetContent) -> Option<String> {
    for cell in &sheet.cells {
        if let Some(obj) = cell.style.as_object() {
            if let Some(v) = obj.get("_page_setup").and_then(|v| v.as_str()) {
                return Some(v.to_string());
            }
        }
    }
    None
}

pub(super) fn get_sheet_tab_color(sheet: &SheetContent) -> String {
    for cell in &sheet.cells {
        if let Some(obj) = cell.style.as_object() {
            if let Some(v) = obj.get("_tab_color").and_then(|v| v.as_str()) {
                return format!(r#" tabColor="{}""#, v);
            }
        }
    }
    String::new()
}
