use std::path::Path;

use tench_document_core::{OfficeContent, RichDocumentContent, TenchDocument};

use super::{export_docx_bytes, import_docx, DOC_SCHEMA};

/// Import DOCX into `OfficeContent::Docs` (backward-compatible).
pub fn import_docx_as_content(path: &Path) -> Result<OfficeContent, String> {
    let doc = import_docx(path)?;
    Ok(tdm_to_docs_content(&doc))
}

/// Export `OfficeContent::Docs` as DOCX bytes (backward-compatible).
pub fn export_docx_bytes_from_content(content: &OfficeContent) -> Result<Vec<u8>, String> {
    let doc = content_to_tdm(content);
    export_docx_bytes(&doc)
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

// ---------------------------------------------------------------------------
// Import helpers
// ---------------------------------------------------------------------------
