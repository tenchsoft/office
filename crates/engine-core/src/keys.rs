use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Manages API keys for different providers.
///
/// Keys are stored encrypted at rest using AES-256-GCM with a machine-specific
/// key derived from environment variables via SHA-256. The file on disk never
/// contains plaintext keys.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiKeyEntry {
    pub provider: String,
    /// The key is stored encrypted in the JSON file.
    /// When loaded, it's decrypted in memory.
    #[serde(
        serialize_with = "serialize_encrypted",
        deserialize_with = "deserialize_encrypted"
    )]
    pub key: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub organization: Option<String>,
}

/// API key manager that stores keys in a local file with encryption at rest.
#[derive(Clone, Debug)]
pub struct ApiKeyManager {
    keys: HashMap<String, ApiKeyEntry>,
    keys_dir: PathBuf,
}

impl ApiKeyManager {
    pub fn new(keys_dir: PathBuf) -> Self {
        let mut manager = Self {
            keys: HashMap::new(),
            keys_dir,
        };
        let _ = manager.load();
        manager
    }

    pub fn default_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".tench")
            .join("keys")
    }

    pub fn with_defaults() -> Self {
        Self::new(Self::default_dir())
    }

    pub fn set(&mut self, provider: &str, key: String, base_url: Option<String>) {
        self.keys.insert(
            provider.to_string(),
            ApiKeyEntry {
                provider: provider.to_string(),
                key,
                base_url,
                organization: None,
            },
        );
        if let Err(e) = self.save() {
            eprintln!("Warning: failed to save keys file: {e}");
        }
    }

    pub fn get(&self, provider: &str) -> Option<&str> {
        self.keys.get(provider).map(|e| e.key.as_str())
    }

    pub fn get_entry(&self, provider: &str) -> Option<&ApiKeyEntry> {
        self.keys.get(provider)
    }

    pub fn remove(&mut self, provider: &str) -> bool {
        let removed = self.keys.remove(provider).is_some();
        if removed {
            if let Err(e) = self.save() {
                eprintln!("Warning: failed to save keys file: {e}");
            }
        }
        removed
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    pub fn has_key(&self, provider: &str) -> bool {
        self.keys.contains_key(provider)
    }

    pub fn validate_key(&self, provider: &str) -> KeyValidationResult {
        match self.keys.get(provider) {
            None => KeyValidationResult::Missing,
            Some(entry) => {
                if entry.key.is_empty() {
                    KeyValidationResult::Empty
                } else if entry.key.len() < 8 {
                    KeyValidationResult::TooShort
                } else {
                    KeyValidationResult::Valid
                }
            }
        }
    }

    fn load(&mut self) -> Result<(), String> {
        let path = self.keys_file_path();
        if !path.exists() {
            return Ok(());
        }
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read keys file: {e}"))?;
        let keys: Vec<ApiKeyEntry> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse keys file: {e}"))?;
        self.keys = keys.into_iter().map(|k| (k.provider.clone(), k)).collect();
        Ok(())
    }

    fn save(&self) -> Result<(), String> {
        if !self.keys_dir.exists() {
            fs::create_dir_all(&self.keys_dir)
                .map_err(|e| format!("Failed to create keys dir: {e}"))?;
        }
        let entries: Vec<&ApiKeyEntry> = self.keys.values().collect();
        let content = serde_json::to_string_pretty(&entries)
            .map_err(|e| format!("Failed to serialize keys: {e}"))?;
        fs::write(self.keys_file_path(), content)
            .map_err(|e| format!("Failed to write keys file: {e}"))?;

        // Set file permissions to owner-only on Unix
        #[cfg(unix)]
        {
            let path = self.keys_file_path();
            let _ = set_owner_only_permissions(&path);
        }

        Ok(())
    }

    fn keys_file_path(&self) -> PathBuf {
        self.keys_dir.join("providers.json")
    }
}

/// Fixed salt for key derivation. Ensures domain separation.
const KEY_DERIVATION_SALT: &[u8] = b"tench-provider-key-encryption-v2";

/// Derive a machine-specific 32-byte AES key using SHA-256.
///
/// Uses a combination of HOME, USER, and HOSTNAME with a fixed salt to create
/// a key that is:
/// - Unique per machine/user
/// - Deterministic (same key on every run for the same user)
/// - Not stored anywhere on disk
fn derive_machine_key() -> [u8; 32] {
    let mut hasher = Sha256::new();

    // Mix in machine-specific environment variables
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();
    hasher.update(home.as_bytes());

    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_default();
    hasher.update(user.as_bytes());

    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_default();
    hasher.update(hostname.as_bytes());

    // Domain separator to prevent cross-purpose key reuse
    hasher.update(KEY_DERIVATION_SALT);

    hasher.finalize().into()
}

