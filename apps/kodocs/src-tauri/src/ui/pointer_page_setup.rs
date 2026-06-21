// ---------------------------------------------------------------------------
// Pointer handlers for page setup
// ---------------------------------------------------------------------------

use tench_document_core::Orientation;
use tench_ui::prelude::*;

use super::state::PAPER_SIZES;
use super::KodocsApp;

impl KodocsApp {
    pub(super) fn handle_page_setup_click(&mut self, ctx: &mut EventCtx, x: f64, y: f64) -> bool {
        if self.state.page_setup_dialog.is_none() {
            return false;
        }
        let modal_w = 420.0;
        let modal_h = 380.0;
        let modal_x = self.state.last_window_size.0 / 2.0 - modal_w / 2.0;
        let modal_y = self.state.last_window_size.1 / 2.0 - modal_h / 2.0;
        let modal = Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);

        if self.handle_page_setup_footer_buttons(ctx, x, y, modal)
            || self.handle_page_setup_paper(ctx, x, y, modal)
            || self.handle_page_setup_orientation(ctx, x, y, modal)
            || self.handle_page_setup_margin_field(ctx, x, y, modal)
        {
            return true;
        }
        if x < modal.x0 || x > modal.x1 || y < modal.y0 || y > modal.y1 {
            self.state.page_setup_dialog = None;
        }
        ctx.request_paint();
        true
    }

    fn handle_page_setup_footer_buttons(
        &mut self,
        ctx: &mut EventCtx,
        x: f64,
        y: f64,
        modal: Rect,
    ) -> bool {
        let btn_y0 = modal.y1 - 44.0;
        let btn_y1 = modal.y1 - 16.0;
        if x >= modal.x1 - 160.0 && x <= modal.x1 - 90.0 && y >= btn_y0 && y <= btn_y1 {
            if let Some(ps) = self.state.page_setup_dialog.take() {
                let result = self.engine().set_page_setup(ps.to_page_setup());
                self.state.apply_edit_result(result);
            }
            ctx.request_paint();
            return true;
        }
        if x >= modal.x1 - 80.0 && x <= modal.x1 - 12.0 && y >= btn_y0 && y <= btn_y1 {
            self.state.page_setup_dialog = None;
            ctx.request_paint();
            return true;
        }
        false
    }

    fn handle_page_setup_paper(&mut self, ctx: &mut EventCtx, x: f64, y: f64, modal: Rect) -> bool {
        let paper_y = modal.y0 + 70.0;
        if x >= modal.x0 + 140.0 && x <= modal.x0 + 320.0 && y >= paper_y && y <= paper_y + 28.0 {
            if let Some(ps) = &mut self.state.page_setup_dialog {
                ps.paper_size_open = !ps.paper_size_open;
            }
            ctx.request_paint();
            return true;
        }
        if self
            .state
            .page_setup_dialog
            .as_ref()
            .is_some_and(|ps| ps.paper_size_open)
        {
            let dropdown_y = paper_y + 28.0;
            if x >= modal.x0 + 140.0 && x <= modal.x0 + 320.0 && y >= dropdown_y {
                let idx = ((y - dropdown_y) / 26.0) as usize;
                if let Some(size) = PAPER_SIZES.get(idx).copied() {
                    if let Some(ps) = &mut self.state.page_setup_dialog {
                        ps.paper_size = size;
                        ps.paper_size_open = false;
                    }
                }
                ctx.request_paint();
                return true;
            }
        }
        false
    }

    fn handle_page_setup_orientation(
        &mut self,
        ctx: &mut EventCtx,
        x: f64,
        y: f64,
        modal: Rect,
    ) -> bool {
        let orient_y = modal.y0 + 110.0;
        if y < orient_y || y > orient_y + 28.0 {
            return false;
        }
        for (offset, orientation) in [
            (140.0, Orientation::Portrait),
            (240.0, Orientation::Landscape),
        ] {
            if x >= modal.x0 + offset && x <= modal.x0 + offset + 80.0 {
                if let Some(ps) = &mut self.state.page_setup_dialog {
                    ps.orientation = orientation;
                }
                ctx.request_paint();
                return true;
            }
        }
        false
    }

    fn handle_page_setup_margin_field(
        &mut self,
        ctx: &mut EventCtx,
        x: f64,
        y: f64,
        modal: Rect,
    ) -> bool {
        let margin_start_y = modal.y0 + 160.0;
        let margin_field_x = modal.x0 + 140.0;
        for i in 0..4 {
            let field_y = margin_start_y + i as f64 * 36.0;
            if x >= margin_field_x
                && x <= margin_field_x + 80.0
                && y >= field_y
                && y <= field_y + 28.0
            {
                if let Some(ps) = &mut self.state.page_setup_dialog {
                    ps.editing_margin_field = Some(i);
                    ps.margin_edit_buffer = match i {
                        0 => format!("{:.1}", ps.margin_top),
                        1 => format!("{:.1}", ps.margin_bottom),
                        2 => format!("{:.1}", ps.margin_left),
                        3 => format!("{:.1}", ps.margin_right),
                        _ => String::new(),
                    };
                }
                ctx.request_paint();
                return true;
            }
        }
        false
    }
}
