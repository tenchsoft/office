use super::*;

// ---------------------------------------------------------------------------
// Sheet XML parsing (import)
// ---------------------------------------------------------------------------

pub(super) struct MergeCellRange {
    ref_str: String,
}

pub(super) struct ColWidth {
    min: u32,
    max: u32,
    width: f64,
}

pub(super) struct RowHeight {
    row: u32,
    height: f64,
}

pub(super) struct CondFmtRule {
    sqref: String,
    priority: u32,
    formula: String,
}

pub(super) struct PageSetupData {
    attrs: String,
}

// type_complexity: sheet parsing returns many collections
#[allow(clippy::type_complexity)]
pub(super) fn parse_sheet_xml_full(
    xml: &str,
    shared_strings: &[String],
    styles: &[XlsxStyle],
) -> (
    Vec<CellContent>,
    Vec<MergeCellRange>,
    Vec<ColWidth>,
    Vec<RowHeight>,
    Vec<CondFmtRule>,
    Option<PageSetupData>,
) {
    let mut cells = Vec::new();
    let mut merge_cells = Vec::new();
    let mut col_widths = Vec::new();
    let mut row_heights = Vec::new();
    let cond_fmts = Vec::new();
    let mut page_setup: Option<PageSetupData> = None;

    let mut current_ref = String::new();
    let mut current_type = String::new();
    let mut current_style_idx: Option<usize> = None;
    let mut in_v = false;
    let mut in_f = false;
    let mut value_text = String::new();
    let mut formula_text = String::new();

    for token in xml.split('<') {
        let token = token.trim();

        // Column widths
        if token.starts_with("col ") {
            let min = extract_attr(token, "min")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            let max = extract_attr(token, "max")
                .and_then(|s| s.parse().ok())
                .unwrap_or(min);
            let width = extract_attr(token, "width")
                .and_then(|s| s.parse().ok())
                .unwrap_or(9.0);
            col_widths.push(ColWidth { min, max, width });
        }
        // Row heights
        else if token.starts_with("row ") {
            if let Some(r) = extract_attr(token, "r").and_then(|s| s.parse::<u32>().ok()) {
                if let Some(ht) = extract_attr(token, "ht").and_then(|s| s.parse::<f64>().ok()) {
                    row_heights.push(RowHeight { row: r, height: ht });
                }
            }
        }
        // Merge cells
        else if token.starts_with("mergeCell ") {
            if let Some(ref_str) = extract_attr(token, "ref") {
                merge_cells.push(MergeCellRange { ref_str });
            }
        }
        // Conditional formatting
        else if token.starts_with("conditionalFormatting ") {
            // sqref handled via rules
        } else if token.starts_with("cfRule ") {
            let sqref = String::new(); // Would need context from parent
            let priority = extract_attr(token, "priority")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            // Formula is in a child element, will be captured below
            let _ = (sqref, priority);
        }
        // Page setup
        else if token.starts_with("pageSetup ") {
            let mut attrs = String::new();
            if let Some(v) = extract_attr(token, "paperSize") {
                attrs.push_str(&format!("paperSize=\"{}\" ", v));
            }
            if let Some(v) = extract_attr(token, "orientation") {
                attrs.push_str(&format!("orientation=\"{}\" ", v));
            }
            page_setup = Some(PageSetupData { attrs });
        }
        // Cell start
        else if token.starts_with("c ") || token.starts_with("c>") {
            // Flush previous cell
            if !current_ref.is_empty() {
                if let Some(cell) = build_cell_full(
                    &current_ref,
                    &current_type,
                    &value_text,
                    &formula_text,
                    shared_strings,
                    styles,
                    current_style_idx,
                ) {
                    cells.push(cell);
                }
            }
            current_ref.clear();
            current_type.clear();
            current_style_idx = None;
            value_text.clear();
            formula_text.clear();

            if let Some(start) = token.find("r=\"") {
                let rest = &token[start + 3..];
                if let Some(end) = rest.find('"') {
                    current_ref = rest[..end].to_string();
                }
            }
            if let Some(start) = token.find("t=\"") {
                let rest = &token[start + 3..];
                if let Some(end) = rest.find('"') {
                    current_type = rest[..end].to_string();
                }
            }
            if let Some(start) = token.find("s=\"") {
                let rest = &token[start + 3..];
                if let Some(end) = rest.find('"') {
                    current_style_idx = rest[..end].parse().ok();
                }
            }
        }
        // Formula
        else if token.starts_with("f>") {
            in_f = true;
            formula_text.clear();
            if let Some(text) = token.strip_prefix("f>") {
                formula_text.push_str(text);
            }
        } else if in_f && token.starts_with("/") {
            in_f = false;
        } else if in_f {
            formula_text.push('<');
            formula_text.push_str(token);
        }
        // Value
        else if token.starts_with("v>") {
            in_v = true;
            value_text.clear();
            if let Some(text) = token.strip_prefix("v>") {
                value_text.push_str(text);
            }
        } else if in_v && token.starts_with("/") {
            in_v = false;
        } else if in_v {
            value_text.push('<');
            value_text.push_str(token);
        }
    }

    // Flush last cell
    if !current_ref.is_empty() {
        if let Some(cell) = build_cell_full(
            &current_ref,
            &current_type,
            &value_text,
            &formula_text,
            shared_strings,
            styles,
            current_style_idx,
        ) {
            cells.push(cell);
        }
    }

    (
        cells,
        merge_cells,
        col_widths,
        row_heights,
        cond_fmts,
        page_setup,
    )
}

