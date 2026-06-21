use crate::error::OfficeIoError;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tench_document_core::{OfficeArtifact, OfficeContent, OfficeRecoveryMetadata};
use tench_storage_core::{office_storage_dir, OfficeStorageArea};

use crate::file_util::{
    compute_checksum, ensure_file_exists, path_to_string, sanitize_file_name, timestamp_millis,
    timestamp_string, write_atomic,
};

/// Maximum number of recovery snapshots to keep per product.
pub const DEFAULT_MAX_RECOVERY_FILES: usize = 10;

/// Internal file format for persisting a recovery snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoverySnapshotFile {
    pub metadata: OfficeRecoveryMetadata,
    pub artifact: OfficeArtifact,
    pub content: OfficeContent,
}

/// Save a recovery snapshot for the given artifact and content.
///
/// `product_id` identifies which office app owns the snapshot.
/// `preview_text` is an optional short text preview stored in the metadata.
pub fn save_recovery_snapshot(
    product_id: &str,
    artifact: OfficeArtifact,
    content: OfficeContent,
    preview_text: Option<String>,
) -> Result<OfficeRecoveryMetadata, OfficeIoError> {
    let recovery_dir = office_storage_dir(product_id, OfficeStorageArea::Recovery);
    fs::create_dir_all(&recovery_dir).map_err(|error| OfficeIoError::Io {
        context: "Failed to create recovery directory".to_string(),
        source: error,
    })?;

    let id = format!("recovery_{}", timestamp_millis());
    let safe_title = sanitize_file_name(&artifact.title);
    let recovery_path = recovery_dir.join(format!("{id}_{safe_title}.recovery.json"));
    let original_modified_at = artifact
        .path
        .as_deref()
        .map(Path::new)
        .and_then(crate::file_util::modified_at_unix);

    let product = artifact.product;
    let mut metadata = OfficeRecoveryMetadata {
        id,
        artifact_id: Some(artifact.id.clone()),
        product,
        original_path: artifact.path.clone(),
        recovery_path: path_to_string(&recovery_path),
        saved_at: timestamp_string(),
        original_modified_at,
        schema_version: artifact.schema_version.clone(),
        checksum: None,
        preview_text,
        content_format: artifact.format,
    };
    let snapshot = RecoverySnapshotFile {
        metadata: metadata.clone(),
        artifact,
        content,
    };
    let raw = serde_json::to_vec_pretty(&snapshot).map_err(|error| OfficeIoError::Serialize {
        context: "Failed to serialize recovery snapshot".to_string(),
        source: Box::new(error),
    })?;
    metadata.checksum = Some(compute_checksum(&raw));
    write_atomic(&recovery_path, &raw)?;
    prune_recovery_files(product_id)?;

    Ok(metadata)
}

/// List all recovery metadata entries for the given product, sorted newest first.
pub fn get_recovery_documents(
    product_id: &str,
) -> Result<Vec<OfficeRecoveryMetadata>, OfficeIoError> {
    let mut snapshots = load_recovery_snapshots(product_id)?
        .into_iter()
        .map(|snapshot| snapshot.metadata)
        .collect::<Vec<_>>();
    snapshots.sort_by(|left, right| right.saved_at.cmp(&left.saved_at));
    Ok(snapshots)
}

/// Open a recovery snapshot and return the artifact + content ready for editing.
///
/// The artifact is marked dirty with a "Recovered - " title prefix and no path.
pub fn open_recovery_document(recovery_path: &str) -> Result<RecoverySnapshotFile, OfficeIoError> {
    let snapshot = read_recovery_snapshot(&PathBuf::from(recovery_path))?;
    Ok(snapshot)
}

/// Delete a single recovery snapshot by its file path.
pub fn delete_recovery_document(
    product_id: &str,
    recovery_path: String,
) -> Result<(), OfficeIoError> {
    let path = PathBuf::from(recovery_path);
    ensure_recovery_path(product_id, &path)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| OfficeIoError::Io {
            context: "Failed to delete recovery snapshot".to_string(),
            source: error,
        })?;
    }
    Ok(())
}

/// Clear all recovery snapshots for a given artifact ID.
pub fn clear_recovery_snapshots(product_id: &str, artifact_id: &str) -> Result<(), OfficeIoError> {
    for snapshot in load_recovery_snapshots(product_id)? {
        if snapshot.metadata.artifact_id.as_deref() == Some(artifact_id) {
            delete_recovery_document(product_id, snapshot.metadata.recovery_path)?;
        }
    }
    Ok(())
}

