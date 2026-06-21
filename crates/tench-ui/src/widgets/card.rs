//! Card — elevated card with title and content area.

use crate::core::types::Color;
use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use kurbo::{Axis, Point, Rect, Size};

/// An elevated card with optional title.
pub struct Card {
    title: Option<String>,
    padding: f64,
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Card {
    pub fn new() -> Self {
        Self {
            title: None,
            padding: 16.0,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_padding(mut self, p: f64) -> Self {
        self.padding = p;
        self
    }
}

impl Widget for Card {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => {
                let base = self.padding * 2.0;
                let title_h = if self.title.is_some() { 24.0 } else { 0.0 };
                base + title_h + 60.0 // content estimate
            }
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut vello::Scene) {
        let theme = ctx.theme().clone();
        let size = ctx.size();
        let mut painter = Painter::new(scene);

        // Card background
        painter.fill_rounded_rect(
            Rect::from_origin_size(Point::ZERO, size),
            Color::rgb8(45, 45, 45),
            8.0,
        );

        // Subtle border
        painter.stroke_rounded_rect(
            Rect::from_origin_size(Point::ZERO, size),
            Color::rgb8(60, 60, 60),
            1.0,
            8.0,
        );

        // Title
        if let Some(ref title) = self.title {
            painter.draw_text(
                title,
                self.padding,
                self.padding + 14.0,
                theme.on_background,
                14.0,
                parley::FontWeight::BOLD,
                false,
            );
        }
    }
}
