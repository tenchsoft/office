use super::{
    empty_tdm, html_to_tdm, markdown_to_tdm, plain_text_to_tdm, tdm_to_html, tdm_to_markdown,
    tdm_to_plain_text,
};
use tench_document_core::{OfficeContent, RichDocumentContent, TenchDocument};

const DOC_SCHEMA: &str = "tench.docs.v1";

/// Create an empty `OfficeContent::Docs`.
pub fn empty_docs_content() -> OfficeContent {
    tdm_to_docs_content(&empty_tdm())
}

/// Convert plain text to `OfficeContent::Docs`.
pub fn plain_text_to_docs_content(text: &str) -> OfficeContent {
    tdm_to_docs_content(&plain_text_to_tdm(text))
}

/// Convert Markdown to `OfficeContent::Docs`.
pub fn markdown_to_docs_content(markdown: &str) -> OfficeContent {
    tdm_to_docs_content(&markdown_to_tdm(markdown))
}

/// Convert HTML to `OfficeContent::Docs`.
pub fn html_to_docs_content(html: &str) -> OfficeContent {
    tdm_to_docs_content(&html_to_tdm(html))
}

/// Extract plain text from `OfficeContent::Docs`.
pub fn docs_content_to_plain_text(content: &OfficeContent) -> String {
    docs_content_to_tdm(content)
        .map(|doc| tdm_to_plain_text(&doc))
        .unwrap_or_default()
}

/// Extract Markdown from `OfficeContent::Docs`.
pub fn docs_content_to_markdown(content: &OfficeContent) -> String {
    docs_content_to_tdm(content)
        .map(|doc| tdm_to_markdown(&doc))
        .unwrap_or_default()
}

/// Extract HTML from `OfficeContent::Docs`.
pub fn docs_content_to_html(content: &OfficeContent) -> String {
    docs_content_to_tdm(content)
        .map(|doc| tdm_to_html(&doc))
        .unwrap_or_else(|| {
            "<!doctype html>\n<html>\n<body>\n<p></p>\n</body>\n</html>\n".to_string()
        })
}

/// Extract the `TenchDocument` from an `OfficeContent::Docs` variant.
pub fn docs_content_to_tdm(content: &OfficeContent) -> Option<TenchDocument> {
    match content {
        OfficeContent::Docs(rich) => rich.document.clone(),
        _ => None,
    }
}

/// Wrap a `TenchDocument` in `OfficeContent::Docs`.
pub fn tdm_to_docs_content(doc: &TenchDocument) -> OfficeContent {
    OfficeContent::Docs(RichDocumentContent {
        schema: DOC_SCHEMA.to_string(),
        document: Some(doc.clone()),
    })
}
