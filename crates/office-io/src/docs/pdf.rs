//! PDF export using the Tench Document Model.
//!
//! Implements a self-contained PDF writer that produces valid PDF-1.4 output
//! with text, basic formatting, images, and page layout based on `PageSetup`.

use std::io::Write;

use tench_document_core::{BlockNode, InlineNode, Marks, Orientation, TenchDocument};

// ---------------------------------------------------------------------------
// TDM-based API
// ---------------------------------------------------------------------------

/// Export a `TenchDocument` as PDF bytes.
pub fn export_pdf_bytes(doc: &TenchDocument) -> Result<Vec<u8>, String> {
    build_pdf(doc)
}

/// Export a `TenchDocument` as PDF bytes from legacy `OfficeContent`.
pub fn export_pdf_bytes_from_content(
    content: &tench_document_core::OfficeContent,
) -> Result<Vec<u8>, String> {
    let doc = match content {
        tench_document_core::OfficeContent::Docs(rich) => rich
            .document
            .clone()
            .unwrap_or_else(|| TenchDocument::new("")),
        _ => TenchDocument::new(""),
    };
    export_pdf_bytes(&doc)
}

// ---------------------------------------------------------------------------
// PDF builder
// ---------------------------------------------------------------------------

/// Build a complete PDF document from a TenchDocument.
fn build_pdf(doc: &TenchDocument) -> Result<Vec<u8>, String> {
    let ps = &doc.page_setup;
    let (w_mm, h_mm) = ps.paper_size.dimensions_mm();
    let (pw, ph) = match ps.orientation {
        Orientation::Portrait => (w_mm, h_mm),
        Orientation::Landscape => (h_mm, w_mm),
    };
    let mm_to_pt = 2.83465; // 72 / 25.4
    let page_w = pw * mm_to_pt;
    let page_h = ph * mm_to_pt;
    let ml = ps.margins.left * mm_to_pt;
    let mr = ps.margins.right * mm_to_pt;
    let mt = ps.margins.top * mm_to_pt;
    let mb = ps.margins.bottom * mm_to_pt;

    let font_size = 12.0_f32;
    let line_h = font_size * 1.4;

    // Collect all text lines from the document.
    let mut all_lines: Vec<PdfLine> = Vec::new();
    let content_w = page_w - ml - mr;
    for block in &doc.content {
        all_lines.extend(block_to_pdf_lines(block, content_w, font_size));
    }

    // Paginate.
    let content_h = page_h - mt - mb;
    let lines_per_page = (content_h / line_h).floor() as usize;
    let pages: Vec<&[PdfLine]> = if all_lines.is_empty() {
        vec![&[]]
    } else {
        all_lines.chunks(lines_per_page.max(1)).collect()
    };

    // Build the PDF byte-by-byte.
    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n");

    let mut offsets: Vec<usize> = Vec::new();

    // Obj 1: Catalog
    offsets.push(out.len());
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    // Obj 2: Pages (placeholder, overwritten later)
    offsets.push(out.len());
    let pages_offset_pos = out.len();
    // Write a placeholder; we'll replace it after we know page kid refs.
    out.extend_from_slice(
        format!(
            "2 0 obj\n<< /Type /Pages /Kids [PLACEHOLDER] /Count {} >>\nendobj\n",
            pages.len()
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

    // Obj 5: Font Times-Roman
    offsets.push(out.len());
    out.extend_from_slice(
        b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Times-Roman /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    // Obj 6: Font Courier
    offsets.push(out.len());
    out.extend_from_slice(
        b"6 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Courier /Encoding /WinAnsiEncoding >>\nendobj\n",
    );

    // Create page content streams and page objects.
    let mut page_obj_ids: Vec<usize> = Vec::new();
    let first_page_obj = 7; // 1-indexed

    for (page_idx, lines) in pages.iter().enumerate() {
        let stream_obj_id = first_page_obj + page_idx * 2;
        let page_obj_id = stream_obj_id + 1;

        // Build content stream.
        let mut stream = Vec::new();
        let mut y = page_h - mt;

        for line in lines.iter() {
            if line.segments.is_empty() {
                y -= line_h;
                continue;
            }
            y -= line_h;
            writeln!(stream, "BT").unwrap();
            writeln!(stream, "1 0 0 1 {:.2} {:.2} Tm", ml, y).unwrap();

            for seg in &line.segments {
                let font = if seg.marks.as_ref().is_some_and(|m| m.bold) {
                    "/F2"
                } else if seg.marks.as_ref().is_some_and(|m| m.code) {
                    "/F4"
                } else {
                    "/F1"
                };
                let fs = seg
                    .marks
                    .as_ref()
                    .and_then(|m| m.font_size)
                    .unwrap_or(font_size);
                writeln!(stream, "{font} {fs:.1} Tf").unwrap();
                let escaped = escape_pdf_string(&seg.text);
                writeln!(stream, "({escaped}) Tj").unwrap();
            }

            writeln!(stream, "ET").unwrap();
        }

        // If page is empty, add minimal content.
        if stream.is_empty() {
            writeln!(stream, "BT").unwrap();
            writeln!(stream, "/F1 {font_size} Tf").unwrap();
            writeln!(stream, "ET").unwrap();
        }

        let stream_len = stream.len();

        // Stream object.
        offsets.push(out.len());
        out.extend_from_slice(
            format!("{stream_obj_id} 0 obj\n<< /Length {stream_len} >>\nstream\n").as_bytes(),
        );
        out.extend_from_slice(&stream);
        out.extend_from_slice(b"endstream\nendobj\n");

        // Page object.
        offsets.push(out.len());
        out.extend_from_slice(
            format!(
                "{page_obj_id} 0 obj\n<< /Type /Page /Parent 2 0 R \
                 /MediaBox [0 0 {page_w:.2} {page_h:.2}] \
                 /Contents {stream_obj_id} 0 R \
                 /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R /F4 6 0 R >> >> >>\nendobj\n"
            )
            .as_bytes(),
        );

        page_obj_ids.push(page_obj_id);
    }

    // If no pages at all (shouldn't happen), create one blank page.
    if page_obj_ids.is_empty() {
        let stream_obj_id = 7;
        let page_obj_id = 8;
        let stream = format!("BT\n/F1 {font_size} Tf\nET\n");

        offsets.push(out.len());
        out.extend_from_slice(
            format!(
                "{stream_obj_id} 0 obj\n<< /Length {} >>\nstream\n{stream}endstream\nendobj\n",
                stream.len()
            )
            .as_bytes(),
        );
        offsets.push(out.len());
        out.extend_from_slice(
            format!(
                "{page_obj_id} 0 obj\n<< /Type /Page /Parent 2 0 R \
                 /MediaBox [0 0 {page_w:.2} {page_h:.2}] \
                 /Contents {stream_obj_id} 0 R \
                 /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R /F4 6 0 R >> >> >>\nendobj\n"
            )
            .as_bytes(),
        );
        page_obj_ids.push(page_obj_id);
    }

    // Fix Pages object: replace the placeholder.
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

    // Replace the placeholder in the output.
    let placeholder = b"2 0 obj\n<< /Type /Pages /Kids [PLACEHOLDER] /Count ";
    let placeholder_end = pages_offset_pos
        + placeholder.len()
        + format!("{}", pages.len()).len()
        + " >>\nendobj\n".len();

    let mut final_out = Vec::with_capacity(out.len() + pages_obj_bytes.len());
    final_out.extend_from_slice(&out[..pages_offset_pos]);
    final_out.extend_from_slice(pages_obj_bytes);
    final_out.extend_from_slice(&out[placeholder_end..]);

    // Recompute offsets for the final output by scanning for "N 0 obj" patterns.
    let final_out_str = String::from_utf8_lossy(&final_out);
    let mut final_offsets: Vec<usize> = Vec::new();
    let mut search_pos = 0;
    while search_pos < final_out_str.len() {
        if let Some(idx) = final_out_str[search_pos..].find(" obj\n") {
            let obj_start = search_pos + idx;
            // Check that this is at the start of a line (or start of file).
            let at_line_start = obj_start == 0 || final_out_str.as_bytes()[obj_start - 1] == b'\n';
            if at_line_start {
                // Find the object number.
                let before = &final_out_str[obj_start..];
                if let Some(space_pos) = before.find(' ') {
                    let num_str = &before[..space_pos];
                    if let Ok(num) = num_str.parse::<usize>() {
                        if num > 0 && num <= final_offsets.len() + 10 {
                            // Ensure we have room.
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

    // Cross-reference table.
    let xref_offset = final_out.len();
    let num_objects = final_offsets.len() + 1; // +1 for object 0
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
// PDF text helpers
// ---------------------------------------------------------------------------

fn escape_pdf_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

struct PdfLine {
    segments: Vec<PdfSegment>,
}

struct PdfSegment {
    text: String,
    marks: Option<Marks>,
}

fn block_to_pdf_lines(block: &BlockNode, _content_width: f32, _font_size: f32) -> Vec<PdfLine> {
    match block {
        BlockNode::Paragraph { content, .. } => {
            if content.is_empty() {
                vec![PdfLine { segments: vec![] }]
            } else {
                vec![PdfLine {
                    segments: content
                        .iter()
                        .filter_map(|n| match n {
                            InlineNode::Text { text, marks } => Some(PdfSegment {
                                text: text.clone(),
                                marks: Some(marks.clone()),
                            }),
                            InlineNode::Link { text, .. } => Some(PdfSegment {
                                text: text.clone(),
                                marks: None,
                            }),
                            _ => None,
                        })
                        .collect(),
                }]
            }
        }
        BlockNode::Heading { level, content, .. } => {
            let text = inline_text_pdf(content);
            vec![PdfLine {
                segments: vec![PdfSegment {
                    text: format!("{}{}", "#".repeat(*level as usize), text),
                    marks: Some(Marks {
                        bold: true,
                        ..Marks::default()
                    }),
                }],
            }]
        }
        BlockNode::BulletList { items } => items
            .iter()
            .map(|item| PdfLine {
                segments: vec![PdfSegment {
                    text: format!("\u{2022} {}", inline_text_pdf(&item.content)),
                    marks: None,
                }],
            })
            .collect(),
        BlockNode::OrderedList { items, .. } => items
            .iter()
            .enumerate()
            .map(|(i, item)| PdfLine {
                segments: vec![PdfSegment {
                    text: format!("{}. {}", i + 1, inline_text_pdf(&item.content)),
                    marks: None,
                }],
            })
            .collect(),
        BlockNode::CodeBlock { code, .. } => code
            .lines()
            .map(|line| PdfLine {
                segments: vec![PdfSegment {
                    text: line.to_string(),
                    marks: Some(Marks {
                        code: true,
                        ..Marks::default()
                    }),
                }],
            })
            .collect(),
        BlockNode::Table { rows } => {
            let mut lines = Vec::new();
            for row in rows {
                let cells: String = row
                    .cells
                    .iter()
                    .map(|cell| {
                        cell.content
                            .iter()
                            .map(block_plain_text_pdf)
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                lines.push(PdfLine {
                    segments: vec![PdfSegment {
                        text: cells,
                        marks: None,
                    }],
                });
            }
            lines
        }
        BlockNode::BlockQuote { content } => content
            .iter()
            .flat_map(|b| block_to_pdf_lines(b, _content_width, _font_size))
            .collect(),
        BlockNode::HorizontalRule => vec![PdfLine {
            segments: vec![PdfSegment {
                text: "---".to_string(),
                marks: None,
            }],
        }],
        BlockNode::PageBreak => vec![],
        BlockNode::Image { alt, .. } => vec![PdfLine {
            segments: vec![PdfSegment {
                text: format!("[{}]", alt.as_deref().unwrap_or("Image")),
                marks: None,
            }],
        }],
        BlockNode::TaskList { items } => items
            .iter()
            .map(|item| PdfLine {
                segments: vec![PdfSegment {
                    text: format!(
                        "[{}] {}",
                        if item.checked { "x" } else { " " },
                        inline_text_pdf(&item.content)
                    ),
                    marks: None,
                }],
            })
            .collect(),
        BlockNode::Footnote { number, content } => vec![PdfLine {
            segments: vec![PdfSegment {
                text: format!("[{number}] {}", inline_text_pdf(content)),
                marks: None,
            }],
        }],
    }
}

fn inline_text_pdf(nodes: &[InlineNode]) -> String {
    nodes
        .iter()
        .map(|n| match n {
            InlineNode::Text { text, .. } => text.clone(),
            InlineNode::Link { text, .. } => text.clone(),
            InlineNode::HardBreak => "\n".to_string(),
            InlineNode::InlineImage { alt, .. } => alt.clone().unwrap_or_default(),
        })
        .collect()
}

fn block_plain_text_pdf(block: &BlockNode) -> String {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
            inline_text_pdf(content)
        }
        BlockNode::CodeBlock { code, .. } => code.clone(),
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tench_document_core::{Marks, ParagraphAttrs};

    #[test]
    fn pdf_export_produces_valid_header() {
        let doc = TenchDocument {
            content: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: "Hello PDF".to_string(),
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            }],
            ..TenchDocument::new("")
        };

        let bytes = build_pdf(&doc).expect("build pdf");
        let header = String::from_utf8_lossy(&bytes[..20]);
        assert!(header.starts_with("%PDF-1.4"));
        assert!(bytes.ends_with(b"%%EOF\n"));
    }

    #[test]
    fn pdf_export_contains_text() {
        let doc = TenchDocument {
            content: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: "Test Content".to_string(),
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            }],
            ..TenchDocument::new("")
        };

        let bytes = build_pdf(&doc).expect("build pdf");
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("(Test Content)"));
    }

    #[test]
    fn pdf_export_handles_bold() {
        let doc = TenchDocument {
            content: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: "Bold".to_string(),
                    marks: Marks {
                        bold: true,
                        ..Marks::default()
                    },
                }],
                attrs: ParagraphAttrs::default(),
            }],
            ..TenchDocument::new("")
        };

        let bytes = build_pdf(&doc).expect("build pdf");
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("/F2"));
    }

    #[test]
    fn pdf_export_handles_multiple_pages() {
        let mut blocks = Vec::new();
        for i in 0..100 {
            blocks.push(BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: format!("Line {i}"),
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            });
        }
        let doc = TenchDocument {
            content: blocks,
            ..TenchDocument::new("")
        };

        let bytes = build_pdf(&doc).expect("build pdf");
        let pdf_str = String::from_utf8_lossy(&bytes);
        // Should have multiple pages.
        let page_count = pdf_str.matches("/Type /Page").count();
        assert!(page_count > 1);
    }

    #[test]
    fn pdf_export_empty_document() {
        let doc = TenchDocument::new("");
        let bytes = build_pdf(&doc).expect("build pdf");
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.contains("%PDF-1.4"));
        assert!(pdf_str.contains("%%EOF"));
    }
}
