use serde::{Deserialize, Serialize};

/// Text formatting marks applied to inline content.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Marks {
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub strikethrough: bool,
    #[serde(default)]
    pub superscript: bool,
    #[serde(default)]
    pub subscript: bool,
    #[serde(default)]
    pub code: bool,
    #[serde(default)]
    pub text_color: Option<String>,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub font_size: Option<f32>,
    #[serde(default)]
    pub font_family: Option<String>,
}

/// A hyperlink with optional display text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub href: String,
    #[serde(default)]
    pub title: Option<String>,
}

/// Inline-level content nodes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InlineNode {
    /// A run of styled text.
    Text {
        text: String,
        #[serde(default)]
        marks: Marks,
    },
    /// A forced line break within a paragraph.
    HardBreak,
    /// An image rendered inline within text.
    InlineImage {
        source: super::ImageSource,
        #[serde(default)]
        alt: Option<String>,
        #[serde(default)]
        width: Option<f32>,
        #[serde(default)]
        height: Option<f32>,
    },
    /// A hyperlink wrapping text.
    Link {
        href: String,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        text: String,
        #[serde(default)]
        marks: Marks,
    },
}
