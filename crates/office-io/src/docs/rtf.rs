//! RTF import/export using the Tench Document Model.
//!
//! Supports basic RTF parsing and generation with font tables, color tables,
//! bold/italic/underline/strike, font size, color, alignment, indentation,
//! and tables.

use std::fs;
use std::path::Path;

use tench_document_core::{
    Alignment, BlockNode, InlineNode, Marks, OfficeContent, ParagraphAttrs, RichDocumentContent,
    TableCell, TableRow, TdmMetadata, TenchDocument,
};

const DOC_SCHEMA: &str = "tench.docs.v1";

// ---------------------------------------------------------------------------
// TDM-based API
// ---------------------------------------------------------------------------

/// Import an RTF file and return a `TenchDocument`.
pub fn import_rtf(path: &Path) -> Result<TenchDocument, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read RTF {}: {e}", path.display()))?;
    parse_rtf_to_tdm(&raw)
}

/// Export a `TenchDocument` as RTF bytes.
pub fn export_rtf_bytes(doc: &TenchDocument) -> Result<Vec<u8>, String> {
    Ok(build_rtf(doc).into_bytes())
}

// ---------------------------------------------------------------------------
// Legacy OfficeContent wrappers
// ---------------------------------------------------------------------------

/// Import RTF into `OfficeContent::Docs` (backward-compatible).
pub fn import_rtf_as_content(path: &Path) -> Result<OfficeContent, String> {
    let doc = import_rtf(path)?;
    Ok(tdm_to_docs_content(&doc))
}

