//! Cross-platform stable device identifier derivation.
//!
//! Strategy:
//! - Windows: `HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid`
//! - macOS:   `IOPlatformUUID` via `/usr/sbin/ioreg`
//! - Linux:   `/etc/machine-id` (fallback `/var/lib/dbus/machine-id`)
//!
//! The raw value is SHA-256 hashed to a 64-char hex string so the server never
//! sees the original OS identifier (privacy: server cannot reconstruct source).
//!
//! If every strategy fails, we return an ephemeral random identifier (prefixed
//! with `EPHEMERAL-`) which signals "cannot persist credentials" to the caller.
//! The ephemeral id is process-scoped only.

use crate::error::LicenseStoreError;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

/// Prefix for fallback identifiers that are not stable across process
/// restarts. The license store refuses to persist credentials when the
/// current device id starts with this prefix.
pub const EPHEMERAL_PREFIX: &str = "EPHEMERAL-";

static CACHE: OnceLock<String> = OnceLock::new();

/// Returns the stable device identifier for the current machine, as a
/// 64-character lowercase hex SHA-256 digest of the OS-native identifier.
///
/// Subsequent calls return the cached value (process-scoped).
///
/// If the OS identifier cannot be derived, returns an `EPHEMERAL-...` string
/// (unhashed, so the prefix survives `is_ephemeral_device_id` checks).
pub fn device_id() -> Result<String, LicenseStoreError> {
    if let Some(cached) = CACHE.get() {
        return Ok(cached.clone());
    }

    if let Ok(value) = std::env::var("TENCH_TEST_DEVICE_ID") {
        let id = hash_hex(value.as_bytes());
        let _ = CACHE.set(id.clone());
        return Ok(id);
    }

    let id = match read_os_identifier() {
        Ok(raw) => hash_hex(raw.as_bytes()),
        Err(_) => {
            let mut bytes = [0u8; 16];
            let _ = getrandom::fill(&mut bytes);
            let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
            format!("{EPHEMERAL_PREFIX}{hex}")
        }
    };
    let _ = CACHE.set(id.clone());
    Ok(id)
}

/// Returns true if the given device id is an ephemeral fallback (process-only).
pub fn is_ephemeral_device_id(id: &str) -> bool {
    id.starts_with(EPHEMERAL_PREFIX)
}

fn hash_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(target_os = "windows")]
fn read_os_identifier() -> Result<String, LicenseStoreError> {
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
        KEY_WOW64_64KEY, REG_SZ,
    };

    // MachineGuid lives in the native 64-bit registry view. Without
    // KEY_WOW64_64KEY a 32-bit process gets redirected to Wow6432Node and
    // misses the key.
    const ACCESS: u32 = KEY_READ | KEY_WOW64_64KEY;

    let sub_key: Vec<u16> = "SOFTWARE\\Microsoft\\Cryptography"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let value_name: Vec<u16> = "MachineGuid"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut handle: HKEY = std::ptr::null_mut();
    // SAFETY: Calling Win32 registry API with properly null-terminated UTF-16
    // strings. The handle is closed on every exit path.
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            sub_key.as_ptr(),
            0,
            ACCESS,
            &mut handle,
        )
    };
    if status != 0 {
        return Err(LicenseStoreError::DeviceIdUnreachable);
    }

    let mut len_bytes: u32 = 0;
    let mut reg_type: u32 = 0;
    // First call to discover size.
    let status = unsafe {
        RegQueryValueExW(
            handle,
            value_name.as_ptr(),
            std::ptr::null_mut(),
            &mut reg_type,
            std::ptr::null_mut(),
            &mut len_bytes,
        )
    };
    if status != 0 {
        unsafe { RegCloseKey(handle) };
        return Err(LicenseStoreError::DeviceIdUnreachable);
    }

    let mut buf = vec![0u8; len_bytes as usize];
    let status = unsafe {
        RegQueryValueExW(
            handle,
            value_name.as_ptr(),
            std::ptr::null_mut(),
            &mut reg_type,
            buf.as_mut_ptr(),
            &mut len_bytes,
        )
    };
    unsafe { RegCloseKey(handle) };
    if status != 0 || reg_type != REG_SZ {
        return Err(LicenseStoreError::DeviceIdUnreachable);
    }

    let utf16_units = (len_bytes as usize) / 2;
    let utf16: Vec<u16> = buf[..(utf16_units * 2)]
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    let trimmed: String = String::from_utf16_lossy(&utf16)
        .trim_end_matches('\u{0}')
        .trim()
        .to_string();
    if trimmed.is_empty() {
        Err(LicenseStoreError::DeviceIdUnreachable)
    } else {
        Ok(trimmed)
    }
}

#[cfg(target_os = "macos")]
fn read_os_identifier() -> Result<String, LicenseStoreError> {
    let output = std::process::Command::new("/usr/sbin/ioreg")
        .args(["-d2", "-c", "IOPlatformExpertDevice"])
        .output()
        .map_err(|_| LicenseStoreError::DeviceIdUnreachable)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("\"IOPlatformUUID\"") {
            // Format: "IOPlatformUUID" = "DEADCAFE-...."
            let after_eq = rest
                .split_once('=')
                .map(|(_, rhs)| rhs.trim())
                .unwrap_or("");
            let value = after_eq.trim_matches(|c: char| c == '"' || c.is_whitespace());
            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
    }
    Err(LicenseStoreError::DeviceIdUnreachable)
}

#[cfg(target_os = "linux")]
fn read_os_identifier() -> Result<String, LicenseStoreError> {
    for path in ["/etc/machine-id", "/var/lib/dbus/machine-id"] {
        if let Ok(bytes) = std::fs::read(path) {
            let s = String::from_utf8_lossy(&bytes).trim().to_string();
            if !s.is_empty() {
                return Ok(s);
            }
        }
    }
    Err(LicenseStoreError::DeviceIdUnreachable)
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn read_os_identifier() -> Result<String, LicenseStoreError> {
    Err(LicenseStoreError::DeviceIdUnreachable)
}
