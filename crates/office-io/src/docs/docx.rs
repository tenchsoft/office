//! DOCX import/export using the Tench Document Model.
//!
//! `import_docx` returns a [`TenchDocument`] and `export_docx_bytes` accepts
//! one. Legacy `OfficeContent` wrappers are also provided for backward
//! compatibility.
//!
//! ## Supported features
//!
//! **Import:** paragraphs, headings, bold/italic/underline/strike, font size,
//! font color, background color, text alignment, indentation, tables (with
//! cell width, colspan/rowspan), images (inline and block-level via
//! relationships), headers/footers, page margins, page size/orientation,
//! default style mapping.
//!
//! **Export:** all of the above plus `word/styles.xml`, proper `<w:sectPr>`
//! with page setup, header/footer XML parts, and image embedding.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use crate::zip_util;
use tench_document_core::{HeadersFooters, TenchDocument};

#[cfg(test)]
use tench_document_core::{
    Alignment, BlockNode, InlineNode, Margins, Marks, Orientation, PageSetup, PaperSize,
    ParagraphAttrs, TableCell, TableRow,
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const DOC_SCHEMA: &str = "tench.docs.v1";

mod export;
mod import;
mod legacy;
mod util;

pub use legacy::{export_docx_bytes_from_content, import_docx_as_content};

// ---------------------------------------------------------------------------
// TDM-based API
// ---------------------------------------------------------------------------

/// Import a DOCX file and return a `TenchDocument`.
pub fn import_docx(path: &Path) -> Result<TenchDocument, String> {
    let file = File::open(path)
        .map_err(|error| format!("Failed to open DOCX {}: {error}", path.display()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| format!("Failed to read DOCX zip: {error}"))?;

    // Apply archive limits
    crate::zip_util::check_archive_limits(&mut archive, &crate::zip_util::ArchiveLimits::desktop())
        .map_err(|e| format!("DOCX archive limit check failed: {e}"))?;

    let mut document_xml = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(|error| format!("DOCX missing word/document.xml: {error}"))?
        .read_to_string(&mut document_xml)
        .map_err(|error| format!("Failed to read DOCX document.xml: {error}"))?;

    // Parse relationships to resolve image references.
    let mut relationships = HashMap::new();
    if let Ok(mut rels_file) = archive.by_name("word/_rels/document.xml.rels") {
        let mut rels_xml = String::new();
        if rels_file.read_to_string(&mut rels_xml).is_ok() {
            relationships = import::parse_relationships(&rels_xml);
        }
    }

    // Collect embedded images from word/media/.
    let mut media_images: HashMap<String, Vec<u8>> = HashMap::new();
    for idx in 0..archive.len() {
        if let Ok(mut entry) = archive.by_index(idx) {
            let name = entry.name().to_string();
            if name.starts_with("word/media/") {
                let mut data = Vec::new();
                if entry.read_to_end(&mut data).is_ok() {
                    media_images.insert(name, data);
                }
            }
        }
    }

    // Parse headers/footers.
    let mut headers_footers = HeadersFooters::default();
    if let Ok(mut hf_file) = archive.by_name("word/header1.xml") {
        let mut xml = String::new();
        if hf_file.read_to_string(&mut xml).is_ok() {
            headers_footers.default_header = Some(import::extract_header_footer_text(&xml));
        }
    }
    if let Ok(mut hf_file) = archive.by_name("word/footer1.xml") {
        let mut xml = String::new();
        if hf_file.read_to_string(&mut xml).is_ok() {
            headers_footers.default_footer = Some(import::extract_header_footer_text(&xml));
        }
    }

    let mut doc = import::parse_document_xml_to_tdm(&document_xml, &relationships, &media_images);
    doc.headers_footers = headers_footers;

    Ok(doc)
}

/// Export a `TenchDocument` as DOCX bytes.
pub fn export_docx_bytes(doc: &TenchDocument) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Collect embedded images for media directory.
    let mut media_entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut rel_counter: u32 = 2; // rId1 is used for document.xml
    let mut extra_rels: Vec<(String, String, String)> = Vec::new(); // (rId, type, target)

    export::collect_embedded_images(doc, &mut media_entries, &mut rel_counter, &mut extra_rels);

    // Header/footer relationships.
    let has_header = doc.headers_footers.default_header.is_some();
    let has_footer = doc.headers_footers.default_footer.is_some();
    let header_rid = if has_header {
        let rid = format!("rId{rel_counter}");
        rel_counter += 1;
        extra_rels.push((
            rid.clone(),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header"
                .to_string(),
            "header1.xml".to_string(),
        ));
        Some(rid)
    } else {
        None
    };
    let footer_rid = if has_footer {
        let rid = format!("rId{rel_counter}");
        rel_counter += 1; // reserved for future relationship entries
        extra_rels.push((
            rid.clone(),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer"
                .to_string(),
            "footer1.xml".to_string(),
        ));
        Some(rid)
    } else {
        None
    };
    let _ = rel_counter; // reserved for future relationship entries

    // Build relationship map: media_name -> rId.
    let image_rid_map: HashMap<String, String> = extra_rels
        .iter()
        .filter(|(_, typ, _)| typ.contains("image"))
        .map(|(rid, _, target)| (target.clone(), rid.clone()))
        .collect();

    let content_types = export::build_content_types(&media_entries, has_header, has_footer);
    let rels_xml = export::build_document_rels(&extra_rels);
    let document_xml =
        export::build_document_xml_from_tdm(doc, &image_rid_map, header_rid, footer_rid);
    let styles_xml = export::build_styles_xml(doc);

    zip_util::write_zip_file(&mut writer, "[Content_Types].xml", &content_types, options)?;
    zip_util::write_zip_file(&mut writer, "_rels/.rels", PACKAGE_RELS_XML, options)?;
    zip_util::write_zip_file(
        &mut writer,
        "word/_rels/document.xml.rels",
        &rels_xml,
        options,
    )?;
    zip_util::write_zip_file(&mut writer, "word/document.xml", &document_xml, options)?;
    zip_util::write_zip_file(&mut writer, "word/styles.xml", &styles_xml, options)?;

    // Write header/footer files.
    if let Some(ref header_text) = doc.headers_footers.default_header {
        let xml = export::build_header_footer_xml(header_text, "header");
        zip_util::write_zip_file(&mut writer, "word/header1.xml", &xml, options)?;
    }
    if let Some(ref footer_text) = doc.headers_footers.default_footer {
        let xml = export::build_header_footer_xml(footer_text, "footer");
        zip_util::write_zip_file(&mut writer, "word/footer1.xml", &xml, options)?;
    }

    // Write media files.
    for (media_name, data) in &media_entries {
        writer
            .start_file(media_name, options)
            .map_err(|e| format!("Failed to add {media_name}: {e}"))?;
        writer
            .write_all(data)
            .map_err(|e| format!("Failed to write {media_name}: {e}"))?;
    }

    writer
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|error| format!("Failed to finish DOCX archive: {error}"))
}

