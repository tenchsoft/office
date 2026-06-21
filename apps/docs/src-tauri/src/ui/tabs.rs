use super::*;
use crate::document_service;

// ---------------------------------------------------------------------------
// Tab management
// ---------------------------------------------------------------------------

impl DocsApp {
    pub(super) fn append_new_document_tab(&mut self) {
        let opened = document_service::create_document(Some("Untitled Document".into()));
        let document = extract_tdm(&opened.content);
        let session = DocumentSession::new(document.clone(), "Untitled Document".into());
        self.sessions.push(session);
        self.active_session_idx = self.sessions.len() - 1;
        self.state.open_tabs.push(TabInfo {
            title: "Untitled Document".into(),
            dirty: false,
        });
        self.state.active_tab_idx = self.state.open_tabs.len() - 1;
        self.state.reset_for_new_document(document);
    }

    pub(super) fn replace_with_single_new_document(&mut self) {
        self.sessions.clear();
        self.state.open_tabs.clear();
        let opened = document_service::create_document(Some("Untitled Document".into()));
        let document = extract_tdm(&opened.content);
        let session = DocumentSession::new(document.clone(), "Untitled Document".into());
        self.sessions.push(session);
        self.active_session_idx = 0;
        self.state.open_tabs.push(TabInfo {
            title: "Untitled Document".into(),
            dirty: false,
        });
        self.state.active_tab_idx = 0;
        self.state.reset_for_new_document(document);
    }

    /// Open a file from a given path (used by drag-and-drop).
    pub(super) fn open_file_from_path(&mut self, path: &str) {
        match document_service::open_document(path.to_string()) {
            Ok(opened) => {
                let document = extract_tdm(&opened.content);
                // Extract filename from path for tab title
                let title = std::path::Path::new(path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .to_string();
                let session = DocumentSession::new(document.clone(), title.clone());
                self.sessions.push(session);
                self.active_session_idx = self.sessions.len() - 1;

                // Create fresh state with the new document applied
                let mut new_state = DocsState::new();
                new_state.apply_edit_result(tench_document_core::EditResult {
                    document,
                    cursor: CursorState::default(),
                    selection: None,
                    dirty: false,
                });
                new_state.open_tabs = self.state.open_tabs.clone();
                new_state.open_tabs.push(TabInfo {
                    title: title.clone(),
                    dirty: false,
                });
                new_state.active_tab_idx = new_state.open_tabs.len() - 1;
                self.state = new_state;
                self.state.show_toast(format!("Opened: {path}"));
            }
            Err(e) => {
                self.state.show_toast(format!("Failed to open file: {e}"));
            }
        }
    }

    pub(super) fn switch_to_tab(&mut self, idx: usize) {
        if idx >= self.state.open_tabs.len() || idx == self.state.active_tab_idx {
            return;
        }
        // Save current session's scroll position and dirty state
        if self.active_session_idx < self.sessions.len() {
            let current = &mut self.sessions[self.active_session_idx];
            current.scroll_y = self.state.scroll_y;
            current.dirty = self.state.is_dirty();
        }
        // Swap to the target session
        self.active_session_idx = idx;
        self.state.active_tab_idx = idx;
        // Restore the target session's state into DocsState
        let session = &self.sessions[idx];
        self.state.scroll_y = session.scroll_y;
        self.state.set_dirty(session.dirty);
        // Re-apply the session's document and cursor to state
        let doc = session.engine.get_document().clone();
        let cursor = session.engine.get_cursor().clone();
        let selection = session.engine.get_selection().clone();
        self.state
            .apply_edit_result(tench_document_core::EditResult {
                document: doc,
                cursor,
                selection,
                dirty: session.dirty,
            });
        self.state.show_toast(format!(
            "Switched to tab: {}",
            self.state.open_tabs[idx].title
        ));
    }

    pub(super) fn close_tab(&mut self, idx: usize) {
        if self.state.open_tabs.len() <= 1 {
            return; // Don't close the last tab
        }
        // Remove the session
        if idx < self.sessions.len() {
            self.sessions.remove(idx);
        }
        self.state.open_tabs.remove(idx);
        // Adjust active tab and session indices
        if self.state.active_tab_idx >= self.state.open_tabs.len() {
            self.state.active_tab_idx = self.state.open_tabs.len() - 1;
        } else if idx < self.state.active_tab_idx {
            self.state.active_tab_idx -= 1;
        } else if idx == self.state.active_tab_idx {
            // Closed the active tab: switch to the one now at this index
            self.state.active_tab_idx = idx.min(self.state.open_tabs.len() - 1);
        }
        self.active_session_idx = self.state.active_tab_idx;
        // Restore the now-active session's state
        if self.active_session_idx < self.sessions.len() {
            let session = &self.sessions[self.active_session_idx];
            self.state.scroll_y = session.scroll_y;
            self.state.set_dirty(session.dirty);
            let doc = session.engine.get_document().clone();
            let cursor = session.engine.get_cursor().clone();
            let selection = session.engine.get_selection().clone();
            self.state
                .apply_edit_result(tench_document_core::EditResult {
                    document: doc,
                    cursor,
                    selection,
                    dirty: session.dirty,
                });
        }
    }
}
