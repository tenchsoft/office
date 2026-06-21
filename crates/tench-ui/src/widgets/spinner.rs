//! Spinner widget.

use kurbo::{Axis, Point, Size};
use vello::Scene;

use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

pub struct Spinner {
    progress: f64,
}

impl Spinner {
    pub fn new() -> Self {
        Self { progress: 0.0 }
    }

    pub fn progress(mut self, progress: f64) -> Self {
        self.progress = progress.rem_euclid(1.0);
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spinner {
    fn measure(&mut self, _ctx: &mut MeasureCtx, _axis: Axis, _available: f64) -> f64 {
        24.0
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size.width.min(ctx.state.size.height);
        let center = Point::new(size / 2.0, size / 2.0);
        let mut p = Painter::new(scene);
        p.stroke_circle(center, size / 2.0 - 2.0, theme.disabled, 2.0);
        let angle = self.progress * std::f64::consts::TAU;
        let end = Point::new(
            center.x + angle.cos() * (size / 2.0 - 2.0),
            center.y + angle.sin() * (size / 2.0 - 2.0),
        );
        p.draw_line(center, end, theme.primary, 2.0);
    }
}
