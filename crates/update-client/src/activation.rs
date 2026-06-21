//! License activation helper — calls tench-web's `/api/device/token` endpoint
//! with the locally-derived device_id and writes the returned device_token
//! into the encrypted `LicenseStore`.
//!
//! Each desktop app's Tauri `license_activate` command delegates here so the
//! HTTP + response handling logic is not duplicated across 4 apps.

use crate::error::UpdateClientError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tench_license_store::LicenseStore;

const DEFAULT_BASE_URL: &str = "https://tenchsoft.com";

#[derive(Serialize)]
struct DeviceTokenRequest<'a> {
    license_key: &'a str,
    device_id: &'a str,
    device_meta: DeviceMeta<'a>,
}

#[derive(Serialize)]
struct DeviceMeta<'a> {
    os: &'static str,
    hostname: String,
    tench_app: &'a str,
    tench_ver: &'a str,
}

#[derive(Deserialize)]
struct DeviceTokenResponse {
    device_token: String,
    expires_at: String,
}

/// Calls the server to bind `license_key` to this device, then persists the
/// returned `device_token` into `store`.
///
/// - `base_url`: override the tench-web origin (defaults to `DEFAULT_BASE_URL`
///   or `TENCH_WEB_BASE_URL` env var if set)
/// - `tench_app`: product identifier — `"docs"`, `"sheets"`, `"slides"`,
///   `"kodocs"`. Used only for analytics/display on the web side.
/// - `tench_ver`: current app version (`env!("CARGO_PKG_VERSION")` from the
///   caller).
pub fn activate_license(
    store: &Arc<LicenseStore>,
    base_url: Option<&str>,
    license_key: &str,
    tench_app: &str,
    tench_ver: &str,
) -> Result<(), UpdateClientError> {
    let license_key = license_key.trim();
    if license_key.is_empty() {
        return Err(UpdateClientError::ManifestUnparseable);
    }

    let state = store.state();
    let device_id = state.device_id.clone();
    let body = DeviceTokenRequest {
        license_key,
        device_id: &device_id,
        device_meta: DeviceMeta {
            os: std::env::consts::OS,
            hostname: hostname(),
            tench_app,
            tench_ver,
        },
    };

    let env_base = std::env::var("TENCH_WEB_BASE_URL").ok();
    let base = base_url.or(env_base.as_deref()).unwrap_or(DEFAULT_BASE_URL);
    let url = format!("{base}/api/device/token");
    let response: DeviceTokenResponse = ureq::post(&url)
        .send_json(serde_json::to_value(&body)?)
        .map_err(map_ureq_error)?
        .into_json()
        .map_err(|e| UpdateClientError::Http(e.to_string()))?;

    store
        .set_activated(
            license_key.to_string(),
            response.device_token,
            response.expires_at,
        )
        .map_err(map_store_error)?;
    Ok(())
}

/// Clears the local license binding (used by the "Release device" UI action).
/// This does NOT call the server — the server's device_activations row will
/// be GC'd by `released_at` staleness later.
pub fn release_license(store: &Arc<LicenseStore>) -> Result<(), UpdateClientError> {
    store.clear().map_err(map_store_error)
}

fn map_ureq_error(e: ureq::Error) -> UpdateClientError {
    match e {
        ureq::Error::Status(code, resp) => {
            let msg = resp.into_string().unwrap_or_default();
            UpdateClientError::BadStatus(code as u16, msg)
        }
        ureq::Error::Transport(t) => UpdateClientError::Http(format!("network error: {t}")),
    }
}

fn map_store_error(e: tench_license_store::LicenseStoreError) -> UpdateClientError {
    UpdateClientError::Http(e.to_string())
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}
