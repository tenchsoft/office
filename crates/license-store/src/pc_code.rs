//! PC Request Code — Direction B (Program → Web) activation flow.
//!
//! Format: `TENCHPC-<base64url(JSON)>`. See
//! `tench-docs/plans/contracts/Licensing/licensing-auth.md` §10 for the wire
//! contract (mirrored from `tench-web/functions/_lib/device_auth.ts`).

use crate::error::LicenseStoreError;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};

pub const PC_CODE_PREFIX: &str = "TENCHPC-";
pub const PC_CODE_TTL_SECS: u64 = 10 * 60;
const NONCE_BYTES: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcRequestCode {
    pub v: u8,
    pub device_id: String,
    pub device_meta: serde_json::Value,
    pub nonce: String,
    pub exp: u64,
}

/// Encodes a one-time PC request code bound to the given device.
///
/// `device_meta` is embedded as-is. The TTL is fixed at 10 minutes from now.
pub fn encode_pc_request_code(
    device_id: &str,
    device_meta: serde_json::Value,
) -> Result<String, LicenseStoreError> {
    let nonce = generate_nonce()?;
    let exp = unix_now() + PC_CODE_TTL_SECS;
    let payload = PcRequestCode {
        v: 1,
        device_id: device_id.to_string(),
        device_meta,
        nonce,
        exp,
    };
    let json = serde_json::to_vec(&payload)?;
    let b64 = URL_SAFE_NO_PAD.encode(json);
    Ok(format!("{PC_CODE_PREFIX}{b64}"))
}

/// Decodes and validates a PC request code. Returns an error if the prefix,
/// base64, JSON, or schema is invalid.
///
/// Expiry is **not** checked here — callers (e.g. the desktop poll loop and
/// the server bind endpoint) decide whether to reject stale codes.
pub fn decode_pc_request_code(code: &str) -> Result<PcRequestCode, LicenseStoreError> {
    let stripped = code
        .strip_prefix(PC_CODE_PREFIX)
        .ok_or(LicenseStoreError::StoreCorrupted)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(stripped.trim().as_bytes())
        .map_err(|_| LicenseStoreError::StoreCorrupted)?;
    let parsed: PcRequestCode =
        serde_json::from_slice(&bytes).map_err(|_| LicenseStoreError::StoreCorrupted)?;
    if parsed.v != 1
        || parsed.device_id.is_empty()
        || parsed.nonce.is_empty()
        || parsed.exp == 0
    {
        return Err(LicenseStoreError::StoreCorrupted);
    }
    Ok(parsed)
}

/// Generates a fresh 16-char hex nonce.
pub fn generate_nonce() -> Result<String, LicenseStoreError> {
    let mut buf = [0u8; NONCE_BYTES];
    getrandom::fill(&mut buf).map_err(|_| LicenseStoreError::DeviceIdUnreachable)?;
    Ok(buf.iter().map(|b| format!("{b:02x}")).collect())
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pc_code_roundtrip_preserves_payload() {
        let meta = serde_json::json!({"os": "linux", "hostname": "box"});
        let encoded =
            encode_pc_request_code("abc123def456", meta.clone()).expect("encode");
        assert!(encoded.starts_with(PC_CODE_PREFIX));
        let decoded = decode_pc_request_code(&encoded).expect("decode");
        assert_eq!(decoded.v, 1);
        assert_eq!(decoded.device_id, "abc123def456");
        assert_eq!(decoded.device_meta, meta);
        assert_eq!(decoded.nonce.len(), 16);
        assert!(decoded.exp > unix_now());
    }

    #[test]
    fn pc_code_rejects_bad_prefix() {
        let result = decode_pc_request_code("NOTPREFIX-payload");
        assert!(result.is_err());
    }

    #[test]
    fn pc_code_rejects_bad_base64() {
        let result = decode_pc_request_code("TENCHPC-@@invalid@@");
        assert!(result.is_err());
    }

    #[test]
    fn pc_code_exp_is_ttl_seconds_in_future() {
        let before = unix_now();
        let encoded = encode_pc_request_code("d", serde_json::Value::Null).unwrap();
        let after = unix_now();
        let decoded = decode_pc_request_code(&encoded).unwrap();
        assert!(decoded.exp >= before + PC_CODE_TTL_SECS - 2);
        assert!(decoded.exp <= after + PC_CODE_TTL_SECS + 2);
    }

    #[test]
    fn nonce_is_hex_and_unique() {
        let a = generate_nonce().unwrap();
        let b = generate_nonce().unwrap();
        assert_eq!(a.len(), 16);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }
}
