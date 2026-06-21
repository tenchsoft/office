pub mod attrs;
pub mod block;
pub mod image;
pub mod inline;
pub mod metadata;

// Re-export all public types from sub-modules.
pub use attrs::*;
pub use block::*;
pub use image::*;
pub use inline::*;
pub use metadata::*;

use serde::{Deserialize, Serialize};

/// The root structure of a Tench Document Model (TDM).
///
/// A `TenchDocument` represents a complete, self-contained document with its
/// metadata, page layout, named styles, block-level content, and
/// headers/footers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TenchDocument {
    pub metadata: TdmMetadata,
    #[serde(default)]
    pub page_setup: attrs::PageSetup,
    #[serde(default)]
    pub styles: Vec<StyleDef>,
    #[serde(default)]
    pub content: Vec<BlockNode>,
    #[serde(default)]
    pub headers_footers: HeadersFooters,
}

impl TenchDocument {
    /// Create an empty document with the given title.
    pub fn new(title: &str) -> Self {
        TenchDocument {
            metadata: TdmMetadata {
                title: title.to_string(),
                author: None,
                created_at: None,
                updated_at: None,
            },
            page_setup: attrs::PageSetup::default(),
            styles: Vec::new(),
            content: Vec::new(),
            headers_footers: HeadersFooters::default(),
        }
    }

    /// Create a document from a plain-text string.
    ///
    /// Each line becomes a separate paragraph. Empty lines are preserved as
    /// empty paragraphs.
    pub fn plain_text(text: &str) -> Self {
        let content: Vec<BlockNode> = text
            .lines()
            .map(|line| BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: line.to_string(),
                    marks: Marks::default(),
                }],
                attrs: attrs::ParagraphAttrs::default(),
            })
            .collect();

        TenchDocument {
            metadata: TdmMetadata {
                title: String::new(),
                author: None,
                created_at: None,
                updated_at: None,
            },
            page_setup: attrs::PageSetup::default(),
            styles: Vec::new(),
            content,
            headers_footers: HeadersFooters::default(),
        }
    }

    /// Convert the document to plain text.
    ///
    /// Block nodes are separated by newlines. Inline formatting is stripped.
    pub fn to_plain_text(&self) -> String {
        let mut result = String::new();
        for (i, block) in self.content.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            append_block_text(&mut result, block);
        }
        result
    }

    /// Count the number of words in the document.
    ///
    /// A word is defined as a contiguous sequence of non-whitespace characters.
    pub fn word_count(&self) -> usize {
        self.to_plain_text().split_whitespace().count()
    }

    /// Count the number of characters in the document (including spaces).
    pub fn character_count(&self) -> usize {
        self.to_plain_text().len()
    }

    /// Count the number of paragraphs (top-level block nodes).
    pub fn paragraph_count(&self) -> usize {
        self.content.len()
    }
}

/// Recursively append the text content of a block node to the output string.
fn append_block_text(out: &mut String, block: &BlockNode) {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
            append_inline_text(out, content);
        }
        BlockNode::BulletList { items } | BlockNode::OrderedList { items, .. } => {
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                append_inline_text(out, &item.content);
            }
        }
        BlockNode::TaskList { items } => {
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                if item.checked {
                    out.push_str("[x] ");
                } else {
                    out.push_str("[ ] ");
                }
                append_inline_text(out, &item.content);
            }
        }
        BlockNode::BlockQuote { content } => {
            for (i, child) in content.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                append_block_text(out, child);
            }
        }
        BlockNode::CodeBlock { code, .. } => {
            out.push_str(code);
        }
        BlockNode::Table { rows } => {
            for (ri, row) in rows.iter().enumerate() {
                if ri > 0 {
                    out.push('\n');
                }
                for (ci, cell) in row.cells.iter().enumerate() {
                    if ci > 0 {
                        out.push('\t');
                    }
                    for (bi, block) in cell.content.iter().enumerate() {
                        if bi > 0 {
                            out.push(' ');
                        }
                        append_block_text(out, block);
                    }
                }
            }
        }
        BlockNode::HorizontalRule | BlockNode::PageBreak => {}
        BlockNode::Image { alt, .. } => {
            if let Some(alt_text) = alt {
                out.push_str(alt_text);
            }
        }
        BlockNode::Footnote { content, .. } => {
            append_inline_text(out, content);
        }
    }
}

/// Append the text content of inline nodes to the output string.
fn append_inline_text(out: &mut String, nodes: &[InlineNode]) {
    for node in nodes {
        match node {
            InlineNode::Text { text, .. } => {
                out.push_str(text);
            }
            InlineNode::Link { text, .. } => {
                out.push_str(text);
            }
            InlineNode::InlineImage { alt, .. } => {
                if let Some(alt_text) = alt {
                    out.push_str(alt_text);
                }
            }
            InlineNode::HardBreak => {
                out.push('\n');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_document() {
        let doc = TenchDocument::new("Test");
        assert_eq!(doc.metadata.title, "Test");
        assert!(doc.content.is_empty());
        assert!(doc.styles.is_empty());
    }

    #[test]
    fn plain_text_creates_paragraphs() {
        let doc = TenchDocument::plain_text("Hello\nWorld");
        assert_eq!(doc.content.len(), 2);
    }

    #[test]
    fn to_plain_text_extracts_text() {
        let doc = TenchDocument::plain_text("Hello\nWorld");
        assert_eq!(doc.to_plain_text(), "Hello\nWorld");
    }

    #[test]
    fn word_count_counts_words() {
        let doc = TenchDocument::plain_text("Hello beautiful world");
        assert_eq!(doc.word_count(), 3);
    }

    #[test]
    fn character_count_includes_spaces() {
        let doc = TenchDocument::plain_text("Hello world");
        assert_eq!(doc.character_count(), 11);
    }

    #[test]
    fn paragraph_count_counts_blocks() {
        let doc = TenchDocument::plain_text("A\nB\nC");
        assert_eq!(doc.paragraph_count(), 3);
    }

    #[test]
    fn empty_document_counts() {
        let doc = TenchDocument::new("Empty");
        assert_eq!(doc.word_count(), 0);
        assert_eq!(doc.character_count(), 0);
        assert_eq!(doc.paragraph_count(), 0);
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let doc = TenchDocument::plain_text("Test content");
        let json = serde_json::to_string(&doc).unwrap();
        let decoded: TenchDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, decoded);
    }
}
