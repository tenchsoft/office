// ---------------------------------------------------------------------------
// Document hit-testing helpers
// ---------------------------------------------------------------------------

use tench_document_core::CursorState;

use super::document_text::extract_block_text;
use super::{state, KodocsApp};

impl KodocsApp {
    /// Convert a click position to a CursorState.
    pub(super) fn click_to_cursor(&self, x: f64, y: f64, workspace_y: f64) -> Option<CursorState> {
        let scale = self.state.zoom / 100.0;
        let doc = self.state.current_document();
        let setup = &doc.page_setup;
        let (page_w_raw, _page_h_raw) = setup.page_size_px();
        let page_w = page_w_raw * scale;
        let mm_to_px = 96.0 / 25.4;
        let margin_left = setup.margins.left as f64 * mm_to_px * scale;
        let margin_right = setup.margins.right as f64 * mm_to_px * scale;
        let margin_top = setup.margins.top as f64 * mm_to_px * scale;

        let page_x = ((x - page_w / 2.0) / 2.0).max(state::PAGE_MARGIN_X);
        let page_y = workspace_y + state::PAGE_MARGIN_Y;

        if x < page_x || x > page_x + page_w || y < page_y {
            return None;
        }

        let content_left = page_x + margin_left;
        let header_h = state::HEADER_H * scale;
        let content_top = page_y + margin_top + header_h;

        if y < content_top {
            return Some(CursorState {
                block_idx: 0,
                offset: 0,
            });
        }

        let rel_y = y - content_top;
        let line_h = 20.0 * scale;
        let line_idx = (rel_y / line_h) as usize;

        let content_w = page_w - margin_left - margin_right;
        let char_width = 7.0 * scale;
        let rel_x = (x - content_left).max(0.0);
        let col = (rel_x / char_width) as usize;

        let mut accumulated_lines = 0usize;

        for (block_idx, block) in doc.content.iter().enumerate() {
            let text = extract_block_text(block);
            let chars_per_line = ((content_w / char_width).max(1.0)) as usize;
            let lines_in_block = if chars_per_line > 0 && !text.is_empty() {
                text.chars().count().div_ceil(chars_per_line)
            } else {
                1
            };

            if accumulated_lines + lines_in_block > line_idx {
                let local_line = line_idx - accumulated_lines;
                let char_idx = (local_line * chars_per_line + col).min(text.chars().count());
                let offset = text
                    .char_indices()
                    .nth(char_idx)
                    .map(|(i, _)| i)
                    .unwrap_or(text.len());
                return Some(CursorState { block_idx, offset });
            }
            accumulated_lines += lines_in_block;
        }

        let last = doc.content.len().saturating_sub(1);
        let text = if last < doc.content.len() {
            extract_block_text(&doc.content[last])
        } else {
            String::new()
        };
        Some(CursorState {
            block_idx: last,
            offset: text.len(),
        })
    }
}
