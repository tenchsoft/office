//! Toggle widget.

use kurbo::{Axis, Point, Rect, Size};
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

pub struct Toggle {
    checked: bool,
    hovered: bool,
    on_change: Option<Box<dyn FnMut(bool) + Send>>,
}

impl Toggle {
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            hovered: false,
            on_change: None,
        }
    }

    pub fn checked(&self) -> bool {
        self.checked
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    pub fn on_change(mut self, f: impl FnMut(bool) + Send + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl Widget for Toggle {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, _available: f64) -> f64 {
        match axis {
            Axis::Horizontal => 42.0,
            Axis::Vertical => 24.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut p = Painter::new(scene);
        let track = Rect::new(0.0, 0.0, size.width, size.height);
        let track_color = if self.checked {
            theme.primary
        } else if self.hovered {
            Color::lerp(theme.disabled, theme.on_surface, 0.2)
        } else {
            theme.disabled
        };
        p.fill_rounded_rect(track, track_color, size.height / 2.0);
        let radius = (size.height - 6.0) * 0.5;
        let cx = if self.checked {
            size.width - radius - 3.0
        } else {
            radius + 3.0
        };
        p.fill_circle(Point::new(cx, size.height / 2.0), radius, theme.on_primary);
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Enter => self.hovered = true,
            PointerEvent::Leave => self.hovered = false,
            PointerEvent::Up(_) => {
                self.checked = !self.checked;
                if let Some(on_change) = &mut self.on_change {
                    on_change(self.checked);
                }
            }
            _ => {}
        }
    }
}
