//! Modal widget — an overlay dialog that blocks interaction with the rest of the UI.

use kurbo::{Axis, Point, Rect, Size};
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetPod};
use crate::render::painter::Painter;
use crate::theme::Theme;

/// A modal overlay with a content widget.
pub struct Modal {
    visible: bool,
    child: WidgetPod,
    on_close: Option<Box<dyn FnMut() + Send>>,
}

impl Modal {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            visible: false,
            child: WidgetPod::new(child),
            on_close: None,
        }
    }

    pub fn visible(mut self, v: bool) -> Self {
        self.visible = v;
        self
    }

    pub fn on_close(mut self, f: impl FnMut() + Send + 'static) -> Self {
        self.on_close = Some(Box::new(f));
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Widget for Modal {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        if !self.visible {
            return 0.0;
        }
        let _ = axis;
        available
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, size: Size) {
        if !self.visible {
            return;
        }

        let modal_width = (size.width * 0.6).clamp(200.0, 600.0);
        let modal_height = (size.height * 0.6).clamp(150.0, 500.0);
        let modal_size = Size::new(modal_width, modal_height);
        let x = (size.width - modal_width) / 2.0;
        let y = (size.height - modal_height) / 2.0;

        self.child.state.position = Point::new(x, y);
        self.child.state.size = modal_size;

        let mut child_ctx = LayoutCtx {
            state: &mut self.child.state,
            global: ctx.global,
        };
        self.child.widget.layout(&mut child_ctx, modal_size);
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        if !self.visible {
            return;
        }

        let theme = Theme::default();
        let size = ctx.state.size;
        let mut painter = Painter::new(scene);

        // Dim overlay
        let overlay_rect = Rect::from_origin_size((0.0, 0.0), size);
        painter.fill_rect(overlay_rect, crate::core::types::Color::rgba8(0, 0, 0, 128));

        // Modal card
        let card_rect = Rect::from_origin_size(self.child.state.position, self.child.state.size);
        painter.fill_rounded_rect(card_rect, theme.surface, theme.border_radius * 2.0);
        painter.stroke_rounded_rect(card_rect, theme.border, 1.0, theme.border_radius * 2.0);

        // Paint child
        let theme = ctx.theme().clone();
        let mut child_ctx = PaintCtx {
            state: &mut self.child.state,
            global: ctx.global,
            theme,
        };
        self.child.widget.paint(&mut child_ctx, scene);
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        if !self.visible {
            return;
        }

        if let PointerEvent::Up(btn) = event {
            // Click outside modal closes it
            let child_pos = self.child.state.position;
            let child_size = self.child.state.size;
            let inside = btn.pos.x >= child_pos.x
                && btn.pos.x <= child_pos.x + child_size.width
                && btn.pos.y >= child_pos.y
                && btn.pos.y <= child_pos.y + child_size.height;

            if !inside {
                if let Some(on_close) = &mut self.on_close {
                    on_close();
                }
                self.visible = false;
            }
        }
    }

    fn children(&self) -> Vec<crate::core::types::WidgetId> {
        if self.visible {
            vec![self.child.state.id]
        } else {
            Vec::new()
        }
    }

    fn child_mut(&mut self, id: crate::core::types::WidgetId) -> Option<&mut WidgetPod> {
        if self.child.state.id == id {
            Some(&mut self.child)
        } else {
            None
        }
    }
}