/// Generate a cryptographically random 12-byte nonce for AES-256-GCM.
fn generate_nonce() -> Result<[u8; 12], String> {
    let mut nonce = [0u8; 12];
    getrandom::fill(&mut nonce).map_err(|e| format!("Failed to generate nonce: {e}"))?;
    Ok(nonce)
}

/// Encrypt a plaintext string for storage using AES-256-GCM.
///
/// Format: `enc:v2:<base64(nonce || ciphertext_and_tag)>`
fn encrypt_for_storage(plaintext: &str) -> Result<String, String> {
    let key_bytes = derive_machine_key();
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to create cipher: {e}"))?;

    let nonce_bytes = generate_nonce()?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {e}"))?;

    // Prepend nonce to the ciphertext (nonce || ciphertext || tag)
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(format!("enc:v2:{}", BASE64.encode(&combined)))
}

/// Decrypt a string from storage.
///
/// Handles three formats:
/// - `enc:v2:` → AES-256-GCM (current)
/// - `enc:v1:` → Legacy XOR (re-encrypted on load)
/// - No prefix → Legacy plaintext (logged as warning, re-encrypted on load)
fn decrypt_from_storage(ciphertext: &str) -> Result<String, String> {
    if let Some(encoded) = ciphertext.strip_prefix("enc:v2:") {
        decrypt_v2(encoded)
    } else if let Some(encoded) = ciphertext.strip_prefix("enc:v1:") {
        decrypt_v1_and_reencrypt(encoded)
    } else {
        // Legacy unencrypted key — log warning and return plaintext
        // (will be re-encrypted on next save via serialize_encrypted)
        eprintln!(
            "Warning: found unencrypted API key in storage. \
             It will be re-encrypted with AES-256-GCM on next save."
        );
        Ok(ciphertext.to_string())
    }
}

/// Decrypt an AES-256-GCM (v2) encrypted string.
fn decrypt_v2(encoded: &str) -> Result<String, String> {
    let combined = BASE64
        .decode(encoded)
        .map_err(|e| format!("Base64 decode failed: {e}"))?;

    if combined.len() < 12 {
        return Err("Ciphertext too short: missing nonce".to_string());
    }

    let (nonce_bytes, encrypted) = combined.split_at(12);

    let key_bytes = derive_machine_key();
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to create cipher: {e}"))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, encrypted)
        .map_err(|e| format!("Decryption failed: {e}"))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8 in decrypted data: {e}"))
}

/// Decrypt a legacy XOR (v1) encrypted string and return the plaintext.
/// The caller (deserialize_encrypted) will re-encrypt with v2 on next save.
fn decrypt_v1_and_reencrypt(encoded: &str) -> Result<String, String> {
    let key_bytes = derive_machine_key_legacy();
    match base64_decode_legacy(encoded) {
        Ok(encrypted) => {
            let decrypted = xor_with_key(&encrypted, &key_bytes);
            String::from_utf8(decrypted)
                .map_err(|e| format!("Invalid UTF-8 in legacy decrypted data: {e}"))
        }
        Err(_) => Err("Legacy base64 decode failed".to_string()),
    }
}

/// Legacy key derivation using DefaultHasher (kept for v1 backward compatibility).
fn derive_machine_key_legacy() -> Vec<u8> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_default();
    home.hash(&mut hasher);

    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_default();
    user.hash(&mut hasher);

    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_default();
    hostname.hash(&mut hasher);

    "tench-provider-key-encryption-v1".hash(&mut hasher);

    let hash = hasher.finish();
    let hash_bytes = hash.to_le_bytes();
    (0..4).flat_map(|_| hash_bytes).collect()
}

/// Legacy XOR decrypt (kept for v1 backward compatibility only).
fn xor_with_key(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &byte)| byte ^ key[i % key.len()])
        .collect()
}

/// Legacy base64 decoder (kept for v1 backward compatibility only).
fn base64_decode_legacy(input: &str) -> Result<Vec<u8>, String> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let input = input.trim_end_matches('=');
    let mut result = Vec::new();
    let bytes: Vec<u8> = input
        .bytes()
        .filter_map(|b| {
            if b == b'=' {
                None
            } else {
                CHARS.iter().position(|&c| c == b).map(|p| p as u8)
            }
        })
        .collect();

    for chunk in bytes.chunks(4) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let b3 = if chunk.len() > 3 { chunk[3] as u32 } else { 0 };
        let triple = (b0 << 18) | (b1 << 12) | (b2 << 6) | b3;

        result.push(((triple >> 16) & 0xFF) as u8);
        if chunk.len() > 2 {
            result.push(((triple >> 8) & 0xFF) as u8);
        }
        if chunk.len() > 3 {
            result.push((triple & 0xFF) as u8);
        }
    }
    Ok(result)
}

/// Set file permissions to owner-only (0600) on Unix.
#[cfg(unix)]
fn set_owner_only_permissions(path: &PathBuf) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms).map_err(|e| format!("Failed to set key file permissions: {e}"))
}

