//! Tench desktop update client.
//!
//! Responsibilities (see plans/spec/docs/ feature chains):
//! - Fetch release manifest from `/api/device/releases`
//! - Verify ED25519 signature with compile-time-embedded public key
//! - Compare versions and surface availability
//! - (TODO) Apply bsdiff delta patches and platform-specific swap
//!
//! This crate is intentionally HTTP-thin: the heavy lifting (bsdiff, swap)
//! lands in follow-up features and is gated behind `cfg`-flagged modules.

mod detect;
mod error;
mod manifest;
mod verify;

pub use detect::{detect_install_method, InstallMethod};
pub use error::UpdateClientError;
pub use manifest::{
    fetch_manifest, AssetRelease, DeltaRelease, Manifest, PlatformRelease, ReleasePriority,
};
pub use verify::{verify_manifest_signature, VerifyingKeyBytes};

/// Default public tench-web origin. Override at runtime by passing a different
/// base URL to [`fetch_manifest`].
pub const DEFAULT_BASE_URL: &str = "https://tenchsoft.com";

/// Compile-time embedded ED25519 public key (32 raw bytes). Replace with the
/// real key produced by the keypair generation script (see
/// `tools/update-keygen/`). Until then, this is a placeholder that will fail
/// verification — by design — so unsigned manifests cannot slip through.
pub const EMBEDDED_PUBLIC_KEY: VerifyingKeyBytes = VerifyingKeyBytes::placeholder();
