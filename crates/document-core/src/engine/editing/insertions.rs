use super::*;

impl DocumentEngine {
    pub fn insert_link(&mut self, href: &str) -> EditResult {
        if self.selection.is_none() {
            let display_text = href.trim();
            if display_text.is_empty() {
                return self.make_result();
            }

            self.push_undo();
            self.ensure_block();

            let block_idx = self.cursor.block_idx;
            let block_text = self.block_text(block_idx);
            let insert_pos = self.cursor.offset.min(block_text.len());
            let link = InlineNode::Link {
                href: href.to_string(),
                title: None,
                text: display_text.to_string(),
                marks: self.active_marks.clone(),
            };

            match &mut self.document.content[block_idx] {
                BlockNode::Paragraph { content, .. }
                | BlockNode::Heading { content, .. }
                | BlockNode::Footnote { content, .. } => {
                    insert_inline_at(content, insert_pos, link);
                }
                block => {
                    let before = &block_text[..insert_pos];
                    let after = &block_text[insert_pos..];
                    let mut content = Vec::new();
                    if !before.is_empty() {
                        content.push(InlineNode::Text {
                            text: before.to_string(),
                            marks: Marks::default(),
                        });
                    }
                    content.push(link);
                    if !after.is_empty() {
                        content.push(InlineNode::Text {
                            text: after.to_string(),
                            marks: Marks::default(),
                        });
                    }
                    *block = BlockNode::Paragraph {
                        content,
                        attrs: ParagraphAttrs::default(),
                    };
                }
            }

            self.cursor.offset = insert_pos + display_text.len();
            self.selection = None;
            self.dirty = true;
            return self.make_result();
        }
        self.push_undo();

        let sel = self.selection.take().expect("checked above");
        let (start, end) = if sel.start <= sel.end {
            (sel.start, sel.end)
        } else {
            (sel.end, sel.start)
        };

        let text = self.block_text(start.block_idx);
        let s = start.offset.min(text.len());
        let e = end.offset.min(text.len());
        let link_text = text[s..e].to_string();
        let before = &text[..s];
        let after = &text[e..];

        let new_content = vec![
            InlineNode::Text {
                text: before.to_string(),
                marks: Marks::default(),
            },
            InlineNode::Link {
                href: href.to_string(),
                title: None,
                text: link_text,
                marks: Marks::default(),
            },
            InlineNode::Text {
                text: after.to_string(),
                marks: Marks::default(),
            },
        ];

        self.document.content[start.block_idx] = BlockNode::Paragraph {
            content: new_content,
            attrs: ParagraphAttrs::default(),
        };
        self.cursor = CursorState {
            block_idx: start.block_idx,
            offset: e,
        };
        self.dirty = true;
        self.make_result()
    }

    pub fn insert_image(&mut self, src: ImageSource, width: f64, height: f64) -> EditResult {
        self.push_undo();
        self.ensure_block();
        let image_block = BlockNode::Image {
            source: src,
            alt: None,
            width: Some(width as f32),
            height: Some(height as f32),
        };
        self.document
            .content
            .insert(self.cursor.block_idx + 1, image_block);
        self.cursor.block_idx += 1;
        self.cursor.offset = 0;
        self.dirty = true;
        self.make_result()
    }

    /// Set the width and height of an image block at the given index.
    pub fn set_image_size(&mut self, block_idx: usize, width: f32, height: f32) -> EditResult {
        self.push_undo();
        if let Some(BlockNode::Image {
            source,
            alt,
            width: _,
            height: _,
        }) = self.document.content.get(block_idx).cloned()
        {
            self.document.content[block_idx] = BlockNode::Image {
                source,
                alt,
                width: Some(width),
                height: Some(height),
            };
            self.dirty = true;
        }
        self.make_result()
    }

    pub fn insert_horizontal_rule(&mut self) -> EditResult {
        self.push_undo();
        self.ensure_block();
        self.document
            .content
            .insert(self.cursor.block_idx + 1, BlockNode::HorizontalRule);
        self.cursor.block_idx += 1;
        self.cursor.offset = 0;
        self.dirty = true;
        self.make_result()
    }

    pub fn insert_page_break(&mut self) -> EditResult {
        self.push_undo();
        self.ensure_block();
        self.document
            .content
            .insert(self.cursor.block_idx + 1, BlockNode::PageBreak);
        self.cursor.block_idx += 1;
        self.cursor.offset = 0;
        self.dirty = true;
        self.make_result()
    }

    /// Insert a footnote block at the end of the document.
    pub fn insert_footnote(&mut self, number: u32) -> EditResult {
        self.push_undo();
        let footnote = BlockNode::Footnote {
            number,
            content: vec![],
        };
        self.document.content.push(footnote);
        self.dirty = true;
        self.make_result()
    }
}
