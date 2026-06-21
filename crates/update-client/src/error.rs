use std::fmt;

#[derive(Debug)]
pub enum UpdateClientError {
    Http(String),
    Json(serde_json::Error),
    BadStatus(u16, String),
    SignatureInvalid,
    SignatureSchemeUnsupported,
    ManifestUnparseable,
    PlatformNotFound,
    NetworkUnavailable,
}

impl fmt::Display for UpdateClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(m) => write!(f, "http error: {m}"),
            Self::Json(e) => write!(f, "json error: {e}"),
            Self::BadStatus(code, msg) => write!(f, "http {code}: {msg}"),
            Self::SignatureInvalid => write!(f, "manifest signature is invalid"),
            Self::SignatureSchemeUnsupported => {
                write!(f, "signature scheme is unsupported (version skew?)")
            }
            Self::ManifestUnparseable => write!(f, "manifest could not be parsed"),
            Self::PlatformNotFound => write!(f, "manifest has no entry for this platform"),
            Self::NetworkUnavailable => write!(f, "network unavailable"),
        }
    }
}

impl std::error::Error for UpdateClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for UpdateClientError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
