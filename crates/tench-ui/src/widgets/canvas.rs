//! Canvas widget — a 2D drawing surface for custom rendering.

use kurbo::{Axis, Size};
use vello::Scene;

use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;

/// A custom drawing callback type.
pub type DrawCallback = Box<dyn FnMut(&mut Painter, Size) + Send>;

/// A 2D canvas for custom rendering via a callback.
pub struct Canvas {
    draw: DrawCallback,
    background: Option<crate::core::types::Color>,
}

impl Canvas {
    pub fn new(draw: impl FnMut(&mut Painter, Size) + Send + 'static) -> Self {
        Self {
            draw: Box::new(draw),
            background: None,
        }
    }

    pub fn with_background(mut self, color: crate::core::types::Color) -> Self {
        self.background = Some(color);
        self
    }
}

impl Widget for Canvas {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        let _ = axis;
        available
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let size = ctx.state.size;
        let mut painter = Painter::new(scene);

        // Optional background
        if let Some(bg) = self.background {
            painter.fill_background(size, bg);
        }

        // Invoke custom draw callback
        (self.draw)(&mut painter, size);
    }
}
