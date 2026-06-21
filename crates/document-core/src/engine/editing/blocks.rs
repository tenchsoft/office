use super::*;

impl DocumentEngine {
    pub fn set_block_type(&mut self, block_type: BlockType) -> EditResult {
        self.push_undo();
        self.ensure_block();

        let idx = self.cursor.block_idx;
        let text = self.block_text(idx);

        let new_block = match block_type {
            BlockType::Paragraph => BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text,
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            },
            BlockType::Heading(level) => BlockNode::Heading {
                level: level.clamp(1, 6),
                content: vec![InlineNode::Text {
                    text,
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            },
            BlockType::BlockQuote => BlockNode::BlockQuote {
                content: vec![BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text,
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                }],
            },
            BlockType::CodeBlock => BlockNode::CodeBlock {
                language: None,
                code: text,
            },
            BlockType::BulletList | BlockType::OrderedList | BlockType::TaskList => {
                BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text,
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                }
            }
            BlockType::Footnote => BlockNode::Footnote {
                number: 1,
                content: vec![InlineNode::Text {
                    text,
                    marks: Marks::default(),
                }],
            },
        };

        self.document.content[idx] = new_block;
        self.dirty = true;
        self.make_result()
    }

    /// Toggle a list type on the current block.
    /// If the block is already the specified list type, convert back to Paragraph.
    /// Otherwise, convert to the list type with the current text as the first item.
    pub fn toggle_list(&mut self, ordered: bool) -> EditResult {
        self.push_undo();
        self.ensure_block();

        let idx = self.cursor.block_idx;
        let text = self.block_text(idx);

        // Check if already the target list type
        let already_is = match &self.document.content[idx] {
            BlockNode::OrderedList { .. } if ordered => true,
            BlockNode::BulletList { .. } if !ordered => true,
            _ => false,
        };

        if already_is {
            // Toggle off → Paragraph
            self.document.content[idx] = BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text,
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            };
        } else {
            let item = ListItem {
                content: vec![InlineNode::Text {
                    text,
                    marks: Marks::default(),
                }],
                children: vec![],
            };
            if ordered {
                self.document.content[idx] = BlockNode::OrderedList {
                    items: vec![item],
                    start: 1,
                };
            } else {
                self.document.content[idx] = BlockNode::BulletList { items: vec![item] };
            }
        }

        self.dirty = true;
        self.make_result()
    }

    /// Toggle a task (checklist) on the current block.
    pub fn toggle_task_list(&mut self) -> EditResult {
        self.push_undo();
        self.ensure_block();

        let idx = self.cursor.block_idx;
        let text = self.block_text(idx);

        let already_is = matches!(&self.document.content[idx], BlockNode::TaskList { .. });

        if already_is {
            self.document.content[idx] = BlockNode::Paragraph {
                content: vec![InlineNode::Text {
                    text,
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            };
        } else {
            let item = TaskItem {
                checked: false,
                content: vec![InlineNode::Text {
                    text,
                    marks: Marks::default(),
                }],
                children: vec![],
            };
            self.document.content[idx] = BlockNode::TaskList { items: vec![item] };
        }

        self.dirty = true;
        self.make_result()
    }

    pub fn set_alignment(&mut self, alignment: Alignment) -> EditResult {
        self.push_undo();
        self.ensure_block();

        let idx = self.cursor.block_idx;
        set_block_alignment(&mut self.document.content[idx], alignment);
        self.dirty = true;
        self.make_result()
    }

    pub fn indent(&mut self) -> EditResult {
        self.push_undo();
        self.ensure_block();

        let idx = self.cursor.block_idx;
        adjust_block_indent(&mut self.document.content[idx], 36.0);
        self.dirty = true;
        self.make_result()
    }

    pub fn outdent(&mut self) -> EditResult {
        self.push_undo();
        self.ensure_block();

        let idx = self.cursor.block_idx;
        adjust_block_indent(&mut self.document.content[idx], -36.0);
        self.dirty = true;
        self.make_result()
    }

    /// Set paragraph indent directly (left indent in px).
    pub fn set_indent_left(&mut self, indent: f32) -> EditResult {
        self.push_undo();
        self.ensure_block();
        let idx = self.cursor.block_idx;
        set_block_indent_left(&mut self.document.content[idx], indent);
        self.dirty = true;
        self.make_result()
    }

    /// Set paragraph right indent directly (in px).
    pub fn set_indent_right(&mut self, indent: f32) -> EditResult {
        self.push_undo();
        self.ensure_block();
        let idx = self.cursor.block_idx;
        set_block_indent_right(&mut self.document.content[idx], indent);
        self.dirty = true;
        self.make_result()
    }

    /// Set paragraph first-line indent directly (in px).
    pub fn set_indent_first_line(&mut self, indent: f32) -> EditResult {
        self.push_undo();
        self.ensure_block();
        let idx = self.cursor.block_idx;
        set_block_indent_first_line(&mut self.document.content[idx], indent);
        self.dirty = true;
        self.make_result()
    }
}
