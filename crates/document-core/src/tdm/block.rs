use serde::{Deserialize, Serialize};

use super::attrs::ParagraphAttrs;
use super::image::ImageSource;
use super::inline::InlineNode;

/// A table cell.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableCell {
    #[serde(default)]
    pub content: Vec<BlockNode>,
    #[serde(default)]
    pub colspan: u32,
    #[serde(default)]
    pub rowspan: u32,
}

impl Default for TableCell {
    fn default() -> Self {
        TableCell {
            content: Vec::new(),
            colspan: 1,
            rowspan: 1,
        }
    }
}

/// A table row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableRow {
    #[serde(default)]
    pub cells: Vec<TableCell>,
}

/// An item in a bullet or ordered list.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    #[serde(default)]
    pub content: Vec<InlineNode>,
    #[serde(default)]
    pub children: Vec<ListItem>,
}

/// A task list item with a checked state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskItem {
    pub checked: bool,
    #[serde(default)]
    pub content: Vec<InlineNode>,
    #[serde(default)]
    pub children: Vec<TaskItem>,
}

/// Block-level content nodes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockNode {
    /// A paragraph of inline content.
    Paragraph {
        #[serde(default)]
        content: Vec<InlineNode>,
        #[serde(default)]
        attrs: ParagraphAttrs,
    },
    /// A heading with level 1-6.
    Heading {
        level: u8,
        #[serde(default)]
        content: Vec<InlineNode>,
        #[serde(default)]
        attrs: ParagraphAttrs,
    },
    /// A bullet (unordered) list.
    BulletList {
        #[serde(default)]
        items: Vec<ListItem>,
    },
    /// An ordered (numbered) list.
    OrderedList {
        #[serde(default)]
        items: Vec<ListItem>,
        #[serde(default)]
        start: u32,
    },
    /// A task (checklist) list.
    TaskList {
        #[serde(default)]
        items: Vec<TaskItem>,
    },
    /// A block quotation.
    BlockQuote {
        #[serde(default)]
        content: Vec<BlockNode>,
    },
    /// A code block with optional language.
    CodeBlock {
        #[serde(default)]
        language: Option<String>,
        code: String,
    },
    /// A table with rows.
    Table {
        #[serde(default)]
        rows: Vec<TableRow>,
    },
    /// A horizontal rule / divider.
    HorizontalRule,
    /// A block-level image.
    Image {
        source: ImageSource,
        #[serde(default)]
        alt: Option<String>,
        #[serde(default)]
        width: Option<f32>,
        #[serde(default)]
        height: Option<f32>,
    },
    /// A page break.
    PageBreak,
    /// A footnote with a reference number and content.
    Footnote {
        /// The footnote reference number.
        number: u32,
        #[serde(default)]
        content: Vec<InlineNode>,
    },
}
