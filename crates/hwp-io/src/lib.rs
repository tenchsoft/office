//! HWP/HWPX file format parser and writer.
//!
//! Supports reading and writing Korean Hangul Word Processor (HWP) binary files
//! and the XML-based HWPX format, converting to/from `TenchDocument` (TDM).
//!
//! Many fields are parsed spec-faithfully but not yet consumed by the
//! conversion layer. Suppress dead-code warnings until the full pipeline
//! is wired up.
#![allow(dead_code)]

mod bin_data;
mod body_text;
mod cfb;
mod char_shape;
mod controls;
mod doc_info;
mod header;
mod hwp_to_tdm;
pub mod hwpx;
mod para_shape;
mod record;
mod tdm_to_hwp;

mod error;

pub use error::HwpError;

use tench_document_core::tdm::TenchDocument;

/// Read an HWP binary file and convert to TenchDocument.
pub fn read_hwp(path: &std::path::Path) -> Result<TenchDocument, HwpError> {
    let data = std::fs::read(path)?;
    read_hwp_bytes(&data)
}

/// Read HWP binary data and convert to TenchDocument.
pub fn read_hwp_bytes(data: &[u8]) -> Result<TenchDocument, HwpError> {
    let mut container = cfb::CfbContainer::open(data)?;
    let file_header = container.read_header()?;

    if file_header.is_encrypted() {
        return Err(HwpError::Encrypted("HWP file is encrypted".into()));
    }

    let doc_info_data = container.read_doc_info(&file_header)?;
    let doc_info = doc_info::parse_doc_info(&doc_info_data, &file_header.version)?;

    let sections = container.read_sections(&file_header)?;
    let paragraphs = body_text::parse_sections(&sections, &doc_info, &file_header.version)?;

    let images = container.read_bin_data(&file_header)?;

    hwp_to_tdm::convert(paragraphs, doc_info, images, &file_header)
}

/// Write a TenchDocument as an HWP binary file.
pub fn write_hwp(doc: &TenchDocument, path: &std::path::Path) -> Result<(), HwpError> {
    let data = write_hwp_bytes(doc)?;
    std::fs::write(path, data)?;
    Ok(())
}

/// Write a TenchDocument as HWP binary data.
pub fn write_hwp_bytes(doc: &TenchDocument) -> Result<Vec<u8>, HwpError> {
    tdm_to_hwp::convert(doc)
}

/// Read an HWPX (XML/ZIP) file and convert to TenchDocument.
pub fn read_hwpx(path: &std::path::Path) -> Result<TenchDocument, HwpError> {
    let data = std::fs::read(path)?;
    read_hwpx_bytes(&data)
}

/// Read HWPX binary data and convert to TenchDocument.
pub fn read_hwpx_bytes(data: &[u8]) -> Result<TenchDocument, HwpError> {
    hwpx::read_hwpx(data)
}

/// Write a TenchDocument as an HWPX file.
pub fn write_hwpx(doc: &TenchDocument, path: &std::path::Path) -> Result<(), HwpError> {
    let data = write_hwpx_bytes(doc)?;
    std::fs::write(path, data)?;
    Ok(())
}

/// Write a TenchDocument as HWPX binary data.
pub fn write_hwpx_bytes(doc: &TenchDocument) -> Result<Vec<u8>, HwpError> {
    hwpx::write_hwpx(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tench_document_core::tdm::{BlockNode, InlineNode, TenchDocument};

    fn simple_doc() -> TenchDocument {
        let mut doc = TenchDocument::new("Test Document");
        doc.content.push(BlockNode::Paragraph {
            attrs: Default::default(),
            content: vec![InlineNode::Text {
                text: "Hello, HWP!".into(),
                marks: Default::default(),
            }],
        });
        doc.content.push(BlockNode::Heading {
            level: 1,
            attrs: Default::default(),
            content: vec![InlineNode::Text {
                text: "Heading 1".into(),
                marks: Default::default(),
            }],
        });
        doc
    }

    #[test]
    fn hwp_roundtrip() {
        let doc = simple_doc();
        let bytes = write_hwp_bytes(&doc).expect("write_hwp_bytes failed");
        let restored = read_hwp_bytes(&bytes).expect("read_hwp_bytes failed");
        assert_eq!(restored.content.len(), doc.content.len());
    }

    #[test]
    fn hwpx_roundtrip() {
        let doc = simple_doc();
        let bytes = write_hwpx_bytes(&doc).expect("write_hwpx_bytes failed");
        let restored = read_hwpx_bytes(&bytes).expect("read_hwpx_bytes failed");
        assert_eq!(restored.content.len(), doc.content.len());
    }

    #[test]
    fn hwp_preserves_paragraph_text() {
        let doc = simple_doc();
        let bytes = write_hwp_bytes(&doc).expect("write_hwp_bytes failed");
        let restored = read_hwp_bytes(&bytes).expect("read_hwp_bytes failed");
        if let BlockNode::Paragraph { content, .. } = &restored.content[0] {
            if let InlineNode::Text { text, .. } = &content[0] {
                assert_eq!(text, "Hello, HWP!");
            } else {
                panic!("Expected Text inline node");
            }
        } else {
            panic!("Expected Paragraph block");
        }
    }

    #[test]
    fn hwpx_preserves_paragraph_text() {
        let doc = simple_doc();
        let bytes = write_hwpx_bytes(&doc).expect("write_hwpx_bytes failed");
        let restored = read_hwpx_bytes(&bytes).expect("read_hwpx_bytes failed");
        if let BlockNode::Paragraph { content, .. } = &restored.content[0] {
            if let InlineNode::Text { text, .. } = &content[0] {
                assert_eq!(text, "Hello, HWP!");
            } else {
                panic!("Expected Text inline node");
            }
        } else {
            panic!("Expected Paragraph block");
        }
    }

    #[test]
    fn hwp_empty_document() {
        let doc = TenchDocument::new("");
        let bytes = write_hwp_bytes(&doc).expect("write_hwp_bytes failed");
        let restored = read_hwp_bytes(&bytes).expect("read_hwp_bytes failed");
        assert!(restored.content.is_empty());
    }

    #[test]
    fn hwpx_empty_document() {
        let doc = TenchDocument::new("");
        let bytes = write_hwpx_bytes(&doc).expect("write_hwpx_bytes failed");
        let restored = read_hwpx_bytes(&bytes).expect("read_hwpx_bytes failed");
        assert!(restored.content.is_empty());
    }

    #[test]
    fn hwp_invalid_data() {
        let result = read_hwp_bytes(b"not a valid hwp file");
        assert!(result.is_err());
    }

    #[test]
    fn hwpx_invalid_data() {
        let result = read_hwpx_bytes(b"not a valid hwpx file");
        assert!(result.is_err());
    }
}
