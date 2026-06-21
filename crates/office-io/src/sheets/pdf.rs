//! PDF export for Tench Sheets.
//!
//! Implements a self-contained PDF writer that produces valid PDF-1.4 output
//! with grid lines, cell text, basic formatting, and page layout.

use std::fmt::Write;

use tench_document_core::{CellValue, OfficeContent, SheetContent, WorkbookContent};

use crate::sheets::format as format_io;
// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Configuration for PDF export.
#[derive(Clone, Debug)]
pub struct PdfExportConfig {
    pub paper_size: PaperSize,
    pub orientation: Orientation,
    pub margins: Margins,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub repeat_rows: Option<usize>,
    pub repeat_cols: Option<usize>,
}

impl Default for PdfExportConfig {
    fn default() -> Self {
        PdfExportConfig {
            paper_size: PaperSize::A4,
            orientation: Orientation::Portrait,
            margins: Margins::default(),
            header: None,
            footer: None,
            repeat_rows: None,
            repeat_cols: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum PaperSize {
    A4,
    Letter,
    Legal,
    A3,
}

impl PaperSize {
    fn dimensions_mm(self) -> (f32, f32) {
        match self {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::Letter => (215.9, 279.4),
            PaperSize::Legal => (215.9, 355.6),
            PaperSize::A3 => (297.0, 420.0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Clone, Copy, Debug)]
pub struct Margins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Default for Margins {
    fn default() -> Self {
        Margins {
            top: 15.0,
            bottom: 15.0,
            left: 15.0,
            right: 15.0,
        }
    }
}

/// Export an OfficeContent (Sheets) as PDF bytes.
pub fn export_pdf_bytes(
    content: &OfficeContent,
    config: &PdfExportConfig,
) -> Result<Vec<u8>, String> {
    let workbook = match content {
        OfficeContent::Sheets(wb) => wb,
        _ => return Err("Expected Sheets content for PDF export.".to_string()),
    };

    build_sheets_pdf(workbook, config)
}

// ---------------------------------------------------------------------------
// PDF builder
// ---------------------------------------------------------------------------

fn build_sheets_pdf(
    workbook: &WorkbookContent,
    config: &PdfExportConfig,
) -> Result<Vec<u8>, String> {
    let (w_mm, h_mm) = config.paper_size.dimensions_mm();
    let (pw, ph) = match config.orientation {
        Orientation::Portrait => (w_mm, h_mm),
        Orientation::Landscape => (h_mm, w_mm),
    };
    let mm_to_pt = 2.83465; // 72 / 25.4
    let page_w = pw * mm_to_pt;
    let page_h = ph * mm_to_pt;
    let ml = config.margins.left * mm_to_pt;
    let mr = config.margins.right * mm_to_pt;
    let mt = config.margins.top * mm_to_pt;
    let mb = config.margins.bottom * mm_to_pt;

    let content_w = page_w - ml - mr;
    let content_h = page_h - mt - mb;

    let default_col_w = 72.0; // ~1 inch
    let row_h = 18.0; // ~0.25 inch
    let font_size = 10.0_f32;
    let header_font_size = 8.0_f32;

    let mut all_pages: Vec<PdfPage> = Vec::new();

    for sheet in &workbook.sheets {
        let pages = render_sheet_pages(
            sheet,
            page_w,
            page_h,
            ml,
            mr,
            mt,
            mb,
            content_w,
            content_h,
            default_col_w,
            row_h,
            font_size,
            header_font_size,
            config,
            &workbook.title,
        );
        all_pages.extend(pages);
    }

    if all_pages.is_empty() {
        all_pages.push(PdfPage {
            streams: vec![format!("BT\n/F1 {} Tf\nET\n", font_size)],
            page_w,
            page_h,
        });
    }

    // Build PDF
    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n");

    let mut offsets: Vec<usize> = Vec::new();

    // Obj 1: Catalog
    offsets.push(out.len());
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Obj 2: Pages (placeholder)
    offsets.push(out.len());
    let pages_offset_pos = out.len();
    out.extend_from_slice(
        format!(
            "2 0 obj\n<< /Type /Pages /Kids [PLACEHOLDER] /Count {} >>\nendobj\n",
            all_pages.len()
        )
        .as_bytes(),
    );

    // Obj 3: Font Helvetica
    offsets.push(out.len());
    out.extend_from_slice(
        b"3 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    // Obj 4: Font Helvetica-Bold
    offsets.push(out.len());
    out.extend_from_slice(
        b"4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    // Create page objects
    let mut page_obj_ids: Vec<usize> = Vec::new();
    let first_page_obj = 5;

    for (page_idx, page) in all_pages.iter().enumerate() {
        let stream_obj_id = first_page_obj + page_idx * 2;
        let page_obj_id = stream_obj_id + 1;

        // Combine all streams
        let stream_data: String = page.streams.join("");
        let stream_bytes = stream_data.as_bytes();
        let stream_len = stream_bytes.len();

        // Stream object
        offsets.push(out.len());
        out.extend_from_slice(
            format!("{stream_obj_id} 0 obj\n<< /Length {stream_len} >>\nstream\n").as_bytes(),
        );
        out.extend_from_slice(stream_bytes);
        out.extend_from_slice(b"endstream\nendobj\n");

        // Page object
        offsets.push(out.len());
        out.extend_from_slice(
            format!(
                "{page_obj_id} 0 obj\n<< /Type /Page /Parent 2 0 R \
                 /MediaBox [0 0 {:.2} {:.2}] \
                 /Contents {stream_obj_id} 0 R \
                 /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> >>\nendobj\n",
                page.page_w, page.page_h
            )
            .as_bytes(),
        );

        page_obj_ids.push(page_obj_id);
    }

    // Fix Pages object
    let kids: String = page_obj_ids
        .iter()
        .map(|id| format!("{id} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");
    let pages_obj = format!(
        "2 0 obj\n<< /Type /Pages /Kids [{kids}] /Count {} >>\nendobj\n",
        page_obj_ids.len()
    );
    let pages_obj_bytes = pages_obj.as_bytes();

    let placeholder = b"2 0 obj\n<< /Type /Pages /Kids [PLACEHOLDER] /Count ";
    let placeholder_end = pages_offset_pos
        + placeholder.len()
        + format!("{}", all_pages.len()).len()
        + " >>\nendobj\n".len();

    let mut final_out = Vec::with_capacity(out.len() + pages_obj_bytes.len());
    final_out.extend_from_slice(&out[..pages_offset_pos]);
    final_out.extend_from_slice(pages_obj_bytes);
    final_out.extend_from_slice(&out[placeholder_end..]);

    // Recompute offsets
    let final_out_str = String::from_utf8_lossy(&final_out);
    let mut final_offsets: Vec<usize> = Vec::new();
    let mut search_pos = 0;
    while search_pos < final_out_str.len() {
        if let Some(idx) = final_out_str[search_pos..].find(" obj\n") {
            let obj_start = search_pos + idx;
            let at_line_start = obj_start == 0 || final_out_str.as_bytes()[obj_start - 1] == b'\n';
            if at_line_start {
                let before = &final_out_str[obj_start..];
                if let Some(space_pos) = before.find(' ') {
                    let num_str = &before[..space_pos];
                    if let Ok(num) = num_str.parse::<usize>() {
                        if num > 0 && num <= final_offsets.len() + 10 + all_pages.len() * 2 {
                            while final_offsets.len() < num {
                                final_offsets.push(0);
                            }
                            final_offsets[num - 1] = obj_start;
                        }
                    }
                }
            }
            search_pos = obj_start + 5;
        } else {
            break;
        }
    }

    // Cross-reference table
    let xref_offset = final_out.len();
    let num_objects = final_offsets.len() + 1;
    final_out.extend_from_slice(format!("xref\n0 {num_objects}\n").as_bytes());
    final_out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in &final_offsets {
        final_out.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
    }

    final_out.extend_from_slice(
        format!(
            "trailer\n<< /Size {num_objects} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
        )
        .as_bytes(),
    );

    Ok(final_out)
}

// ---------------------------------------------------------------------------
// Sheet rendering
// ---------------------------------------------------------------------------

struct PdfPage {
    streams: Vec<String>,
    page_w: f32,
    page_h: f32,
}

// too_many_arguments: sheet rendering requires many layout parameters
#[allow(clippy::too_many_arguments)]
fn render_sheet_pages(
    sheet: &SheetContent,
    page_w: f32,
    page_h: f32,
    ml: f32,
    _mr: f32,
    mt: f32,
    mb: f32,
    content_w: f32,
    content_h: f32,
    default_col_w: f32,
    row_h: f32,
    font_size: f32,
    header_font_size: f32,
    config: &PdfExportConfig,
    workbook_title: &str,
) -> Vec<PdfPage> {
    if sheet.cells.is_empty() {
        let mut stream = String::new();
        if let Some(ref header) = config.header {
            let y = page_h - mt + header_font_size + 2.0;
            writeln!(stream, "BT").unwrap();
            writeln!(stream, "/F1 {header_font_size} Tf").unwrap();
            writeln!(stream, "1 0 0 1 {:.2} {:.2} Tm", ml, y).unwrap();
            writeln!(stream, "({}) Tj", escape_pdf_string(header)).unwrap();
            writeln!(stream, "ET").unwrap();
        }
        if let Some(ref footer) = config.footer {
            let y = mb - header_font_size - 2.0;
            writeln!(stream, "BT").unwrap();
            writeln!(stream, "/F1 {header_font_size} Tf").unwrap();
            writeln!(stream, "1 0 0 1 {:.2} {:.2} Tm", ml, y).unwrap();
            writeln!(stream, "({}) Tj", escape_pdf_string(footer)).unwrap();
            writeln!(stream, "ET").unwrap();
        }
        if stream.is_empty() {
            stream = format!("BT\n/F1 {font_size} Tf\nET\n");
        }
        return vec![PdfPage {
            streams: vec![stream],
            page_w,
            page_h,
        }];
    }

    // Find dimensions
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
        return vec![PdfPage {
            streams: vec![format!("BT\n/F1 {font_size} Tf\nET\n")],
            page_w,
            page_h,
        }];
    }

    // Calculate column widths
    let col_widths = calculate_col_widths(sheet, max_col, default_col_w);

    // Calculate how many columns and rows fit per page
    let rows_per_page = (content_h / row_h).floor() as usize;
    let mut cols_per_page = 0usize;
    let mut width_sum = 0.0f32;
    for col in 1..=max_col {
        let w = col_widths.get(&col).copied().unwrap_or(default_col_w);
        if width_sum + w > content_w {
            break;
        }
        width_sum += w;
        cols_per_page += 1;
    }
    if cols_per_page == 0 {
        cols_per_page = 1;
    }

    // Build cell lookup
    let cell_map: std::collections::HashMap<(u32, u32), &tench_document_core::CellContent> = sheet
        .cells
        .iter()
        .filter_map(|c| {
            let row = format_io::row_from_address(&c.address)?;
            let col = format_io::col_from_address(&c.address)?;
            Some(((row, col), c))
        })
        .collect();

    let mut pages = Vec::new();

    // Paginate by columns first, then rows
    let mut col_start = 1u32;
    while col_start <= max_col {
        let mut row_start = 1u32;
        while row_start <= max_row {
            let mut stream = String::new();

            // Draw header
            if let Some(ref header) = config.header {
                let y = page_h - mt + header_font_size + 2.0;
                writeln!(stream, "BT").unwrap();
                writeln!(stream, "/F1 {header_font_size} Tf").unwrap();
                writeln!(stream, "1 0 0 1 {:.2} {:.2} Tm", ml, y).unwrap();
                writeln!(stream, "({}) Tj", escape_pdf_string(header)).unwrap();
                writeln!(stream, "ET").unwrap();
            }

            // Draw footer
            if let Some(ref footer) = config.footer {
                let y = mb - header_font_size - 2.0;
                writeln!(stream, "BT").unwrap();
                writeln!(stream, "/F1 {header_font_size} Tf").unwrap();
                writeln!(stream, "1 0 0 1 {:.2} {:.2} Tm", ml, y).unwrap();
                writeln!(stream, "({}) Tj", escape_pdf_string(footer)).unwrap();
                writeln!(stream, "ET").unwrap();
            }

            // Draw sheet name
            let y = page_h - mt + 2.0;
            writeln!(stream, "BT").unwrap();
            writeln!(stream, "/F2 {header_font_size} Tf").unwrap();
            writeln!(stream, "1 0 0 1 {:.2} {:.2} Tm", ml, y).unwrap();
            writeln!(
                stream,
                "({}) Tj",
                escape_pdf_string(&format!("{} - {}", workbook_title, sheet.name))
            )
            .unwrap();
            writeln!(stream, "ET").unwrap();

            // Repeat rows (title rows)
            let repeat_rows = config.repeat_rows.unwrap_or(0);
            let repeat_cols = config.repeat_cols.unwrap_or(0);

            // Draw grid and cells
            let col_end = (col_start + cols_per_page as u32 - 1).min(max_col);
            let row_end = (row_start + rows_per_page as u32 - 1).min(max_row);

            // Draw column headers (A, B, C...)
            let header_row_h = 14.0;
            let y_header = page_h - mt;

            // Row number column background
            write!(stream, "0.9 0.9 0.9 rg ").unwrap();
            write!(
                stream,
                "{} {} {} {} re f ",
                ml,
                y_header - header_row_h,
                20.0,
                header_row_h
            )
            .unwrap();
            write!(stream, "0 0 0 rg ").unwrap();

            let mut x = ml + 20.0;
            for col in col_start..=col_end {
                let w = col_widths.get(&col).copied().unwrap_or(default_col_w);
                write!(stream, "0.9 0.9 0.9 rg ").unwrap();
                write!(
                    stream,
                    "{} {} {} {} re f ",
                    x,
                    y_header - header_row_h,
                    w,
                    header_row_h
                )
                .unwrap();
                write!(stream, "0 0 0 rg ").unwrap();

                // Column letter
                let letter = format_io::col_to_letter(col);
                writeln!(stream, "BT").unwrap();
                writeln!(stream, "/F1 {header_font_size} Tf").unwrap();
                writeln!(
                    stream,
                    "1 0 0 1 {:.2} {:.2} Tm",
                    x + 2.0,
                    y_header - header_row_h + 3.0
                )
                .unwrap();
                writeln!(stream, "({}) Tj", escape_pdf_string(&letter)).unwrap();
                writeln!(stream, "ET").unwrap();

                x += w;
            }

            // Draw rows
            for row in row_start..=row_end {
                let actual_row = row;
                let display_y = y_header - header_row_h - (row - row_start) as f32 * row_h;

                if display_y < mb {
                    break;
                }

                // Row number
                write!(stream, "0.95 0.95 0.95 rg ").unwrap();
                write!(
                    stream,
                    "{} {} {} {} re f ",
                    ml,
                    display_y - row_h,
                    20.0,
                    row_h
                )
                .unwrap();
                write!(stream, "0 0 0 rg ").unwrap();

                writeln!(stream, "BT").unwrap();
                writeln!(stream, "/F1 {header_font_size} Tf").unwrap();
                writeln!(
                    stream,
                    "1 0 0 1 {:.2} {:.2} Tm",
                    ml + 2.0,
                    display_y - row_h + 4.0
                )
                .unwrap();
                writeln!(stream, "({actual_row}) Tj").unwrap();
                writeln!(stream, "ET").unwrap();

                // Draw cells
                let mut cell_x = ml + 20.0;
                for col in col_start..=col_end {
                    let w = col_widths.get(&col).copied().unwrap_or(default_col_w);

                    // Determine if this is a repeated row/col header
                    let is_repeat = (repeat_rows > 0 && actual_row <= repeat_rows as u32)
                        || (repeat_cols > 0 && col <= repeat_cols as u32);

                    if is_repeat {
                        write!(stream, "0.93 0.93 1.0 rg ").unwrap();
                    }

                    // Grid line
                    write!(
                        stream,
                        "0.7 0.7 0.7 RG 0.5 w {} {} {} {} re S ",
                        cell_x,
                        display_y - row_h,
                        w,
                        row_h
                    )
                    .unwrap();

                    if is_repeat {
                        write!(stream, "0 0 0 rg ").unwrap();
                    }

                    // Cell content
                    if let Some(cell) = cell_map.get(&(actual_row, col)) {
                        let text = cell_value_to_text(&cell.value);
                        if !text.is_empty() {
                            let bold = cell
                                .style
                                .get("bold")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            let font = if bold { "/F2" } else { "/F1" };
                            let alignment = cell
                                .style
                                .get("alignment_h")
                                .and_then(|v| v.as_str())
                                .unwrap_or("left");

                            let text_x = match alignment {
                                "center" => cell_x + w / 2.0,
                                "right" => cell_x + w - 3.0,
                                _ => cell_x + 3.0,
                            };

                            let align_op = match alignment {
                                "center" => " 1 0 0 1",
                                "right" => " 1 0 0 1",
                                _ => " 1 0 0 1",
                            };

                            writeln!(stream, "BT").unwrap();
                            writeln!(stream, "{font} {font_size} Tf").unwrap();
                            if alignment == "center" {
                                writeln!(
                                    stream,
                                    "{align_op} {:.2} {:.2} Tm",
                                    text_x - (text.len() as f32 * font_size * 0.3),
                                    display_y - row_h + 4.0
                                )
                                .unwrap();
                            } else {
                                writeln!(
                                    stream,
                                    "{align_op} {:.2} {:.2} Tm",
                                    text_x,
                                    display_y - row_h + 4.0
                                )
                                .unwrap();
                            }
                            writeln!(stream, "({}) Tj", escape_pdf_string(&text)).unwrap();
                            writeln!(stream, "ET").unwrap();
                        }
                    }

                    cell_x += w;
                }
            }

            if stream.is_empty() {
                writeln!(stream, "BT").unwrap();
                writeln!(stream, "/F1 {font_size} Tf").unwrap();
                writeln!(stream, "ET").unwrap();
            }

            pages.push(PdfPage {
                streams: vec![stream],
                page_w,
                page_h,
            });

            row_start += rows_per_page as u32;
        }
        col_start += cols_per_page as u32;
    }

    if pages.is_empty() {
        pages.push(PdfPage {
            streams: vec![format!("BT\n/F1 {font_size} Tf\nET\n")],
            page_w,
            page_h,
        });
    }

    pages
}

fn calculate_col_widths(
    sheet: &SheetContent,
    max_col: u32,
    default: f32,
) -> std::collections::HashMap<u32, f32> {
    let mut widths = std::collections::HashMap::new();
    for cell in &sheet.cells {
        if let Some(obj) = cell.style.as_object() {
            if let Some(v) = obj.get("_col_width").and_then(|v| v.as_f64()) {
                if let Some(col) = format_io::col_from_address(&cell.address) {
                    widths.insert(col, v as f32 * 6.0); // Approximate char to points
                }
            }
        }
    }
    // Fill defaults
    for col in 1..=max_col {
        widths.entry(col).or_insert(default);
    }
    widths
}

fn cell_value_to_text(value: &CellValue) -> String {
    match value {
        CellValue::String(s) => s.clone(),
        CellValue::Number(n) => {
            if *n == (*n as i64) as f64 {
                format!("{}", *n as i64)
            } else {
                format!("{n}")
            }
        }
        CellValue::Boolean(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        CellValue::Empty => String::new(),
    }
}

fn escape_pdf_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_export_produces_valid_header() {
        let content = format_io::empty_workbook_content("Test");
        let config = PdfExportConfig::default();
        let bytes = export_pdf_bytes(&content, &config).expect("export");
        let header = String::from_utf8_lossy(&bytes[..20]);
        assert!(header.starts_with("%PDF-1.4"));
        assert!(bytes.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn pdf_export_contains_text() {
        let mut content = format_io::empty_workbook_content("Test");
        if let OfficeContent::Sheets(ref mut wb) = content {
            wb.sheets[0].cells.push(tench_document_core::CellContent {
                address: "A1".to_string(),
                value: CellValue::String("Hello PDF".to_string()),
                formula: None,
                style: serde_json::json!({}),
            });
        }

        let config = PdfExportConfig::default();
        let bytes = export_pdf_bytes(&content, &config).expect("export");
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("(Hello PDF)"));
    }

    #[test]
    fn pdf_export_with_landscape() {
        let content = format_io::empty_workbook_content("Test");
        let config = PdfExportConfig {
            orientation: Orientation::Landscape,
            ..PdfExportConfig::default()
        };
        let bytes = export_pdf_bytes(&content, &config).expect("export");
        let pdf_str = String::from_utf8_lossy(&bytes);
        // Landscape A4: width > height in points
        assert!(pdf_str.contains("841.89")); // 297mm in points
    }

    #[test]
    fn pdf_export_with_header_footer() {
        let content = format_io::empty_workbook_content("Test");
        let config = PdfExportConfig {
            header: Some("My Header".to_string()),
            footer: Some("Page 1".to_string()),
            ..PdfExportConfig::default()
        };
        let bytes = export_pdf_bytes(&content, &config).expect("export");
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("(My Header)"));
        assert!(pdf_str.contains("(Page 1)"));
    }
}