/// Export `OfficeContent::Docs` as RTF bytes (backward-compatible).
pub fn export_rtf_bytes_from_content(content: &OfficeContent) -> Result<Vec<u8>, String> {
    let doc = content_to_tdm(content);
    export_rtf_bytes(&doc)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tdm_to_docs_content(doc: &TenchDocument) -> OfficeContent {
    OfficeContent::Docs(RichDocumentContent {
        schema: DOC_SCHEMA.to_string(),
        document: Some(doc.clone()),
    })
}

fn content_to_tdm(content: &OfficeContent) -> TenchDocument {
    match content {
        OfficeContent::Docs(rich) => rich
            .document
            .clone()
            .unwrap_or_else(|| TenchDocument::new("")),
        _ => TenchDocument::new(""),
    }
}

// ---------------------------------------------------------------------------
// RTF Import
// ---------------------------------------------------------------------------

fn parse_rtf_to_tdm(raw: &str) -> Result<TenchDocument, String> {
    if !raw.starts_with("{\\rtf") {
        return Err("Not a valid RTF file: missing header".to_string());
    }

    // Parse color table.
    let colors = parse_color_table(raw);

    // Parse font table (simplified).
    let _fonts = parse_font_table(raw);

    // Strip outer braces and header.
    let body = extract_rtf_body(raw);

    let mut blocks = Vec::new();
    let mut current_marks = Marks::default();
    let mut current_attrs = ParagraphAttrs::default();
    let mut text_buf = String::new();

    let mut i = 0;
    let bytes = body.as_bytes();
    let len = bytes.len();

    while i < len {
        let ch = bytes[i] as char;

        if ch == '\\' && i + 1 < len {
            let (keyword, value, advance) = parse_rtf_keyword(&body, i + 1);
            match keyword.as_str() {
                "par" | "pard" => {
                    flush_rtf_paragraph(
                        &mut text_buf,
                        &mut current_marks,
                        &mut current_attrs,
                        &mut blocks,
                    );
                    if keyword == "pard" {
                        current_attrs = ParagraphAttrs::default();
                        current_marks = Marks::default();
                    }
                }
                "b" => {
                    flush_rtf_text(
                        &mut text_buf,
                        &mut current_marks,
                        &mut blocks,
                        &current_attrs,
                    );
                    current_marks.bold = value.is_none() || value.as_deref() == Some("1");
                }
                "b0" => {
                    flush_rtf_text(
                        &mut text_buf,
                        &mut current_marks,
                        &mut blocks,
                        &current_attrs,
                    );
                    current_marks.bold = false;
                }
                "i" => {
                    flush_rtf_text(
                        &mut text_buf,
                        &mut current_marks,
                        &mut blocks,
                        &current_attrs,
                    );
                    current_marks.italic = value.is_none() || value.as_deref() == Some("1");
                }
                "i0" => {
                    flush_rtf_text(
                        &mut text_buf,
                        &mut current_marks,
                        &mut blocks,
                        &current_attrs,
                    );
                    current_marks.italic = false;
                }
                "ul" => {
                    flush_rtf_text(
                        &mut text_buf,
                        &mut current_marks,
                        &mut blocks,
                        &current_attrs,
                    );
                    current_marks.underline = value.is_none() || value.as_deref() == Some("1");
                }
                "ulnone" | "ul0" => {
                    flush_rtf_text(
                        &mut text_buf,
                        &mut current_marks,
                        &mut blocks,
                        &current_attrs,
                    );
                    current_marks.underline = false;
                }
                "strike" => {
                    flush_rtf_text(
                        &mut text_buf,
                        &mut current_marks,
                        &mut blocks,
                        &current_attrs,
                    );
                    current_marks.strikethrough = value.is_none() || value.as_deref() == Some("1");
                }
                "strike0" => {
                    flush_rtf_text(
                        &mut text_buf,
                        &mut current_marks,
                        &mut blocks,
                        &current_attrs,
                    );
                    current_marks.strikethrough = false;
                }
                "fs" => {
                    flush_rtf_text(
                        &mut text_buf,
                        &mut current_marks,
                        &mut blocks,
                        &current_attrs,
                    );
                    if let Some(ref v) = value {
                        if let Ok(half_pts) = v.parse::<f32>() {
                            current_marks.font_size = Some(half_pts / 2.0);
                        }
                    }
                }
                "cf" => {
                    flush_rtf_text(
                        &mut text_buf,
                        &mut current_marks,
                        &mut blocks,
                        &current_attrs,
                    );
                    if let Some(ref idx) = value {
                        if let Ok(ci) = idx.parse::<usize>() {
                            if ci > 0 && ci <= colors.len() {
                                current_marks.text_color = Some(colors[ci - 1].clone());
                            }
                        }
                    }
                }
                "ql" => current_attrs.alignment = Alignment::Left,
                "qc" => current_attrs.alignment = Alignment::Center,
                "qr" => current_attrs.alignment = Alignment::Right,
                "qj" => current_attrs.alignment = Alignment::Justify,
                "li" => {
                    if let Some(ref v) = value {
                        if let Ok(tw) = v.parse::<f32>() {
                            current_attrs.indent_left = tw / 20.0; // twips to half-points, approx mm
                        }
                    }
                }
                "ri" => {
                    if let Some(ref v) = value {
                        if let Ok(tw) = v.parse::<f32>() {
                            current_attrs.indent_right = tw / 20.0;
                        }
                    }
                }
                "fi" => {
                    if let Some(ref v) = value {
                        if let Ok(tw) = v.parse::<f32>() {
                            current_attrs.indent_first_line = tw / 20.0;
                        }
                    }
                }
                "trowd" => {
                    // Table row start - simplified: parse cells.
                    flush_rtf_paragraph(
                        &mut text_buf,
                        &mut current_marks,
                        &mut current_attrs,
                        &mut blocks,
                    );
                    let (table, advance_to) = parse_rtf_table_row(&body, i);
                    if let Some(table_block) = table {
                        blocks.push(table_block);
                    }
                    i = advance_to;
                    continue;
                }
                "cell" => {
                    // Cell separator in table - handled in table parsing.
                }
                "row" => {
                    // Row end - handled in table parsing.
                }
                _ => {
                    // Unknown keyword, skip.
                }
            }
            i += advance;
            continue;
        }

        if ch == '{' || ch == '}' {
            i += 1;
            continue;
        }

        if ch == '\n' || ch == '\r' {
            i += 1;
            continue;
        }

        text_buf.push(ch);
        i += 1;
    }

    // Flush remaining text.
    flush_rtf_paragraph(
        &mut text_buf,
        &mut current_marks,
        &mut current_attrs,
        &mut blocks,
    );

    if blocks.is_empty() {
        blocks.push(BlockNode::Paragraph {
            content: Vec::new(),
            attrs: ParagraphAttrs::default(),
        });
    }

    Ok(TenchDocument {
        content: blocks,
        metadata: TdmMetadata::default(),
        ..TenchDocument::new("")
    })
}

fn parse_rtf_keyword(body: &str, start: usize) -> (String, Option<String>, usize) {
    let bytes = body.as_bytes();
    let len = bytes.len();
    let mut i = start;

    // Read keyword characters (a-z, *).
    let mut keyword = String::new();
    while i < len {
        let ch = bytes[i] as char;
        if ch.is_ascii_alphabetic() || ch == '*' {
            keyword.push(ch);
            i += 1;
        } else {
            break;
        }
    }

    // Read optional numeric value.
    let mut value = String::new();
    if i < len && (bytes[i] == b'-' || (bytes[i] as char).is_ascii_digit()) {
        value.push(bytes[i] as char);
        i += 1;
        while i < len && (bytes[i] as char).is_ascii_digit() {
            value.push(bytes[i] as char);
            i += 1;
        }
    }

    // Consume trailing space.
    if i < len && bytes[i] == b' ' {
        i += 1;
    }

    let val_opt = if value.is_empty() { None } else { Some(value) };

    (keyword, val_opt, i - start)
}

fn parse_color_table(raw: &str) -> Vec<String> {
    let mut colors = Vec::new();
    if let Some(start) = raw.find("{\\colortbl") {
        let rest = &raw[start + "{\\colortbl".len()..];
        if let Some(end) = rest.find('}') {
            let table = &rest[..end];
            for entry in table.split(';') {
                let entry = entry.trim();
                if entry.is_empty() {
                    continue;
                }
                // Parse \redN \greenN \blueN.
                let mut r: u8 = 0;
                let mut g: u8 = 0;
                let mut b: u8 = 0;
                for part in entry.split('\\') {
                    let part = part.trim();
                    if let Some(rest) = part.strip_prefix("red") {
                        r = rest.parse().unwrap_or(0);
                    } else if let Some(rest) = part.strip_prefix("green") {
                        g = rest.parse().unwrap_or(0);
                    } else if let Some(rest) = part.strip_prefix("blue") {
                        b = rest.parse().unwrap_or(0);
                    }
                }
                colors.push(format!("#{:02x}{:02x}{:02x}", r, g, b));
            }
        }
    }
    colors
}

fn parse_font_table(_raw: &str) -> Vec<String> {
    // Simplified: return empty.
    Vec::new()
}

fn extract_rtf_body(raw: &str) -> String {
    // Strip the outer {\rtf1... and closing }.
    let start = raw.find("\\ansi").map(|i| i + 5).unwrap_or(0);
    let end = raw.rfind('}').unwrap_or(raw.len());
    raw[start..end].to_string()
}

fn flush_rtf_text(
    text_buf: &mut String,
    marks: &mut Marks,
    blocks: &mut [BlockNode],
    _attrs: &ParagraphAttrs,
) {
    if !text_buf.is_empty() {
        // Add text to current paragraph's inline content.
        // Find or create current paragraph.
        let text = std::mem::take(text_buf);
        if let Some(BlockNode::Paragraph { content, .. }) = blocks.last_mut() {
            content.push(InlineNode::Text {
                text,
                marks: marks.clone(),
            });
        }
    }
}

fn flush_rtf_paragraph(
    text_buf: &mut String,
    marks: &mut Marks,
    attrs: &mut ParagraphAttrs,
    blocks: &mut Vec<BlockNode>,
) {
    flush_rtf_text(text_buf, marks, blocks, attrs);
    if let Some(BlockNode::Paragraph { content, .. }) = blocks.last() {
        if content.is_empty() {
            // Don't add empty paragraphs.
            return;
        }
    }
    blocks.push(BlockNode::Paragraph {
        content: Vec::new(),
        attrs: attrs.clone(),
    });
}

fn parse_rtf_table_row(body: &str, start: usize) -> (Option<BlockNode>, usize) {
    let mut cells: Vec<TableCell> = Vec::new();
    let mut cell_text = String::new();
    let mut cell_blocks = Vec::new();
    let mut i = start;

    let bytes = body.as_bytes();
    let len = bytes.len();

    while i < len {
        let ch = bytes[i] as char;

        if ch == '\\' && i + 1 < len {
            let (keyword, _value, advance) = parse_rtf_keyword(body, i + 1);
            match keyword.as_str() {
                "cell" => {
                    if !cell_text.is_empty() {
                        cell_blocks.push(BlockNode::Paragraph {
                            content: vec![InlineNode::Text {
                                text: std::mem::take(&mut cell_text),
                                marks: Marks::default(),
                            }],
                            attrs: ParagraphAttrs::default(),
                        });
                    }
                    cells.push(TableCell {
                        content: std::mem::take(&mut cell_blocks),
                        ..TableCell::default()
                    });
                }
                "row" => {
                    i += advance;
                    break;
                }
                _ => {}
            }
            i += advance;
            continue;
        }

        if ch == '{' || ch == '}' {
            i += 1;
            continue;
        }

        if ch != '\n' && ch != '\r' {
            cell_text.push(ch);
        }
        i += 1;
    }

    if !cells.is_empty() {
        (
            Some(BlockNode::Table {
                rows: vec![TableRow { cells }],
            }),
            i,
        )
    } else {
        (None, i)
    }
}

// ---------------------------------------------------------------------------
// RTF Export
// ---------------------------------------------------------------------------

fn build_rtf(doc: &TenchDocument) -> String {
    let mut out = String::new();

    // Header.
    out.push_str("{\\rtf1\\ansi\\deff0\n");

    // Font table.
    out.push_str("{\\fonttbl{\\f0 Helvetica;}{\\f1 Courier New;}}\n");

    // Color table.
    out.push_str(
        "{\\colortbl;\\red0\\green0\\blue0;\\red255\\green0\\blue0;\\red0\\green0\\blue255;}\n",
    );

    // Document content.
    for block in &doc.content {
        block_to_rtf(block, &mut out);
    }

    out.push_str("}\n");
    out
}

fn block_to_rtf(block: &BlockNode, out: &mut String) {
    match block {
        BlockNode::Paragraph { content, attrs } => {
            write_rtf_alignment(attrs, out);
            write_rtf_indent(attrs, out);
            for inline in content {
                inline_to_rtf(inline, out);
            }
            out.push_str("\\par\n");
        }
        BlockNode::Heading {
            level,
            content,
            attrs,
        } => {
            write_rtf_alignment(attrs, out);
            let size = match level {
                1 => 44,
                2 => 36,
                3 => 32,
                4 => 28,
                5 => 24,
                _ => 22,
            };
            out.push_str(&format!("{{\\fs{size}\\b "));
            for inline in content {
                inline_to_rtf(inline, out);
            }
            out.push_str("}\\par\n");
        }
        BlockNode::BulletList { items } => {
            for item in items {
                out.push_str("\\pard{\\pntext\\bullet\\tab}");
                for inline in &item.content {
                    inline_to_rtf(inline, out);
                }
                out.push_str("\\par\n");
            }
        }
        BlockNode::OrderedList { items, .. } => {
            for (i, item) in items.iter().enumerate() {
                out.push_str(&format!("\\pard{{\\pntext {}\\tab}}", i + 1));
                for inline in &item.content {
                    inline_to_rtf(inline, out);
                }
                out.push_str("\\par\n");
            }
        }
        BlockNode::BlockQuote { content } => {
            out.push_str("\\pard\\li720\\ri720\\i ");
            for child in content {
                block_to_rtf(child, out);
            }
            out.push_str("\\par\n");
        }
        BlockNode::CodeBlock { code, .. } => {
            out.push_str("\\pard\\f1 ");
            out.push_str(&rtf_escape(code));
            out.push_str("\\par\n");
        }
        BlockNode::Table { rows } => {
            for row in rows {
                out.push_str("\\trowd\\trautofit1\\intbl\n");
                for cell in &row.cells {
                    for b in &cell.content {
                        block_to_rtf(b, out);
                    }
                    out.push_str("\\cell\n");
                }
                out.push_str("\\row\n");
            }
        }
        BlockNode::HorizontalRule => {
            out.push_str("\\pard\\brdrb\\brdrs\\brdrw10\\brsp20 {\\fs2 }\\par\n");
        }
        BlockNode::PageBreak => {
            out.push_str("\\page\n");
        }
        BlockNode::Image { alt, .. } => {
            let text = alt.as_deref().unwrap_or("[Image]");
            out.push_str(&format!("{}\\par\n", rtf_escape(text)));
        }
        BlockNode::TaskList { items } => {
            for item in items {
                let check = if item.checked { "[x] " } else { "[ ] " };
                out.push_str(&rtf_escape(check));
                for inline in &item.content {
                    inline_to_rtf(inline, out);
                }
                out.push_str("\\par\n");
            }
        }
        BlockNode::Footnote { number, content } => {
            out.push_str(&format!("\\pard\\super [{}]\\super0 ", number));
            for inline in content {
                inline_to_rtf(inline, out);
            }
            out.push_str("\\par\n");
        }
    }
}

fn inline_to_rtf(node: &InlineNode, out: &mut String) {
    match node {
        InlineNode::Text { text, marks } => {
            let mut prefix = String::new();
            let mut suffix = String::new();
            if marks.bold {
                prefix.push_str("\\b ");
                suffix.push_str("\\b0 ");
            }
            if marks.italic {
                prefix.push_str("\\i ");
                suffix.push_str("\\i0 ");
            }
            if marks.underline {
                prefix.push_str("\\ul ");
                suffix.push_str("\\ulnone ");
            }
            if marks.strikethrough {
                prefix.push_str("\\strike ");
                suffix.push_str("\\strike0 ");
            }
            if let Some(fs) = marks.font_size {
                let half_pts = (fs * 2.0) as i32;
                prefix.push_str(&format!("\\fs{half_pts} "));
                suffix.push_str("\\fs24 ");
            }
            if let Some(ref color) = marks.text_color {
                let idx = rtf_color_index(color);
                prefix.push_str(&format!("\\cf{idx} "));
                suffix.push_str("\\cf0 ");
            }
            out.push_str(&prefix);
            out.push_str(&rtf_escape(text));
            out.push_str(&suffix);
        }
        InlineNode::HardBreak => {
            out.push_str("\\line\n");
        }
        InlineNode::Link { text, .. } => {
            out.push_str(&rtf_escape(text));
        }
        InlineNode::InlineImage { alt, .. } => {
            out.push_str(&rtf_escape(alt.as_deref().unwrap_or("[Image]")));
        }
    }
}

fn write_rtf_alignment(attrs: &ParagraphAttrs, out: &mut String) {
    match attrs.alignment {
        Alignment::Left => out.push_str("\\pard\\ql "),
        Alignment::Center => out.push_str("\\pard\\qc "),
        Alignment::Right => out.push_str("\\pard\\qr "),
        Alignment::Justify => out.push_str("\\pard\\qj "),
    }
}

fn write_rtf_indent(attrs: &ParagraphAttrs, out: &mut String) {
    if attrs.indent_left != 0.0 {
        let twips = (attrs.indent_left * 20.0) as i32;
        out.push_str(&format!("\\li{twips} "));
    }
    if attrs.indent_right != 0.0 {
        let twips = (attrs.indent_right * 20.0) as i32;
        out.push_str(&format!("\\ri{twips} "));
    }
    if attrs.indent_first_line != 0.0 {
        let twips = (attrs.indent_first_line * 20.0) as i32;
        out.push_str(&format!("\\fi{twips} "));
    }
}

fn rtf_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('\n', "\\line\n")
}

