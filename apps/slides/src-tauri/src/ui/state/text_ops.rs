use super::*;

impl SlidesState {
    pub fn begin_text_edit(&mut self, element_idx: usize) {
        let (text_len, font_size, bold, text_color) = {
            let Some(slide) = self.current_slide() else {
                return;
            };
            let Some(elem) = slide.elements.get(element_idx) else {
                return;
            };
            let len = elem.text.as_deref().map(|t| t.len()).unwrap_or(0);
            let is_title = elem.kind == "title";
            let size = if is_title { 24.0 } else { 16.0 };
            let is_bold = is_title;
            let color = if elem.fill.is_some() {
                Color::WHITE
            } else {
                Color::BLACK
            };
            (len, size, is_bold, color)
        };
        self.text_edit = TextEditState {
            editing: true,
            element_index: element_idx,
            cursor_pos: text_len,
            selection_start: None,
            font_size,
            bold,
            italic: false,
            text_color,
        };
    }

    pub fn end_text_edit(&mut self) {
        self.text_edit.editing = false;
    }

    /// Insert a character at the current cursor position in the text being edited.
    pub fn text_edit_insert_char(&mut self, c: char) {
        let idx = self.text_edit.element_index;
        let pos = self.text_edit.cursor_pos;
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                if let Some(text) = &mut elem.text {
                    let pos = pos.min(text.len());
                    text.insert(pos, c);
                    self.text_edit.cursor_pos = pos + c.len_utf8();
                }
            }
        }
        self.sync_content_from_slides();
    }

    /// Delete the character before the cursor (backspace) during text editing.
    pub fn text_edit_backspace(&mut self) {
        let idx = self.text_edit.element_index;
        let cursor_pos = self.text_edit.cursor_pos;
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                if let Some(text) = &mut elem.text {
                    if cursor_pos > 0 && !text.is_empty() {
                        let prev = text[..cursor_pos]
                            .char_indices()
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        text.drain(prev..cursor_pos);
                        self.text_edit.cursor_pos = prev;
                    }
                }
            }
        }
        self.sync_content_from_slides();
    }

    /// Delete the character at the cursor (delete key) during text editing.
    pub fn text_edit_delete(&mut self) {
        let idx = self.text_edit.element_index;
        let cursor_pos = self.text_edit.cursor_pos;
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                if let Some(text) = &mut elem.text {
                    if cursor_pos < text.len() {
                        let next = text[cursor_pos..]
                            .char_indices()
                            .nth(1)
                            .map(|(i, _)| cursor_pos + i)
                            .unwrap_or(text.len());
                        text.drain(cursor_pos..next);
                    }
                }
            }
        }
        self.sync_content_from_slides();
    }

    /// Move cursor left/right by one character.
    pub fn text_edit_move_cursor(&mut self, delta: isize) {
        if delta < 0 {
            let text_len = self
                .current_slide()
                .and_then(|s| s.elements.get(self.text_edit.element_index))
                .and_then(|e| e.text.as_deref())
                .map(|t| t.len())
                .unwrap_or(0);
            let text = self
                .current_slide()
                .and_then(|s| s.elements.get(self.text_edit.element_index))
                .and_then(|e| e.text.as_deref())
                .unwrap_or("");
            let pos = self.text_edit.cursor_pos;
            if pos > 0 {
                let prev = text[..pos]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                self.text_edit.cursor_pos = prev;
            }
            let _ = text_len;
        } else {
            let text = self
                .current_slide()
                .and_then(|s| s.elements.get(self.text_edit.element_index))
                .and_then(|e| e.text.as_deref())
                .unwrap_or("");
            let pos = self.text_edit.cursor_pos;
            if pos < text.len() {
                let next = text[pos..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| pos + i)
                    .unwrap_or(text.len());
                self.text_edit.cursor_pos = next;
            }
        }
    }

    pub fn update_element_text(&mut self, element_idx: usize, text: String) {
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(element_idx) {
                elem.text = Some(text);
            }
        }
        self.sync_content_from_slides();
    }
}
