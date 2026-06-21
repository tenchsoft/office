use super::*;

impl DocumentEngine {
    // ----- editing operations -----

    pub fn insert_text(&mut self, text: &str) -> EditResult {
        if text.is_empty() {
            return self.make_result();
        }

        // If there is a selection, delete it first.
        if self.selection.is_some() {
            self.delete_selection_inner();
        }

        self.push_undo();
        self.ensure_block();

        let block_text = self.block_text(self.cursor.block_idx);
        let insert_pos = self.cursor.offset.min(block_text.len());

        // Build the new inline content by splitting the existing text and
        // inserting the new characters with the current active marks.
        let new_text = format!(
            "{}{}{}",
            &block_text[..insert_pos],
            text,
            &block_text[insert_pos..]
        );
        self.document.content[self.cursor.block_idx] = BlockNode::Paragraph {
            content: vec![InlineNode::Text {
                text: new_text,
                marks: self.active_marks.clone(),
            }],
            attrs: ParagraphAttrs::default(),
        };
        self.cursor.offset = insert_pos + text.len();
        self.dirty = true;
        self.make_result()
    }

    pub fn backspace(&mut self) -> EditResult {
        self.ensure_block();

        // If selection exists, delete it.
        if self.selection.is_some() {
            self.push_undo();
            self.delete_selection_inner();
            self.dirty = true;
            return self.make_result();
        }

        let block_text = self.block_text(self.cursor.block_idx);
        if self.cursor.offset == 0 {
            // Merge with previous block if possible.
            if self.cursor.block_idx == 0 {
                return self.make_result();
            }
            self.push_undo();
            let prev_idx = self.cursor.block_idx - 1;
            let prev_text = self.block_text(prev_idx);
            let merged_offset = prev_text.len();
            let merged = format!("{}{}", prev_text, block_text);

            self.document.content[prev_idx] = BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: merged,
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            };
            self.document.content.remove(self.cursor.block_idx);
            self.cursor.block_idx = prev_idx;
            self.cursor.offset = merged_offset;
            self.dirty = true;
            return self.make_result();
        }

        // Delete one character before cursor within the same block.
        self.push_undo();
        let text = block_text;
        let offset = self.cursor.offset.min(text.len());
        if offset == 0 {
            return self.make_result();
        }
        let char_start = text[..offset]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        let new_text = format!("{}{}", &text[..char_start], &text[offset..]);
        self.document.content[self.cursor.block_idx] = BlockNode::Paragraph {
            content: vec![InlineNode::Text {
                text: new_text,
                marks: Marks::default(),
            }],
            attrs: ParagraphAttrs::default(),
        };
        self.cursor.offset = char_start;
        self.dirty = true;
        self.make_result()
    }

    pub fn delete_forward(&mut self) -> EditResult {
        self.ensure_block();

        if self.selection.is_some() {
            self.push_undo();
            self.delete_selection_inner();
            self.dirty = true;
            return self.make_result();
        }

        let block_idx = self.cursor.block_idx;
        let block_text = self.block_text(block_idx);
        let offset = self.cursor.offset.min(block_text.len());

        if offset < block_text.len() {
            self.push_undo();
            let next_end = block_text[offset..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| offset + i)
                .unwrap_or(block_text.len());
            let new_text = format!("{}{}", &block_text[..offset], &block_text[next_end..]);
            self.document.content[block_idx] = BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: new_text,
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            };
            self.cursor.offset = offset;
            self.dirty = true;
            return self.make_result();
        }

        if block_idx + 1 >= self.document.content.len() {
            return self.make_result();
        }

        self.push_undo();
        let next_text = self.block_text(block_idx + 1);
        let merged = format!("{}{}", block_text, next_text);
        self.document.content[block_idx] = BlockNode::Paragraph {
            content: vec![InlineNode::Text {
                text: merged,
                marks: Marks::default(),
            }],
            attrs: ParagraphAttrs::default(),
        };
        self.document.content.remove(block_idx + 1);
        self.cursor.offset = offset;
        self.dirty = true;
        self.make_result()
    }

    pub fn delete_selection(&mut self) -> EditResult {
        if self.selection.is_none() {
            return self.make_result();
        }
        self.push_undo();
        self.delete_selection_inner();
        self.dirty = true;
        self.make_result()
    }

    pub(in crate::engine) fn delete_selection_inner(&mut self) {
        let sel = match self.selection.take() {
            Some(s) => s,
            None => return,
        };

        let (start, end) = if sel.start <= sel.end {
            (sel.start, sel.end)
        } else {
            (sel.end, sel.start)
        };

        if start.block_idx == end.block_idx {
            let text = self.block_text(start.block_idx);
            let s_off = start.offset.min(text.len());
            let e_off = end.offset.min(text.len());
            let new_text = format!("{}{}", &text[..s_off], &text[e_off..]);
            self.document.content[start.block_idx] = BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: new_text,
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            };
            self.cursor = CursorState {
                block_idx: start.block_idx,
                offset: s_off,
            };
        } else {
            // Multi-block selection: merge start and end blocks, remove in-between.
            let start_text = self.block_text(start.block_idx);
            let end_text = self.block_text(end.block_idx);
            let s_off = start.offset.min(start_text.len());
            let e_off = end.offset.min(end_text.len());
            let merged = format!("{}{}", &start_text[..s_off], &end_text[e_off..]);

            self.document.content[start.block_idx] = BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text: merged,
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            };
            let remove_count = end.block_idx - start.block_idx;
            for _ in 0..remove_count {
                self.document.content.remove(start.block_idx + 1);
            }
            self.cursor = CursorState {
                block_idx: start.block_idx,
                offset: s_off,
            };
        }
    }
}
