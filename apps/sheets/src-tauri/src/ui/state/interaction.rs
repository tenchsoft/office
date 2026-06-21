use super::*;
use std::time::Instant;

impl SheetsState {
    /// Legacy single-character delete without undo.
    ///
    /// Retained for backward compatibility with existing call-sites.
    #[allow(dead_code)] // public API — legacy method kept for compatibility
    pub fn delete_from_active_cell(&mut self) -> bool {
        let Some(cell) = self.active_cell_mut() else {
            return false;
        };
        if cell.value.is_empty() {
            return false;
        }
        cell.value.pop();
        cell.is_formula = cell.value.starts_with('=');
        self.formula_draft = cell.value.clone();
        self.sync_content_from_grid();
        true
    }

    /// Legacy single-character append without undo.
    ///
    /// Retained for backward compatibility with existing call-sites.
    #[allow(dead_code)] // public API — legacy method kept for compatibility
    pub fn push_text_to_active_cell(&mut self, text: &str) -> bool {
        let Some(cell) = self.active_cell_mut() else {
            return false;
        };
        cell.value.push_str(text);
        cell.is_formula = cell.value.starts_with('=');
        self.formula_draft = cell.value.clone();
        self.sync_content_from_grid();
        true
    }

    pub fn open_modal(&mut self, modal_type: ModalType) {
        self.active_modal = Some(modal_type);
    }

    /// Set a toast notification with auto-dismiss timestamp.
    pub fn set_toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now()));
    }

    /// Copy the current selection to the system clipboard as TSV.
    /// Stub: copies to internal clipboard only. Tauri integration needed.
    pub fn copy_to_system_clipboard(&mut self) -> bool {
        self.edit_copy()
    }

    /// Paste from the system clipboard.
    /// Stub: pastes from internal clipboard only. Tauri integration needed.
    pub fn paste_from_system_clipboard(&mut self) -> bool {
        self.edit_paste()
    }
}
