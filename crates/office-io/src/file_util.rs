use crate::error::OfficeIoError;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Write bytes to a file atomically using a temporary file.
///
/// Creates parent directories if needed. Writes to a `.atomictmp` sidecar
/// with a random suffix (PID + timestamp), calls `sync_all` for durability,
/// then renames into place so a crash during write does not corrupt the target.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), OfficeIoError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| OfficeIoError::Io {
            context: format!("Failed to create directory {}", parent.display()),
            source: error,
        })?;
    }

    let suffix = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    );
    let temp_path = path.with_extension(format!("{}.atomictmp", suffix));
    {
        let mut file = std::fs::File::create(&temp_path).map_err(|error| OfficeIoError::Io {
            context: format!("Failed to create temp file {}", temp_path.display()),
            source: error,
        })?;
        std::io::Write::write_all(&mut file, bytes).map_err(|error| OfficeIoError::Io {
            context: format!("Failed to write {}", temp_path.display()),
            source: error,
        })?;
        file.sync_all().map_err(|error| OfficeIoError::Io {
            context: format!("Failed to sync {}", temp_path.display()),
            source: error,
        })?;
    }

    match fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(first_error) => {
            if path.exists() {
                fs::remove_file(path).map_err(|remove_error| OfficeIoError::Io {
                    context: format!(
                        "Failed to replace {} after rename error ({first_error}): {remove_error}",
                        path.display()
                    ),
                    source: remove_error,
                })?;
                fs::rename(&temp_path, path).map_err(|error| OfficeIoError::Io {
                    context: format!("Failed to replace {}", path.display()),
                    source: error,
                })
            } else {
                let _ = fs::remove_file(&temp_path);
                Err(OfficeIoError::Io {
                    context: format!("Failed to save {}", path.display()),
                    source: first_error,
                })
            }
        }
    }
}

/// Return `Ok(())` when `path` exists, or an error describing the missing file.
pub fn ensure_file_exists(path: &Path) -> Result<(), OfficeIoError> {
    if path.exists() {
        Ok(())
    } else {
        Err(OfficeIoError::NotFound(format!(
            "File not found: {}",
            path.display()
        )))
    }
}

/// Replace characters that are invalid in file names with underscores.
pub fn sanitize_file_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}

/// Current time as seconds since UNIX epoch, wrapped in `Some`.
pub fn timestamp_string() -> Option<String> {
    Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string()),
    )
}

/// Current time as milliseconds since UNIX epoch.
pub fn timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

/// File modification time as seconds since UNIX epoch.
pub fn modified_at_unix(path: &Path) -> Option<String> {
    fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs().to_string())
}

/// Compute a deterministic integrity checksum for the given data.
/// Uses FNV-1a-based hashing to produce a 64-character hex string
/// suitable for verifying file integrity without external dependencies.
pub fn compute_checksum(data: &[u8]) -> String {
    let mut hash1: u64 = 0xcbf29ce484222325;
    let mut hash2: u64 = 0x9e3779b97f4a7c15;
    let mut hash3: u64 = 0x6c62272e07bb0142;
    let mut hash4: u64 = 0x517cc1b727220a95;
    for &byte in data {
        hash1 ^= u64::from(byte);
        hash1 = hash1.wrapping_mul(0x100000001b3);
        hash2 ^= u64::from(byte);
        hash2 = hash2.wrapping_mul(0x100000001b3);
        hash3 ^= u64::from(byte);
        hash3 = hash3.wrapping_mul(0x100000001b3);
        hash4 ^= u64::from(byte);
        hash4 = hash4.wrapping_mul(0x100000001b3);
    }
    // Mix the hashes together
    hash1 ^= hash2.wrapping_mul(0x517cc1b727220a95);
    hash3 ^= hash4.wrapping_mul(0x9e3779b97f4a7c15);
    hash1 ^= hash3;
    format!("{:016x}{:016x}{:016x}{:016x}", hash1, hash2, hash3, hash4)
}

/// Lossy conversion of a `Path` to a `String`.
pub fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn write_atomic_creates_file_with_content() {
        let dir = std::env::temp_dir().join(format!(
            "tench_office_io_write_atomic_{}_{}",
            std::process::id(),
            timestamp_millis()
        ));
        fs::create_dir_all(&dir).expect("test dir");
        let target = dir.join("test.txt");

        write_atomic(&target, b"hello").expect("write");
        assert_eq!(fs::read_to_string(&target).unwrap(), "hello");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn write_atomic_creates_parent_directories() {
        let dir = std::env::temp_dir().join(format!(
            "tench_office_io_nested_{}_{}",
            std::process::id(),
            timestamp_millis()
        ));
        let target = dir.join("a/b/c/test.txt");

        write_atomic(&target, b"nested").expect("write nested");
        assert_eq!(fs::read_to_string(&target).unwrap(), "nested");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ensure_file_exists_returns_ok_for_existing() {
        let dir = std::env::temp_dir().join(format!(
            "tench_office_io_exists_{}_{}",
            std::process::id(),
            timestamp_millis()
        ));
        fs::create_dir_all(&dir).expect("test dir");
        let file = dir.join("exists.txt");
        fs::write(&file, b"").expect("write");

        assert!(ensure_file_exists(&file).is_ok());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ensure_file_exists_returns_err_for_missing() {
        let path = PathBuf::from("/tmp/tench_office_io_nonexistent_12345");
        assert!(ensure_file_exists(&path).is_err());
    }

    #[test]
    fn sanitize_file_name_replaces_special_chars() {
        assert_eq!(
            sanitize_file_name("a/b\\c:d*e?f\"g<h>i|j"),
            "a_b_c_d_e_f_g_h_i_j"
        );
    }

    #[test]
    fn sanitize_file_name_preserves_normal_chars() {
        assert_eq!(sanitize_file_name("hello world.txt"), "hello world.txt");
    }

    #[test]
    fn timestamp_string_returns_some() {
        assert!(timestamp_string().is_some());
    }

    #[test]
    fn timestamp_millis_is_nonzero() {
        assert!(timestamp_millis() > 0);
    }

    #[test]
    fn path_to_string_converts_path() {
        let path = PathBuf::from("/tmp/test.txt");
        assert_eq!(path_to_string(&path), "/tmp/test.txt");
    }
}
