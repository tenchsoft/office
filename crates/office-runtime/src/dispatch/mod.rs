use std::path::Path;

use tench_document_core::{
    ImportExportDiagnostic, OfficeArtifact, OfficeContent, OfficeFileFormat, OfficeProductKind,
};
use tench_office_io::office_file;

mod docs;
mod sheets;
mod slides;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OfficeDispatchKind {
    Docs,
    Kodocs,
    Sheets,
    Slides,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OfficeDispatchProfile {
    product: OfficeProductKind,
    id_prefix: &'static str,
    schema_version: &'static str,
    fallback_title: &'static str,
    default_format: OfficeFileFormat,
    native_extensions: &'static [&'static str],
    kind: OfficeDispatchKind,
}

pub const DOCS_DISPATCH: OfficeDispatchProfile = OfficeDispatchProfile {
    product: OfficeProductKind::Docs,
    id_prefix: "doc",
    schema_version: "tench.docs.v1",
    fallback_title: "Untitled Document",
    default_format: OfficeFileFormat::Docx,
    native_extensions: &[],
    kind: OfficeDispatchKind::Docs,
};

pub const KODOCS_DISPATCH: OfficeDispatchProfile = OfficeDispatchProfile {
    product: OfficeProductKind::Docs,
    id_prefix: "hwp",
    schema_version: "tench.kodocs.v1",
    fallback_title: "Untitled HWP Document",
    default_format: OfficeFileFormat::Hwpx,
    native_extensions: &[],
    kind: OfficeDispatchKind::Kodocs,
};

pub const SHEETS_DISPATCH: OfficeDispatchProfile = OfficeDispatchProfile {
    product: OfficeProductKind::Sheets,
    id_prefix: "sheet",
    schema_version: "tench.sheets.v1",
    fallback_title: "Untitled Workbook",
    default_format: OfficeFileFormat::Xlsx,
    native_extensions: &["tenchsheet", "json"],
    kind: OfficeDispatchKind::Sheets,
};

pub const SLIDES_DISPATCH: OfficeDispatchProfile = OfficeDispatchProfile {
    product: OfficeProductKind::Slides,
    id_prefix: "slide",
    schema_version: "tench.slides.v1",
    fallback_title: "Untitled Presentation",
    default_format: OfficeFileFormat::Pptx,
    native_extensions: &[],
    kind: OfficeDispatchKind::Slides,
};

impl OfficeDispatchProfile {
    pub fn new_artifact(
        self,
        title: String,
        format: OfficeFileFormat,
        path: Option<String>,
    ) -> OfficeArtifact {
        office_file::build_office_artifact(
            self.product,
            self.id_prefix,
            self.schema_version,
            title,
            format,
            path,
        )
    }

    pub fn title_from_path(self, path: &Path) -> String {
        office_file::title_from_path(path, self.fallback_title)
    }

    pub fn normalized_title(self, title: Option<String>) -> String {
        office_file::normalize_title(title, self.fallback_title)
    }

    pub fn is_native_path(self, path: &Path) -> bool {
        !self.native_extensions.is_empty()
            && office_file::is_native_extension(path, self.native_extensions)
    }

    pub fn preview_text(self, content: &OfficeContent) -> Option<String> {
        let text = match self.kind {
            OfficeDispatchKind::Docs | OfficeDispatchKind::Kodocs => docs::preview_text(content),
            OfficeDispatchKind::Sheets => sheets::preview_text(content),
            OfficeDispatchKind::Slides => slides::preview_text(content),
        };
        let preview = text.trim().chars().take(180).collect::<String>();
        (!preview.is_empty()).then_some(preview)
    }

