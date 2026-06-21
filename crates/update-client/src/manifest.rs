//! Release manifest types and HTTP fetch.
//!
//! Wire format matches Tauri 2 updater's `latest.json` schema so the
//! `tauri-plugin-updater` can consume the response directly. Our server's
//! `/api/device/releases` endpoint authenticates the request via the
//! `Authorization: Tench-Device` header and then returns the manifest below
//! (or 401 if the device's license is no longer valid, which silently
//! disables updates for that client).

use crate::error::UpdateClientError;
use serde::{Deserialize, Serialize};

const SUPPORTED_APPS: &[&str] = &["docs", "sheets", "slides", "kodocs"];
const SUPPORTED_CHANNELS: &[&str] = &["stable", "beta"];

/// Tauri-compatible release manifest. See Tauri 2 updater docs for the
/// canonical schema; we mirror it here so the plugin can parse the response
/// without an intermediate translation layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestResponse {
    pub version: String,
    #[serde(default)]
    pub notes: String,
    pub pub_date: String,
    /// Platform → {signature, url}. Tauri uses `darwin-*` for macOS.
    pub platforms: std::collections::HashMap<String, PlatformEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEntry {
    /// minisign signature over the downloaded archive.
    pub signature: String,
    /// URL of the platform-specific update bundle (.msi.zip on Windows,
    /// .AppImage.tar.gz on Linux, .app.tar.gz on macOS).
    pub url: String,
}

/// Fetches the latest Tauri-format manifest for `app` on `channel`.
///
/// The server returns 401 when the device's license has expired or been
/// revoked — callers should treat that as "no update available, surface the
/// reactivation UI".
pub fn fetch_manifest(
    base_url: &str,
    app: &str,
    channel: &str,
    device_token: &str,
) -> Result<ManifestResponse, UpdateClientError> {
    if !SUPPORTED_APPS.contains(&app) {
        return Err(UpdateClientError::ManifestUnparseable);
    }
    if !SUPPORTED_CHANNELS.contains(&channel) {
        return Err(UpdateClientError::ManifestUnparseable);
    }

    let url = format!("{base_url}/api/device/releases?app={app}&channel={channel}");
    let response = ureq::get(&url)
        .set("Authorization", &format!("Tench-Device {device_token}"))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, resp) => {
                let msg = resp.into_string().unwrap_or_default();
                UpdateClientError::BadStatus(code as u16, msg)
            }
            ureq::Error::Transport(_) => UpdateClientError::NetworkUnavailable,
        })?;

    let body: ManifestResponse = response
        .into_json()
        .map_err(|e| UpdateClientError::Http(e.to_string()))?;
    Ok(body)
}

/// Returns the canonical platform key used in Tauri manifests.
/// Tauri uses `darwin-*` for macOS (NOT `macos-*`) and `windows-*` /
/// `linux-*` for the other two.
pub fn current_platform_key() -> String {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };
    format!("{os}-{arch}")
}

/// Loose semver "strictly newer" check used to decide whether to surface an
/// update notification before Tauri's own updater runs.
pub fn is_newer_version(candidate: &str, current: &str) -> bool {
    match (parse_semver(candidate), parse_semver(current)) {
        (Some(c), Some(cur)) => c > cur,
        _ => candidate != current,
    }
}

fn parse_semver(v: &str) -> Option<(u64, u64, u64, Option<&str>)> {
    let (core, pre) = v.split_once('-').map(|(c, p)| (c, Some(p))).unwrap_or((v, None));
    let mut nums = core.split('.');
    let major: u64 = nums.next()?.parse().ok()?;
    let minor: u64 = nums.next()?.parse().ok()?;
    let patch: u64 = nums.next()?.parse().ok()?;
    if nums.next().is_some() {
        return None;
    }
    Some((major, minor, patch, pre))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compare_basic() {
        assert!(is_newer_version("0.2.0", "0.1.0"));
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(!is_newer_version("0.0.9", "0.1.0"));
    }

    #[test]
    fn platform_key_uses_tauri_darwin_prefix_on_macos() {
        // Tauri uses `darwin-*` for macOS. Verify our key follows that
        // convention on macOS hosts; on other hosts we just sanity-check
        // the prefix matches the host OS as Tauri expects.
        let key = current_platform_key();
        let prefix = if cfg!(target_os = "windows") {
            "windows-"
        } else if cfg!(target_os = "macos") {
            "darwin-"
        } else if cfg!(target_os = "linux") {
            "linux-"
        } else {
            return;
        };
        assert!(key.starts_with(prefix));
    }

    #[test]
    fn manifest_parses_tauri_format() {
        let json = r#"{
            "version": "0.2.0",
            "notes": "bug fixes",
            "pub_date": "2026-06-21T12:00:00Z",
            "platforms": {
                "windows-x86_64": {
                    "signature": "sig",
                    "url": "https://example.com/msi.zip"
                }
            }
        }"#;
        let parsed: ManifestResponse = serde_json::from_str(json).expect("parse");
        assert_eq!(parsed.version, "0.2.0");
        assert_eq!(parsed.platforms.len(), 1);
        assert_eq!(
            parsed.platforms["windows-x86_64"].url,
            "https://example.com/msi.zip"
        );
    }
}
