// ---------------------------------------------------------------------------
// Pointer handlers for find/replace and page setup
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;

use super::KodocsApp;

impl KodocsApp {
    pub(super) fn handle_find_replace_click(&mut self, ctx: &mut EventCtx, x: f64, y: f64) -> bool {
        if self.state.find_replace.is_none() {
            return false;
        }
        let modal_w = 400.0;
        let modal_h = if self
            .state
            .find_replace
            .as_ref()
            .is_some_and(|fr| fr.show_replace)
        {
            240.0
        } else {
            180.0
        };
        let modal_x = self.state.last_window_size.0 / 2.0 - modal_w / 2.0;
        let modal_y = self.state.last_window_size.1 / 2.0 - modal_h / 2.0;
        let btn_y = modal_y + 100.0;
        let btn_h = 28.0;

        if self.handle_find_replace_close(ctx, x, y, modal_x, modal_y, modal_w) {
            return true;
        }
        if self.handle_find_navigation_click(ctx, x, y, modal_x, btn_y, btn_h) {
            return true;
        }
        if self.handle_find_toggle_click(ctx, x, y, modal_x, btn_y, btn_h) {
            return true;
        }
        if self.handle_replace_click(ctx, x, y, modal_x, btn_y, btn_h) {
            return true;
        }
        if x < modal_x || x > modal_x + modal_w || y < modal_y || y > modal_y + modal_h {
            self.engine().clear_search();
            self.state.find_replace = None;
        }
        ctx.request_paint();
        true
    }

    fn handle_find_replace_close(
        &mut self,
        ctx: &mut EventCtx,
        x: f64,
        y: f64,
        modal_x: f64,
        modal_y: f64,
        modal_w: f64,
    ) -> bool {
        let close_x = modal_x + modal_w - 28.0;
        if x >= close_x && x <= close_x + 20.0 && y >= modal_y + 4.0 && y <= modal_y + 24.0 {
            self.engine().clear_search();
            self.state.find_replace = None;
            ctx.request_paint();
            return true;
        }
        false
    }

    fn handle_find_navigation_click(
        &mut self,
        ctx: &mut EventCtx,
        x: f64,
        y: f64,
        modal_x: f64,
        btn_y: f64,
        btn_h: f64,
    ) -> bool {
        for (offset, width, forward) in [(16.0, 80.0, true), (104.0, 80.0, false)] {
            if x >= modal_x + offset
                && x <= modal_x + offset + width
                && y >= btn_y
                && y <= btn_y + btn_h
            {
                self.refresh_find_matches();
                let result = if forward {
                    self.engine().find_next()
                } else {
                    self.engine().find_prev()
                };
                self.state.apply_edit_result(result);
                self.update_find_match_index();
                ctx.request_paint();
                return true;
            }
        }
        false
    }

    fn handle_find_toggle_click(
        &mut self,
        ctx: &mut EventCtx,
        x: f64,
        y: f64,
        modal_x: f64,
        btn_y: f64,
        btn_h: f64,
    ) -> bool {
        for (offset, regex) in [(196.0, false), (228.0, true)] {
            if x >= modal_x + offset
                && x <= modal_x + offset + 24.0
                && y >= btn_y
                && y <= btn_y + btn_h
            {
                if let Some(fr) = &mut self.state.find_replace {
                    if regex {
                        fr.use_regex = !fr.use_regex;
                    } else {
                        fr.case_sensitive = !fr.case_sensitive;
                    }
                    self.refresh_find_matches();
                }
                ctx.request_paint();
                return true;
            }
        }
        false
    }

    fn handle_replace_click(
        &mut self,
        ctx: &mut EventCtx,
        x: f64,
        y: f64,
        modal_x: f64,
        btn_y: f64,
        btn_h: f64,
    ) -> bool {
        if !self
            .state
            .find_replace
            .as_ref()
            .is_some_and(|fr| fr.show_replace)
        {
            return false;
        }
        let replace_y = btn_y + 36.0;
        if x >= modal_x + 16.0 && x <= modal_x + 96.0 && y >= replace_y && y <= replace_y + btn_h {
            self.refresh_find_matches();
            if let Some(replacement) = self
                .state
                .find_replace
                .as_ref()
                .map(|fr| fr.replacement.clone())
            {
                let result = self.engine().replace_next(&replacement);
                self.state.apply_edit_result(result);
            }
            ctx.request_paint();
            return true;
        }
        if x >= modal_x + 104.0 && x <= modal_x + 194.0 && y >= replace_y && y <= replace_y + btn_h
        {
            self.refresh_find_matches();
            let fr_data = self.state.find_replace.as_ref().map(|fr| {
                (
                    fr.query.clone(),
                    fr.replacement.clone(),
                    fr.case_sensitive,
                    fr.use_regex,
                )
            });
            if let Some((query, replacement, case_sensitive, use_regex)) = fr_data {
                let count =
                    self.engine()
                        .replace_all(&query, &replacement, case_sensitive, use_regex);
                self.state.toast = Some((format!("{}개 바꿈", count), 0.0));
            }
            ctx.request_paint();
            return true;
        }
        false
    }
}
