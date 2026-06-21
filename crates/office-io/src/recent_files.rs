use crate::error::OfficeIoError;
use std::fs;
use std::path::Path;

use tench_document_core::{OfficeArtifact, OfficeRecentFile};
use tench_storage_core::{office_storage_dir, OfficeStorageArea};

use crate::file_util::{timestamp_string, write_atomic};

/// Default maximum number of recent files to track per product.
pub const DEFAULT_MAX_RECENT_FILES: usize = 20;

/// Add an artifact to the recent files list for the given product.
///
/// If the artifact has no path, this is a no-op. Deduplicates by path,
/// moves the entry to the front, and truncates to the maximum count.
pub fn add_recent_file(product_id: &str, artifact: &OfficeArtifact) -> Result<(), OfficeIoError> {
    if artifact.path.is_none() {
        return Ok(());
    }

    let mut recent = load_recent_files(product_id)?;
    recent.retain(|item| item.artifact.path != artifact.path);
    recent.insert(
        0,
        OfficeRecentFile {
            artifact: artifact.clone(),
            opened_at: timestamp_string(),
            exists: artifact
                .path
                .as_ref()
                .is_some_and(|path| Path::new(path).exists()),
            thumbnail: None,
        },
    );
    recent.truncate(DEFAULT_MAX_RECENT_FILES);
    save_recent_files(product_id, &recent)
}

/// Load the recent files list for the given product.
pub fn load_recent_files(product_id: &str) -> Result<Vec<OfficeRecentFile>, OfficeIoError> {
    let path = recent_files_path(product_id);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&path).map_err(|error| OfficeIoError::Io {
        context: "Failed to read recent files".to_string(),
        source: error,
    })?;
    serde_json::from_str(&raw).map_err(|error| OfficeIoError::Parse {
        context: "Failed to parse recent files".to_string(),
        source: Box::new(error),
    })
}

/// Get recent documents with existence checks.
pub fn get_recent_documents(product_id: &str) -> Result<Vec<OfficeRecentFile>, OfficeIoError> {
    let mut recent = load_recent_files(product_id)?;
    for item in &mut recent {
        item.exists = item
            .artifact
            .path
            .as_ref()
            .is_some_and(|path| Path::new(path).exists());
    }
    Ok(recent)
}

/// Save the recent files list for the given product.
fn save_recent_files(product_id: &str, recent: &[OfficeRecentFile]) -> Result<(), OfficeIoError> {
    let path = recent_files_path(product_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| OfficeIoError::Io {
            context: "Failed to create recent files directory".to_string(),
            source: error,
        })?;
    }
    let raw = serde_json::to_vec_pretty(recent).map_err(|error| OfficeIoError::Serialize {
        context: "Failed to serialize recent files".to_string(),
        source: Box::new(error),
    })?;
    write_atomic(&path, &raw)
}

/// Get the path to the recent files JSON for the given product.
pub fn recent_files_path(product_id: &str) -> std::path::PathBuf {
    office_storage_dir(product_id, OfficeStorageArea::RecentFiles).join("recent.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_util::timestamp_millis;
    use tench_document_core::{OfficeFileFormat, OfficeProductKind};

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        crate::test_util::env_guard()
    }

    fn test_artifact(title: &str, path: Option<String>) -> OfficeArtifact {
        OfficeArtifact {
            id: format!("doc_{}", timestamp_millis()),
            title: title.to_string(),
            product: OfficeProductKind::Docs,
            format: OfficeFileFormat::Docx,
            path,
            schema_version: "test".to_string(),
            created_at: None,
            updated_at: None,
            dirty: false,
            tags: vec![],
            assets: vec![],
        }
    }

    fn isolated_storage(name: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = env_guard();
        let dir = std::env::temp_dir().join(format!(
            "tench_office_io_recent_{name}_{}_{}",
            std::process::id(),
            timestamp_millis()
        ));
        crate::test_util::set_test_data_home(&dir);
        std::fs::create_dir_all(&dir).expect("test dir");
        guard
    }

    #[test]
    fn add_and_load_recent_files() {
        let _guard = isolated_storage("add_load");
        let product_id = "tench-office-io-test";
        let file_dir = std::env::temp_dir().join(format!(
            "tench_office_io_recent_files_{}_{}",
            std::process::id(),
            timestamp_millis()
        ));
        std::fs::create_dir_all(&file_dir).expect("test dir");
        let file_path = file_dir.join("doc.docx");
        std::fs::write(&file_path, b"").expect("write");

        let artifact = test_artifact(
            "Test Doc",
            Some(crate::file_util::path_to_string(&file_path)),
        );
        add_recent_file(product_id, &artifact).expect("add recent");

        let recent = load_recent_files(product_id).expect("load recent");
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].artifact.title, "Test Doc");
    }

    #[test]
    fn add_recent_deduplicates_by_path() {
        let _guard = isolated_storage("dedup");
        let product_id = "tench-office-io-test";
        let file_dir = std::env::temp_dir().join(format!(
            "tench_office_io_recent_dedup_{}_{}",
            std::process::id(),
            timestamp_millis()
        ));
        std::fs::create_dir_all(&file_dir).expect("test dir");
        let file_path = file_dir.join("doc.docx");
        std::fs::write(&file_path, b"").expect("write");
        let path_str = crate::file_util::path_to_string(&file_path);

        let artifact1 = test_artifact("First", Some(path_str.clone()));
        add_recent_file(product_id, &artifact1).expect("add first");
        let artifact2 = test_artifact("Second", Some(path_str));
        add_recent_file(product_id, &artifact2).expect("add second");

        let recent = load_recent_files(product_id).expect("load");
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].artifact.title, "Second");
    }
}
