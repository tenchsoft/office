// ---------------------------------------------------------------------------
// Version history actions
// ---------------------------------------------------------------------------

use crate::document_service;

use super::state::VersionEntry;
use super::DocsApp;

impl DocsApp {
    pub(super) fn refresh_version_history(&mut self) {
        match document_service::get_recovery_documents() {
            Ok(snapshots) => {
                self.state.version_history = snapshots
                    .into_iter()
                    .map(|s| {
                        let path = s.recovery_path.clone();
                        let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                        let timestamp_label =
                            s.saved_at.as_deref().unwrap_or("Unknown time").to_string();
                        VersionEntry {
                            timestamp_label,
                            path: path.clone(),
                            size_bytes,
                            label: s.original_path.as_deref().unwrap_or(&path).to_string(),
                        }
                    })
                    .collect();
                if self.state.version_history.is_empty() {
                    self.state.show_toast("No version history available");
                } else {
                    self.state.show_toast(format!(
                        "{} version(s) found",
                        self.state.version_history.len()
                    ));
                }
            }
            Err(e) => {
                self.state
                    .show_toast(format!("Failed to load versions: {e}"));
            }
        }
    }
}
