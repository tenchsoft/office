// ---------------------------------------------------------------------------
// Cursor and tab actions
// ---------------------------------------------------------------------------

use tench_document_core::MoveDirection;

use super::KodocsApp;

impl KodocsApp {
    pub(super) fn reset_cursor_blink(&mut self) {
        self.state.cursor_visible = true;
        self.cursor_timer.reset();
    }

    pub(super) fn extend_selection(&mut self, direction: MoveDirection) {
        let result = self.engine().move_cursor(direction);
        self.state.apply_edit_result(result);
        self.reset_cursor_blink();
    }

    pub(super) fn switch_to_tab(&mut self, idx: usize) {
        if idx >= self.state.open_tabs.len() || idx == self.state.active_tab_idx {
            return;
        }
        self.state.active_tab_idx = idx;
    }

    pub(super) fn close_tab(&mut self, idx: usize) {
        if self.state.open_tabs.len() <= 1 || idx >= self.state.open_tabs.len() {
            return;
        }
        self.state.open_tabs.remove(idx);
        if self.state.active_tab_idx >= self.state.open_tabs.len() {
            self.state.active_tab_idx = self.state.open_tabs.len() - 1;
        } else if idx < self.state.active_tab_idx {
            self.state.active_tab_idx -= 1;
        }
    }
}
