// ---------------------------------------------------------------------------
// Find/replace state helpers
// ---------------------------------------------------------------------------

use super::document_text::extract_block_text;
use super::{state, DocsApp};

impl DocsApp {
    pub(super) fn open_find_replace(&mut self, show_replace: bool) {
        self.state.prepare_modal_open();
        let existing = self.state.find_replace.take();
        let mut fr = existing.unwrap_or_default();
        fr.show_replace = show_replace;
        self.state.find_replace = Some(fr);
        self.state.active_modal = None;
    }

    pub(super) fn refresh_find_matches(&mut self) {
        if let Some(fr) = self.state.find_replace.as_ref() {
            let query = fr.query.clone();
            let case_sensitive = fr.case_sensitive;
            let use_regex = fr.use_regex;
            let matches = self.engine().find(&query, case_sensitive, use_regex);
            let current_idx = if matches.is_empty() { None } else { Some(0) };
            if let Some(fr) = &mut self.state.find_replace {
                fr.matches = matches;
                fr.current_match_idx = current_idx;
            }
        }
    }

    pub(super) fn update_find_match_index(&mut self) {
        let (_, idx) = self.engine().get_search_state();
        if let Some(fr) = &mut self.state.find_replace {
            fr.current_match_idx = idx;
        }
    }

    /// Auto-scroll viewport to show the current find match.
    pub(super) fn auto_scroll_to_match(&mut self) {
        if let Some(fr) = &self.state.find_replace {
            if let Some(match_item) = fr.current_match_idx.and_then(|idx| fr.matches.get(idx)) {
                let scale = self.state.zoom / 100.0;
                let doc = self.state.current_document();
                let setup = &doc.page_setup;
                let (_page_w_raw, page_h_raw) = setup.page_size_px();
                let page_h = page_h_raw * scale;
                let line_h = 20.0 * scale;
                let header_h = state::HEADER_H * scale;
                let mm_to_px = 96.0 / 25.4;
                let margin_top = setup.margins.top as f64 * mm_to_px * scale;

                let mut y_offset = 0.0;
                for (i, block) in doc.content.iter().enumerate() {
                    if i == match_item.block_idx {
                        break;
                    }
                    let text = extract_block_text(block);
                    let lines = if text.is_empty() {
                        1
                    } else {
                        let content_w = 600.0 * scale;
                        let chars_per_line = ((content_w / (7.0 * scale)).max(1.0)) as usize;
                        text.chars().count().div_ceil(chars_per_line)
                    };
                    y_offset += lines as f64 * line_h;
                }

                let char_width = 7.0 * scale;
                let content_w = 600.0 * scale;
                let chars_per_line = ((content_w / char_width).max(1.0)) as usize;
                let match_line = match_item
                    .start_offset
                    .checked_div(chars_per_line)
                    .unwrap_or(0);
                y_offset += match_line as f64 * line_h;

                let total_y =
                    state::PAGE_MARGIN_Y + margin_top + header_h + y_offset + line_h * 2.0;
                let page_total = page_h + state::PAGE_MARGIN_Y * 2.0;
                let page_idx = (total_y / page_total) as usize;
                let target_scroll = page_idx as f64 * page_total;

                let visible_top = self.state.scroll_y;
                let visible_bottom = self.state.scroll_y + 600.0;
                if total_y < visible_top || total_y > visible_bottom {
                    self.state.scroll_y = target_scroll.max(0.0);
                }
            }
        }
    }
}
