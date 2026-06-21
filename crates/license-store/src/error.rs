use std::fmt;

#[derive(Debug)]
pub enum LicenseStoreError {
    Io(std::io::Error),
    Encryption(tench_storage_core::EncryptionError),
    Json(serde_json::Error),
    DeviceIdUnreachable,
    StoreCorrupted,
    AtomicRenameFailed,
}

impl fmt::Display for LicenseStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Encryption(e) => write!(f, "encryption error: {e}"),
            Self::Json(e) => write!(f, "serialization error: {e}"),
            Self::DeviceIdUnreachable => write!(
                f,
                "failed to derive a stable device identifier from the host OS"
            ),
            Self::StoreCorrupted => {
                write!(f, "license store file is corrupted and cannot be decoded")
            }
            Self::AtomicRenameFailed => write!(f, "atomic rename of license store failed"),
        }
    }
}

impl std::error::Error for LicenseStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Encryption(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for LicenseStoreError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<tench_storage_core::EncryptionError> for LicenseStoreError {
    fn from(e: tench_storage_core::EncryptionError) -> Self {
        Self::Encryption(e)
    }
}

impl From<serde_json::Error> for LicenseStoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
