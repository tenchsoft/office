//! EPUB export using the Tench Document Model.
//!
//! Produces a valid EPUB 3.0 archive with:
//! - mimetype (stored, uncompressed)
//! - META-INF/container.xml
//! - OEBPS/content.opf
//! - OEBPS/toc.ncx
//! - OEBPS/chapter1.html
//!
//! Images are embedded in OEBPS/media/.

use std::io::{Cursor, Write};

use crate::zip_util;
use tench_document_core::{BlockNode, ImageSource, InlineNode, OfficeContent, TenchDocument};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

// ---------------------------------------------------------------------------
// TDM-based API
// ---------------------------------------------------------------------------

/// Export a `TenchDocument` as EPUB bytes.
pub fn export_epub_bytes(doc: &TenchDocument) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let stored = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o644);
    let deflated = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // mimetype must be first and stored uncompressed.
    zip_util::write_zip_file(&mut writer, "mimetype", "application/epub+zip", stored)?;

    // META-INF/container.xml
    zip_util::write_zip_file(
        &mut writer,
        "META-INF/container.xml",
        CONTAINER_XML,
        deflated,
    )?;

    // Collect images.
    let mut media_entries: Vec<(String, Vec<u8>)> = Vec::new();
    collect_epub_images(doc, &mut media_entries);

    // Generate chapter HTML.
    let chapter_html = build_chapter_html(doc);

    // Generate TOC from headings.
    let toc_ncx = build_toc_ncx(doc);
    let content_opf = build_content_opf(doc, &media_entries);

    zip_util::write_zip_file(&mut writer, "OEBPS/content.opf", &content_opf, deflated)?;
    zip_util::write_zip_file(&mut writer, "OEBPS/toc.ncx", &toc_ncx, deflated)?;
    zip_util::write_zip_file(&mut writer, "OEBPS/chapter1.html", &chapter_html, deflated)?;

    // Write media files.
    for (name, data) in &media_entries {
        writer
            .start_file(name, deflated)
            .map_err(|e| format!("Failed to add {name}: {e}"))?;
        writer
            .write_all(data)
            .map_err(|e| format!("Failed to write {name}: {e}"))?;
    }

    writer
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|error| format!("Failed to finish EPUB archive: {error}"))
}

// ---------------------------------------------------------------------------
// Legacy OfficeContent wrapper
// ---------------------------------------------------------------------------

/// Export `OfficeContent::Docs` as EPUB bytes (backward-compatible).
pub fn export_epub_bytes_from_content(content: &OfficeContent) -> Result<Vec<u8>, String> {
    let doc = match content {
        OfficeContent::Docs(rich) => rich
            .document
            .clone()
            .unwrap_or_else(|| TenchDocument::new("")),
        _ => TenchDocument::new(""),
    };
    export_epub_bytes(&doc)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn collect_epub_images(doc: &TenchDocument, media_entries: &mut Vec<(String, Vec<u8>)>) {
    for block in &doc.content {
        collect_images_from_block(block, media_entries);
    }
}

fn collect_images_from_block(block: &BlockNode, media_entries: &mut Vec<(String, Vec<u8>)>) {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
            for inline in content {
                if let InlineNode::InlineImage {
                    source: ImageSource::Embedded { data },
                    ..
                } = inline
                {
                    let name = format!("OEBPS/media/image{}.png", media_entries.len() + 1);
                    media_entries.push((name, data.clone()));
                }
            }
        }
        BlockNode::Image {
            source: ImageSource::Embedded { data },
            ..
        } => {
            let name = format!("OEBPS/media/image{}.png", media_entries.len() + 1);
            media_entries.push((name, data.clone()));
        }
        BlockNode::Table { rows } => {
            for row in rows {
                for cell in &row.cells {
                    for b in &cell.content {
                        collect_images_from_block(b, media_entries);
                    }
                }
            }
        }
        BlockNode::BlockQuote { content } => {
            for child in content {
                collect_images_from_block(child, media_entries);
            }
        }
        _ => {}
    }
}

fn build_chapter_html(doc: &TenchDocument) -> String {
    let body: String = doc
        .content
        .iter()
        .map(block_to_epub_html)
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><meta charset="utf-8"/><title>{}</title></head>
<body>
{body}
</body>
</html>"#,
        crate::xml_util::escape_xml(&doc.metadata.title)
    )
}

