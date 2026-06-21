use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// FX-DATA-002: SafeId Validation
// ---------------------------------------------------------------------------

/// Error returned when an identifier fails safe-id validation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SafeIdError {
    pub id: String,
    pub reason: String,
}

impl std::fmt::Display for SafeIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid safe id {:?}: {}", self.id, self.reason)
    }
}

impl std::error::Error for SafeIdError {}

/// Returns `true` if `id` is non-empty and contains only alphanumeric
/// characters, hyphens, and underscores.
pub fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Validates that `id` is a safe identifier, returning `Err(SafeIdError)` if
/// it contains disallowed characters or is empty.
pub fn validate_safe_id(id: &str) -> Result<(), SafeIdError> {
    if id.is_empty() {
        return Err(SafeIdError {
            id: id.to_string(),
            reason: "id must not be empty".to_string(),
        });
    }
    if !is_safe_id(id) {
        return Err(SafeIdError {
            id: id.to_string(),
            reason: "id contains disallowed characters".to_string(),
        });
    }
    Ok(())
}

pub fn validate_safe_file_name(file_name: &str) -> Result<(), SafeIdError> {
    if file_name.is_empty() {
        return Err(SafeIdError {
            id: file_name.to_string(),
            reason: "file name must not be empty".to_string(),
        });
    }
    if file_name == "." || file_name == ".." {
        return Err(SafeIdError {
            id: file_name.to_string(),
            reason: "file name must not be a relative path component".to_string(),
        });
    }
    if file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains("..")
        || file_name.chars().any(|ch| ch.is_control())
    {
        return Err(SafeIdError {
            id: file_name.to_string(),
            reason: "file name contains disallowed path characters".to_string(),
        });
    }
    let stem = file_name.split('.').next().unwrap_or_default();
    if is_windows_reserved_name(stem) {
        return Err(SafeIdError {
            id: file_name.to_string(),
            reason: "file name uses a reserved Windows device name".to_string(),
        });
    }
    Ok(())
}

fn is_windows_reserved_name(stem: &str) -> bool {
    let upper = stem.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

/// Filters `value` to only safe characters (alphanumeric, hyphens,
/// underscores). If the result is empty, returns `fallback` instead.
pub fn sanitize_id(value: &str, fallback: &str) -> String {
    let sanitized: String = value
        .chars()
        .filter(|&c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        .collect();
    if sanitized.is_empty() {
        fallback.to_string()
    } else {
        sanitized
    }
}
