// ---------------------------------------------------------------------------
// Clipboard actions
// ---------------------------------------------------------------------------

use super::DocsApp;

impl DocsApp {
    pub(super) fn cut_selection(&mut self) {
        let content = self.engine().cut();
        if !content.plain_text.is_empty() {
            self.engine().set_clipboard(content.clone());
            self.state.clipboard_text = content.plain_text.clone();
            self.state.clipboard_node_count = content.tdm_nodes.len();
            let edit_result = tench_document_core::EditResult {
                document: self.engine().get_document().clone(),
                cursor: self.engine().get_cursor().clone(),
                selection: self.engine().get_selection().clone(),
                dirty: true,
            };
            self.state.apply_edit_result(edit_result);
            self.state.set_status("Cut to clipboard");
            self.reset_cursor_blink();
        }
    }

    pub(super) fn copy_selection(&mut self) {
        let content = self.engine().copy();
        if !content.plain_text.is_empty() {
            self.engine().set_clipboard(content.clone());
            self.state.clipboard_text = content.plain_text.clone();
            self.state.clipboard_node_count = content.tdm_nodes.len();
            self.state.set_status("Copied to clipboard");
            self.state.show_toast("Copied");
        }
    }

    pub(super) fn paste_clipboard(&mut self) {
        if let Some(content) = self.engine().get_clipboard().clone() {
            let result = self.engine().paste(content);
            self.state.apply_edit_result(result);
            self.state.set_status("Pasted from clipboard");
            self.reset_cursor_blink();
        }
    }
}
