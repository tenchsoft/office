//! Release manifest types and HTTP fetch.
//!
//! Wire contract mirrors `tench-docs/plans/contracts/Licensing/licensing-auth.md`
//! §12.2 and `tench-web/functions/api/device/releases.ts`.
//!
//! Several functions in this module are flagged `#[allow(dead_code)]` because
//! they are part of the public API consumed by the upcoming
//! `update-check-weekly` and `update-install-flow` feature chains (not yet
//! wired into a binary).

#![allow(dead_code)]

use crate::error::UpdateClientError;
use serde::{Deserialize, Serialize};

const SUPPORTED_APPS: &[&str] = &["docs", "sheets", "slides", "kodocs"];
const SUPPORTED_CHANNELS: &[&str] = &["stable", "beta"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleasePriority {
    Normal,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRelease {
    pub url: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaRelease {
    pub from: String,
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub patch_algo: String,
    pub target_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRelease {
    pub full: AssetRelease,
    #[serde(default)]
    pub deltas: Vec<DeltaRelease>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub app: String,
    pub version: String,
    pub released_at: String,
    pub priority: ReleasePriority,
    pub platforms: std::collections::HashMap<String, PlatformRelease>,
}

/// Server-side response envelope. `manifest_str` is the **raw** JSON string
/// that the client must use for ED25519 verification (NOT a re-serialized
/// version of `manifest`).
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestResponse {
    pub app: String,
    pub channel: String,
    pub version: String,
    pub priority: String,
    pub released_at: String,
    pub notes: Option<String>,
    pub manifest_str: String,
    pub signature: String,
}

impl ManifestResponse {
    pub fn priority(&self) -> Result<ReleasePriority, UpdateClientError> {
        match self.priority.as_str() {
            "normal" => Ok(ReleasePriority::Normal),
            "critical" => Ok(ReleasePriority::Critical),
            _ => Err(UpdateClientError::ManifestUnparseable),
        }
    }

    pub fn parse_manifest(&self) -> Result<Manifest, UpdateClientError> {
        serde_json::from_str::<Manifest>(&self.manifest_str)
            .map_err(|_| UpdateClientError::ManifestUnparseable)
    }
}

/// Fetches the latest manifest for `app` on `channel` from the tench-web
/// server. `device_token` is sent as `Authorization: Tench-Device <token>`.
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

    let url = format!(
        "{base_url}/api/device/releases?app={app}&channel={channel}"
    );
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

    let body: ManifestResponse =
        response.into_json().map_err(|e| UpdateClientError::Http(e.to_string()))?;
    Ok(body)
}

/// Returns the platform entry for the running OS/arch, or an error if the
/// manifest does not cover this platform.
pub fn platform_release_for_current<'a>(
    manifest: &'a Manifest,
) -> Result<&'a PlatformRelease, UpdateClientError> {
    let key = current_platform_key();
    manifest
        .platforms
        .get(&key)
        .ok_or(UpdateClientError::PlatformNotFound)
}

/// Returns the canonical platform key used in manifests, e.g.
/// `windows-x86_64`, `linux-x86_64`, `macos-arm64`.
pub fn current_platform_key() -> String {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
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

/// Returns true if `candidate` is strictly newer than `current` per loose
/// semver rules (major.minor.patch). Pre-release suffixes are compared
/// lexicographically when the rest is equal.
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
    fn version_compare_prerelease() {
        // 0.2.0-beta is treated equal-but-different to 0.2.0.
        assert!(!is_newer_version("0.2.0", "0.2.0"));
        assert!(is_newer_version("0.2.0", "0.2.0-beta"));
    }

    #[test]
    fn platform_key_starts_with_os() {
        let key = current_platform_key();
        let os_prefix = if cfg!(target_os = "windows") {
            "windows-"
        } else if cfg!(target_os = "macos") {
            "macos-"
        } else if cfg!(target_os = "linux") {
            "linux-"
        } else {
            return;
        };
        assert!(key.starts_with(os_prefix));
    }
}
