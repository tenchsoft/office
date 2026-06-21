//! Badge — small count/status indicator.

use crate::core::types::Color;
use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use kurbo::{Axis, Point, Rect, Size};

/// A small badge showing a count or status.
pub struct Badge {
    text: String,
    color: Color,
}

impl Badge {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Color::rgb8(220, 50, 50),
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl Widget for Badge {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, _available: f64) -> f64 {
        let char_count = self.text.len().max(1) as f64;
        match axis {
            Axis::Horizontal => char_count * 7.0 + 12.0,
            Axis::Vertical => 18.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut vello::Scene) {
        let size = ctx.size();
        let mut painter = Painter::new(scene);

        painter.fill_rounded_rect(
            Rect::from_origin_size(Point::ZERO, size),
            self.color,
            size.height / 2.0,
        );

        painter.draw_text(
            &self.text,
            6.0,
            size.height / 2.0 + 4.0,
            Color::WHITE,
            11.0,
            parley::FontWeight::BOLD,
            false,
        );
    }
}
