//! Simple table widget.

use kurbo::{Axis, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    row_height: f64,
}

impl Table {
    pub fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        Self {
            headers,
            rows,
            row_height: 28.0,
        }
    }

    pub fn row_height(mut self, row_height: f64) -> Self {
        self.row_height = row_height.max(20.0);
        self
    }
}

impl Widget for Table {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => self.row_height * (self.rows.len() as f64 + 1.0),
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut p = Painter::new(scene);
        let columns = self.headers.len().max(1);
        let col_w = size.width / columns as f64;
        p.fill_rect(
            Rect::new(0.0, 0.0, size.width, self.row_height),
            theme.surface,
        );
        for (idx, header) in self.headers.iter().enumerate() {
            p.draw_text(
                header,
                idx as f64 * col_w + 8.0,
                18.0,
                theme.secondary,
                theme.font_size_small,
                FontWeight::BOLD,
                false,
            );
        }
        for (row_idx, row) in self.rows.iter().enumerate() {
            let y = self.row_height * (row_idx as f64 + 1.0);
            if y > size.height {
                break;
            }
            p.draw_line(
                kurbo::Point::new(0.0, y),
                kurbo::Point::new(size.width, y),
                theme.border,
                0.5,
            );
            for (col_idx, value) in row.iter().take(columns).enumerate() {
                p.draw_text(
                    value,
                    col_idx as f64 * col_w + 8.0,
                    y + 18.0,
                    theme.on_surface,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
            }
        }
    }
}
