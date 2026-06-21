//! Local encrypted credential store and device identifier derivation for
//! Tench desktop apps.
//!
//! See `plans/spec/docs/license-persistence.md` for the behavioral contract and
//! `plans/background/docs/license-persistence.md` for the runtime contract.
//!
//! ## Quick start
//!
//! ```no_run
//! use tench_license_store::LicenseStore;
//!
//! let store = LicenseStore::load_or_init("docs")?;
//! println!("device_id = {}", store.state().device_id);
//! println!("license_key = {:?}", store.state().license_key);
//! # Ok::<(), tench_license_store::LicenseStoreError>(())
//! ```

mod device_id;
mod error;
mod pc_code;
mod store;

pub use device_id::{device_id, is_ephemeral_device_id};
pub use error::LicenseStoreError;
pub use pc_code::{
    decode_pc_request_code, encode_pc_request_code, generate_nonce, PcRequestCode,
    PC_CODE_PREFIX, PC_CODE_TTL_SECS,
};
pub use store::{
    file_path_for_product, LicenseState, LicenseStatus, LicenseStore, FILE_NAME, FORMAT_VERSION,
};

/// Number of seconds the device token TTL has on the server side. Used by the
/// update-client to schedule token refreshes before expiry. Kept here because
/// the server constant lives in another repo (`tench-web` `_types.ts`).
pub const DEVICE_TOKEN_TTL_SECS: u64 = 10 * 24 * 60 * 60;
