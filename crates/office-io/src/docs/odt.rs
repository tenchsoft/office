//! ODT import/export using the Tench Document Model.
//!
//! ## Supported features
//!
//! **Import:** paragraphs, headings, text:span (bold/italic/underline/strike,
//! font-size, color), tables (table:table, table:table-row, table:table-cell),
//! images (draw:image), text alignment (fo:text-align), indentation
//! (fo:margin-left, fo:margin-right), automatic styles.
//!
//! **Export:** all of the above plus office:automatic-styles, manifest entries
//! for images, and proper table structures.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use crate::{xml_util, zip_util};
use tench_document_core::{
    Alignment, BlockNode, ImageSource, InlineNode, Marks, OfficeContent, ParagraphAttrs,
    RichDocumentContent, TableCell, TableRow, TdmMetadata, TenchDocument,
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const DOC_SCHEMA: &str = "tench.docs.v1";
const ODT_MIMETYPE: &str = "application/vnd.oasis.opendocument.text";

mod export;
mod import;

pub use export::export_odt_bytes;
pub use import::import_odt;

// ---------------------------------------------------------------------------
// Legacy OfficeContent wrappers
// ---------------------------------------------------------------------------

/// Import ODT into `OfficeContent::Docs` (backward-compatible).
pub fn import_odt_as_content(path: &Path) -> Result<OfficeContent, String> {
    let doc = import_odt(path)?;
    Ok(tdm_to_docs_content(&doc))
}

/// Export `OfficeContent::Docs` as ODT bytes (backward-compatible).
pub fn export_odt_bytes_from_content(content: &OfficeContent) -> Result<Vec<u8>, String> {
    let doc = content_to_tdm(content);
    export_odt_bytes(&doc)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odt_export_import_round_trips_basic_content() {
        let doc = TenchDocument {
            content: vec![
                BlockNode::Heading {
                    level: 2,
                    content: vec![InlineNode::Text {
                        text: "ODT Report".to_string(),
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                },
                BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text: "ODT body".to_string(),
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                },
            ],
            ..TenchDocument::new("")
        };

        let bytes = export_odt_bytes(&doc).expect("export odt");
        let path = std::env::temp_dir().join(format!(
            "tench_docs_odt_roundtrip_{}_{}.odt",
            std::process::id(),
            bytes.len()
        ));
        std::fs::write(&path, bytes).expect("write odt");

        let imported = import_odt(&path).expect("import odt");
        let text = imported.to_plain_text();

        assert!(text.contains("ODT Report"));
        assert!(text.contains("ODT body"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn odt_export_preserves_table() {
        let doc = TenchDocument {
            content: vec![BlockNode::Table {
                rows: vec![TableRow {
                    cells: vec![
                        TableCell {
                            content: vec![BlockNode::Paragraph {
                                content: vec![InlineNode::Text {
                                    text: "Cell A".to_string(),
                                    marks: Marks::default(),
                                }],
                                attrs: ParagraphAttrs::default(),
                            }],
                            ..TableCell::default()
                        },
                        TableCell {
                            content: vec![BlockNode::Paragraph {
                                content: vec![InlineNode::Text {
                                    text: "Cell B".to_string(),
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

        let bytes = export_odt_bytes(&doc).expect("export odt");
        let xml = extract_zip_xml(&bytes, "content.xml");
        assert!(xml.contains("<table:table>"));
        assert!(xml.contains("<table:table-row>"));
        assert!(xml.contains("<table:table-cell>"));
        assert!(xml.contains("Cell A"));
    }

    #[test]
    fn odt_export_preserves_inline_formatting() {
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

        let bytes = export_odt_bytes(&doc).expect("export odt");
        let xml = extract_zip_xml(&bytes, "content.xml");
        assert!(xml.contains("fo:font-weight=\"bold\""));
        assert!(xml.contains("fo:font-style=\"italic\""));
    }

    #[test]
    fn odt_import_rejects_unsafe_zip_entries_security_regression() {
        let path =
            std::env::temp_dir().join(format!("tench_docs_odt_unsafe_{}.odt", std::process::id()));
        let file = std::fs::File::create(&path).expect("create odt");
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer
            .start_file("content.xml", options)
            .expect("content entry");
        writer
            .write_all(br#"<office:document-content/>"#)
            .expect("content");
        writer
            .start_file("../evil.txt", options)
            .expect("evil entry");
        writer.write_all(b"evil").expect("evil");
        writer.finish().expect("finish");

        let result = import_odt(&path);

        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("safety limits"),
            "ODT import should reject unsafe archive entries"
        );
        let _ = std::fs::remove_file(path);
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