/// Load all recovery snapshots for the given product.
fn load_recovery_snapshots(product_id: &str) -> Result<Vec<RecoverySnapshotFile>, OfficeIoError> {
    let recovery_dir = office_storage_dir(product_id, OfficeStorageArea::Recovery);
    if !recovery_dir.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();
    for entry in fs::read_dir(&recovery_dir).map_err(|error| OfficeIoError::Io {
        context: "Failed to read recovery directory".to_string(),
        source: error,
    })? {
        let entry = entry.map_err(|error| OfficeIoError::Io {
            context: "Failed to read recovery entry".to_string(),
            source: error,
        })?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            snapshots.push(read_recovery_snapshot(&path)?);
        }
    }
    Ok(snapshots)
}

/// Read a single recovery snapshot from disk.
///
/// Verifies the integrity checksum if one is present in the metadata.
fn read_recovery_snapshot(path: &Path) -> Result<RecoverySnapshotFile, OfficeIoError> {
    ensure_file_exists(path)?;
    let raw = fs::read_to_string(path).map_err(|error| OfficeIoError::Io {
        context: format!("Failed to read recovery snapshot {}", path.display()),
        source: error,
    })?;
    let snapshot: RecoverySnapshotFile =
        serde_json::from_str(&raw).map_err(|error| OfficeIoError::Parse {
            context: format!("Failed to parse recovery snapshot {}", path.display()),
            source: Box::new(error),
        })?;
    // Verify checksum if present
    if let Some(ref expected) = snapshot.metadata.checksum {
        let actual = compute_checksum(raw.as_bytes());
        if expected != &actual {
            return Err(OfficeIoError::General(format!(
                "Recovery snapshot checksum mismatch: expected {}, got {}",
                expected, actual
            )));
        }
    }
    Ok(snapshot)
}

/// Remove oldest snapshots exceeding the maximum count.
fn prune_recovery_files(product_id: &str) -> Result<(), OfficeIoError> {
    let mut snapshots = load_recovery_snapshots(product_id)?;
    snapshots.sort_by(|left, right| right.metadata.saved_at.cmp(&left.metadata.saved_at));
    for snapshot in snapshots.into_iter().skip(DEFAULT_MAX_RECOVERY_FILES) {
        delete_recovery_document(product_id, snapshot.metadata.recovery_path)?;
    }
    Ok(())
}

/// Verify that a recovery path is within the product's recovery directory.
fn ensure_recovery_path(product_id: &str, path: &Path) -> Result<(), OfficeIoError> {
    let recovery_dir = office_storage_dir(product_id, OfficeStorageArea::Recovery);
    let expected = recovery_dir
        .canonicalize()
        .unwrap_or_else(|_| recovery_dir.clone());
    let actual_parent = path
        .parent()
        .ok_or_else(|| {
            OfficeIoError::General("Recovery snapshot path has no parent directory.".to_string())
        })?
        .canonicalize()
        .unwrap_or_else(|_| path.parent().expect("checked parent").to_path_buf());

    if actual_parent == expected {
        Ok(())
    } else {
        Err(OfficeIoError::Permission(
            "Recovery snapshot path is outside the recovery directory.".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_util::timestamp_millis;
    use tench_document_core::RichDocumentContent;
    use tench_document_core::{OfficeFileFormat, OfficeProductKind};

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

    fn test_content() -> OfficeContent {
        OfficeContent::Docs(RichDocumentContent {
            schema: "tench.docs.v1".to_string(),
            document: None,
        })
    }

    fn isolated_storage(name: &str) -> (std::sync::MutexGuard<'static, ()>, PathBuf) {
        let guard = env_guard();
        let dir = std::env::temp_dir().join(format!(
            "tench_office_io_recovery_{name}_{}_{}",
            std::process::id(),
            timestamp_millis()
        ));
        crate::test_util::set_test_data_home(&dir);
        std::fs::create_dir_all(&dir).expect("test dir");
        (guard, dir)
    }

    #[test]
    fn recovery_snapshot_round_trip() {
        let (_guard, _dir) = isolated_storage("roundtrip");
        let product_id = format!("tench-office-io-test-{}", timestamp_millis());
        let artifact = test_artifact("Recoverable");
        let content = test_content();

        let snapshot = save_recovery_snapshot(
            &product_id,
            artifact.clone(),
            content.clone(),
            Some("test preview".to_string()),
        )
        .expect("save recovery");

        let snapshots = get_recovery_documents(&product_id).expect("list recovery");
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].id, snapshot.id);
        assert_eq!(snapshots[0].preview_text, Some("test preview".to_string()));

        let recovered = open_recovery_document(&snapshot.recovery_path).expect("open recovery");
        assert_eq!(recovered.artifact.title, "Recoverable");

        clear_recovery_snapshots(&product_id, &artifact.id).expect("clear recovery");
        assert!(get_recovery_documents(&product_id)
            .expect("list after clear")
            .is_empty());
    }
}
