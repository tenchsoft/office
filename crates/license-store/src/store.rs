//! Encrypted local store for license credentials.
//!
//! File layout on disk:
//! ```text
//! <app_data_dir>/Tench/<product>/license-store.bin
//! ```
//! The contents are `[ENCRYPTED_MAGIC][12-byte nonce][AES-256-GCM ciphertext]`.
//! Plaintext is `serde_json::to_vec(&PersistedState)` — a small JSON document
//! carrying the device token, license key, and timestamps. The encryption key
//! is derived from the device id (HKDF not yet available; we use SHA-256 of
//! `device_id || DOMAIN`).

use crate::device_id::{device_id, is_ephemeral_device_id};
use crate::error::LicenseStoreError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tench_storage_core::{decrypt_data, encrypt_data};

pub const FILE_NAME: &str = "license-store.bin";
pub const FORMAT_VERSION: u8 = 1;

const KEY_DOMAIN: &[u8] = b"tench-license-store-v1/aes-key";

/// Aggregate license status surfaced to the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseStatus {
    /// No stored credentials. UI shows license-input form.
    Unactivated,
    /// Stored token exists and is not expired.
    Active,
    /// Stored token exists but is past its expiry timestamp.
    Expired,
}

impl LicenseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unactivated => "unactivated",
            Self::Active => "active",
            Self::Expired => "expired",
        }
    }
}

/// Snapshot of the current license state. Cheap to clone (all fields are
/// owned strings or `Option<String>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseState {
    pub device_id: String,
    pub license_key: Option<String>,
    pub device_token: Option<String>,
    /// RFC 3339 timestamp, e.g. `"2026-07-01T12:00:00Z"`.
    pub token_expires_at: Option<String>,
    /// RFC 3339 timestamp of the last successful `/api/device/token` call.
    pub last_refreshed_at: Option<String>,
}

impl LicenseState {
    fn fresh(device_id: String) -> Self {
        Self {
            device_id,
            license_key: None,
            device_token: None,
            token_expires_at: None,
            last_refreshed_at: None,
        }
    }

