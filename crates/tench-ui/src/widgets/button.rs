//! Button widget.

use kurbo::{Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{
    AccessRole, AccessibilityNode, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetState,
};
use crate::render::painter::Painter;
use crate::theme::Theme;

/// A push button with a label.
pub struct Button {
    label: String,
    hovered: bool,
    pressed: bool,
    on_click: Option<Box<dyn FnMut() + Send>>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            hovered: false,
            pressed: false,
            on_click: None,
        }
    }

    /// Set the click handler.
    pub fn on_click(mut self, f: impl FnMut() + Send + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl Widget for Button {
    fn measure(&mut self, _ctx: &mut MeasureCtx, _axis: kurbo::Axis, _available: f64) -> f64 {
        // Rough text width estimate: ~8px per char + padding
        let text_width = self.label.len() as f64 * 8.0 + 32.0;
        let height = Theme::default().button_height;
        match _axis {
            kurbo::Axis::Horizontal => text_width,
            kurbo::Axis::Vertical => height,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, size: Size) {
        // No children to layout
        let _ = size;
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;

        // Determine background color
        let bg = if self.pressed {
            Color::lerp(theme.primary, Color::BLACK, 0.2)
        } else if self.hovered {
            Color::lerp(theme.primary, Color::WHITE, 0.15)
        } else {
            theme.primary
        };

        // Draw rounded rect background
        let rect = Rect::from_origin_size((0.0, 0.0), size);
        let mut painter = Painter::new(scene);
        painter.fill_rounded_rect(rect, bg, theme.border_radius);

        // Draw label centered
        let text_x = size.width / 2.0;
        let text_y = size.height / 2.0;
        painter.draw_text(
            &self.label,
            text_x,
            text_y,
            theme.on_primary,
            theme.font_size,
            FontWeight::MEDIUM,
            true, // center aligned
        );
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Enter => {
                self.hovered = true;
            }
            PointerEvent::Leave => {
                self.hovered = false;
                self.pressed = false;
            }
            PointerEvent::Down(_) => {
                self.pressed = true;
            }
            PointerEvent::Up(_) if self.pressed => {
                self.pressed = false;
                if let Some(on_click) = &mut self.on_click {
                    on_click();
                }
            }
            _ => {}
        }
    }

    fn accessibility_tree(&self, state: &WidgetState) -> AccessibilityNode {
        AccessibilityNode {
            role: AccessRole::Button,
            label: Some(self.label.clone()),
            value: None,
            focused: state.is_focused,
            disabled: state.is_disabled,
            children: Vec::new(),
        }
    }
}
