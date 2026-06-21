use std::fs;
use std::path::{Path, PathBuf};

use tench_document_core::{
    detect_office_file_format, OfficeArtifact, OfficeContent, OfficeExportResponse,
    OfficeFileFormat, OfficeOpenResponse, OfficeRecentFile, OfficeRecoveryMetadata,
    OfficeSaveResponse,
};
use tench_office_io::{file_util, office_file, recent_files, recovery};
use tench_office_runtime::{
    export_sheets_content_bytes, serialize_sheets_for_target, SHEETS_DISPATCH,
};

use tench_office_io::sheets::format as format_io;

const PRODUCT_ID: &str = "tench-sheets";

pub fn create_workbook(title: Option<String>) -> OfficeOpenResponse {
    let title = SHEETS_DISPATCH.normalized_title(title);
    let artifact = SHEETS_DISPATCH.new_artifact(title, OfficeFileFormat::Xlsx, None);
    let content = format_io::empty_workbook_content(&artifact.title);

    OfficeOpenResponse {
        artifact,
        content,
        diagnostics: Vec::new(),
    }
}

pub fn open_workbook(path: String) -> Result<OfficeOpenResponse, String> {
    let path = PathBuf::from(&path);
    if SHEETS_DISPATCH.is_native_path(&path) {
        open_internal_workbook(&path)
    } else {
        import_workbook(file_util::path_to_string(&path), None)
    }
}

pub fn import_workbook(
    source_path: String,
    source_format: Option<OfficeFileFormat>,
) -> Result<OfficeOpenResponse, String> {
    let path = PathBuf::from(&source_path);
    file_util::ensure_file_exists(&path)?;

    let format = source_format
        .or_else(|| detect_office_file_format(&source_path))
        .ok_or_else(|| format!("Unsupported spreadsheet format: {}", path.display()))?;

    let (content, diagnostics) = SHEETS_DISPATCH.import_binary(&path, format)?;

    let mut artifact = SHEETS_DISPATCH.new_artifact(
        SHEETS_DISPATCH.title_from_path(&path),
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

pub fn save_workbook(
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
        serialize_sheets_for_target,
    )
}

pub fn export_workbook(
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
        export_sheets_content_bytes,
    )
}

pub fn get_recent_workbooks() -> Result<Vec<OfficeRecentFile>, String> {
    recent_files::get_recent_documents(PRODUCT_ID).map_err(|e| e.to_string())
}

pub fn save_recovery_snapshot(
    artifact: OfficeArtifact,
    content: OfficeContent,
) -> Result<OfficeRecoveryMetadata, String> {
    let preview = SHEETS_DISPATCH.preview_text(&content);
    office_file::save_recovery_snapshot(PRODUCT_ID, artifact, content, preview)
}

pub fn get_recovery_workbooks() -> Result<Vec<OfficeRecoveryMetadata>, String> {
    recovery::get_recovery_documents(PRODUCT_ID).map_err(|e| e.to_string())
}

pub fn open_recovery_workbook(recovery_path: String) -> Result<OfficeOpenResponse, String> {
    office_file::open_recovery_as_unsaved(recovery_path)
}

pub fn delete_recovery_workbook(recovery_path: String) -> Result<(), String> {
    recovery::delete_recovery_document(PRODUCT_ID, recovery_path).map_err(|e| e.to_string())
}

pub fn clear_recovery_snapshots(artifact_id: String) -> Result<(), String> {
    recovery::clear_recovery_snapshots(PRODUCT_ID, &artifact_id).map_err(|e| e.to_string())
}

fn open_internal_workbook(path: &Path) -> Result<OfficeOpenResponse, String> {
    file_util::ensure_file_exists(path)?;
    let raw =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let saved: office_file::SavedOfficeFile = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;

    recent_files::add_recent_file(PRODUCT_ID, &saved.artifact)?;
    Ok(OfficeOpenResponse {
        artifact: saved.artifact,
        content: saved.content,
        diagnostics: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn isolated_storage(name: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::test_util::env_guard();
        let dir = std::env::temp_dir().join(format!(
            "tench_sheets_test_{name}_{}_{}",
            std::process::id(),
            file_util::timestamp_millis()
        ));
        std::fs::create_dir_all(&dir).expect("test temp dir");
        crate::test_util::set_test_data_home(&dir);
        guard
    }

    #[test]
    fn create_workbook_has_one_sheet() {
        let wb = create_workbook(Some("Test".to_string()));
        assert_eq!(wb.artifact.title, "Test");
        assert!(matches!(wb.content, OfficeContent::Sheets(_)));
    }

    #[test]
    fn save_and_open_internal_round_trips() {
        let _guard = isolated_storage("internal");
        let dir = std::env::temp_dir().join(format!(
            "tench_sheets_internal_files_{}_{}",
            std::process::id(),
            file_util::timestamp_millis()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("test.tenchsheet");

        let created = create_workbook(Some("Roundtrip".to_string()));
        let saved = save_workbook(
            created.artifact,
            created.content,
            Some(file_util::path_to_string(&target)),
            None,
        )
        .expect("save");
        let reopened = open_workbook(file_util::path_to_string(&target)).expect("open");

        assert_eq!(reopened.artifact.title, "Roundtrip");
        assert!(!saved.artifact.dirty);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn xlsx_export_and_import_round_trips() {
        let _guard = isolated_storage("xlsx_rt");
        let dir = std::env::temp_dir().join(format!(
            "tench_xlsx_rt_files_{}_{}",
            std::process::id(),
            file_util::timestamp_millis()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("test.xlsx");

        let created = create_workbook(Some("XLSX Test".to_string()));
        export_workbook(
            created.artifact.id.clone(),
            created.content,
            OfficeFileFormat::Xlsx,
            file_util::path_to_string(&target),
        )
        .expect("export xlsx");

        let imported = import_workbook(file_util::path_to_string(&target), None).expect("import");
        assert_eq!(imported.artifact.format, OfficeFileFormat::Xlsx);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn csv_export_and_import_round_trips() {
        let _guard = isolated_storage("csv_rt");
        let dir = std::env::temp_dir().join(format!(
            "tench_csv_rt_files_{}_{}",
            std::process::id(),
            file_util::timestamp_millis()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("test.csv");

        let created = create_workbook(Some("CSV Test".to_string()));
        export_workbook(
            created.artifact.id.clone(),
            created.content,
            OfficeFileFormat::Csv,
            file_util::path_to_string(&target),
        )
        .expect("export csv");

        let imported = import_workbook(file_util::path_to_string(&target), None).expect("import");
        assert_eq!(imported.artifact.format, OfficeFileFormat::Csv);

        let _ = std::fs::remove_dir_all(dir);
    }
}