fn build_cell_full(
    address: &str,
    cell_type: &str,
    value_text: &str,
    formula_text: &str,
    shared_strings: &[String],
    styles: &[XlsxStyle],
    style_idx: Option<usize>,
) -> Option<CellContent> {
    let value = match cell_type {
        "s" => {
            let idx: usize = value_text.parse().ok()?;
            shared_strings
                .get(idx)
                .map(|s| CellValue::String(s.clone()))
                .unwrap_or(CellValue::Empty)
        }
        "b" => CellValue::Boolean(value_text == "1"),
        "str" => CellValue::String(value_text.to_string()),
        _ => {
            if value_text.is_empty() && !formula_text.is_empty() {
                CellValue::Empty
            } else if let Ok(n) = value_text.parse::<f64>() {
                CellValue::Number(n)
            } else if value_text.is_empty() {
                CellValue::Empty
            } else {
                CellValue::String(value_text.to_string())
            }
        }
    };

    let formula = if formula_text.is_empty() {
        None
    } else {
        Some(formula_text.to_string())
    };

    let style = if let Some(idx) = style_idx {
        if let Some(s) = styles.get(idx) {
            s.to_json()
        } else {
            serde_json::json!({})
        }
    } else {
        serde_json::json!({})
    };

    Some(CellContent {
        address: address.to_string(),
        value,
        formula,
        style,
    })
}

pub(super) fn apply_merge_cells(sheet: &mut SheetContent, merge_cells: &[MergeCellRange]) {
    for mc in merge_cells {
        let parts: Vec<&str> = mc.ref_str.split(':').collect();
        if parts.len() == 2 {
            // Mark the top-left cell with merge_range
            let top_left = parts[0];
            if let Some(cell) = sheet.cells.iter_mut().find(|c| c.address == top_left) {
                if let Some(obj) = cell.style.as_object_mut() {
                    obj.insert("merge_range".into(), Value::String(mc.ref_str.clone()));
                    obj.insert("merged".into(), Value::Bool(true));
                }
            }
        }
    }
}

pub(super) fn apply_col_row_metadata(
    sheet: &mut SheetContent,
    col_widths: &[ColWidth],
    row_heights: &[RowHeight],
) {
    // Store column widths in first row cells
    for cw in col_widths {
        for col in cw.min..=cw.max {
            let address = format::address_from_row_col(1, col);
            if let Some(cell) = sheet.cells.iter_mut().find(|c| c.address == address) {
                if let Some(obj) = cell.style.as_object_mut() {
                    obj.insert(
                        "_col_width".into(),
                        serde_json::Number::from_f64(cw.width)
                            .map(Value::Number)
                            .unwrap_or(Value::Null),
                    );
                }
            }
        }
    }

    // Store row heights in first column cells
    for rh in row_heights {
        let address = format::address_from_row_col(rh.row, 1);
        if let Some(cell) = sheet.cells.iter_mut().find(|c| c.address == address) {
            if let Some(obj) = cell.style.as_object_mut() {
                obj.insert(
                    "_row_height".into(),
                    serde_json::Number::from_f64(rh.height)
                        .map(Value::Number)
                        .unwrap_or(Value::Null),
                );
            }
        }
    }
}

pub(super) fn apply_conditional_formatting(sheet: &mut SheetContent, cond_fmts: &[CondFmtRule]) {
    // Store conditional formatting rules as metadata on the first cell or a sentinel cell.
    // Each rule is serialized into the cell's style JSON under "_conditional_formats".
    if cond_fmts.is_empty() {
        return;
    }
    let rules_json: Vec<serde_json::Value> = cond_fmts
        .iter()
        .map(|r| {
            serde_json::json!({
                "range": r.sqref,
                "priority": r.priority,
                "formula": r.formula,
            })
        })
        .collect();
    // Find or create a metadata cell at A1
    if let Some(cell) = sheet.cells.first_mut() {
        if let Some(obj) = cell.style.as_object_mut() {
            obj.insert(
                "_conditional_formats".to_string(),
                serde_json::Value::Array(rules_json),
            );
        }
    }
}

pub(super) fn apply_page_setup(sheet: &mut SheetContent, page_setup: Option<&PageSetupData>) {
    // Store page setup as metadata on the first cell's style JSON.
    let Some(ps) = page_setup else {
        return;
    };
    if let Some(cell) = sheet.cells.first_mut() {
        if let Some(obj) = cell.style.as_object_mut() {
            obj.insert("_page_setup".to_string(), serde_json::json!(ps.attrs));
        }
    }
}

pub(super) fn apply_tab_color(sheet: &mut SheetContent, tab_color: Option<&str>) {
    let Some(color) = tab_color else {
        return;
    };
    if let Some(cell) = sheet.cells.first_mut() {
        if let Some(obj) = cell.style.as_object_mut() {
            obj.insert("_tab_color".to_string(), serde_json::json!(color));
        }
    }
}
