// ---------------------------------------------------------------------------
// Pointer handlers for popup/modal clicks
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;

use super::{equation_editor, KodocsApp};

impl KodocsApp {
    pub(super) fn handle_hanja_popup_click(&mut self, ctx: &mut EventCtx, x: f64, y: f64) -> bool {
        if self.state.hanja_popup.is_none() {
            return false;
        }
        if let Some(hanja_state) = self.state.hanja_popup.take() {
            let modal_w = 300.0;
            let row_h = 28.0;
            let visible = hanja_state.candidates.len().min(8);
            let modal_h = 68.0 + visible as f64 * row_h;
            let modal_x = self.state.last_window_size.0 / 2.0 - modal_w / 2.0;
            let modal_y = self.state.last_window_size.1 / 2.0 - modal_h / 2.0;

            if x >= modal_x + 8.0 && x <= modal_x + modal_w - 8.0 {
                let rel_y = y - modal_y - 40.0;
                if rel_y >= 0.0 {
                    let idx = (rel_y / row_h) as usize;
                    if let Some(candidate) =
                        hanja_state.candidates.get(idx).filter(|_| idx < visible)
                    {
                        let hanja_text = candidate.split('(').next().unwrap_or(candidate).trim();
                        let result = self.engine().insert_text(hanja_text);
                        self.state.apply_edit_result(result);
                        self.reset_cursor_blink();
                    }
                }
            }
        }
        ctx.request_paint();
        true
    }

    pub(super) fn handle_equation_editor_click(
        &mut self,
        ctx: &mut EventCtx,
        x: f64,
        y: f64,
    ) -> bool {
        if self.state.equation_editor.is_none() {
            return false;
        }
        let modal_w = 480.0;
        let modal_h = 280.0;
        let modal_x = self.state.last_window_size.0 / 2.0 - modal_w / 2.0;
        let modal_y = self.state.last_window_size.1 / 2.0 - modal_h / 2.0;

        let ok_rect = Rect::new(
            modal_x + modal_w - 160.0,
            modal_y + modal_h - 40.0,
            modal_x + modal_w - 90.0,
            modal_y + modal_h - 12.0,
        );
        if contains(ok_rect, x, y) {
            if let Some(eq_state) = self.state.equation_editor.take() {
                if !eq_state.input.is_empty() {
                    let preview = equation_editor::render_equation_preview(&eq_state.input);
                    let result = self.engine().insert_text(&preview);
                    self.state.apply_edit_result(result);
                    self.reset_cursor_blink();
                }
            }
            ctx.request_paint();
            return true;
        }

        let cancel_rect = Rect::new(
            modal_x + modal_w - 80.0,
            modal_y + modal_h - 40.0,
            modal_x + modal_w - 12.0,
            modal_y + modal_h - 12.0,
        );
        if contains(cancel_rect, x, y) {
            self.state.equation_editor = None;
            ctx.request_paint();
            return true;
        }

        if self.handle_equation_symbol_click(ctx, x, y, modal_x, modal_y) {
            return true;
        }
        if x < modal_x || x > modal_x + modal_w || y < modal_y || y > modal_y + modal_h {
            self.state.equation_editor = None;
        }
        ctx.request_paint();
        true
    }

    pub(super) fn handle_link_modal_click(&mut self, ctx: &mut EventCtx, x: f64, y: f64) -> bool {
        if self.state.link_modal.is_none() {
            return false;
        }
        let modal_w = 400.0;
        let modal_h = 160.0;
        let modal_x = self.state.last_window_size.0 / 2.0 - modal_w / 2.0;
        let modal_y = self.state.last_window_size.1 / 2.0 - modal_h / 2.0;
        let btn_y0 = modal_y + modal_h - 40.0;
        let btn_y1 = modal_y + modal_h - 12.0;

        if x >= modal_x + modal_w - 160.0
            && x <= modal_x + modal_w - 90.0
            && y >= btn_y0
            && y <= btn_y1
        {
            if let Some(link_state) = self.state.link_modal.take() {
                if !link_state.url.is_empty() {
                    let result = self.engine().insert_link(&link_state.url);
                    self.state.apply_edit_result(result);
                }
            }
            ctx.request_paint();
            return true;
        }
        if x >= modal_x + modal_w - 80.0
            && x <= modal_x + modal_w - 12.0
            && y >= btn_y0
            && y <= btn_y1
        {
            self.state.link_modal = None;
            ctx.request_paint();
            return true;
        }
        if x < modal_x || x > modal_x + modal_w || y < modal_y || y > modal_y + modal_h {
            self.state.link_modal = None;
        }
        ctx.request_paint();
        true
    }

    fn handle_equation_symbol_click(
        &mut self,
        ctx: &mut EventCtx,
        x: f64,
        y: f64,
        modal_x: f64,
        modal_y: f64,
    ) -> bool {
        let symbols = [
            "+", "-", "\u{00D7}", "\u{00F7}", "=", "\u{2260}", "\u{2264}", "\u{2265}", "\u{221A}",
            "\u{03C0}", "\u{2211}", "\u{222B}", "\u{00B2}", "\u{00B3}", "(", ")", "{", "}", "[",
            "]",
        ];
        let symbol_y = modal_y + 204.0;
        let (symbol_w, symbol_h, symbol_gap) = (28.0, 24.0, 4.0);
        for (i, sym) in symbols.iter().enumerate() {
            let col = i % 10;
            let row = i / 10;
            let sx = modal_x + 16.0 + col as f64 * (symbol_w + symbol_gap);
            let rect = Rect::new(
                sx,
                symbol_y + row as f64 * (symbol_h + symbol_gap),
                sx + symbol_w,
                symbol_y + row as f64 * (symbol_h + symbol_gap) + symbol_h,
            );
            if contains(rect, x, y) {
                if let Some(eq_state) = &mut self.state.equation_editor {
                    eq_state.input.insert_str(eq_state.cursor_pos, sym);
                    eq_state.cursor_pos += sym.len();
                }
                ctx.request_paint();
                return true;
            }
        }
        false
    }
}

fn contains(rect: Rect, x: f64, y: f64) -> bool {
    x >= rect.x0 && x <= rect.x1 && y >= rect.y0 && y <= rect.y1
}
