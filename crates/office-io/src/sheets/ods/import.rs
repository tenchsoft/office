use super::*;

/// Import an ODS file into OfficeContent.
pub fn import_ods(path: &Path) -> Result<OfficeContent, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open ODS: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ODS: {e}"))?;

    // Apply archive limits
    crate::zip_util::check_archive_limits(&mut archive, &crate::zip_util::ArchiveLimits::desktop())
        .map_err(|e| format!("ODS archive limit check failed: {e}"))?;

    // Read styles.xml for cell style mapping
    let ods_styles = read_ods_styles(&mut archive);

    let mut sheets = Vec::new();

    // Try to read content.xml
    if let Ok(mut content_file) = archive.by_name("content.xml") {
        let mut xml = String::new();
        content_file.read_to_string(&mut xml).unwrap_or_default();
        sheets = parse_ods_content(&xml, &ods_styles);
    }

    if sheets.is_empty() {
        sheets.push(SheetContent {
            id: "sheet_0".to_string(),
            name: "Sheet1".to_string(),
            index: 0,
            cells: Vec::new(),
            row_count: Some(1000),
            column_count: Some(26),
        });
    }

    for (idx, sheet) in sheets.iter_mut().enumerate() {
        sheet.id = format!("sheet_{idx}");
        sheet.index = idx as u32;
    }

    let active_id = sheets[0].id.clone();
    Ok(OfficeContent::Sheets(WorkbookContent {
        id: format!(
            "wb_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ),
        title: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string(),
        sheets,
        active_sheet_id: Some(active_id),
    }))
}

// ---------------------------------------------------------------------------
// ODS import style helpers
// ---------------------------------------------------------------------------

/// A parsed ODS style from styles.xml
#[derive(Default)]
struct OdsStyle {
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

impl OdsStyle {
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        if self.bold {
            obj.insert("bold".into(), Value::Bool(true));
        }
        if self.italic {
            obj.insert("italic".into(), Value::Bool(true));
        }
        if let Some(ref fs) = self.font_size {
            obj.insert(
                "font_size".into(),
                serde_json::Number::from_f64(*fs as f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null),
            );
        }
        if let Some(ref ff) = self.font_family {
            obj.insert("font_family".into(), Value::String(ff.clone()));
        }
        if let Some(ref tc) = self.text_color {
            obj.insert("text_color".into(), Value::String(tc.clone()));
        }
        if let Some(ref bc) = self.background_color {
            obj.insert("background_color".into(), Value::String(bc.clone()));
        }
        if let Some(ref ah) = self.alignment_h {
            obj.insert("alignment_h".into(), Value::String(ah.clone()));
        }
        if let Some(ref av) = self.alignment_v {
            obj.insert("alignment_v".into(), Value::String(av.clone()));
        }
        if self.wrap_text {
            obj.insert("wrap_text".into(), Value::Bool(true));
        }
        Value::Object(obj)
    }
}

fn read_ods_styles(archive: &mut zip::ZipArchive<std::fs::File>) -> HashMap<String, OdsStyle> {
    let mut styles = HashMap::new();

    let mut xml = String::new();
    {
        let Ok(mut file) = archive.by_name("styles.xml") else {
            return styles;
        };
        file.read_to_string(&mut xml).unwrap_or_default();
    }

    // Also try content.xml for automatic styles
    let mut content_styles_xml = String::new();
    if let Ok(mut cf) = archive.by_name("content.xml") {
        cf.read_to_string(&mut content_styles_xml)
            .unwrap_or_default();
    }

    // Parse styles from styles.xml
    parse_ods_style_elements(&xml, &mut styles);
    // Parse automatic styles from content.xml
    parse_ods_style_elements(&content_styles_xml, &mut styles);

    styles
}