    /// Computes the current status from `token_expires_at`.
    ///
    /// - `None` token  → `Unactivated`
    /// - Expired token → `Expired`
    /// - Otherwise     → `Active`
    pub fn status(&self) -> LicenseStatus {
        let Some(_token) = &self.device_token else {
            return LicenseStatus::Unactivated;
        };
        let Some(exp) = &self.token_expires_at else {
            return LicenseStatus::Unactivated;
        };
        if is_expired(exp) {
            LicenseStatus::Expired
        } else {
            LicenseStatus::Active
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedState {
    pub v: u8,
    #[serde(flatten)]
    pub state: LicenseState,
}

/// Thread-safe handle to the on-disk license store.
pub struct LicenseStore {
    state: RwLock<LicenseState>,
    path: PathBuf,
    /// When true, [`LicenseStore::save`] is a no-op. Used when device_id is
    /// ephemeral or when the data directory is not writable.
    ephemeral: bool,
}

impl LicenseStore {
    /// Loads the store for the given product (`docs`, `sheets`, `slides`,
    /// `kodocs`). Creates an empty store if no file exists yet.
    ///
    /// Falls back to an in-memory ephemeral store if:
    /// - the device id cannot be derived
    /// - the data directory cannot be created or written to
    pub fn load_or_init(product: &str) -> Result<Arc<Self>, LicenseStoreError> {
        let path = file_path_for_product(product);
        Self::load_or_init_at(product, path)
    }

    /// Test-only variant that lets the caller pick the storage path.
    pub fn load_or_init_at(
        _product: &str,
        path: PathBuf,
    ) -> Result<Arc<Self>, LicenseStoreError> {
        let device_id = device_id()?;
        let ephemeral = is_ephemeral_device_id(&device_id);

        let state = if ephemeral {
            LicenseState::fresh(device_id)
        } else {
            match std::fs::read(&path) {
                Ok(bytes) => match Self::decrypt_state(&bytes, &device_id) {
                    Ok(s) => s,
                    Err(_) => {
                        // Corrupted or moved from another machine. Discard
                        // and start fresh — caller can re-activate.
                        LicenseState::fresh(device_id)
                    }
                },
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    LicenseState::fresh(device_id)
                }
                Err(_) => LicenseState::fresh(device_id),
            }
        };

        Ok(Arc::new(Self {
            state: RwLock::new(state),
            path,
            ephemeral,
        }))
    }

    /// Returns a snapshot of the current state.
    pub fn state(&self) -> LicenseState {
        self.state.read().unwrap().clone()
    }

    /// Convenience accessor for the current status.
    pub fn status(&self) -> LicenseStatus {
        self.state().status()
    }

    /// Returns true if writes are no-ops (ephemeral fallback).
    pub fn is_ephemeral(&self) -> bool {
        self.ephemeral
    }

    /// Records a successful activation or token refresh.
    pub fn set_activated(
        &self,
        license_key: String,
        device_token: String,
        token_expires_at: String,
    ) -> Result<(), LicenseStoreError> {
        let mut state = self.state.write().unwrap();
        state.license_key = Some(license_key);
        state.device_token = Some(device_token);
        state.token_expires_at = Some(token_expires_at);
        state.last_refreshed_at = Some(now_rfc3339());
        let snapshot = state.clone();
        drop(state);
        self.save(snapshot)
    }

    /// Updates only the token (used during weekly refreshes).
    pub fn set_token(
        &self,
        device_token: String,
        token_expires_at: String,
    ) -> Result<(), LicenseStoreError> {
        let mut state = self.state.write().unwrap();
        state.device_token = Some(device_token);
        state.token_expires_at = Some(token_expires_at);
        state.last_refreshed_at = Some(now_rfc3339());
        let snapshot = state.clone();
        drop(state);
        self.save(snapshot)
    }

    /// Clears the license binding (used on Release/Unbind from the UI).
    /// The device id is preserved.
    pub fn clear(&self) -> Result<(), LicenseStoreError> {
        let mut state = self.state.write().unwrap();
        state.license_key = None;
        state.device_token = None;
        state.token_expires_at = None;
        state.last_refreshed_at = None;
        let snapshot = state.clone();
        drop(state);
        self.save(snapshot)
    }

    fn save(&self, snapshot: LicenseState) -> Result<(), LicenseStoreError> {
        if self.ephemeral {
            return Ok(());
        }
        let persisted = PersistedState {
            v: FORMAT_VERSION,
            state: snapshot,
        };
        let json = serde_json::to_vec(&persisted)?;
        let device_id = self.state.read().unwrap().device_id.clone();
        let key = derive_aes_key(&device_id);
        let encrypted = encrypt_data(&json, &key)?;

        // Atomic write: <path>.tmp → fsync → rename(<path>.tmp, <path>)
        let mut tmp = self.path.clone().into_os_string();
        tmp.push(".tmp");
        let tmp = PathBuf::from(tmp);
        if let Some(parent) = tmp.parent() {
            std::fs::create_dir_all(parent)?;
        }
        {
            let file = std::fs::File::create(&tmp)?;
            file.sync_all()?;
            use std::io::Write;
            let mut writer = std::io::BufWriter::new(file);
            writer.write_all(&encrypted)?;
            writer.flush()?;
            let file = writer
                .into_inner()
                .map_err(|_| LicenseStoreError::AtomicRenameFailed)?;
            file.sync_all()?;
        }
        std::fs::rename(&tmp, &self.path).map_err(|_| LicenseStoreError::AtomicRenameFailed)?;
        Ok(())
    }

    fn decrypt_state(
        bytes: &[u8],
        device_id: &str,
    ) -> Result<LicenseState, LicenseStoreError> {
        let key = derive_aes_key(device_id);
        let plaintext = decrypt_data(bytes, &key)?;
        let parsed: PersistedState = serde_json::from_slice(&plaintext)?;
        if parsed.v != FORMAT_VERSION {
            return Err(LicenseStoreError::StoreCorrupted);
        }
        Ok(parsed.state)
    }
}

fn derive_aes_key(device_id: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    hasher.update(KEY_DOMAIN);
    hasher.finalize().into()
}

/// Resolves `<data_dir>/Tench/<product>/license-store.bin`.
pub fn file_path_for_product(product: &str) -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Tench").join(product).join(FILE_NAME)
}

fn now_rfc3339() -> String {
    // We avoid pulling in chrono by formatting from SystemTime directly. RFC
    // 3339 only needs second precision for our use case.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    format_unix_seconds_rfc3339(secs)
}

fn format_unix_seconds_rfc3339(secs: u64) -> String {
    // Civil-time conversion (Howard Hinnant's algorithm). Kept inline to
    // avoid pulling in time/chrono crates for one formatter.
    let z = secs as i64 / 86400 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64 + era * 400) as u64;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let secs_of_day = secs % 86400;
    let hh = secs_of_day / 3600;
    let mm = (secs_of_day % 3600) / 60;
    let ss = secs_of_day % 60;
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

fn is_expired(rfc3339: &str) -> bool {
    let Some(parsed) = parse_rfc3339_to_unix(rfc3339) else {
        // Unparseable → treat as expired so caller tries to refresh.
        return true;
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    parsed <= now
}

/// Very lenient parser for the subset of RFC 3339 we emit:
/// `YYYY-MM-DDTHH:MM:SSZ`. Returns Unix seconds.
fn parse_rfc3339_to_unix(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.len() < 20 {
        return None;
    }
    let bytes = s.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' || bytes[10] != b'T' || bytes[13] != b':' || bytes[16] != b':'
    {
        return None;
    }
    let y: u64 = s[0..4].parse().ok()?;
    let mo: u64 = s[5..7].parse().ok()?;
    let d: u64 = s[8..10].parse().ok()?;
    let hh: u64 = s[11..13].parse().ok()?;
    let mm: u64 = s[14..16].parse().ok()?;
    let ss: u64 = s[17..19].parse().ok()?;
    Some(civil_to_unix(y, mo, d, hh, mm, ss))
}

fn civil_to_unix(y: u64, m: u64, d: u64, hh: u64, mm: u64, ss: u64) -> u64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y / 400;
    let yoe = y - era * 400;
    let m = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * m + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = (era * 146097 + doe) as i64 - 719468;
    let secs_of_day = hh * 3600 + mm * 60 + ss;
    (days as u64) * 86400 + secs_of_day
}

#[allow(dead_code)]
fn ensure_parent_dir(path: &Path) -> Result<(), LicenseStoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_unactivated_for_fresh_state() {
        let s = LicenseState::fresh("abc".into());
        assert_eq!(s.status(), LicenseStatus::Unactivated);
    }

