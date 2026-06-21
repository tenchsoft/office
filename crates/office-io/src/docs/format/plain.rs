use tench_document_core::{BlockNode, ParagraphAttrs, TenchDocument};

pub fn empty_tdm() -> TenchDocument {
    let mut doc = TenchDocument::new("");
    doc.content.push(BlockNode::Paragraph {
        content: Vec::new(),
        attrs: ParagraphAttrs::default(),
    });
    doc
}

/// Convert plain text to a `TenchDocument`.
pub fn plain_text_to_tdm(text: &str) -> TenchDocument {
    TenchDocument::plain_text(text)
}

/// Convert a `TenchDocument` to plain text.
pub fn tdm_to_plain_text(doc: &TenchDocument) -> String {
    doc.to_plain_text()
}