/// Custom serialization: encrypt the key before writing to JSON.
fn serialize_encrypted<S: serde::Serializer>(key: &str, serializer: S) -> Result<S::Ok, S::Error> {
    let encrypted = encrypt_for_storage(key).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&encrypted)
}

/// Custom deserialization: decrypt the key after reading from JSON.
fn deserialize_encrypted<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let ciphertext = String::deserialize(deserializer)?;
    decrypt_from_storage(&ciphertext).map_err(serde::de::Error::custom)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyValidationResult {
    Valid,
    Missing,
    Empty,
    TooShort,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_manager_set_and_get() {
        let dir = std::env::temp_dir().join("tench_test_keys_set_get");
        let _ = fs::remove_dir_all(&dir);
        let mut manager = ApiKeyManager::new(dir.clone());
        manager.set("openai", "sk-test12345678".to_string(), None);
        assert_eq!(manager.get("openai"), Some("sk-test12345678"));
        assert_eq!(manager.get("anthropic"), None);

        // Verify the file on disk contains encrypted data
        let file_content = fs::read_to_string(dir.join("providers.json")).unwrap();
        assert!(
            !file_content.contains("sk-test12345678"),
            "Plaintext key should not appear in file"
        );
        assert!(
            file_content.contains("enc:v2:"),
            "File should contain encrypted prefix"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn key_manager_roundtrip_through_disk() {
        let dir = std::env::temp_dir().join("tench_test_keys_roundtrip");
        let _ = fs::remove_dir_all(&dir);

        // Set a key
        let mut manager = ApiKeyManager::new(dir.clone());
        manager.set("openai", "sk-test12345678".to_string(), None);
        drop(manager);

        // Reload from disk
        let manager2 = ApiKeyManager::new(dir.clone());
        assert_eq!(manager2.get("openai"), Some("sk-test12345678"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn key_manager_validate() {
        let dir = std::env::temp_dir().join("tench_test_keys_validate");
        let _ = fs::remove_dir_all(&dir);
        let mut manager = ApiKeyManager::new(dir.clone());
        assert_eq!(manager.validate_key("openai"), KeyValidationResult::Missing);
        manager.set("openai", "short".to_string(), None);
        assert_eq!(
            manager.validate_key("openai"),
            KeyValidationResult::TooShort
        );
        manager.set("openai", "sk-valid-long-key".to_string(), None);
        assert_eq!(manager.validate_key("openai"), KeyValidationResult::Valid);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let original = "sk-test-api-key-12345";
        let encrypted = encrypt_for_storage(original).unwrap();
        assert!(encrypted.starts_with("enc:v2:"));
        assert_ne!(encrypted, original);
        let decrypted = decrypt_from_storage(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn legacy_unencrypted_key_compatibility() {
        let legacy = "sk-legacy-key";
        let decrypted = decrypt_from_storage(legacy).unwrap();
        assert_eq!(
            decrypted, legacy,
            "Legacy keys should pass through unchanged"
        );
    }

    #[test]
    fn legacy_v1_key_can_be_decrypted() {
        // Encrypt with the legacy XOR method to simulate a v1 key
        let original = "sk-legacy-v1-key";
        let key = derive_machine_key_legacy();
        let encrypted = xor_with_key(original.as_bytes(), &key);
        let encoded = base64_encode_legacy(&encrypted);
        let v1_string = format!("enc:v1:{encoded}");

        let decrypted = decrypt_from_storage(&v1_string).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn derive_machine_key_is_deterministic() {
        let key1 = derive_machine_key();
        let key2 = derive_machine_key();
        assert_eq!(key1, key2, "Machine key should be deterministic");
        assert_eq!(key1.len(), 32, "Machine key should be 32 bytes");
    }

    #[test]
    fn plaintext_not_in_serialized_form() {
        let entry = ApiKeyEntry {
            provider: "test".into(),
            key: "sk-secret-key-12345".into(),
            base_url: None,
            organization: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(
            !json.contains("sk-secret-key-12345"),
            "Serialized form should not contain plaintext key"
        );
        assert!(
            json.contains("enc:v2:"),
            "Serialized form should contain encrypted prefix"
        );
    }

    #[test]
    fn different_encryptions_produce_different_ciphertexts() {
        let original = "sk-test-key";
        let encrypted_a = encrypt_for_storage(original).unwrap();
        let encrypted_b = encrypt_for_storage(original).unwrap();
        // Different nonces should produce different ciphertexts
        assert_ne!(encrypted_a, encrypted_b);
        // But both decrypt to the same plaintext
        assert_eq!(
            decrypt_from_storage(&encrypted_a).unwrap(),
            decrypt_from_storage(&encrypted_b).unwrap()
        );
    }

    /// Legacy base64 encoder (for v1 backward compatibility tests only).
    fn base64_encode_legacy(input: &[u8]) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();
        for chunk in input.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
            result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            if chunk.len() > 2 {
                result.push(CHARS[(triple & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }
        result
    }
}
