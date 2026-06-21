use crate::error::OfficeIoError;
use std::fs;
use std::path::Path;

use tench_document_core::OfficeBackupMetadata;
use tench_storage_core::{office_storage_dir, OfficeStorageArea};

use crate::file_util::{
    compute_checksum, path_to_string, sanitize_file_name, timestamp_millis, timestamp_string,
};

/// Create a backup copy of `original` for the given `artifact` under the
/// product-specific backup directory.
///
/// `product_id` identifies which office app (e.g. `"tench-docs"`) owns the backup.
pub fn create_backup(
    product_id: &str,
    artifact: &tench_document_core::OfficeArtifact,
    original: &Path,
) -> Result<OfficeBackupMetadata, OfficeIoError> {
    let backup_dir = office_storage_dir(product_id, OfficeStorageArea::Backups);
    fs::create_dir_all(&backup_dir).map_err(|error| OfficeIoError::Io {
        context: "Failed to create backup directory".to_string(),
        source: error,
    })?;

    let file_name = original
        .file_name()
        .and_then(|value| value.to_str())
        .map(sanitize_file_name)
        .unwrap_or_else(|| "document".to_string());
    let id = format!("backup_{}", timestamp_millis());
    let backup_path = backup_dir.join(format!("{id}_{file_name}"));
    fs::copy(original, &backup_path).map_err(|error| OfficeIoError::Io {
        context: format!("Failed to create backup {}", backup_path.display()),
        source: error,
    })?;

    let backup_bytes = fs::read(&backup_path).map_err(|error| OfficeIoError::Io {
        context: format!(
            "Failed to read backup for checksum {}",
            backup_path.display()
        ),
        source: error,
    })?;
    let checksum = Some(compute_checksum(&backup_bytes));

    Ok(OfficeBackupMetadata {
        id,
        artifact_id: artifact.id.clone(),
        original_path: path_to_string(original),
        backup_path: path_to_string(&backup_path),
        created_at: timestamp_string(),
        checksum,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_util::{timestamp_millis, write_atomic};
    use tench_document_core::{OfficeArtifact, OfficeFileFormat, OfficeProductKind};

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        crate::test_util::env_guard()
    }

    fn test_artifact(title: &str) -> OfficeArtifact {
        OfficeArtifact {
            id: format!("doc_{}", timestamp_millis()),
            title: title.to_string(),
            product: OfficeProductKind::Docs,
            format: OfficeFileFormat::Docx,
            path: None,
            schema_version: "test".to_string(),
            created_at: None,
            updated_at: None,
            dirty: false,
            tags: vec![],
            assets: vec![],
        }
    }

    #[test]
    fn create_backup_copies_file_to_backup_dir() {
        let _guard = env_guard();
        let dir = std::env::temp_dir().join(format!(
            "tench_office_io_backup_{}_{}",
            std::process::id(),
            timestamp_millis()
        ));
        std::fs::create_dir_all(&dir).expect("test dir");
        crate::test_util::set_test_data_home(&dir);

        let original = dir.join("report.docx");
        write_atomic(&original, b"doc content").expect("write original");

        let artifact = test_artifact("Report");
        let backup = create_backup("tench-docs-test", &artifact, &original).expect("create backup");

        assert!(Path::new(&backup.backup_path).exists());
        assert_eq!(
            std::fs::read_to_string(&backup.backup_path).unwrap(),
            "doc content"
        );

        let _ = std::fs::remove_dir_all(dir);
    }
}
