// ---------------------------------------------------------------------------
// Version history actions
// ---------------------------------------------------------------------------

use crate::document_service;

use super::state::VersionEntry;
use super::KodocsApp;

impl KodocsApp {
    pub(super) fn refresh_version_history(&mut self) {
        match document_service::get_recovery_documents() {
            Ok(snapshots) => {
                self.state.version_history = snapshots
                    .into_iter()
                    .map(|s| VersionEntry {
                        timestamp: 0,
                        path: s.recovery_path.clone(),
                        size_bytes: 0,
                        label: s
                            .original_path
                            .as_deref()
                            .unwrap_or(&s.recovery_path)
                            .to_string(),
                    })
                    .collect();
                if self.state.version_history.is_empty() {
                    self.state.toast = Some(("버전 기록이 없습니다".into(), 0.0));
                } else {
                    self.state.toast = Some((
                        format!("{}개 버전 발견", self.state.version_history.len()),
                        0.0,
                    ));
                }
            }
            Err(e) => {
                self.state.toast = Some((format!("버전 불러오기 실패: {e}"), 0.0));
            }
        }
    }
}