fn block_to_epub_html(block: &BlockNode) -> String {
    match block {
        BlockNode::Paragraph { content, attrs } => {
            let style = paragraph_style_html(attrs);
            let inner = inline_to_epub_html(content);
            format!("<p{style}>{inner}</p>")
        }
        BlockNode::Heading { level, content, .. } => {
            let l = (*level as usize).clamp(1, 6);
            let inner = inline_to_epub_html(content);
            format!("<h{l}>{inner}</h{l}>")
        }
        BlockNode::BulletList { items } => {
            let items_html: String = items
                .iter()
                .map(|i| format!("<li>{}</li>", inline_to_epub_html(&i.content)))
                .collect();
            format!("<ul>{items_html}</ul>")
        }
        BlockNode::OrderedList { items, .. } => {
            let items_html: String = items
                .iter()
                .map(|i| format!("<li>{}</li>", inline_to_epub_html(&i.content)))
                .collect();
            format!("<ol>{items_html}</ol>")
        }
        BlockNode::BlockQuote { content } => {
            let inner: String = content
                .iter()
                .map(block_to_epub_html)
                .collect::<Vec<_>>()
                .join("\n");
            format!("<blockquote>{inner}</blockquote>")
        }
        BlockNode::CodeBlock { code, .. } => {
            format!(
                "<pre><code>{}</code></pre>",
                crate::xml_util::escape_xml(code)
            )
        }
        BlockNode::Table { rows } => {
            let mut html = String::from("<table>");
            for row in rows {
                html.push_str("<tr>");
                for cell in &row.cells {
                    html.push_str("<td>");
                    for block in &cell.content {
                        html.push_str(&block_to_epub_html(block));
                    }
                    html.push_str("</td>");
                }
                html.push_str("</tr>");
            }
            html.push_str("</table>");
            html
        }
        BlockNode::HorizontalRule => "<hr/>".to_string(),
        BlockNode::PageBreak => "<hr/>".to_string(),
        BlockNode::Image { source, alt, .. } => {
            let src = match source {
                ImageSource::Embedded { .. } => {
                    format!("media/image{}.png", 1) // simplified
                }
                ImageSource::Referenced { path } => path.clone(),
            };
            let a = alt.as_deref().unwrap_or("");
            format!(
                "<img src=\"{}\" alt=\"{}\"/>",
                crate::xml_util::escape_html(&src),
                crate::xml_util::escape_html(a)
            )
        }
        BlockNode::TaskList { items } => {
            let items_html: String = items
                .iter()
                .map(|i| {
                    let checked = if i.checked { " checked" } else { "" };
                    format!(
                        "<li><input type=\"checkbox\"{checked}/>{}</li>",
                        inline_to_epub_html(&i.content)
                    )
                })
                .collect();
            format!("<ul>{items_html}</ul>")
        }
        BlockNode::Footnote { number, content } => {
            let inner = inline_to_epub_html(content);
            format!("<p>[{number}] {inner}</p>")
        }
    }
}

fn inline_to_epub_html(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            InlineNode::Text { text, marks } => {
                let mut t = crate::xml_util::escape_html(text);
                if marks.bold {
                    t = format!("<strong>{t}</strong>");
                }
                if marks.italic {
                    t = format!("<em>{t}</em>");
                }
                if marks.underline {
                    t = format!("<u>{t}</u>");
                }
                if marks.strikethrough {
                    t = format!("<del>{t}</del>");
                }
                if let Some(ref color) = marks.text_color {
                    t = format!("<span style=\"color:{color}\">{t}</span>");
                }
                if let Some(ref bg) = marks.background_color {
                    t = format!("<span style=\"background-color:{bg}\">{t}</span>");
                }
                if let Some(fs) = marks.font_size {
                    t = format!("<span style=\"font-size:{fs}pt\">{t}</span>");
                }
                out.push_str(&t);
            }
            InlineNode::Link { href, text, .. } => {
                out.push_str(&format!(
                    "<a href=\"{}\">{}</a>",
                    crate::xml_util::escape_html(href),
                    crate::xml_util::escape_html(text)
                ));
            }
            InlineNode::HardBreak => out.push_str("<br/>"),
            InlineNode::InlineImage { source, alt, .. } => {
                let src = match source {
                    ImageSource::Embedded { .. } => "media/image1.png".to_string(),
                    ImageSource::Referenced { path } => path.clone(),
                };
                let a = alt.as_deref().unwrap_or("");
                out.push_str(&format!(
                    "<img src=\"{}\" alt=\"{}\"/>",
                    crate::xml_util::escape_html(&src),
                    crate::xml_util::escape_html(a)
                ));
            }
        }
    }
    out
}

