use super::helpers::{
    block_text_of, block_text_of_from_node, html_escape, insert_inline_at, trim_block,
};
use super::*;

// ---------------------------------------------------------------------------
// Clipboard
// ---------------------------------------------------------------------------

impl DocumentEngine {
    /// Cut the current selection and return clipboard content.
    pub fn cut(&mut self) -> ClipboardContent {
        let content = self.copy();
        if self.selection.is_some() {
            self.push_undo();
            self.delete_selection_inner();
        }
        content
    }

    /// Copy the current selection and return clipboard content.
    pub fn copy(&self) -> ClipboardContent {
        let sel = match &self.selection {
            Some(s) => s,
            None => {
                return ClipboardContent {
                    tdm_nodes: Vec::new(),
                    html: String::new(),
                    plain_text: String::new(),
                }
            }
        };

        let start = &sel.start;
        let end = &sel.end;

        let mut tdm_nodes = Vec::new();
        let mut plain_parts = Vec::new();
        let mut html_parts = Vec::new();

        let block_start = start.block_idx.min(end.block_idx);
        let block_end = start.block_idx.max(end.block_idx);

        for bi in block_start..=block_end {
            if bi >= self.document.content.len() {
                break;
            }
            let block = &self.document.content[bi];
            let text = block_text_of(&self.document.content, bi);

            let (so, eo) = if block_start == block_end {
                (start.offset.min(end.offset), start.offset.max(end.offset))
            } else if bi == block_start {
                (start.offset.min(end.offset), text.len())
            } else if bi == block_end {
                (0, start.offset.max(end.offset))
            } else {
                (0, text.len())
            };

            let so = so.min(text.len());
            let eo = eo.min(text.len());

            let slice = &text[so..eo];
            plain_parts.push(slice.to_string());
            html_parts.push(format!("<p>{}</p>", html_escape(slice)));

            // Clone the block and trim its content
            let mut trimmed = block.clone();
            trim_block(&mut trimmed, so, eo);
            tdm_nodes.push(trimmed);
        }

        ClipboardContent {
            tdm_nodes,
            html: html_parts.join("\n"),
            plain_text: plain_parts.join("\n"),
        }
    }

    /// Paste clipboard content into the document.
    pub fn paste(&mut self, content: ClipboardContent) -> EditResult {
        if content.tdm_nodes.is_empty() && content.plain_text.is_empty() {
            return self.make_result();
        }

        self.push_undo();

        // Delete selection if present
        if self.selection.is_some() {
            self.delete_selection_inner();
        }

        // Prefer TDM nodes if available (preserves formatting)
        if !content.tdm_nodes.is_empty() {
            // Flatten all node texts and join with newlines
            let full_text: String = content
                .tdm_nodes
                .iter()
                .map(block_text_of_from_node)
                .collect::<Vec<_>>()
                .join("\n");
            if !full_text.is_empty() {
                self.insert_text_no_undo(&full_text);
            }
        } else {
            // Fall back to plain text
            self.insert_text_no_undo(&content.plain_text);
        }

        self.dirty = true;
        self.make_result()
    }

    /// Store clipboard content internally.
    pub fn set_clipboard(&mut self, content: ClipboardContent) {
        self.clipboard = Some(content);
    }

    /// Retrieve the last clipboard content.
    pub fn get_clipboard(&self) -> &Option<ClipboardContent> {
        &self.clipboard
    }

    /// Insert text without pushing an undo snapshot (used internally by paste).
    fn insert_text_no_undo(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.ensure_block();
        let bi = self.cursor.block_idx;
        let block = &mut self.document.content[bi];

        match block {
            BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
                let node = InlineNode::Text {
                    text: text.to_string(),
                    marks: self.active_marks.clone(),
                };
                insert_inline_at(content, self.cursor.offset, node);
                self.cursor.offset += text.len();
            }
            BlockNode::CodeBlock { code, .. } => {
                code.insert_str(self.cursor.offset, text);
                self.cursor.offset += text.len();
            }
            _ => {}
        }
        self.dirty = true;
    }
}
