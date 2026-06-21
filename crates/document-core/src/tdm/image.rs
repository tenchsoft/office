use serde::{Deserialize, Serialize};

/// Source of an image embedded in a document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ImageSource {
    /// Image data stored directly in the document.
    Embedded { data: Vec<u8> },
    /// Reference to an external file path.
    Referenced { path: String },
}
