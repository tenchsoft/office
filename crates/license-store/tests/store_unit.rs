//! Integration tests for `tench-license-store` covering file roundtrip,
//! corruption fallback, and clear semantics.
//!
//! Test names embed `license_persistence_*` so they fall under the
//! `license_persistence` test doc gate (see `plans/test/docs/license-persistence.md`).

use std::sync::OnceLock;
use tench_license_store::{LicenseStore, LicenseStatus};

static TEMPDIR_HOLDER: OnceLock<tempfile::TempDir> = OnceLock::new();

fn fixture_path(filename: &str) -> std::path::PathBuf {
    let dir = TEMPDIR_HOLDER.get_or_init(|| tempfile::tempdir().expect("tempdir"));
    dir.path().join(filename)
}

fn fresh_store(filename: &str) -> std::sync::Arc<LicenseStore> {
    let path = fixture_path(filename);
    // Ensure no leftover.
    let _ = std::fs::remove_file(&path);
    LicenseStore::load_or_init_at("docs", path).expect("load_or_init")
}

#[test]
fn license_persistence_fresh_start_unactivated() {
    let store = fresh_store("fresh.bin");
    let state = store.state();
    assert!(state.license_key.is_none());
    assert!(state.device_token.is_none());
    assert!(!state.device_id.is_empty());
    assert_eq!(store.status(), LicenseStatus::Unactivated);
}

#[test]
fn license_persistence_restores_activated_state() {
    let path = fixture_path("restore.bin");
    let _ = std::fs::remove_file(&path);

    let store = LicenseStore::load_or_init_at("docs", path.clone()).expect("init");
    let future = future_rfc3339(3600);
    store
        .set_activated(
            "TENCH-ABCD".into(),
            "tok-123".into(),
            future.clone(),
        )
        .expect("activate");

    // Drop the in-memory instance and reload from disk.
    drop(store);
    let reloaded = LicenseStore::load_or_init_at("docs", path).expect("reload");
    let state = reloaded.state();
    assert_eq!(state.license_key.as_deref(), Some("TENCH-ABCD"));
    assert_eq!(state.device_token.as_deref(), Some("tok-123"));
    assert_eq!(state.token_expires_at.as_deref(), Some(future.as_str()));
    assert_eq!(reloaded.status(), LicenseStatus::Active);
}

#[test]
fn license_persistence_missing_file_is_unactivated() {
    let store = fresh_store("missing.bin");
    assert_eq!(store.status(), LicenseStatus::Unactivated);
}

#[test]
fn license_persistence_corrupted_file_falls_back() {
    let path = fixture_path("corrupt.bin");
    std::fs::write(&path, b"definitely not a valid encrypted payload").expect("write");

    let store = LicenseStore::load_or_init_at("docs", path).expect("load");
    // Falls back to unactivated without panicking.
    assert_eq!(store.status(), LicenseStatus::Unactivated);
    assert!(store.state().license_key.is_none());
}

#[test]
fn license_persistence_clear_keeps_device_id() {
    let store = fresh_store("clear.bin");
    let device_id_before = store.state().device_id.clone();
    store
        .set_activated(
            "TENCH-X".into(),
            "tok".into(),
            future_rfc3339(3600),
        )
        .expect("activate");
    store.clear().expect("clear");
    let state = store.state();
    assert!(state.license_key.is_none());
    assert!(state.device_token.is_none());
    assert_eq!(state.device_id, device_id_before);
    assert_eq!(store.status(), LicenseStatus::Unactivated);
}

#[test]
fn license_persistence_roundtrip_preserves_state() {
    let path = fixture_path("roundtrip.bin");
    let _ = std::fs::remove_file(&path);

    let store = LicenseStore::load_or_init_at("docs", path.clone()).expect("init");
    let original_device_id = store.state().device_id.clone();
    store
        .set_activated(
            "TENCH-R".into(),
            "tok-r".into(),
            future_rfc3339(7200),
        )
        .expect("activate");
    drop(store);

    let reloaded = LicenseStore::load_or_init_at("docs", path).expect("reload");
    let state = reloaded.state();
    assert_eq!(state.device_id, original_device_id);
    assert_eq!(state.license_key.as_deref(), Some("TENCH-R"));
    assert_eq!(state.device_token.as_deref(), Some("tok-r"));
}

#[test]
fn license_persistence_orphaned_tmp_ignored() {
    let path = fixture_path("orphan.bin");
    let tmp = path.with_extension("bin.tmp");
    let _ = std::fs::remove_file(&path);
    std::fs::write(&tmp, b"partial").expect("write tmp");

    let store = LicenseStore::load_or_init_at("docs", path).expect("load");
    assert_eq!(store.status(), LicenseStatus::Unactivated);
}

#[test]
fn license_persistence_set_token_preserves_license_key() {
    let store = fresh_store("settoken.bin");
    store
        .set_activated(
            "TENCH-S".into(),
            "tok-1".into(),
            future_rfc3339(60),
        )
        .expect("activate");
    store
        .set_token("tok-2".into(), future_rfc3339(7200))
        .expect("refresh");
    let state = store.state();
    assert_eq!(state.license_key.as_deref(), Some("TENCH-S"));
    assert_eq!(state.device_token.as_deref(), Some("tok-2"));
}

#[test]
fn license_persistence_expired_status_after_ttl() {
    let store = fresh_store("expired.bin");
    let past = past_rfc3339(60);
    store
        .set_activated("TENCH-E".into(), "tok".into(), past)
        .expect("activate");
    assert_eq!(store.status(), LicenseStatus::Expired);
}

fn future_rfc3339(offset_secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format_unix_seconds_rfc3339(now + offset_secs)
}

fn past_rfc3339(offset_secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format_unix_seconds_rfc3339(now.saturating_sub(offset_secs))
}

// Local copy of the formatter (private in store.rs) — keep in sync. We don't
// expose it because tests outside the crate should use real RFC 3339 strings
// in the long run; for now this keeps the test self-contained.
fn format_unix_seconds_rfc3339(secs: u64) -> String {
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