fn parse_ods_style_elements(xml: &str, styles: &mut HashMap<String, OdsStyle>) {
    let mut current_style_name = String::new();
    let mut current_style = OdsStyle::default();
    let mut in_style = false;

    for token in xml.split('<') {
        let token = token.trim();

        if token.starts_with("style:style ") {
            in_style = true;
            current_style = OdsStyle::default();
            current_style_name.clear();

            if let Some(name) = extract_ods_attr(token, "style:name") {
                current_style_name = name;
            }
        } else if in_style && token.starts_with("/style:style") {
            if !current_style_name.is_empty() {
                styles.insert(
                    current_style_name.clone(),
                    std::mem::take(&mut current_style),
                );
            }
            in_style = false;
        } else if in_style
            && (token.starts_with("style:text-properties ")
                || token.starts_with("style:text-properties>"))
        {
            if let Some(w) = extract_ods_attr(token, "fo:font-weight") {
                current_style.bold = w == "bold";
            }
            if let Some(fs) = extract_ods_attr(token, "fo:font-style") {
                current_style.italic = fs == "italic";
            }
            if let Some(fs) = extract_ods_attr(token, "fo:font-size") {
                current_style.font_size = fs.trim_end_matches("pt").parse().ok();
            }
            if let Some(ff) = extract_ods_attr(token, "fo:font-family") {
                current_style.font_family = Some(ff.trim_matches('"').to_string());
            }
            if let Some(tc) = extract_ods_attr(token, "fo:color") {
                current_style.text_color = Some(tc.to_string());
            }
        } else if in_style
            && (token.starts_with("style:table-cell-properties ")
                || token.starts_with("style:table-cell-properties>"))
        {
            if let Some(bc) = extract_ods_attr(token, "fo:background-color") {
                if bc != "transparent" {
                    current_style.background_color = Some(bc.to_string());
                }
            }
            if let Some(ah) = extract_ods_attr(token, "fo:text-align") {
                current_style.alignment_h = Some(ah.to_string());
            }
            if let Some(av) = extract_ods_attr(token, "style:vertical-align") {
                current_style.alignment_v = Some(av.to_string());
            }
            if token.contains("fo:wrap-option=\"wrap\"") {
                current_style.wrap_text = true;
            }
        }
    }
}

