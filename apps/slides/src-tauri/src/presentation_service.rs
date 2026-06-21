use std::path::PathBuf;

use tench_document_core::{
    detect_office_file_format, OfficeArtifact, OfficeContent, OfficeExportResponse,
    OfficeFileFormat, OfficeOpenResponse, OfficeRecentFile, OfficeRecoveryMetadata,
    OfficeSaveResponse,
};
use tench_office_io::{file_util, office_file, recent_files, recovery};
use tench_office_runtime::{
    export_slides_content_bytes, serialize_slides_for_target, SLIDES_DISPATCH,
};

use tench_office_io::slides::format as format_io;

const PRODUCT_ID: &str = "tench-slides";

pub fn create_presentation(title: Option<String>) -> OfficeOpenResponse {
    let title = SLIDES_DISPATCH.normalized_title(title);
    let artifact = SLIDES_DISPATCH.new_artifact(title, OfficeFileFormat::Pptx, None);
    let content = format_io::empty_presentation_content(&artifact.title);

    OfficeOpenResponse {
        artifact,
        content,
        diagnostics: Vec::new(),
    }
}

pub fn open_presentation(path: String) -> Result<OfficeOpenResponse, String> {
    import_presentation(file_util::path_to_string(&PathBuf::from(&path)), None)
}

pub fn import_presentation(
    source_path: String,
    source_format: Option<OfficeFileFormat>,
) -> Result<OfficeOpenResponse, String> {
    let path = PathBuf::from(&source_path);
    file_util::ensure_file_exists(&path)?;

    let format = source_format
        .or_else(|| detect_office_file_format(&source_path))
        .ok_or_else(|| format!("Unsupported presentation format: {}", path.display()))?;

    let (content, diagnostics) = SLIDES_DISPATCH.import_binary(&path, format)?;

    let mut artifact = SLIDES_DISPATCH.new_artifact(
        SLIDES_DISPATCH.title_from_path(&path),
        format,
        Some(file_util::path_to_string(&path)),
    );
    artifact.updated_at = file_util::modified_at_unix(&path);
    recent_files::add_recent_file(PRODUCT_ID, &artifact)?;

    Ok(OfficeOpenResponse {
        artifact,
        content,
        diagnostics,
    })
}

pub fn save_presentation(
    artifact: OfficeArtifact,
    content: OfficeContent,
    target_path: Option<String>,
    format: Option<OfficeFileFormat>,
) -> Result<OfficeSaveResponse, String> {
    office_file::save_office_file(
        PRODUCT_ID,
        artifact,
        &content,
        target_path,
        format,
        serialize_slides_for_target,
    )
}

pub fn export_presentation(
    artifact_id: String,
    content: OfficeContent,
    target_format: OfficeFileFormat,
    output_path: String,
) -> Result<OfficeExportResponse, String> {
    office_file::export_office_file(
        artifact_id,
        &content,
        target_format,
        output_path,
        "exported_from_editor",
        export_slides_content_bytes,
    )
}

pub fn get_recent_presentations() -> Result<Vec<OfficeRecentFile>, String> {
    recent_files::get_recent_documents(PRODUCT_ID).map_err(|e| e.to_string())
}

pub fn save_recovery_snapshot(
    artifact: OfficeArtifact,
    content: OfficeContent,
) -> Result<OfficeRecoveryMetadata, String> {
    let preview = SLIDES_DISPATCH.preview_text(&content);
    office_file::save_recovery_snapshot(PRODUCT_ID, artifact, content, preview)
}

pub fn get_recovery_presentations() -> Result<Vec<OfficeRecoveryMetadata>, String> {
    recovery::get_recovery_documents(PRODUCT_ID).map_err(|e| e.to_string())
}

pub fn open_recovery_presentation(recovery_path: String) -> Result<OfficeOpenResponse, String> {
    office_file::open_recovery_as_unsaved(recovery_path)
}

pub fn delete_recovery_presentation(recovery_path: String) -> Result<(), String> {
    recovery::delete_recovery_document(PRODUCT_ID, recovery_path).map_err(|e| e.to_string())
}

pub fn clear_recovery_snapshots(artifact_id: String) -> Result<(), String> {
    recovery::clear_recovery_snapshots(PRODUCT_ID, &artifact_id).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn isolated_storage(name: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::test_util::env_guard();
        let dir = std::env::temp_dir().join(format!(
            "tench_slides_test_{name}_{}_{}",
            std::process::id(),
            file_util::timestamp_millis()
        ));
        std::fs::create_dir_all(&dir).expect("test temp dir");
        crate::test_util::set_test_data_home(&dir);
        guard
    }

    #[test]
    fn create_presentation_has_one_slide() {
        let pres = create_presentation(Some("Test".to_string()));
        assert_eq!(pres.artifact.title, "Test");
        assert!(matches!(pres.content, OfficeContent::Slides(_)));
    }

    #[test]
    fn pptx_export_and_import_round_trips() {
        let _guard = isolated_storage("pptx_rt");
        let dir = std::env::temp_dir().join(format!(
            "tench_pptx_rt_files_{}_{}",
            std::process::id(),
            file_util::timestamp_millis()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("test.pptx");

        let created = create_presentation(Some("PPTX Test".to_string()));
        export_presentation(
            created.artifact.id.clone(),
            created.content,
            OfficeFileFormat::Pptx,
            file_util::path_to_string(&target),
        )
        .expect("export pptx");

        let imported =
            import_presentation(file_util::path_to_string(&target), None).expect("import");
        assert_eq!(imported.artifact.format, OfficeFileFormat::Pptx);

        let _ = std::fs::remove_dir_all(dir);
    }
}
