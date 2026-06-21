//! ED25519 signature verification for release manifests.
//!
//! See `tench-docs/plans/contracts/Licensing/licensing-auth.md` §13.
//! Public key is embedded at compile time (see [`crate::EMBEDDED_PUBLIC_KEY`]).

use crate::error::UpdateClientError;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Wrapper around the 32 raw bytes of an ED25519 public key. This is the only
/// type the embed-site (lib.rs) needs to know about.
#[derive(Debug, Clone, Copy)]
pub struct VerifyingKeyBytes(pub [u8; 32]);

impl VerifyingKeyBytes {
    /// Placeholder key. **MUST be replaced with the real public key produced
    /// by `tools/update-keygen/` before any signed release is shipped.**
    /// Until replaced, verification always fails — by design.
    pub const fn placeholder() -> Self {
        Self([0; 32])
    }

    /// Returns true if this is still the all-zeros placeholder.
    pub fn is_placeholder(&self) -> bool {
        self.0.iter().all(|b| *b == 0)
    }
}

impl From<[u8; 32]> for VerifyingKeyBytes {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

/// Verifies that `signature_hex` is a valid ED25519 signature over
/// `manifest_str.as_bytes()` using the provided public key.
///
/// Hex decoding must yield exactly 64 bytes; otherwise returns
/// [`UpdateClientError::SignatureInvalid`].
pub fn verify_manifest_signature(
    manifest_str: &str,
    signature_hex: &str,
    public_key: VerifyingKeyBytes,
) -> Result<(), UpdateClientError> {
    if public_key.is_placeholder() {
        // Refuse to verify anything against the placeholder key.
        return Err(UpdateClientError::SignatureSchemeUnsupported);
    }

    let signature_bytes = decode_hex_64(signature_hex)?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| UpdateClientError::SignatureInvalid)?;
    let verifying = VerifyingKey::from_bytes(&public_key.0)
        .map_err(|_| UpdateClientError::SignatureSchemeUnsupported)?;

    verifying
        .verify(manifest_str.as_bytes(), &signature)
        .map_err(|_| UpdateClientError::SignatureInvalid)
}

fn decode_hex_64(s: &str) -> Result<[u8; 64], UpdateClientError> {
    let s = s.trim();
    if s.len() != 128 {
        return Err(UpdateClientError::SignatureInvalid);
    }
    let mut out = [0u8; 64];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        let hi = hex_value(chunk[0])?;
        let lo = hex_value(chunk[1])?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

fn hex_value(b: u8) -> Result<u8, UpdateClientError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(UpdateClientError::SignatureInvalid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand_core::OsRng;

    fn fake_manifest_str() -> &'static str {
        r#"{"app":"docs","version":"0.2.0","released_at":"2026-06-21T12:00:00Z","priority":"normal","platforms":{}}"#
    }

    #[test]
    fn placeholder_key_rejects_everything() {
        let err = verify_manifest_signature(
            fake_manifest_str(),
            "00".repeat(64).as_str(),
            VerifyingKeyBytes::placeholder(),
        )
        .unwrap_err();
        assert!(matches!(err, UpdateClientError::SignatureSchemeUnsupported));
    }

    #[test]
    fn real_key_verifies_real_signature() {
        let mut csprng = OsRng;
        let signing = SigningKey::generate(&mut csprng);
        let verifying_bytes = signing.verifying_key().to_bytes();
        let pubkey = VerifyingKeyBytes::from(verifying_bytes);

        let manifest = fake_manifest_str();
        let signature = signing.sign(manifest.as_bytes());
        let hex: String = signature.to_bytes().iter().map(|b| format!("{b:02x}")).collect();

        verify_manifest_signature(manifest, &hex, pubkey).expect("must verify");
    }

    #[test]
    fn rejects_tampered_manifest() {
        let mut csprng = OsRng;
        let signing = SigningKey::generate(&mut csprng);
        let pubkey = VerifyingKeyBytes::from(signing.verifying_key().to_bytes());

        let original = fake_manifest_str();
        let signature = signing.sign(original.as_bytes());
        let hex: String = signature.to_bytes().iter().map(|b| format!("{b:02x}")).collect();

        let tampered = original.replace("0.2.0", "9.9.9");
        let err =
            verify_manifest_signature(&tampered, &hex, pubkey).unwrap_err();
        assert!(matches!(err, UpdateClientError::SignatureInvalid));
    }

    #[test]
    fn rejects_short_hex() {
        let mut csprng = OsRng;
        let signing = SigningKey::generate(&mut csprng);
        let pubkey = VerifyingKeyBytes::from(signing.verifying_key().to_bytes());

        let err = verify_manifest_signature(fake_manifest_str(), "abc", pubkey).unwrap_err();
        assert!(matches!(err, UpdateClientError::SignatureInvalid));
    }

    #[test]
    fn hex_value_decodes_all_digits() {
        for c in b'0'..=b'9' {
            assert!(hex_value(c).is_ok());
        }
        for c in b'a'..=b'f' {
            assert!(hex_value(c).is_ok());
        }
        for c in b'A'..=b'F' {
            assert!(hex_value(c).is_ok());
        }
        assert!(hex_value(b'g').is_err());
        assert!(hex_value(b' ').is_err());
    }
}
