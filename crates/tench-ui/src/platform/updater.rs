//! Auto-update system — checks for new versions and applies updates.
//!
//! Uses `reqwest` for HTTP requests and `tokio` for async runtime.
//! The update process:
//! 1. Check a configurable URL for the latest version info
//! 2. Download the update to a temp file
//! 3. Replace the current binary (platform-specific)

use serde::Deserialize;
use std::io;
use std::path::{Path, PathBuf};

/// Metadata about an available update, returned by the update server.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInfo {
    /// The latest available version string (e.g. "0.2.0").
    pub version: String,
    /// URL to download the update binary.
    pub download_url: String,
    /// SHA-256 checksum of the update binary (hex-encoded).
    pub sha256: Option<String>,
    /// Human-readable release notes.
    pub notes: Option<String>,
}

/// Errors that can occur during the update process.
#[derive(Debug)]
pub enum UpdateError {
    /// HTTP request failed.
    Http(reqwest::Error),
    /// I/O error (file operations).
    Io(io::Error),
    /// The update response could not be parsed.
    Parse(String),
    /// The downloaded file failed checksum verification.
    ChecksumMismatch { expected: String, actual: String },
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Parse(msg) => write!(f, "Parse error: {msg}"),
            Self::ChecksumMismatch { expected, actual } => {
                write!(f, "Checksum mismatch: expected {expected}, got {actual}")
            }
        }
    }
}

impl std::error::Error for UpdateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Http(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for UpdateError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

impl From<io::Error> for UpdateError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// The auto-update client.
pub struct Updater {
    /// Base URL for the update server.
    base_url: String,
    /// HTTP client.
    client: reqwest::Client,
    /// Current application version.
    current_version: String,
    /// Platform identifier (e.g. "linux-x86_64", "macos-arm64").
    platform: String,
}

impl Updater {
    /// Create a new updater.
    ///
    /// - `base_url`: The update server base URL (e.g. "https://updates.tench.app").
    /// - `current_version`: The current application version (e.g. "0.1.0").
    /// - `platform`: The target platform string (e.g. "linux-x86_64").
    pub fn new(
        base_url: impl Into<String>,
        current_version: impl Into<String>,
        platform: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
            current_version: current_version.into(),
            platform: platform.into(),
        }
    }

    /// Detect the current platform string.
    pub fn detect_platform() -> String {
        let os = if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "unknown"
        };
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "unknown"
        };
        format!("{os}-{arch}")
    }

    /// Check for updates by querying the update server.
    ///
    /// Returns `Some(UpdateInfo)` if an update is available, `None` if the
    /// current version is up to date.
    pub async fn check(&self) -> Result<Option<UpdateInfo>, UpdateError> {
        let url = format!(
            "{}/update?platform={}&version={}",
            self.base_url, self.platform, self.current_version
        );

        let response = self.client.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        let info: UpdateInfo = response
            .json()
            .await
            .map_err(|e| UpdateError::Parse(format!("Failed to parse update response: {e}")))?;

        if info.version == self.current_version {
            Ok(None)
        } else {
            Ok(Some(info))
        }
    }

    /// Download the update to a temporary file.
    ///
    /// Returns the path to the downloaded file.
    pub async fn download(&self, info: &UpdateInfo) -> Result<PathBuf, UpdateError> {
        let response = self.client.get(&info.download_url).send().await?;

        let bytes = response.bytes().await?;

        // Create a temp file
        let temp_dir = std::env::temp_dir();
        let file_name = format!("tench-update-{}", info.version);
        let temp_path = temp_dir.join(file_name);

        std::fs::write(&temp_path, &bytes)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&temp_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&temp_path, perms)?;
        }

        Ok(temp_path)
    }

    /// Apply the update by replacing the current binary.
    ///
    /// On success, the application should exit so the new binary can take over.
    /// The `pending_path` is the downloaded update file.
    pub async fn apply(&self, pending_path: &Path) -> Result<(), UpdateError> {
        let current_exe = std::env::current_exe()?;

        // Write a small updater script / do the swap
        #[cfg(unix)]
        {
            self.apply_unix(&current_exe, pending_path)?;
        }

        #[cfg(windows)]
        {
            self.apply_windows(&current_exe, pending_path)?;
        }

        Ok(())
    }

    /// Unix-specific binary replacement.
    ///
    /// Renames the downloaded file to the current executable path.
    #[cfg(unix)]
    fn apply_unix(&self, current_exe: &Path, pending_path: &Path) -> Result<(), UpdateError> {
        // On Unix, we can simply rename the file (atomic on same filesystem)
        // or copy + remove if cross-filesystem.
        if let Err(_) = std::fs::rename(pending_path, current_exe) {
            // Fallback: copy then remove
            std::fs::copy(pending_path, current_exe)?;
            let _ = std::fs::remove_file(pending_path);
        }
        Ok(())
    }

    /// Windows-specific binary replacement.
    ///
    /// On Windows, we cannot replace a running executable. Instead we
    /// schedule the replacement for the next reboot using `MoveFileEx`.
    #[cfg(windows)]
    fn apply_windows(&self, current_exe: &Path, pending_path: &Path) -> Result<(), UpdateError> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let source: Vec<u16> = OsStr::new(pending_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let target: Vec<u16> = OsStr::new(current_exe)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // MOVEFILE_REPLACE_EXISTING | MOVEFILE_DELAY_UNTIL_REBOOT
        unsafe {
            let result = windows_sys::Win32::Storage::FileSystem::MoveFileExW(
                source.as_ptr(),
                target.as_ptr(),
                0x1 | 0x4,
            );
            if result == 0 {
                return Err(UpdateError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "MoveFileEx failed",
                )));
            }
        }
        Ok(())
    }
}