fn extract_ods_attr(token: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = token.find(&pattern) {
        let rest = &token[start + pattern.len()..];
        if let Some(end) = rest.find('"') {
            return Some(xml_unescape(&rest[..end]));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Import parsing
// ---------------------------------------------------------------------------

fn parse_ods_content(xml: &str, ods_styles: &HashMap<String, OdsStyle>) -> Vec<SheetContent> {
    let mut sheets = Vec::new();
    let mut current_sheet_name = String::new();
    let mut current_cells = Vec::new();
    let mut current_row = 0u32;
    let mut current_col = 0u32;
    let mut in_cell = false;
    let mut in_paragraph = false;
    let mut cell_value_type = String::new();
    let mut paragraph_text = String::new();
    let mut repeated_rows = 1u32;
    let mut repeated_cols = 1u32;
    let mut cell_style_name = String::new();
    let mut cell_formula = String::new();
    let mut col_span: u32 = 1;
    let mut row_span: u32 = 1;

    for token in xml.split('<') {
        let token = token.trim();

        if token.starts_with("table:table ") || token.starts_with("table:table>") {
            // New sheet
            if !current_sheet_name.is_empty() {
                sheets.push(SheetContent {
                    id: String::new(),
                    name: current_sheet_name.clone(),
                    index: 0,
                    cells: std::mem::take(&mut current_cells),
                    row_count: Some(1000),
                    column_count: Some(26),
                });
            }
            current_sheet_name.clear();
            current_cells.clear();
            current_row = 0;
            current_col = 0;

            if let Some(start) = token.find("table:name=\"") {
                let rest = &token[start + 12..];
                if let Some(end) = rest.find('"') {
                    current_sheet_name = rest[..end].to_string();
                }
            }
            if current_sheet_name.is_empty() {
                current_sheet_name = format!("Sheet{}", sheets.len() + 1);
            }
        } else if token.starts_with("table:table-row") {
            repeated_rows = 1;
            if let Some(start) = token.find("table:number-rows-repeated=\"") {
                let rest = &token[start + 28..];
                if let Some(end) = rest.find('"') {
                    repeated_rows = rest[..end].parse().unwrap_or(1);
                }
            }
        } else if token.starts_with("/table:table-row") {
            current_row += repeated_rows;
            current_col = 0;
        } else if token.starts_with("table:table-cell")
            || token.starts_with("table:covered-table-cell")
        {
            in_cell = true;
            cell_value_type.clear();
            paragraph_text.clear();
            cell_style_name.clear();
            cell_formula.clear();
            repeated_cols = 1;
            col_span = 1;
            row_span = 1;

            if let Some(start) = token.find("office:value-type=\"") {
                let rest = &token[start + 19..];
                if let Some(end) = rest.find('"') {
                    cell_value_type = rest[..end].to_string();
                }
            }
            if let Some(start) = token.find("table:number-columns-repeated=\"") {
                let rest = &token[start + 31..];
                if let Some(end) = rest.find('"') {
                    repeated_cols = rest[..end].parse().unwrap_or(1);
                }
            }
            if let Some(start) = token.find("table:style-name=\"") {
                let rest = &token[start + 18..];
                if let Some(end) = rest.find('"') {
                    cell_style_name = rest[..end].to_string();
                }
            }
            if let Some(start) = token.find("table:formula=\"") {
                let rest = &token[start + 15..];
                if let Some(end) = rest.find('"') {
                    cell_formula = xml_unescape(&rest[..end]);
                    // Strip "of:=" prefix
                    if let Some(stripped) = cell_formula.strip_prefix("of:=") {
                        cell_formula = convert_formula_from_ods(stripped);
                    }
                }
            }
            if let Some(start) = token.find("table:number-columns-spanned=\"") {
                let rest = &token[start + 30..];
                if let Some(end) = rest.find('"') {
                    col_span = rest[..end].parse().unwrap_or(1);
                }
            }
            if let Some(start) = token.find("table:number-rows-spanned=\"") {
                let rest = &token[start + 27..];
                if let Some(end) = rest.find('"') {
                    row_span = rest[..end].parse().unwrap_or(1);
                }
            }

            // Read inline float value
            if cell_value_type == "float" {
                if let Some(start) = token.find("office:value=\"") {
                    let rest = &token[start + 14..];
                    if let Some(end) = rest.find('"') {
                        paragraph_text = rest[..end].to_string();
                    }
                }
            }
        } else if token.starts_with("/table:table-cell")
            || token.starts_with("/table:covered-table-cell")
        {
            if in_cell {
                let mut style_json = if let Some(ods_style) = ods_styles.get(&cell_style_name) {
                    ods_style.to_json()
                } else {
                    serde_json::json!({})
                };

                // Add merge info
                if col_span > 1 || row_span > 1 {
                    let start_col = current_col + 1;
                    let start_row = current_row + 1;
                    let end_col = start_col + col_span - 1;
                    let end_row = start_row + row_span - 1;
                    let start_addr = format::address_from_row_col(start_row, start_col);
                    let end_addr = format::address_from_row_col(end_row, end_col);
                    if let Some(obj) = style_json.as_object_mut() {
                        obj.insert("merged".into(), Value::Bool(true));
                        obj.insert(
                            "merge_range".into(),
                            Value::String(format!("{}:{}", start_addr, end_addr)),
                        );
                    }
                }

                for offset in 0..repeated_cols {
                    let col = current_col + offset + 1;
                    let row = current_row + 1;
                    let address = format::address_from_row_col(row, col);
                    let value = match cell_value_type.as_str() {
                        "float" => paragraph_text
                            .parse::<f64>()
                            .map(CellValue::Number)
                            .unwrap_or(CellValue::String(paragraph_text.clone())),
                        "boolean" => CellValue::Boolean(paragraph_text == "TRUE"),
                        _ => {
                            if paragraph_text.is_empty() {
                                CellValue::Empty
                            } else {
                                CellValue::String(paragraph_text.clone())
                            }
                        }
                    };

                    let formula = if cell_formula.is_empty() {
                        None
                    } else {
                        Some(cell_formula.clone())
                    };

                    current_cells.push(CellContent {
                        address,
                        value,
                        formula,
                        style: style_json.clone(),
                    });
                }
            }
            current_col += repeated_cols;
            in_cell = false;
        } else if token.starts_with("text:p") {
            in_paragraph = true;
            paragraph_text.clear();
            if let Some(text) = token.strip_prefix("text:p>") {
                paragraph_text.push_str(text);
            }
        } else if in_paragraph && token.starts_with('/') {
            in_paragraph = false;
        } else if in_paragraph {
            paragraph_text.push('<');
            paragraph_text.push_str(token);
        }
    }

    // Don't forget the last sheet
    if !current_sheet_name.is_empty() {
        sheets.push(SheetContent {
            id: String::new(),
            name: current_sheet_name,
            index: 0,
            cells: current_cells,
            row_count: Some(1000),
            column_count: Some(26),
        });
    }

    sheets
}

fn convert_formula_from_ods(formula: &str) -> String {
    // Convert ODS OpenFormula to Excel-like formula
    // Replace [.A1] with A1
    let mut result = formula.to_string();
    // Simple approach: remove [. and ] around cell references
    result = result.replace("[.", "").replace("]", "");
    result
}
