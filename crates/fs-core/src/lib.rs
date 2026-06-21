use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileEntryKind {
    File,
    Directory,
    Symlink,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: String,
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub kind: FileEntryKind,
    pub size_bytes: Option<u64>,
    pub modified_at_unix: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FolderScanOptions {
    pub recursive: bool,
    pub include_hidden: bool,
    pub allowed_extensions: Vec<String>,
    pub max_depth: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilePermissionScope {
    SingleFile,
    Directory,
    RecursiveDirectory,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileAccessGrant {
    pub scope: FilePermissionScope,
    pub path: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileWatchEventKind {
    ExternalModified,
    Deleted,
    Moved,
    PermissionDenied,
    SelfSaved,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OfficeFileWatchEvent {
    pub product_id: String,
    pub path: String,
    pub kind: FileWatchEventKind,
    pub previous_path: Option<String>,
    pub marker: Option<String>,
    pub observed_at: Option<String>,
}

pub fn normalize_extension(extension: impl AsRef<str>) -> String {
    extension
        .as_ref()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

pub fn is_hidden_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|file_name| file_name.starts_with('.'))
}

pub fn file_entry_from_path(path: impl AsRef<Path>) -> std::io::Result<FileEntry> {
    let path = path.as_ref();
    let metadata = fs::symlink_metadata(path)?;
    let kind = if metadata.file_type().is_file() {
        FileEntryKind::File
    } else if metadata.file_type().is_dir() {
        FileEntryKind::Directory
    } else if metadata.file_type().is_symlink() {
        FileEntryKind::Symlink
    } else {
        FileEntryKind::Unknown
    };
    let display_path = path.to_string_lossy().to_string();
    let canonical = path
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(path))
        .to_string_lossy()
        .to_string();
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(normalize_extension);
    let size_bytes = metadata.is_file().then_some(metadata.len());
    let modified_at_unix = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());

    Ok(FileEntry {
        id: canonical,
        path: display_path,
        name,
        extension,
        kind,
        size_bytes,
        modified_at_unix,
    })
}

pub fn scan_folder(
    root: impl AsRef<Path>,
    options: &FolderScanOptions,
) -> std::io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    scan_folder_inner(root.as_ref(), options, 0, &mut entries)?;
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

fn scan_folder_inner(
    dir: &Path,
    options: &FolderScanOptions,
    depth: u8,
    entries: &mut Vec<FileEntry>,
) -> std::io::Result<()> {
    if options.max_depth.is_some_and(|max_depth| depth > max_depth) {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !options.include_hidden && is_hidden_path(&path) {
            continue;
        }

        let file_entry = file_entry_from_path(&path)?;
        let is_allowed = extension_allowed(file_entry.extension.as_deref(), options);
        let is_dir = file_entry.kind == FileEntryKind::Directory;
        if is_allowed {
            entries.push(file_entry);
        }

        if options.recursive && is_dir {
            scan_folder_inner(&path, options, depth.saturating_add(1), entries)?;
        }
    }

    Ok(())
}

fn extension_allowed(extension: Option<&str>, options: &FolderScanOptions) -> bool {
    if options.allowed_extensions.is_empty() {
        return true;
    }
    let Some(extension) = extension else {
        return false;
    };
    options
        .allowed_extensions
        .iter()
        .map(normalize_extension)
        .any(|allowed| allowed == extension)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn office_file_watch_event_serializes_kind() {
        let event = OfficeFileWatchEvent {
            product_id: "tench-docs".to_string(),
            path: "/tmp/report.docx".to_string(),
            kind: FileWatchEventKind::ExternalModified,
            previous_path: None,
            marker: Some("save_1".to_string()),
            observed_at: None,
        };
        let value = serde_json::to_value(event).expect("watch event json");

        assert_eq!(value["product_id"], "tench-docs");
        assert_eq!(value["kind"], "external_modified");
        assert_eq!(value["marker"], "save_1");
    }

    #[test]
    fn normalize_extension_accepts_leading_dot() {
        assert_eq!(normalize_extension(".DOCX"), "docx");
    }

    // Edge case tests
    #[test]
    fn normalize_extension_empty_string() {
        assert_eq!(normalize_extension(""), "");
    }

    #[test]
    fn normalize_extension_whitespace_only() {
        assert_eq!(normalize_extension("   "), "   ");
    }

    #[test]
    fn normalize_extension_multiple_dots() {
        assert_eq!(normalize_extension(".tar.gz"), "tar.gz");
    }

    #[test]
    fn normalize_extension_unicode() {
        // to_ascii_lowercase does not lowercase Cyrillic; it returns the original string
        assert_eq!(normalize_extension(".ДОК"), "ДОК");
    }

    #[test]
    fn is_hidden_path_empty_path() {
        assert!(!is_hidden_path(Path::new("")));
    }

    #[test]
    fn is_hidden_path_root() {
        assert!(!is_hidden_path(Path::new("/")));
    }

    #[test]
    fn is_hidden_path_dotfile() {
        assert!(is_hidden_path(Path::new(".gitignore")));
    }

    #[test]
    fn is_hidden_path_not_hidden() {
        assert!(!is_hidden_path(Path::new("document.docx")));
    }

    #[test]
    fn file_entry_from_path_preserves_unicode_name_and_extension() {
        let root = temp_test_dir("file-entry-unicode");
        let file = root.join("심장 구조.PDF");
        fs::write(&file, b"%PDF-1.7").expect("write pdf");

        let entry = file_entry_from_path(&file).expect("entry");

        assert_eq!(entry.name, "심장 구조.PDF");
        assert_eq!(entry.extension.as_deref(), Some("pdf"));
        assert_eq!(entry.kind, FileEntryKind::File);
        assert_eq!(entry.size_bytes, Some(8));

        fs::remove_dir_all(root).expect("remove temp dir");
    }

    #[test]
    fn scan_folder_filters_extension_hidden_and_depth() {
        let root = temp_test_dir("scan-folder");
        fs::write(root.join("a.pdf"), b"a").expect("write a");
        fs::write(root.join(".hidden.pdf"), b"h").expect("write hidden");
        fs::write(root.join("notes.txt"), b"n").expect("write txt");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("nested dir");
        fs::write(nested.join("b.pdf"), b"b").expect("write b");

        let entries = scan_folder(
            &root,
            &FolderScanOptions {
                recursive: true,
                include_hidden: false,
                allowed_extensions: vec!["pdf".to_string()],
                max_depth: Some(1),
            },
        )
        .expect("scan");

        let names = entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["a.pdf", "b.pdf"]);

        fs::remove_dir_all(root).expect("remove temp dir");
    }

    #[test]
    fn all_file_entry_kind_variants_roundtrip() {
        for variant in [
            FileEntryKind::File,
            FileEntryKind::Directory,
            FileEntryKind::Symlink,
            FileEntryKind::Unknown,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: FileEntryKind = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_file_permission_scope_variants_roundtrip() {
        for variant in [
            FilePermissionScope::SingleFile,
            FilePermissionScope::Directory,
            FilePermissionScope::RecursiveDirectory,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: FilePermissionScope = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_file_watch_event_kind_variants_roundtrip() {
        for variant in [
            FileWatchEventKind::ExternalModified,
            FileWatchEventKind::Deleted,
            FileWatchEventKind::Moved,
            FileWatchEventKind::PermissionDenied,
            FileWatchEventKind::SelfSaved,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: FileWatchEventKind = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn file_entry_none_fields() {
        let entry = FileEntry {
            id: String::new(),
            path: String::new(),
            name: String::new(),
            extension: None,
            kind: FileEntryKind::Unknown,
            size_bytes: None,
            modified_at_unix: None,
        };
        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: FileEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn file_entry_zero_size() {
        let entry = FileEntry {
            id: String::new(),
            path: String::new(),
            name: String::new(),
            extension: Some(String::new()),
            kind: FileEntryKind::File,
            size_bytes: Some(0),
            modified_at_unix: Some(0),
        };
        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: FileEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn file_entry_very_large_size() {
        let entry = FileEntry {
            id: String::new(),
            path: String::new(),
            name: String::new(),
            extension: Some(String::new()),
            kind: FileEntryKind::File,
            size_bytes: Some(u64::MAX),
            modified_at_unix: Some(u64::MAX),
        };
        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: FileEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn folder_scan_options_empty_extensions() {
        let opts = FolderScanOptions {
            recursive: false,
            include_hidden: false,
            allowed_extensions: vec![],
            max_depth: None,
        };
        let serialized = serde_json::to_string(&opts).unwrap();
        let deserialized: FolderScanOptions = serde_json::from_str(&serialized).unwrap();
        assert_eq!(opts, deserialized);
    }

    #[test]
    fn folder_scan_options_zero_max_depth() {
        let opts = FolderScanOptions {
            recursive: true,
            include_hidden: true,
            allowed_extensions: vec!["docx".to_string()],
            max_depth: Some(0),
        };
        let serialized = serde_json::to_string(&opts).unwrap();
        let deserialized: FolderScanOptions = serde_json::from_str(&serialized).unwrap();
        assert_eq!(opts, deserialized);
    }

    #[test]
    fn file_access_grant_empty_reason() {
        let grant = FileAccessGrant {
            scope: FilePermissionScope::SingleFile,
            path: String::new(),
            reason: String::new(),
        };
        let serialized = serde_json::to_string(&grant).unwrap();
        let deserialized: FileAccessGrant = serde_json::from_str(&serialized).unwrap();
        assert_eq!(grant, deserialized);
    }

    #[test]
    fn office_file_watch_event_none_fields() {
        let event = OfficeFileWatchEvent {
            product_id: String::new(),
            path: String::new(),
            kind: FileWatchEventKind::Deleted,
            previous_path: None,
            marker: None,
            observed_at: None,
        };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: OfficeFileWatchEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }

    fn temp_test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("tench-fs-core-{name}-{unique}"));
        fs::create_dir_all(&root).expect("create temp dir");
        root
    }
}
