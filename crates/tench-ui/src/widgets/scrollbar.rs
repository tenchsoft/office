//! Scrollbar widget.

use kurbo::{Axis, Rect, Size};
use vello::Scene;

use crate::core::events::{PointerButton, PointerEvent};
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarOrientation {
    Vertical,
    Horizontal,
}

pub struct Scrollbar {
    orientation: ScrollbarOrientation,
    position: f64,
    viewport_fraction: f64,
    dragging: bool,
    on_change: Option<Box<dyn FnMut(f64) + Send>>,
}

impl Scrollbar {
    pub fn new(orientation: ScrollbarOrientation) -> Self {
        Self {
            orientation,
            position: 0.0,
            viewport_fraction: 0.2,
            dragging: false,
            on_change: None,
        }
    }

    pub fn position(mut self, position: f64) -> Self {
        self.position = position.clamp(0.0, 1.0);
        self
    }

    pub fn viewport_fraction(mut self, fraction: f64) -> Self {
        self.viewport_fraction = fraction.clamp(0.02, 1.0);
        self
    }

    pub fn on_change(mut self, f: impl FnMut(f64) + Send + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    fn set_from_pointer(&mut self, x: f64, y: f64, size: Size) {
        let length = match self.orientation {
            ScrollbarOrientation::Vertical => size.height,
            ScrollbarOrientation::Horizontal => size.width,
        };
        let coord = match self.orientation {
            ScrollbarOrientation::Vertical => y,
            ScrollbarOrientation::Horizontal => x,
        };
        self.position = (coord / length).clamp(0.0, 1.0);
        if let Some(on_change) = &mut self.on_change {
            on_change(self.position);
        }
    }
}

impl Widget for Scrollbar {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match (self.orientation, axis) {
            (ScrollbarOrientation::Vertical, Axis::Horizontal)
            | (ScrollbarOrientation::Horizontal, Axis::Vertical) => 10.0,
            _ => available,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut p = Painter::new(scene);
        p.fill_rounded_rect(
            Rect::new(0.0, 0.0, size.width, size.height),
            theme.background,
            5.0,
        );
        let thumb = match self.orientation {
            ScrollbarOrientation::Vertical => {
                let h = (size.height * self.viewport_fraction).max(20.0);
                let y = (size.height - h) * self.position;
                Rect::new(1.0, y, size.width - 1.0, y + h)
            }
            ScrollbarOrientation::Horizontal => {
                let w = (size.width * self.viewport_fraction).max(20.0);
                let x = (size.width - w) * self.position;
                Rect::new(x, 1.0, x + w, size.height - 1.0)
            }
        };
        p.fill_rounded_rect(thumb, theme.disabled, 5.0);
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Down(e) if e.button == PointerButton::Primary => {
                self.dragging = true;
                self.set_from_pointer(e.pos.x, e.pos.y, ctx.state.size);
            }
            PointerEvent::Move(e) if self.dragging => {
                self.set_from_pointer(e.pos.x, e.pos.y, ctx.state.size);
            }
            PointerEvent::Up(_) | PointerEvent::Leave => self.dragging = false,
            _ => {}
        }
    }
}