fn paragraph_style_html(attrs: &tench_document_core::ParagraphAttrs) -> String {
    let mut styles: Vec<String> = Vec::new();
    match attrs.alignment {
        tench_document_core::Alignment::Center => styles.push("text-align:center".to_string()),
        tench_document_core::Alignment::Right => styles.push("text-align:right".to_string()),
        tench_document_core::Alignment::Justify => styles.push("text-align:justify".to_string()),
        tench_document_core::Alignment::Left => {}
    }
    if attrs.indent_left != 0.0 {
        styles.push(format!("margin-left:{}em", attrs.indent_left));
    }
    if styles.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", styles.join(";"))
    }
}

fn build_toc_ncx(doc: &TenchDocument) -> String {
    let mut nav_points = String::new();
    let mut play_order = 0;

    for block in &doc.content {
        if let BlockNode::Heading {
            level: _, content, ..
        } = block
        {
            play_order += 1;
            let text: String = content
                .iter()
                .map(|n| match n {
                    InlineNode::Text { text, .. } => text.clone(),
                    InlineNode::Link { text, .. } => text.clone(),
                    _ => String::new(),
                })
                .collect();
            let escaped = crate::xml_util::escape_xml(&text);
            nav_points.push_str(&format!(
                "    <navPoint id=\"nav_{play_order}\" playOrder=\"{play_order}\">\
                 <navLabel><text>{escaped}</text></navLabel>\
                 <content src=\"chapter1.html\"/></navPoint>\n"
            ));
        }
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="urn:uuid:tench-docs-epub"/>
    <meta name="dtb:depth" content="1"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle><text>{}</text></docTitle>
  <navMap>
{nav_points}  </navMap>
</ncx>"#,
        crate::xml_util::escape_xml(&doc.metadata.title)
    )
}

fn build_content_opf(doc: &TenchDocument, media_entries: &[(String, Vec<u8>)]) -> String {
    let title = crate::xml_util::escape_xml(&doc.metadata.title);
    let author = crate::xml_util::escape_xml(doc.metadata.author.as_deref().unwrap_or("Unknown"));

    let mut manifest = String::from(
        "<item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>\
         <item id=\"chapter1\" href=\"chapter1.html\" media-type=\"application/xhtml+xml\"/>",
    );
    for (i, (name, _)) in media_entries.iter().enumerate() {
        let mime = if name.ends_with(".png") {
            "image/png"
        } else if name.ends_with(".jpg") || name.ends_with(".jpeg") {
            "image/jpeg"
        } else {
            "application/octet-stream"
        };
        let href = name.trim_start_matches("OEBPS/");
        manifest.push_str(&format!(
            "<item id=\"img{i}\" href=\"{href}\" media-type=\"{mime}\"/>"
        ));
    }

    let spine = String::from("<itemref idref=\"chapter1\"/>");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" unique-identifier="uid" version="3.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uid">urn:uuid:tench-docs-epub</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:creator>{author}</dc:creator>
    <dc:language>en</dc:language>
    <meta property="dcterms:modified">2024-01-01T00:00:00Z</meta>
  </metadata>
  <manifest>{manifest}</manifest>
  <spine toc="ncx">{spine}</spine>
</package>"#
    )
}

const CONTAINER_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tench_document_core::{Marks, ParagraphAttrs};

    #[test]
    fn epub_export_produces_valid_structure() {
        let doc = TenchDocument {
            content: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: "EPUB Test".to_string(),
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            }],
            ..TenchDocument::new("Test Book")
        };

        let bytes = export_epub_bytes(&doc).expect("export epub");
        assert!(bytes.len() > 100);

        // Verify the zip structure.
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        assert!(archive.by_name("mimetype").is_ok());
        assert!(archive.by_name("META-INF/container.xml").is_ok());
        assert!(archive.by_name("OEBPS/content.opf").is_ok());
        assert!(archive.by_name("OEBPS/chapter1.html").is_ok());
    }

    #[test]
    fn epub_export_contains_text() {
        let doc = TenchDocument {
            content: vec![
                BlockNode::Heading {
                    level: 1,
                    content: vec![InlineNode::Text {
                        text: "Chapter 1".to_string(),
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                },
                BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text: "Body text".to_string(),
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                },
            ],
            ..TenchDocument::new("Test Book")
        };

        let bytes = export_epub_bytes(&doc).expect("export epub");
        // Extract the chapter HTML from the ZIP archive.
        let html = extract_zip_xml(&bytes, "OEBPS/chapter1.html");
        assert!(html.contains("Chapter 1"));
        assert!(html.contains("Body text"));
    }

    /// Helper: extract a text file from a ZIP archive.
    fn extract_zip_xml(bytes: &[u8], path: &str) -> String {
        use std::io::Read;
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let mut file = archive.by_name(path).unwrap_or_else(|e| {
            panic!("Failed to find {path} in zip archive: {e}");
        });
        let mut content = String::new();
        file.read_to_string(&mut content).expect("read content");
        content
    }
}