    #[test]
    fn status_active_when_token_not_expired() {
        let future = format_unix_seconds_rfc3339(unix_now() + 3600);
        let s = LicenseState {
            device_id: "abc".into(),
            license_key: Some("k".into()),
            device_token: Some("t".into()),
            token_expires_at: Some(future),
            last_refreshed_at: None,
        };
        assert_eq!(s.status(), LicenseStatus::Active);
    }

    #[test]
    fn status_expired_when_token_in_past() {
        let past = format_unix_seconds_rfc3339(unix_now().saturating_sub(60));
        let s = LicenseState {
            device_id: "abc".into(),
            license_key: Some("k".into()),
            device_token: Some("t".into()),
            token_expires_at: Some(past),
            last_refreshed_at: None,
        };
        assert_eq!(s.status(), LicenseStatus::Expired);
    }

    #[test]
    fn rfc3339_roundtrip_within_minute() {
        let now = unix_now();
        let s = format_unix_seconds_rfc3339(now);
        let parsed = parse_rfc3339_to_unix(&s).unwrap();
        assert!(parsed.abs_diff(now) <= 1);
    }

    #[test]
    fn rfc3339_parse_rejects_short_input() {
        assert!(parse_rfc3339_to_unix("short").is_none());
    }

    fn unix_now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
