//! HTML export for Tench Sheets.
//!
//! Converts spreadsheet content to a simple HTML document with inline CSS styling.

use tench_document_core::{CellValue, OfficeContent};

use crate::sheets::format as format_io;

/// Export OfficeContent (Sheets) as HTML bytes.
pub fn export_html_bytes(content: &OfficeContent) -> Result<Vec<u8>, String> {
    let workbook = match content {
        OfficeContent::Sheets(wb) => wb,
        _ => return Err("Expected Sheets content for HTML export.".to_string()),
    };

    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str(&format!(
        "<title>{}</title>\n",
        html_escape(&workbook.title)
    ));
    html.push_str("<style>\n");
    html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; }\n");
    html.push_str("h1 { font-size: 1.5em; margin-bottom: 0.5em; }\n");
    html.push_str(
        "h2 { font-size: 1.2em; margin-top: 1.5em; margin-bottom: 0.3em; color: #333; }\n",
    );
    html.push_str("table { border-collapse: collapse; margin-bottom: 1em; }\n");
    html.push_str("th, td { border: 1px solid #ccc; padding: 6px 10px; text-align: left; }\n");
    html.push_str("th { background-color: #f0f0f0; font-weight: bold; }\n");
    html.push_str(".header-row { background-color: #e8e8e8; font-weight: bold; }\n");
    html.push_str(".number { text-align: right; }\n");
    html.push_str(".boolean { text-align: center; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");

    html.push_str(&format!("<h1>{}</h1>\n", html_escape(&workbook.title)));

    for sheet in &workbook.sheets {
        html.push_str(&format!("<h2>{}</h2>\n", html_escape(&sheet.name)));

        if sheet.cells.is_empty() {
            html.push_str("<p><em>Empty sheet</em></p>\n");
            continue;
        }

        let max_row = sheet
            .cells
            .iter()
            .filter_map(|c| format_io::row_from_address(&c.address))
            .max()
            .unwrap_or(0);
        let max_col = sheet
            .cells
            .iter()
            .filter_map(|c| format_io::col_from_address(&c.address))
            .max()
            .unwrap_or(0);

        if max_row == 0 || max_col == 0 {
            html.push_str("<p><em>Empty sheet</em></p>\n");
            continue;
        }

        html.push_str("<table>\n");

        // Column headers
        html.push_str("<thead><tr><th></th>");
        for col in 1..=max_col {
            html.push_str(&format!(
                "<th>{}</th>",
                html_escape(&format_io::col_to_letter(col))
            ));
        }
        html.push_str("</tr></thead>\n");

        html.push_str("<tbody>\n");

        for row in 1..=max_row {
            html.push_str("<tr>");
            // Row number
            html.push_str(&format!("<td class=\"header-row\">{}</td>", row));

            for col in 1..=max_col {
                let address = format_io::address_from_row_col(row, col);
                if let Some(cell) = sheet.cells.iter().find(|c| c.address == address) {
                    let (text, class, style) = format_cell_html(&cell.value, &cell.style);
                    let attrs = if class.is_empty() && style.is_empty() {
                        String::new()
                    } else {
                        let mut a = String::new();
                        if !class.is_empty() {
                            a.push_str(&format!(" class=\"{}\"", class));
                        }
                        if !style.is_empty() {
                            a.push_str(&format!(" style=\"{}\"", style));
                        }
                        a
                    };
                    html.push_str(&format!("<td{}>{}</td>", attrs, text));
                } else {
                    html.push_str("<td></td>");
                }
            }
            html.push_str("</tr>\n");
        }

        html.push_str("</tbody>\n</table>\n");
    }

    html.push_str("</body>\n</html>\n");

    Ok(html.into_bytes())
}

