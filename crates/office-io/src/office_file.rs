use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tench_document_core::{
    detect_office_file_format, DiagnosticSeverity, ImportExportDiagnostic, OfficeArtifact,
    OfficeAssetRef, OfficeContent, OfficeExportResponse, OfficeFileFormat, OfficeOpenResponse,
    OfficeProductKind, OfficeRecoveryMetadata, OfficeSaveResponse,
};

use crate::{backup, diagnostic, file_util, recent_files, recovery};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedOfficeFile {
    pub artifact: OfficeArtifact,
    pub content: OfficeContent,
}

pub fn build_office_artifact(
    product: OfficeProductKind,
    id_prefix: &str,
    schema_version: &str,
    title: String,
    format: OfficeFileFormat,
    path: Option<String>,
) -> OfficeArtifact {
    let now = file_util::timestamp_string();
    OfficeArtifact {
        id: format!("{}_{}", id_prefix, file_util::timestamp_millis()),
        title,
        product,
        format,
        path,
        schema_version: schema_version.to_string(),
        created_at: now.clone(),
        updated_at: now,
        dirty: false,
        tags: Vec::new(),
        assets: Vec::<OfficeAssetRef>::new(),
    }
}

pub fn normalize_title(title: Option<String>, fallback: &str) -> String {
    title
        .and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .unwrap_or_else(|| fallback.to_string())
}

pub fn title_from_path(path: &Path, fallback: &str) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|| fallback.to_string())
}

pub fn is_native_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .is_some_and(|extension| extensions.iter().any(|candidate| *candidate == extension))
}

pub fn save_office_file<F>(
    product_id: &str,
    mut artifact: OfficeArtifact,
    content: &OfficeContent,
    target_path: Option<String>,
    format: Option<OfficeFileFormat>,
    serialize: F,
) -> Result<OfficeSaveResponse, String>
where
    F: FnOnce(
        &OfficeArtifact,
        &OfficeContent,
        &Path,
        Option<OfficeFileFormat>,
    ) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String>,
{
    let target_path = target_path
        .or_else(|| artifact.path.clone())
        .ok_or_else(|| "A target path is required for the first save.".to_string())?;
    let target = PathBuf::from(&target_path);
    let target_format = format.or_else(|| detect_office_file_format(&target_path));

    let backup_result = if target.exists() {
        Some(backup::create_backup(product_id, &artifact, &target)?)
    } else {
        None
    };

    artifact.path = Some(file_util::path_to_string(&target));
    artifact.updated_at = file_util::timestamp_string();
    artifact.dirty = false;
    if let Some(format) = target_format {
        artifact.format = format;
    }

    let (bytes, diagnostics) = serialize(&artifact, content, &target, target_format)?;
    file_util::write_atomic(&target, &bytes)?;
    recent_files::add_recent_file(product_id, &artifact)?;

    Ok(OfficeSaveResponse {
        artifact,
        backup: backup_result,
        diagnostics,
    })
}

pub fn export_office_file<F>(
    artifact_id: String,
    content: &OfficeContent,
    target_format: OfficeFileFormat,
    output_path: String,
    diagnostic_code: &str,
    export: F,
) -> Result<OfficeExportResponse, String>
where
    F: FnOnce(
        &OfficeContent,
        OfficeFileFormat,
    ) -> Result<(Vec<u8>, Vec<ImportExportDiagnostic>), String>,
{
    let output = PathBuf::from(&output_path);
    let (bytes, mut diagnostics) = export(content, target_format)?;
    file_util::write_atomic(&output, &bytes)?;
    diagnostics.push(diagnostic::diagnostic(
        DiagnosticSeverity::Info,
        diagnostic_code,
        &format!("Exported {artifact_id} from the current editor state."),
        true,
    ));

    Ok(OfficeExportResponse {
        output_path,
        format: target_format,
        diagnostics,
    })
}

pub fn save_recovery_snapshot(
    product_id: &str,
    artifact: OfficeArtifact,
    content: OfficeContent,
    preview_text: Option<String>,
) -> Result<OfficeRecoveryMetadata, String> {
    recovery::save_recovery_snapshot(product_id, artifact, content, preview_text)
        .map_err(|error| error.to_string())
}

pub fn open_recovery_as_unsaved(recovery_path: String) -> Result<OfficeOpenResponse, String> {
    let snapshot = recovery::open_recovery_document(&recovery_path)?;
    let mut artifact = snapshot.artifact;
    artifact.path = None;
    artifact.title = format!("Recovered - {}", artifact.title);
    artifact.dirty = true;

    Ok(OfficeOpenResponse {
        artifact,
        content: snapshot.content,
        diagnostics: vec![diagnostic::diagnostic(
            DiagnosticSeverity::Info,
            "opened_recovery_snapshot",
            "Opened a recovery copy. Save it to choose a final file path.",
            true,
        )],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tench_document_core::{OfficeProductKind, RichDocumentContent};

    #[test]
    fn title_helpers_trim_or_fallback() {
        assert_eq!(
            normalize_title(Some("  Budget  ".to_string()), "Untitled"),
            "Budget"
        );
        assert_eq!(
            normalize_title(Some(" ".to_string()), "Untitled"),
            "Untitled"
        );
        assert_eq!(
            title_from_path(Path::new("/tmp/report.docx"), "Untitled"),
            "report"
        );
        assert_eq!(title_from_path(Path::new("/"), "Untitled"), "Untitled");
    }

    #[test]
    fn native_extension_matches_case_insensitively() {
        assert!(is_native_extension(
            Path::new("/tmp/report.TENCHDOC"),
            &["tenchdoc", "json"]
        ));
        assert!(!is_native_extension(
            Path::new("/tmp/report.docx"),
            &["tenchdoc", "json"]
        ));
    }

    #[test]
    fn build_artifact_sets_product_metadata() {
        let artifact = build_office_artifact(
            OfficeProductKind::Docs,
            "doc",
            "tench.docs.v1",
            "Draft".to_string(),
            OfficeFileFormat::Docx,
            None,
        );

        assert!(artifact.id.starts_with("doc_"));
        assert_eq!(artifact.product, OfficeProductKind::Docs);
        assert_eq!(artifact.schema_version, "tench.docs.v1");
        assert!(!artifact.dirty);
    }

    #[test]
    fn export_office_file_writes_bytes_and_appends_diagnostic() {
        let dir = std::env::temp_dir().join(format!(
            "tench_office_file_export_{}_{}",
            std::process::id(),
            file_util::timestamp_millis()
        ));
        std::fs::create_dir_all(&dir).expect("test dir");
        let target = dir.join("export.txt");
        let content = OfficeContent::Docs(RichDocumentContent {
            schema: "tench.docs.v1".to_string(),
            document: None,
        });

        let result = export_office_file(
            "doc_1".to_string(),
            &content,
            OfficeFileFormat::Txt,
            file_util::path_to_string(&target),
            "exported_from_test",
            |_content, _format| Ok((b"hello".to_vec(), Vec::new())),
        )
        .expect("export");

        assert_eq!(result.format, OfficeFileFormat::Txt);
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "hello");
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "exported_from_test"));

        let _ = std::fs::remove_dir_all(dir);
    }
}
