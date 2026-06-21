use crate::error::OfficeIoError;
use std::io::{Cursor, Read, Seek, Write};

use zip::write::SimpleFileOptions;
use zip::ZipArchive;
use zip::ZipWriter;

/// Limits for ZIP archive extraction to guard against zip bombs and malformed archives.
#[derive(Clone, Debug)]
pub struct ArchiveLimits {
    /// Maximum allowed size of the archive file itself.
    pub max_archive_bytes: u64,
    /// Maximum number of entries in the archive.
    pub max_entry_count: usize,
    /// Maximum uncompressed size of a single entry.
    pub max_entry_bytes: u64,
    /// Maximum total uncompressed size across all entries.
    pub max_total_uncompressed_bytes: u64,
    /// Maximum compression ratio (uncompressed / compressed) before flagging as a zip bomb.
    pub max_compression_ratio: u64,
}

impl Default for ArchiveLimits {
    fn default() -> Self {
        Self::desktop()
    }
}

impl ArchiveLimits {
    /// Limits appropriate for desktop platforms with more memory.
    pub fn desktop() -> Self {
        Self {
            max_archive_bytes: 500 * 1024 * 1024, // 500 MB
            max_entry_count: 10000,
            max_entry_bytes: 100 * 1024 * 1024, // 100 MB
            max_total_uncompressed_bytes: 2 * 1024 * 1024 * 1024, // 2 GB
            max_compression_ratio: 1000,        // 1000x
        }
    }

    /// Limits appropriate for mobile platforms with constrained memory.
    pub fn mobile() -> Self {
        Self {
            max_archive_bytes: 100 * 1024 * 1024, // 100 MB
            max_entry_count: 5000,
            max_entry_bytes: 50 * 1024 * 1024, // 50 MB
            max_total_uncompressed_bytes: 500 * 1024 * 1024, // 500 MB
            max_compression_ratio: 500,
        }
    }
}

/// Validate a ZIP entry name against path traversal and unsafe patterns.
///
/// Rejects:
/// - Absolute paths (starting with `/`)
/// - Path traversal components (`..`)
/// - Backslash characters (potential traversal on Windows)
pub fn validate_zip_entry_name(name: &str) -> Result<(), OfficeIoError> {
    if name.starts_with('/') {
        return Err(OfficeIoError::General(format!(
            "ZIP entry has absolute path: {name}"
        )));
    }
    for component in name.split('/') {
        if component == ".." {
            return Err(OfficeIoError::General(format!(
                "ZIP entry contains path traversal: {name}"
            )));
        }
    }
    if name.contains('\\') {
        return Err(OfficeIoError::General(format!(
            "ZIP entry contains backslash: {name}"
        )));
    }
    Ok(())
}

/// Check a ZIP archive against the given limits before extraction.
///
/// Validates entry count, individual entry sizes, total uncompressed size,
/// compression ratios (zip bomb detection), and entry name safety.
pub fn check_archive_limits<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    limits: &ArchiveLimits,
) -> Result<(), OfficeIoError> {
    let entry_count = archive.len();
    if entry_count > limits.max_entry_count {
        return Err(OfficeIoError::General(format!(
            "Archive contains {entry_count} entries, exceeding limit of {}",
            limits.max_entry_count
        )));
    }

    let mut total_uncompressed: u64 = 0;
    for index in 0..entry_count {
        let file = archive
            .by_index(index)
            .map_err(|error| OfficeIoError::Parse {
                context: format!("Failed to read ZIP entry at index {index}"),
                source: Box::new(error),
            })?;
        let name = file.name().to_string();
        validate_zip_entry_name(&name)?;

        let uncompressed = file.size();
        if uncompressed > limits.max_entry_bytes {
            return Err(OfficeIoError::General(format!(
                "ZIP entry \"{name}\" is {uncompressed} bytes, exceeding limit of {}",
                limits.max_entry_bytes
            )));
        }

        let compressed = file.compressed_size();
        if let Some(ratio) = uncompressed.checked_div(compressed) {
            if ratio > limits.max_compression_ratio {
                return Err(OfficeIoError::General(format!(
                    "ZIP entry \"{name}\" has compression ratio {ratio}, exceeding limit of {}",
                    limits.max_compression_ratio
                )));
            }
        }

        total_uncompressed = total_uncompressed
            .checked_add(uncompressed)
            .ok_or_else(|| {
                OfficeIoError::General(
                    "Total uncompressed size overflow while checking archive limits".to_string(),
                )
            })?;
        if total_uncompressed > limits.max_total_uncompressed_bytes {
            return Err(OfficeIoError::General(format!(
                "Total uncompressed size exceeds limit of {}",
                limits.max_total_uncompressed_bytes
            )));
        }
    }

    Ok(())
}

/// Write a string entry into a ZIP archive.
pub fn write_zip_file(
    writer: &mut ZipWriter<Cursor<Vec<u8>>>,
    path: &str,
    contents: &str,
    options: SimpleFileOptions,
) -> Result<(), OfficeIoError> {
    writer
        .start_file(path, options)
        .map_err(|error| OfficeIoError::Serialize {
            context: format!("Failed to add {path} to ZIP"),
            source: Box::new(error),
        })?;
    writer
        .write_all(contents.as_bytes())
        .map_err(|error| OfficeIoError::Io {
            context: format!("Failed to write {path} to ZIP"),
            source: error,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use zip::{CompressionMethod, ZipArchive};

    #[test]
    fn write_zip_file_adds_readable_entry() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .unix_permissions(0o644);

        write_zip_file(&mut writer, "test.txt", "hello", options).expect("write");
        let cursor = writer.finish().expect("finish");
        let mut archive = ZipArchive::new(cursor).expect("open");

        let mut file = archive.by_name("test.txt").expect("find entry");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("read");
        assert_eq!(content, "hello");
    }

    #[test]
    fn validate_zip_entry_name_rejects_absolute_path() {
        assert!(validate_zip_entry_name("/etc/passwd").is_err());
    }

    #[test]
    fn validate_zip_entry_name_rejects_traversal() {
        assert!(validate_zip_entry_name("../../etc/passwd").is_err());
        assert!(validate_zip_entry_name("foo/../../bar").is_err());
    }

    #[test]
    fn validate_zip_entry_name_rejects_backslash() {
        assert!(validate_zip_entry_name("foo\\bar").is_err());
    }

    #[test]
    fn validate_zip_entry_name_accepts_valid_names() {
        assert!(validate_zip_entry_name("foo/bar.txt").is_ok());
        assert!(validate_zip_entry_name("document.xml").is_ok());
    }

    #[test]
    fn check_archive_limits_passes_for_small_archive() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .unix_permissions(0o644);

        write_zip_file(&mut writer, "test.txt", "hello", options).expect("write");
        let cursor = writer.finish().expect("finish");
        let mut archive = ZipArchive::new(cursor).expect("open");

        assert!(check_archive_limits(&mut archive, &ArchiveLimits::desktop()).is_ok());
    }
}
