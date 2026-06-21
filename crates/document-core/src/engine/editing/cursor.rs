use super::*;

impl DocumentEngine {
    pub fn move_cursor(&mut self, direction: MoveDirection) -> EditResult {
        self.selection = None;
        self.ensure_block();

        let block_text = self.block_text(self.cursor.block_idx);
        let text_len = block_text.len();

        match direction {
            MoveDirection::Left => {
                if self.cursor.offset > 0 {
                    let prev = block_text[..self.cursor.offset]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.cursor.offset = prev;
                } else if self.cursor.block_idx > 0 {
                    self.cursor.block_idx -= 1;
                    self.cursor.offset = self.block_text(self.cursor.block_idx).len();
                }
            }
            MoveDirection::Right => {
                if self.cursor.offset < text_len {
                    let next = block_text[self.cursor.offset..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.cursor.offset + i)
                        .unwrap_or(text_len);
                    self.cursor.offset = next;
                } else if self.cursor.block_idx + 1 < self.document.content.len() {
                    self.cursor.block_idx += 1;
                    self.cursor.offset = 0;
                }
            }
            MoveDirection::Up => {
                if self.cursor.block_idx > 0 {
                    self.cursor.block_idx -= 1;
                    self.cursor.offset = self
                        .cursor
                        .offset
                        .min(self.block_text(self.cursor.block_idx).len());
                }
            }
            MoveDirection::Down => {
                if self.cursor.block_idx + 1 < self.document.content.len() {
                    self.cursor.block_idx += 1;
                    self.cursor.offset = self
                        .cursor
                        .offset
                        .min(self.block_text(self.cursor.block_idx).len());
                }
            }
            MoveDirection::Home => {
                self.cursor.offset = 0;
            }
            MoveDirection::End => {
                self.cursor.offset = text_len;
            }
            MoveDirection::WordLeft => {
                let text = &block_text[..self.cursor.offset];
                let trimmed = text.trim_end();
                if trimmed.is_empty() {
                    self.cursor.offset = 0;
                } else {
                    let word_start = trimmed
                        .char_indices()
                        .rev()
                        .skip(1)
                        .find(|(_, c)| c.is_whitespace())
                        .map(|(i, _)| i + 1)
                        .unwrap_or(0);
                    self.cursor.offset = word_start;
                }
            }
            MoveDirection::WordRight => {
                let rest = &block_text[self.cursor.offset..];
                let trimmed = rest.trim_start();
                if trimmed.is_empty() {
                    self.cursor.offset = text_len;
                } else {
                    let word_end = trimmed
                        .char_indices()
                        .skip(1)
                        .find(|(_, c)| c.is_whitespace())
                        .map(|(i, _)| self.cursor.offset + (rest.len() - trimmed.len()) + i)
                        .unwrap_or(text_len);
                    self.cursor.offset = word_end;
                }
            }
            MoveDirection::DocStart => {
                self.cursor.block_idx = 0;
                self.cursor.offset = 0;
            }
            MoveDirection::DocEnd => {
                let last = self.document.content.len().saturating_sub(1);
                self.cursor.block_idx = last;
                self.cursor.offset = self.block_text(last).len();
            }
        }

        self.make_result()
    }

    pub fn select(&mut self, start: CursorState, end: CursorState) -> EditResult {
        self.selection = Some(SelectionRange {
            start,
            end: end.clone(),
        });
        self.cursor = end;
        self.make_result()
    }

    pub fn select_all(&mut self) -> EditResult {
        let last = self.document.content.len().saturating_sub(1);
        let end_offset = self.block_text(last).len();
        self.selection = Some(SelectionRange {
            start: CursorState {
                block_idx: 0,
                offset: 0,
            },
            end: CursorState {
                block_idx: last,
                offset: end_offset,
            },
        });
        self.cursor = CursorState {
            block_idx: last,
            offset: end_offset,
        };
        self.make_result()
    }

    pub fn click_position(&mut self, _x: f64, _y: f64, _layout_info: &LayoutInfo) -> EditResult {
        // Placeholder: actual click-to-cursor mapping requires layout info
        // from Parley. For now, do nothing.
        self.make_result()
    }
}
