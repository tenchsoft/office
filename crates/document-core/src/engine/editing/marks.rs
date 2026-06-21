use super::*;

impl DocumentEngine {
    pub fn toggle_mark(&mut self, mark: MarkType) -> EditResult {
        match mark {
            MarkType::Bold => self.active_marks.bold = !self.active_marks.bold,
            MarkType::Italic => self.active_marks.italic = !self.active_marks.italic,
            MarkType::Underline => self.active_marks.underline = !self.active_marks.underline,
            MarkType::Strikethrough => {
                self.active_marks.strikethrough = !self.active_marks.strikethrough;
            }
            MarkType::Superscript => {
                self.active_marks.superscript = !self.active_marks.superscript;
                if self.active_marks.superscript {
                    self.active_marks.subscript = false;
                }
            }
            MarkType::Subscript => {
                self.active_marks.subscript = !self.active_marks.subscript;
                if self.active_marks.subscript {
                    self.active_marks.superscript = false;
                }
            }
            MarkType::Code => self.active_marks.code = !self.active_marks.code,
        }

        // If there is a selection, apply the toggle to the selected text.
        if let Some(sel) = &self.selection.clone() {
            self.push_undo();
            let (start, end) = if sel.start <= sel.end {
                (sel.start.clone(), sel.end.clone())
            } else {
                (sel.end.clone(), sel.start.clone())
            };

            if start.block_idx == end.block_idx {
                let text = self.block_text(start.block_idx);
                let s = start.offset.min(text.len());
                let e = end.offset.min(text.len());
                let before = &text[..s];
                let target = &text[s..e];
                let after = &text[e..];

                let marks = self.active_marks.clone();

                let new_text = format!("{before}{target}{after}");
                self.document.content[start.block_idx] = BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text: new_text,
                        marks,
                    }],
                    attrs: ParagraphAttrs::default(),
                };
            }
            self.dirty = true;
        }

        self.make_result()
    }

    /// Clear all marks on the current selection (or cursor position).
    /// Resets bold, italic, underline, strikethrough, superscript, subscript,
    /// code, font_size, text_color, background_color, and font_family.
    pub fn clear_marks(&mut self) -> EditResult {
        self.active_marks = Marks::default();

        if let Some(sel) = &self.selection.clone() {
            self.push_undo();
            let (start, end) = if sel.start <= sel.end {
                (sel.start.clone(), sel.end.clone())
            } else {
                (sel.end.clone(), sel.start.clone())
            };

            if start.block_idx == end.block_idx {
                let text = self.block_text(start.block_idx);
                let s = start.offset.min(text.len());
                let e = end.offset.min(text.len());
                let before = &text[..s];
                let target = &text[s..e];
                let after = &text[e..];

                let new_text = format!("{before}{target}{after}");
                self.document.content[start.block_idx] = BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text: new_text,
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                };
            }
            self.dirty = true;
        }

        self.make_result()
    }

    pub fn set_font_size(&mut self, size: f32) -> EditResult {
        self.active_marks.font_size = Some(size);
        self.make_result()
    }

    pub fn set_font_family(&mut self, family: String) -> EditResult {
        if family == "Default" {
            self.active_marks.font_family = None;
        } else {
            self.active_marks.font_family = Some(family);
        }
        self.make_result()
    }

    pub fn set_text_color(&mut self, color: String) -> EditResult {
        self.active_marks.text_color = Some(color);
        self.make_result()
    }

    pub fn set_background_color(&mut self, color: String) -> EditResult {
        self.active_marks.background_color = Some(color);
        self.make_result()
    }
}
