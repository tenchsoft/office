use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// FX-DATA-001: Encryption Infrastructure
// ---------------------------------------------------------------------------

/// Magic prefix written before encrypted payloads so readers can distinguish
/// encrypted files from legacy plaintext JSON.
pub const ENCRYPTED_MAGIC: &[u8; 8] = b"TCHENC01";

pub(crate) const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

/// Error returned when decryption fails.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EncryptionError {
    /// The ciphertext is shorter than the required header (magic + nonce).
    DataTooShort,
    /// The ciphertext data does not start with the expected magic bytes.
    InvalidMagic,
    /// Failed to generate a random nonce.
    NonceGenerationFailed,
    /// AES-256-GCM encryption failed.
    EncryptionFailed,
    /// AES-256-GCM decryption failed (wrong key or corrupted data).
    DecryptionFailed,
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::DataTooShort => {
                write!(f, "ciphertext too short: missing magic header or nonce")
            }
            EncryptionError::InvalidMagic => {
                write!(f, "ciphertext does not have the expected magic prefix")
            }
            EncryptionError::NonceGenerationFailed => {
                write!(f, "failed to generate random nonce")
            }
            EncryptionError::EncryptionFailed => {
                write!(f, "AES-256-GCM encryption failed")
            }
            EncryptionError::DecryptionFailed => {
                write!(
                    f,
                    "AES-256-GCM decryption failed: wrong key or corrupted data"
                )
            }
        }
    }
}

impl std::error::Error for EncryptionError {}

/// Derives a 32-byte key from arbitrary input using SHA-256.
///
/// The input is hashed with SHA-256 to produce a uniformly distributed
/// 32-byte key suitable for use with AES-256-GCM.
pub fn derive_key(input: &[u8]) -> [u8; KEY_SIZE] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

/// AES-256-GCM encryptor with authenticated encryption.
///
/// Provides proper AEAD encryption using AES-256-GCM with a cryptographically
/// random 12-byte nonce. The nonce is stored prepended to the ciphertext.
pub struct Aes256GcmEncryptor {
    key: [u8; KEY_SIZE],
}

impl Aes256GcmEncryptor {
    /// Creates a new encryptor, deriving a 32-byte key from the provided input.
    pub fn new(key: &[u8]) -> Self {
        Self {
            key: derive_key(key),
        }
    }

    /// Encrypts `plaintext` using AES-256-GCM and prepending the magic prefix
    /// and a 12-byte random nonce.
    ///
    /// Output format: `[ENCRYPTED_MAGIC][12-byte nonce][ciphertext + auth tag]`
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let nonce_bytes = generate_nonce()?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|_| EncryptionError::EncryptionFailed)?;

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| EncryptionError::EncryptionFailed)?;

        let mut output = Vec::with_capacity(ENCRYPTED_MAGIC.len() + NONCE_SIZE + ciphertext.len());
        output.extend_from_slice(ENCRYPTED_MAGIC);
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    /// Decrypts `ciphertext`. Returns an error if the data is too short,
    /// has an invalid magic prefix, or authentication fails.
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let header_len = ENCRYPTED_MAGIC.len() + NONCE_SIZE;
        if ciphertext.len() < header_len {
            return Err(EncryptionError::DataTooShort);
        }
        let (magic, rest) = ciphertext.split_at(ENCRYPTED_MAGIC.len());
        if magic != ENCRYPTED_MAGIC.as_slice() {
            return Err(EncryptionError::InvalidMagic);
        }
        let (nonce_bytes, encrypted) = rest.split_at(NONCE_SIZE);

        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|_| EncryptionError::DecryptionFailed)?;

        cipher
            .decrypt(nonce, encrypted)
            .map_err(|_| EncryptionError::DecryptionFailed)
    }
}

/// Generates a cryptographically random 12-byte nonce using `getrandom`.
fn generate_nonce() -> Result<[u8; NONCE_SIZE], EncryptionError> {
    let mut nonce = [0u8; NONCE_SIZE];
    getrandom::fill(&mut nonce).map_err(|_| EncryptionError::NonceGenerationFailed)?;
    Ok(nonce)
}

// ---------------------------------------------------------------------------
// EncryptionKey – machine-specific key derivation
// ---------------------------------------------------------------------------

/// A key derived from machine-specific information (hostname, username, and a
/// fixed salt) for use with at-rest encryption.
///
/// The key is deterministic on a given machine so that encrypted files can be
/// read back on the same device, but will differ across machines/users.
#[derive(Debug, Clone)]
pub struct EncryptionKey {
    raw: Vec<u8>,
}

impl EncryptionKey {
    /// Domain separator included in key derivation to prevent cross-purpose
    /// key reuse between different Tench subsystems.
    const SALT: &'static [u8] = b"tench-storage-core-encrypted-at-rest-v1";

    /// Derives an encryption key from the current machine's environment.
    ///
    /// Uses `HOME`/`USERPROFILE`, `USER`/`USERNAME`, and `HOSTNAME`/`COMPUTERNAME`
    /// environment variables combined with a fixed salt.
    pub fn from_machine() -> Self {
        let mut material = Vec::new();

        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();
        material.extend_from_slice(home.as_bytes());
        material.push(0); // separator

        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_default();
        material.extend_from_slice(user.as_bytes());
        material.push(0);

        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_default();
        material.extend_from_slice(hostname.as_bytes());
        material.push(0);

        material.extend_from_slice(Self::SALT);

        Self { raw: material }
    }

    /// Returns the derived 32-byte key.
    pub fn derived_key(&self) -> [u8; KEY_SIZE] {
        derive_key(&self.raw)
    }
}

// ---------------------------------------------------------------------------
// Convenience encrypt/decrypt functions
// ---------------------------------------------------------------------------

/// Encrypts `data` using AES-256-GCM with a key derived from the provided input.
///
/// The output is: `[ENCRYPTED_MAGIC][12-byte nonce][ciphertext + auth tag]`.
pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let encryptor = Aes256GcmEncryptor::new(key);
    encryptor.encrypt(data)
}

/// Decrypts `data` that was previously encrypted with [`encrypt_data`].
///
/// Returns the original plaintext bytes. Returns an error if the key is wrong
/// or the data is corrupted.
pub fn decrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let encryptor = Aes256GcmEncryptor::new(key);
    encryptor.decrypt(data)
}

/// Returns `true` if `data` starts with the encrypted payload magic bytes,
/// indicating it was written by the encrypted-at-rest write path.
pub fn is_encrypted_payload(data: &[u8]) -> bool {
    data.len() >= ENCRYPTED_MAGIC.len() && data[..ENCRYPTED_MAGIC.len()] == ENCRYPTED_MAGIC[..]
}
