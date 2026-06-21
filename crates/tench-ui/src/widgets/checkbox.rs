//! Checkbox widget — a toggleable boolean input.

use kurbo::{Axis, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::widget::{
    AccessRole, AccessibilityNode, EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetState,
};
use crate::render::painter::Painter;
use crate::theme::Theme;

/// A checkbox with an optional label.
pub struct Checkbox {
    label: String,
    checked: bool,
    hovered: bool,
    on_change: Option<Box<dyn FnMut(bool) + Send>>,
}

impl Checkbox {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked: false,
            hovered: false,
            on_change: None,
        }
    }

    pub fn checked(mut self, value: bool) -> Self {
        self.checked = value;
        self
    }

    pub fn on_change(mut self, f: impl FnMut(bool) + Send + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn set_checked(&mut self, value: bool) {
        self.checked = value;
    }
}

impl Widget for Checkbox {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        let theme = Theme::default();
        let box_size = theme.font_size as f64;
        let gap = 8.0;
        let text_width = self.label.len() as f64 * (theme.font_size as f64 * 0.6);
        match axis {
            Axis::Horizontal => box_size + gap + text_width.min(available),
            Axis::Vertical => theme.input_height,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let box_size = theme.font_size as f64;
        let y_center = size.height / 2.0;

        let mut painter = Painter::new(scene);

        // Draw checkbox box
        let box_rect = Rect::new(
            0.0,
            y_center - box_size / 2.0,
            box_size,
            y_center + box_size / 2.0,
        );
        let border_color = if self.hovered {
            theme.primary
        } else {
            theme.border
        };
        painter.stroke_rounded_rect(box_rect, border_color, 1.5, 3.0);

        if self.checked {
            let fill_color = theme.primary;
            painter.fill_rounded_rect(box_rect, fill_color, 3.0);
            // Draw checkmark (two lines forming a "V" rotated)
            let cx = box_size / 2.0;
            let cy = y_center;
            let s = box_size * 0.3;
            painter.draw_line(
                kurbo::Point::new(cx - s, cy),
                kurbo::Point::new(cx - s * 0.3, cy + s * 0.7),
                theme.on_primary,
                2.0,
            );
            painter.draw_line(
                kurbo::Point::new(cx - s * 0.3, cy + s * 0.7),
                kurbo::Point::new(cx + s, cy - s * 0.5),
                theme.on_primary,
                2.0,
            );
        }

        // Draw label
        let text_x = box_size + 8.0;
        painter.draw_text(
            &self.label,
            text_x,
            y_center,
            theme.on_surface,
            theme.font_size,
            FontWeight::NORMAL,
            false,
        );
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Enter => {
                self.hovered = true;
            }
            PointerEvent::Leave => {
                self.hovered = false;
            }
            PointerEvent::Up(_) => {
                self.checked = !self.checked;
                if let Some(on_change) = &mut self.on_change {
                    on_change(self.checked);
                }
            }
            _ => {}
        }
    }

    fn accessibility_tree(&self, state: &WidgetState) -> AccessibilityNode {
        AccessibilityNode {
            role: AccessRole::CheckBox,
            label: Some(self.label.clone()),
            value: Some(self.checked.to_string()),
            focused: state.is_focused,
            disabled: state.is_disabled,
            children: Vec::new(),
        }
    }
}
