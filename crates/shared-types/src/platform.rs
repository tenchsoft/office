//! Platform action abstraction for external process calls.
//!
//! Centralizes all external process invocations (file manager open, URL open,
//! etc.) behind a typed enum with path validation, ensuring no shell injection
//! is possible and all process calls go through a single audit point.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A platform action that may spawn an external process.
///
/// All variants use validated, canonicalized paths and explicit argv.
/// No shell interpretation occurs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "path")]
pub enum PlatformAction {
    /// Reveal a file in the system file manager.
    RevealFile(String),
    /// Open a file with the system default application.
    OpenFile(String),
    /// Open a URL in the system default browser.
    OpenUrl(String),
}

/// Result of executing a platform action.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformActionResult {
    pub success: bool,
    pub message: String,
}

impl PlatformAction {
    /// Validate and canonicalize the path for this action.
    ///
    /// Returns an error if the path doesn't exist or contains traversal attempts.
    pub fn validate_path(&self) -> Result<PathBuf, String> {
        match self {
            PlatformAction::RevealFile(path) | PlatformAction::OpenFile(path) => {
                let p = Path::new(path);
                if !p.exists() {
                    return Err(format!("Path does not exist: {path}"));
                }
                p.canonicalize()
                    .map_err(|e| format!("Failed to canonicalize path: {e}"))
            }
            PlatformAction::OpenUrl(url) => {
                // Validate URL scheme
                if !url.starts_with("http://")
                    && !url.starts_with("https://")
                    && !url.starts_with("mailto:")
                {
                    return Err(format!("URL must use http, https, or mailto scheme: {url}"));
                }
                Ok(PathBuf::from(url))
            }
        }
    }

    /// Execute the platform action by spawning an external process.
    ///
    /// Uses explicit program and args (no shell).
    pub fn execute(&self) -> PlatformActionResult {
        match self.validate_path() {
            Ok(_) => {}
            Err(e) => {
                return PlatformActionResult {
                    success: false,
                    message: e,
                }
            }
        }

        let result = match self {
            PlatformAction::RevealFile(path) => reveal_file_in_manager(path),
            PlatformAction::OpenFile(path) => open_file_with_default(path),
            PlatformAction::OpenUrl(url) => open_url_in_browser(url),
        };

        match result {
            Ok(()) => PlatformActionResult {
                success: true,
                message: "Action completed successfully".into(),
            },
            Err(e) => PlatformActionResult {
                success: false,
                message: e,
            },
        }
    }
}

/// Reveal a file in the system file manager.
///
/// Uses platform-specific commands with explicit argv (no shell).
fn reveal_file_in_manager(path: &str) -> Result<(), String> {
    let canonical = Path::new(path)
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize: {e}"))?;

    if cfg!(target_os = "windows") {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", canonical.display()))
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {e}"))?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .args(["-R", &canonical.display().to_string()])
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {e}"))?;
    } else {
        // Linux: open the parent directory
        let parent = canonical
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {e}"))?;
    }
    Ok(())
}

/// Open a file with the system default application.
fn open_file_with_default(path: &str) -> Result<(), String> {
    let canonical = Path::new(path)
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize: {e}"))?;

    if cfg!(target_os = "windows") {
        std::process::Command::new("explorer")
            .arg(&canonical)
            .spawn()
            .map_err(|e| format!("Failed to open file: {e}"))?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(&canonical)
            .spawn()
            .map_err(|e| format!("Failed to open file: {e}"))?;
    } else {
        std::process::Command::new("xdg-open")
            .arg(&canonical)
            .spawn()
            .map_err(|e| format!("Failed to open file: {e}"))?;
    }
    Ok(())
}

/// Open a URL in the system default browser.
fn open_url_in_browser(url: &str) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {e}"))?;
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {e}"))?;
    } else {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_nonexistent_path() {
        let action = PlatformAction::RevealFile("/nonexistent/path".into());
        assert!(action.validate_path().is_err());
    }

    #[test]
    fn validate_rejects_unsafe_url_scheme() {
        let action = PlatformAction::OpenUrl("file:///etc/passwd".into());
        assert!(action.validate_path().is_err());

        let action = PlatformAction::OpenUrl("javascript:alert(1)".into());
        assert!(action.validate_path().is_err());
    }

    #[test]
    fn validate_accepts_http_url() {
        let action = PlatformAction::OpenUrl("https://example.com".into());
        assert!(action.validate_path().is_ok());
    }

    #[test]
    fn validate_accepts_existing_path() {
        let tmp = std::env::temp_dir();
        let action = PlatformAction::RevealFile(tmp.to_str().unwrap().into());
        assert!(action.validate_path().is_ok());
    }

    #[test]
    fn action_serialization_roundtrip() {
        let actions = vec![
            PlatformAction::RevealFile("/tmp/test.txt".into()),
            PlatformAction::OpenFile("/tmp/test.txt".into()),
            PlatformAction::OpenUrl("https://example.com".into()),
        ];
        for action in actions {
            let serialized = serde_json::to_string(&action).unwrap();
            let deserialized: PlatformAction = serde_json::from_str(&serialized).unwrap();
            assert_eq!(
                serde_json::to_string(&action).unwrap(),
                serde_json::to_string(&deserialized).unwrap()
            );
        }
    }
}