fn rtf_color_index(_color: &str) -> u32 {
    // Simplified: return 1 (black) for now.
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtf_export_produces_valid_header() {
        let doc = TenchDocument {
            content: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: "Hello RTF".to_string(),
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            }],
            ..TenchDocument::new("")
        };

        let rtf = build_rtf(&doc);
        assert!(rtf.starts_with("{\\rtf1\\ansi"));
        assert!(rtf.contains("Hello RTF"));
        assert!(rtf.ends_with("}\n"));
    }

    #[test]
    fn rtf_export_preserves_bold_italic() {
        let doc = TenchDocument {
            content: vec![BlockNode::Paragraph {
                content: vec![
                    InlineNode::Text {
                        text: "Bold".to_string(),
                        marks: Marks {
                            bold: true,
                            ..Marks::default()
                        },
                    },
                    InlineNode::Text {
                        text: "Italic".to_string(),
                        marks: Marks {
                            italic: true,
                            ..Marks::default()
                        },
                    },
                ],
                attrs: ParagraphAttrs::default(),
            }],
            ..TenchDocument::new("")
        };

        let rtf = build_rtf(&doc);
        assert!(rtf.contains("\\b "));
        assert!(rtf.contains("\\i "));
    }

    #[test]
    fn rtf_export_preserves_table() {
        let doc = TenchDocument {
            content: vec![BlockNode::Table {
                rows: vec![TableRow {
                    cells: vec![
                        TableCell {
                            content: vec![BlockNode::Paragraph {
                                content: vec![InlineNode::Text {
                                    text: "A".to_string(),
                                    marks: Marks::default(),
                                }],
                                attrs: ParagraphAttrs::default(),
                            }],
                            ..TableCell::default()
                        },
                        TableCell {
                            content: vec![BlockNode::Paragraph {
                                content: vec![InlineNode::Text {
                                    text: "B".to_string(),
                                    marks: Marks::default(),
                                }],
                                attrs: ParagraphAttrs::default(),
                            }],
                            ..TableCell::default()
                        },
                    ],
                }],
            }],
            ..TenchDocument::new("")
        };

        let rtf = build_rtf(&doc);
        assert!(rtf.contains("\\trowd"));
        assert!(rtf.contains("\\cell"));
        assert!(rtf.contains("\\row"));
    }

    #[test]
    fn rtf_import_parses_basic_document() {
        let rtf = r#"{\rtf1\ansi{\fonttbl{\f0 Helvetica;}}{\colortbl;\red0\green0\blue0;}\pard\ql Hello World\par }"#;
        let doc = parse_rtf_to_tdm(rtf).expect("parse rtf");
        let text = doc.to_plain_text();
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn rtf_import_parses_bold_text() {
        let rtf = r#"{\rtf1\ansi\pard\ql \b Bold Text\b0 \par }"#;
        let doc = parse_rtf_to_tdm(rtf).expect("parse rtf");
        let text = doc.to_plain_text();
        assert!(text.contains("Bold Text"));
    }

    #[test]
    fn rtf_roundtrip_preserves_text() {
        let doc = TenchDocument {
            content: vec![
                BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text: "Round trip".to_string(),
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                },
                BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text: "Second paragraph".to_string(),
                        marks: Marks {
                            bold: true,
                            ..Marks::default()
                        },
                    }],
                    attrs: ParagraphAttrs::default(),
                },
            ],
            ..TenchDocument::new("")
        };

        let rtf = build_rtf(&doc);
        let reimported = parse_rtf_to_tdm(&rtf).expect("reimport rtf");
        let text = reimported.to_plain_text();
        assert!(text.contains("Round trip"));
        assert!(text.contains("Second paragraph"));
    }
}
