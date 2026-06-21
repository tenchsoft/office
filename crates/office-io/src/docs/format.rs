//! Format conversion between text-based formats and the Tench Document Model.
//!
//! All functions now operate on [`tench_document_core::TenchDocument`] internally. The legacy
//! `OfficeContent`-based API is preserved as thin wrappers that extract the
//! TDM document from the `OfficeContent::Docs` variant.

mod html_parse;
mod html_render;
mod legacy;
mod markdown;
mod plain;
#[cfg(test)]
mod tests;

pub use html_parse::html_to_tdm;
pub use html_render::tdm_to_html;
pub use legacy::{
    docs_content_to_html, docs_content_to_markdown, docs_content_to_plain_text,
    docs_content_to_tdm, empty_docs_content, html_to_docs_content, markdown_to_docs_content,
    plain_text_to_docs_content, tdm_to_docs_content,
};
pub use markdown::{markdown_to_tdm, tdm_to_markdown};
pub use plain::{empty_tdm, plain_text_to_tdm, tdm_to_plain_text};
