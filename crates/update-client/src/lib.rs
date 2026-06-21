//! Tench desktop update client.
//!
//! Thin orchestration layer around `tauri-plugin-updater` that:
//! - Loads the device token from `tench-license-store`
//! - Configures the Tauri updater endpoint with `Authorization: Tench-Device`
//!   so the server can gate updates by license validity
//! - Reports availability / install status back to the UI
//!
//! The actual download, signature verification, and platform-specific swap
//! are handled by `tauri-plugin-updater` (see AGENTS.md "Self-Implementation
//! Principle > Per-Component Decisions" for the rationale). We use Tauri's
//! standard `latest.json` manifest format and minisign signatures — the
//! server emits both from a single source of truth.
//!
//! See `tench-docs/plans/contracts/Licensing/licensing-auth.md` for the
//! end-to-end protocol.

mod detect;
mod error;
mod manifest;

pub use detect::{detect_install_method, InstallMethod};
pub use error::UpdateClientError;
pub use manifest::{
    current_platform_key, fetch_manifest, is_newer_version, ManifestResponse,
};

/// Default public tench-web origin.
pub const DEFAULT_BASE_URL: &str = "https://tenchsoft.com";
