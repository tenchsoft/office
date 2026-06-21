// ---------------------------------------------------------------------------
// Find/replace state helpers
// ---------------------------------------------------------------------------

use super::KodocsApp;

impl KodocsApp {
    pub(super) fn open_find_replace(&mut self, show_replace: bool) {
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
}