    pub fn serialize_for_target(
        self,
        artifact: &OfficeArtifact,
        content: &OfficeContent,
        target: &Path,
        format: Option<OfficeFileFormat>,
    ) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
        match self.kind {
            OfficeDispatchKind::Docs | OfficeDispatchKind::Kodocs | OfficeDispatchKind::Slides => {
                self.export_content_bytes(content, format.unwrap_or(self.default_format))
            }
            OfficeDispatchKind::Sheets => {
                if self.is_native_path(target) || format.is_none() {
                    let saved = office_file::SavedOfficeFile {
                        artifact: artifact.clone(),
                        content: content.clone(),
                    };
                    return serde_json::to_vec_pretty(&saved)
                        .map(|bytes| (bytes, Vec::new()))
                        .map_err(|e| format!("Failed to serialize Tench Sheets file: {e}"));
                }
                self.export_content_bytes(content, format.expect("checked above"))
            }
        }
    }

    pub fn import_content(
        self,
        raw: &str,
        format: OfficeFileFormat,
    ) -> Result<(OfficeContent, Vec<ImportExportDiagnostic>), String> {
        match self.kind {
            OfficeDispatchKind::Docs => docs::import_text(raw, format, true),
            OfficeDispatchKind::Kodocs => docs::import_text(raw, format, false),
            OfficeDispatchKind::Sheets | OfficeDispatchKind::Slides => Err(format!(
                "{} is not a text import format for this product.",
                format.extension()
            )),
        }
    }

    pub fn import_binary(
        self,
        path: &Path,
        format: OfficeFileFormat,
    ) -> Result<(OfficeContent, Vec<ImportExportDiagnostic>), String> {
        match self.kind {
            OfficeDispatchKind::Docs => docs::import_binary(path, format, false),
            OfficeDispatchKind::Kodocs => docs::import_binary(path, format, true),
            OfficeDispatchKind::Sheets => sheets::import_binary(path, format),
            OfficeDispatchKind::Slides => slides::import_binary(path, format),
        }
    }

    pub fn export_content_bytes(
        self,
        content: &OfficeContent,
        format: OfficeFileFormat,
    ) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
        match self.kind {
            OfficeDispatchKind::Docs => docs::export_content(content, format, false),
            OfficeDispatchKind::Kodocs => docs::export_content(content, format, true),
            OfficeDispatchKind::Sheets => sheets::export_content(content, format),
            OfficeDispatchKind::Slides => slides::export_content(content, format),
        }
    }
}

pub fn serialize_docs_for_target(
    artifact: &OfficeArtifact,
    content: &OfficeContent,
    target: &Path,
    format: Option<OfficeFileFormat>,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    DOCS_DISPATCH.serialize_for_target(artifact, content, target, format)
}

pub fn export_docs_content_bytes(
    content: &OfficeContent,
    format: OfficeFileFormat,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    DOCS_DISPATCH.export_content_bytes(content, format)
}

pub fn serialize_kodocs_for_target(
    artifact: &OfficeArtifact,
    content: &OfficeContent,
    target: &Path,
    format: Option<OfficeFileFormat>,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    KODOCS_DISPATCH.serialize_for_target(artifact, content, target, format)
}

pub fn export_kodocs_content_bytes(
    content: &OfficeContent,
    format: OfficeFileFormat,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    KODOCS_DISPATCH.export_content_bytes(content, format)
}

pub fn serialize_sheets_for_target(
    artifact: &OfficeArtifact,
    content: &OfficeContent,
    target: &Path,
    format: Option<OfficeFileFormat>,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    SHEETS_DISPATCH.serialize_for_target(artifact, content, target, format)
}

pub fn export_sheets_content_bytes(
    content: &OfficeContent,
    format: OfficeFileFormat,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    SHEETS_DISPATCH.export_content_bytes(content, format)
}

pub fn serialize_slides_for_target(
    artifact: &OfficeArtifact,
    content: &OfficeContent,
    target: &Path,
    format: Option<OfficeFileFormat>,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    SLIDES_DISPATCH.serialize_for_target(artifact, content, target, format)
}

pub fn export_slides_content_bytes(
    content: &OfficeContent,
    format: OfficeFileFormat,
) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String> {
    SLIDES_DISPATCH.export_content_bytes(content, format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_preserve_default_titles_and_formats() {
        assert_eq!(DOCS_DISPATCH.normalized_title(None), "Untitled Document");
        assert_eq!(
            KODOCS_DISPATCH
                .new_artifact("Doc".to_string(), OfficeFileFormat::Hwpx, None)
                .format,
            OfficeFileFormat::Hwpx
        );
        assert!(SHEETS_DISPATCH.is_native_path(Path::new("budget.TENCHSHEET")));
        assert_eq!(
            SLIDES_DISPATCH.title_from_path(Path::new("/tmp/talk.pptx")),
            "talk"
        );
    }

    #[test]
    fn docs_and_kodocs_keep_product_specific_error_messages() {
        let docs = DOCS_DISPATCH
            .import_content("", OfficeFileFormat::Xlsx)
            .expect_err("docs rejects xlsx text import");
        let kodocs = KODOCS_DISPATCH
            .import_content("", OfficeFileFormat::Xlsx)
            .expect_err("kodocs rejects xlsx text import");

        assert!(docs.contains("and RTF"));
        assert!(!kodocs.contains("and RTF"));
    }
}
