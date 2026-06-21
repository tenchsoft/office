//! Avatar widget.

use kurbo::{Axis, Point, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::types::Color;
use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

pub struct Avatar {
    initials: String,
    color: Color,
    size: f64,
}

impl Avatar {
    pub fn new(initials: impl Into<String>) -> Self {
        Self {
            initials: initials.into(),
            color: Theme::default().primary,
            size: 36.0,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: f64) -> Self {
        self.size = size.max(12.0);
        self
    }
}

impl Widget for Avatar {
    fn measure(&mut self, _ctx: &mut MeasureCtx, _axis: Axis, _available: f64) -> f64 {
        self.size
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size.width.min(ctx.state.size.height);
        let mut p = Painter::new(scene);
        p.fill_circle(Point::new(size / 2.0, size / 2.0), size / 2.0, self.color);
        p.draw_text(
            &self.initials,
            size / 2.0 - 7.0,
            size / 2.0 + 5.0,
            theme.on_primary,
            (size * 0.36) as f32,
            FontWeight::BOLD,
            false,
        );
    }
}
