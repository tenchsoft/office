// ---------------------------------------------------------------------------
// Sheet and document tab actions
// ---------------------------------------------------------------------------

use super::{DocumentSession, SheetsState};

impl SheetsState {
    /// Add a new sheet.
    pub fn add_sheet(&mut self) -> bool {
        let idx = self.sheet_names.len();
        self.sheet_names.push(format!("Sheet{}", idx + 1));
        true
    }

    /// Save the current session state for the active tab.
    pub fn save_current_session(&mut self) {
        if let Some(tab) = self.doc_tabs.get(self.active_tab_idx) {
            let session_id = tab.session_id.clone();
            let session = DocumentSession::capture(self);
            self.sessions.insert(session_id, session);
        }
    }

    /// Restore a session by session_id.
    pub fn restore_session(&mut self, session_id: &str) {
        let session = match self.sessions.get(session_id).cloned() {
            Some(s) => s,
            None => return,
        };
        session.restore(self);
        self.recalculate_status();
    }

    /// Switch to a document tab by index, saving current and restoring target.
    pub fn switch_to_tab(&mut self, tab_idx: usize) -> bool {
        if tab_idx >= self.doc_tabs.len() || tab_idx == self.active_tab_idx {
            return false;
        }
        self.save_current_session();
        self.active_tab_idx = tab_idx;
        let session_id = self.doc_tabs[tab_idx].session_id.clone();
        self.restore_session(&session_id);
        true
    }

    /// Duplicate the current sheet (copies grid to a new sheet).
    pub fn duplicate_sheet(&mut self, sheet_idx: usize) -> bool {
        if sheet_idx >= self.sheet_names.len() {
            return false;
        }
        let name = format!("{} (copy)", self.sheet_names[sheet_idx]);
        self.sheet_names.push(name);
        true
    }

    /// Move a sheet to a new position.
    pub fn move_sheet(&mut self, from: usize, to: usize) -> bool {
        if from >= self.sheet_names.len() || to >= self.sheet_names.len() || from == to {
            return false;
        }
        let name = self.sheet_names.remove(from);
        self.sheet_names.insert(to, name);
        if self.active_sheet == from {
            self.active_sheet = to;
        } else if from < self.active_sheet && to >= self.active_sheet {
            self.active_sheet -= 1;
        } else if from > self.active_sheet && to <= self.active_sheet {
            self.active_sheet += 1;
        }
        true
    }
}
