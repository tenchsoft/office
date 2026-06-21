use std::fmt;

/// Structured error type for office I/O operations.
///
/// Replaces ad-hoc `String` errors with categorized, descriptive variants
/// that can be matched on and formatted for user-facing messages.
#[derive(Debug)]
pub enum OfficeIoError {
    /// File system I/O failure (read, write, create directory, etc.).
    Io {
        context: String,
        source: std::io::Error,
    },
    /// Failed to parse a file (JSON, XML, ZIP, etc.).
    Parse {
        context: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Failed to serialize content to a target format.
    Serialize {
        context: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// The requested file or resource was not found.
    NotFound(String),
    /// Permission denied accessing a file or directory.
    Permission(String),
    /// A format or operation is not supported.
    Unsupported(String),
    /// A general operation failure with a descriptive message.
    General(String),
}

impl fmt::Display for OfficeIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OfficeIoError::Io { context, source } => {
                write!(f, "{context}: {source}")
            }
            OfficeIoError::Parse { context, source } => {
                write!(f, "{context}: {source}")
            }
            OfficeIoError::Serialize { context, source } => {
                write!(f, "{context}: {source}")
            }
            OfficeIoError::NotFound(msg) => write!(f, "Not found: {msg}"),
            OfficeIoError::Permission(msg) => write!(f, "Permission denied: {msg}"),
            OfficeIoError::Unsupported(msg) => write!(f, "Unsupported: {msg}"),
            OfficeIoError::General(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for OfficeIoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OfficeIoError::Io { source, .. } => Some(source),
            OfficeIoError::Parse { source, .. } => Some(source.as_ref()),
            OfficeIoError::Serialize { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<std::io::Error> for OfficeIoError {
    fn from(error: std::io::Error) -> Self {
        OfficeIoError::Io {
            context: "I/O error".to_string(),
            source: error,
        }
    }
}

impl From<OfficeIoError> for String {
    fn from(error: OfficeIoError) -> String {
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_displays_context_and_source() {
        let err = OfficeIoError::Io {
            context: "Failed to read file".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "no such file"),
        };
        let msg = err.to_string();
        assert!(msg.contains("Failed to read file"));
        assert!(msg.contains("no such file"));
    }

    #[test]
    fn not_found_error_formats() {
        let err = OfficeIoError::NotFound("document.docx".to_string());
        assert!(err.to_string().contains("document.docx"));
    }

    #[test]
    fn unsupported_error_formats() {
        let err = OfficeIoError::Unsupported("PDF import".to_string());
        assert!(err.to_string().contains("PDF import"));
    }

    #[test]
    fn error_converts_to_string() {
        let err = OfficeIoError::General("Something went wrong".to_string());
        let msg: String = err.into();
        assert_eq!(msg, "Something went wrong");
    }
}