// ---------------------------------------------------------------------------
// Legacy OfficeContent wrappers
// ---------------------------------------------------------------------------

const PACKAGE_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

#[cfg(test)]
mod tests {
    use super::util::{extract_attr, mm_to_twip, twip_to_mm};
    use super::*;

    #[test]
    fn docx_export_import_round_trips_basic_content() {
        let doc = TenchDocument {
            content: vec![
                BlockNode::Heading {
                    level: 1,
                    content: vec![InlineNode::Text {
                        text: "Report".to_string(),
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                },
                BlockNode::Paragraph {
                    content: vec![
                        InlineNode::Text {
                            text: "Alpha ".to_string(),
                            marks: Marks {
                                bold: true,
                                ..Marks::default()
                            },
                        },
                        InlineNode::Text {
                            text: "Beta".to_string(),
                            marks: Marks {
                                italic: true,
                                ..Marks::default()
                            },
                        },
                    ],
                    attrs: ParagraphAttrs::default(),
                },
            ],
            ..TenchDocument::new("")
        };

        let bytes = export_docx_bytes(&doc).expect("export docx");
        let path = std::env::temp_dir().join(format!(
            "tench_docs_docx_roundtrip_{}_{}.docx",
            std::process::id(),
            bytes.len()
        ));
        std::fs::write(&path, bytes).expect("write docx");

        let imported = import_docx(&path).expect("import docx");
        let text = imported.to_plain_text();

        assert!(text.contains("Report"));
        assert!(text.contains("Alpha"));
        assert!(text.contains("Beta"));

        // Verify marks are preserved
        match &imported.content[1] {
            BlockNode::Paragraph { content, .. } => {
                match &content[0] {
                    InlineNode::Text { marks, .. } => assert!(marks.bold),
                    other => panic!("expected text node, got {other:?}"),
                }
                match &content[1] {
                    InlineNode::Text { marks, .. } => assert!(marks.italic),
                    other => panic!("expected text node, got {other:?}"),
                }
            }
            other => panic!("expected paragraph, got {other:?}"),
        }

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn docx_export_preserves_page_setup() {
        let doc = TenchDocument {
            page_setup: PageSetup {
                paper_size: PaperSize::Letter,
                orientation: Orientation::Landscape,
                margins: Margins {
                    top: 30.0,
                    bottom: 30.0,
                    left: 20.0,
                    right: 20.0,
                },
            },
            content: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: "Test".to_string(),
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            }],
            ..TenchDocument::new("")
        };

        let bytes = export_docx_bytes(&doc).expect("export");
        let xml = extract_zip_xml(&bytes, "word/document.xml");
        assert!(xml.contains("w:orient=\"landscape\""));
    }

    #[test]
    fn docx_export_preserves_alignment_and_indent() {
        let doc = TenchDocument {
            content: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: "Centered".to_string(),
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs {
                    alignment: Alignment::Center,
                    indent_left: 10.0,
                    indent_first_line: 5.0,
                    ..ParagraphAttrs::default()
                },
            }],
            ..TenchDocument::new("")
        };

