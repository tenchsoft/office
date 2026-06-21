use tench_document_core::{DiagnosticSeverity, ImportExportDiagnostic};

/// Create a diagnostic entry with the given severity, code, message, and recoverability.
pub fn diagnostic(
    severity: DiagnosticSeverity,
    code: &str,
    message: &str,
    recoverable: bool,
) -> ImportExportDiagnostic {
    ImportExportDiagnostic {
        severity,
        code: code.to_string(),
        message: message.to_string(),
        location: None,
        recoverable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_creates_entry() {
        let d = diagnostic(
            DiagnosticSeverity::Warning,
            "test_code",
            "test message",
            true,
        );
        assert_eq!(d.severity, DiagnosticSeverity::Warning);
        assert_eq!(d.code, "test_code");
        assert_eq!(d.message, "test message");
        assert!(d.recoverable);
        assert!(d.location.is_none());
    }
}