fn format_cell_html(value: &CellValue, style: &serde_json::Value) -> (String, String, String) {
    let (text, class) = match value {
        CellValue::String(s) => (html_escape(s), String::new()),
        CellValue::Number(n) => {
            let display = if *n == (*n as i64) as f64 {
                format!("{}", *n as i64)
            } else {
                format!("{n}")
            };
            (display, "number".to_string())
        }
        CellValue::Boolean(b) => (
            if *b { "TRUE" } else { "FALSE" }.to_string(),
            "boolean".to_string(),
        ),
        CellValue::Empty => (String::new(), String::new()),
    };

    let mut css_style = String::new();

    if let Some(obj) = style.as_object() {
        if obj.get("bold").and_then(|v| v.as_bool()).unwrap_or(false) {
            css_style.push_str("font-weight: bold;");
        }
        if obj.get("italic").and_then(|v| v.as_bool()).unwrap_or(false) {
            css_style.push_str("font-style: italic;");
        }
        if let Some(fs) = obj.get("font_size").and_then(|v| v.as_f64()) {
            css_style.push_str(&format!("font-size: {:.0}px;", fs));
        }
        if let Some(ff) = obj.get("font_family").and_then(|v| v.as_str()) {
            css_style.push_str(&format!("font-family: {};", ff));
        }
        if let Some(tc) = obj.get("text_color").and_then(|v| v.as_str()) {
            css_style.push_str(&format!("color: {};", tc));
        }
        if let Some(bc) = obj.get("background_color").and_then(|v| v.as_str()) {
            css_style.push_str(&format!("background-color: {};", bc));
        }
        if let Some(ah) = obj.get("alignment_h").and_then(|v| v.as_str()) {
            css_style.push_str(&format!("text-align: {};", ah));
        }
    }

    (text, class, css_style)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_export_produces_valid_html() {
        let content = format_io::empty_workbook_content("Test");
        let bytes = export_html_bytes(&content).expect("export");
        let html = String::from_utf8(bytes).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test</title>"));
        assert!(html.contains("<meta charset=\"UTF-8\">"));
    }

    #[test]
    fn html_export_contains_cell_data() {
        let mut content = format_io::empty_workbook_content("Test");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets[0].cells.push(tench_document_core::CellContent {
                address: "A1".to_string(),
                value: CellValue::String("Hello HTML".to_string()),
                formula: None,
                style: serde_json::json!({}),
            });
            wb.sheets[0].cells.push(tench_document_core::CellContent {
                address: "B1".to_string(),
                value: CellValue::Number(42.0),
                formula: None,
                style: serde_json::json!({}),
            });
        }

        let bytes = export_html_bytes(&content).expect("export");
        let html = String::from_utf8(bytes).unwrap();
        assert!(html.contains("Hello HTML"));
        assert!(html.contains("42"));
        assert!(html.contains("class=\"number\""));
    }

    #[test]
    fn html_export_with_styles() {
        let mut content = format_io::empty_workbook_content("Test");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets[0].cells.push(tench_document_core::CellContent {
                address: "A1".to_string(),
                value: CellValue::String("Bold Red".to_string()),
                formula: None,
                style: serde_json::json!({
                    "bold": true,
                    "text_color": "#FF0000",
                    "background_color": "#FFFF00"
                }),
            });
        }

        let bytes = export_html_bytes(&content).expect("export");
        let html = String::from_utf8(bytes).unwrap();
        assert!(html.contains("font-weight: bold;"));
        assert!(html.contains("color: #FF0000;"));
        assert!(html.contains("background-color: #FFFF00;"));
    }

    #[test]
    fn html_export_multiple_sheets() {
        let mut content = format_io::empty_workbook_content("Multi");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets.push(tench_document_core::SheetContent {
                id: "sheet_1".to_string(),
                name: "Sheet2".to_string(),
                index: 1,
                cells: vec![tench_document_core::CellContent {
                    address: "A1".to_string(),
                    value: CellValue::String("Second Sheet".to_string()),
                    formula: None,
                    style: serde_json::json!({}),
                }],
                row_count: Some(1000),
                column_count: Some(26),
            });
        }

        let bytes = export_html_bytes(&content).expect("export");
        let html = String::from_utf8(bytes).unwrap();
        assert!(html.contains("<h2>Sheet1</h2>"));
        assert!(html.contains("<h2>Sheet2</h2>"));
        assert!(html.contains("Second Sheet"));
    }
}