        let bytes = export_docx_bytes(&doc).expect("export");
        let xml = extract_zip_xml(&bytes, "word/document.xml");
        assert!(xml.contains("w:val=\"center\""));
        assert!(xml.contains("w:ind"));
    }

    #[test]
    fn docx_export_preserves_font_size_and_color() {
        let doc = TenchDocument {
            content: vec![BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: "Styled".to_string(),
                    marks: Marks {
                        bold: true,
                        font_size: Some(14.0),
                        text_color: Some("#FF0000".to_string()),
                        background_color: Some("#FFFF00".to_string()),
                        ..Marks::default()
                    },
                }],
                attrs: ParagraphAttrs::default(),
            }],
            ..TenchDocument::new("")
        };

        let bytes = export_docx_bytes(&doc).expect("export");
        let xml = extract_zip_xml(&bytes, "word/document.xml");
        assert!(xml.contains("<w:b/>"));
        assert!(xml.contains("w:val=\"28\"")); // 14pt = 28 half-points
        assert!(xml.contains("w:val=\"FF0000\""));
        assert!(xml.contains("w:fill=\"FFFF00\""));
    }

    #[test]
    fn docx_export_preserves_table() {
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

        let bytes = export_docx_bytes(&doc).expect("export");
        let xml = extract_zip_xml(&bytes, "word/document.xml");
        assert!(xml.contains("<w:tbl>"));
        assert!(xml.contains("<w:tr>"));
        assert!(xml.contains("<w:tc>"));
    }

    #[test]
    fn docx_export_preserves_headers_footers() {
        let mut doc = TenchDocument::new("Test");
        doc.headers_footers.default_header = Some("Header Text".to_string());
        doc.headers_footers.default_footer = Some("Page {{page}}".to_string());
        doc.content.push(BlockNode::Paragraph {
            content: vec![InlineNode::Text {
                text: "Body".to_string(),
                marks: Marks::default(),
            }],
            attrs: ParagraphAttrs::default(),
        });

        let bytes = export_docx_bytes(&doc).expect("export");
        // Check that header/footer relationship entries exist in .rels.
        let rels = extract_zip_xml(&bytes, "word/_rels/document.xml.rels");
        assert!(rels.contains("header1.xml"));
        assert!(rels.contains("footer1.xml"));
    }

    #[test]
    fn extract_attr_finds_value() {
        let tag = r#"Id="rId5" Type="image" Target="media/image1.png""#;
        assert_eq!(extract_attr(tag, "Id"), Some("rId5".to_string()));
        assert_eq!(
            extract_attr(tag, "Target"),
            Some("media/image1.png".to_string())
        );
        assert_eq!(extract_attr(tag, "Missing"), None);
    }

    #[test]
    fn mm_twip_roundtrip() {
        let mm = 25.4;
        let twips = mm_to_twip(mm);
        let back = twip_to_mm(twips);
        assert!((back - mm).abs() < 0.1);
    }

    /// Helper: extract an XML file from a ZIP archive.
    fn extract_zip_xml(bytes: &[u8], path: &str) -> String {
        use std::io::Read;
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let mut file = archive.by_name(path).unwrap_or_else(|e| {
            panic!("Failed to find {path} in zip archive: {e}");
        });
        let mut content = String::new();
        file.read_to_string(&mut content).expect("read xml");
        content
    }
}
