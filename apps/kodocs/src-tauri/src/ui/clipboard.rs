// ---------------------------------------------------------------------------
// Clipboard actions
// ---------------------------------------------------------------------------

use super::KodocsApp;

impl KodocsApp {
    pub(super) fn cut_selection(&mut self) {
        let content = self.engine().cut();
        if !content.plain_text.is_empty() {
            self.engine().set_clipboard(content);
            let edit_result = tench_document_core::EditResult {
                document: self.engine().get_document().clone(),
                cursor: self.engine().get_cursor().clone(),
                selection: self.engine().get_selection().clone(),
                dirty: true,
            };
            self.state.apply_edit_result(edit_result);
            self.state.set_status("잘라내기");
            self.reset_cursor_blink();
        }
    }

    pub(super) fn copy_selection(&mut self) {
        let content = self.engine().copy();
        if !content.plain_text.is_empty() {
            self.engine().set_clipboard(content);
            self.state.set_status("복사됨");
            self.state.toast = Some(("복사됨".into(), 0.0));
        }
    }

    pub(super) fn paste_clipboard(&mut self) {
        if let Some(content) = self.engine().get_clipboard().clone() {
            let result = self.engine().paste(content);
            self.state.apply_edit_result(result);
            self.state.set_status("붙여넣기");
            self.reset_cursor_blink();
        }
    }
}
