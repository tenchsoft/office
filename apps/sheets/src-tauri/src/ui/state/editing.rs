// ---------------------------------------------------------------------------
// Cell editing actions
// ---------------------------------------------------------------------------

use super::{EditingCell, SheetsState};

impl SheetsState {
    /// Enter edit mode, replacing the current cell value.
    pub fn begin_edit_replace(&mut self) {
        let value = self
            .active_cell()
            .map(|c| c.value.clone())
            .unwrap_or_default();
        let original = value.clone();
        self.editing_cell = Some(EditingCell {
            row: self.selected_row,
            col: self.selected_col,
            draft: String::new(),
            cursor_pos: 0,
            original_value: original,
            is_formula_edit: false,
            autocomplete: None,
        });
        self.formula_refs.clear();
    }

    /// Enter edit mode in append mode (F2 / double-click), cursor at end.
    pub fn begin_edit_append(&mut self) {
        let value = self
            .active_cell()
            .map(|c| c.value.clone())
            .unwrap_or_default();
        let original = value.clone();
        let cursor_pos = value.len();
        let is_formula = value.starts_with('=');
        let mut edit = EditingCell {
            row: self.selected_row,
            col: self.selected_col,
            draft: value,
            cursor_pos,
            original_value: original,
            is_formula_edit: is_formula,
            autocomplete: None,
        };
        if is_formula {
            self.parse_formula_refs(&edit.draft);
            self.update_autocomplete(&mut edit);
        }
        self.editing_cell = Some(edit);
    }

    /// Commit the current edit to the cell.
    pub fn commit_edit(&mut self) -> bool {
        let Some(edit) = self.editing_cell.take() else {
            return false;
        };
        self.push_undo();
        if let Some(cell) = self
            .grid
            .get_mut(edit.row)
            .and_then(|r| r.get_mut(edit.col))
        {
            cell.value = edit.draft.clone();
            cell.is_formula = edit.draft.starts_with('=');
        }
        self.formula_draft = edit.draft;
        self.formula_refs.clear();
        self.sync_content_from_grid();
        true
    }

    /// Cancel the current edit, restoring the original value.
    pub fn cancel_edit(&mut self) -> bool {
        if self.editing_cell.take().is_some() {
            self.formula_draft = self
                .active_cell()
                .map(|c| c.value.clone())
                .unwrap_or_default();
            self.formula_refs.clear();
            return true;
        }
        false
    }

    /// Insert text at the cursor position in the editing draft.
    pub fn edit_insert_text(&mut self, text: &str) -> bool {
        let (draft, is_formula) = {
            let Some(edit) = self.editing_cell.as_mut() else {
                return false;
            };
            edit.draft.insert_str(edit.cursor_pos, text);
            edit.cursor_pos += text.len();
            edit.is_formula_edit = edit.draft.starts_with('=');
            self.formula_draft = edit.draft.clone();
            (edit.draft.clone(), edit.is_formula_edit)
        };
        if is_formula {
            self.parse_formula_refs(&draft);
            self.rebuild_autocomplete();
        } else {
            self.formula_refs.clear();
            if let Some(edit) = self.editing_cell.as_mut() {
                edit.autocomplete = None;
            }
        }
        true
    }

    /// Delete one character before the cursor (Backspace).
    pub fn edit_backspace(&mut self) -> bool {
        let (draft, is_formula) = {
            let Some(edit) = self.editing_cell.as_mut() else {
                return false;
            };
            if edit.cursor_pos == 0 {
                return false;
            }
            let prev = edit.draft[..edit.cursor_pos]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            edit.draft.drain(prev..edit.cursor_pos);
            edit.cursor_pos = prev;
            self.formula_draft = edit.draft.clone();
            (edit.draft.clone(), edit.is_formula_edit)
        };
        if is_formula {
            self.parse_formula_refs(&draft);
            self.rebuild_autocomplete();
        }
        true
    }

    /// Delete one character after the cursor (Delete key in edit mode).
    pub fn edit_delete_forward(&mut self) -> bool {
        let (draft, is_formula) = {
            let Some(edit) = self.editing_cell.as_mut() else {
                return false;
            };
            if edit.cursor_pos >= edit.draft.len() {
                return false;
            }
            let next = edit.draft[..edit.cursor_pos]
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(edit.draft.len());
            edit.draft.drain(edit.cursor_pos..next);
            self.formula_draft = edit.draft.clone();
            (edit.draft.clone(), edit.is_formula_edit)
        };
        if is_formula {
            self.parse_formula_refs(&draft);
            self.rebuild_autocomplete();
        }
        true
    }

    /// Move cursor left/right within the editing draft.
    pub fn edit_move_cursor(&mut self, delta: isize) -> bool {
        let Some(edit) = self.editing_cell.as_mut() else {
            return false;
        };
        let chars: Vec<(usize, char)> = edit.draft.char_indices().collect();
        let current_char_idx = chars
            .iter()
            .position(|(i, _)| *i >= edit.cursor_pos)
            .unwrap_or(chars.len());
        let new_char_idx = if delta > 0 {
            (current_char_idx + 1).min(chars.len())
        } else {
            current_char_idx.saturating_sub(1)
        };
        let new_pos = chars
            .get(new_char_idx)
            .map(|(i, _)| *i)
            .unwrap_or(edit.draft.len());
        if new_pos == edit.cursor_pos {
            return false;
        }
        edit.cursor_pos = new_pos;
        true
    }

    /// Move cursor to start of draft.
    pub fn edit_cursor_home(&mut self) -> bool {
        let Some(edit) = self.editing_cell.as_mut() else {
            return false;
        };
        if edit.cursor_pos == 0 {
            return false;
        }
        edit.cursor_pos = 0;
        true
    }

    /// Move cursor to end of draft.
    pub fn edit_cursor_end(&mut self) -> bool {
        let Some(edit) = self.editing_cell.as_mut() else {
            return false;
        };
        let len = edit.draft.len();
        if edit.cursor_pos == len {
            return false;
        }
        edit.cursor_pos = len;
        true
    }
}
