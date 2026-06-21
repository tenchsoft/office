//! Slider widget — a horizontal range input.

use kurbo::{Axis, Rect, Size};
use vello::Scene;

use crate::core::events::{PointerButton, PointerEvent};
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

/// A horizontal slider for selecting a numeric value within a range.
pub struct Slider {
    min: f64,
    max: f64,
    value: f64,
    step: f64,
    dragging: bool,
    hovered: bool,
    on_change: Option<Box<dyn FnMut(f64) + Send>>,
}

impl Slider {
    pub fn new(min: f64, max: f64, value: f64) -> Self {
        Self {
            min,
            max,
            value: value.clamp(min, max),
            step: 0.0,
            dragging: false,
            hovered: false,
            on_change: None,
        }
    }

    pub fn with_step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn on_change(mut self, f: impl FnMut(f64) + Send + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn set_value(&mut self, v: f64) {
        self.value = v.clamp(self.min, self.max);
    }

    fn normalized(&self) -> f64 {
        if (self.max - self.min).abs() < f64::EPSILON {
            0.0
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }

    fn value_from_x(&self, x: f64, width: f64) -> f64 {
        let ratio = (x / width).clamp(0.0, 1.0);
        let raw = self.min + ratio * (self.max - self.min);
        if self.step > 0.0 {
            (raw / self.step).round() * self.step
        } else {
            raw
        }
        .clamp(self.min, self.max)
    }

    fn thumb_radius() -> f64 {
        8.0
    }

    fn track_height() -> f64 {
        4.0
    }
}

impl Widget for Slider {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => Self::thumb_radius() * 2.0 + 4.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut painter = Painter::new(scene);

        let track_y = size.height / 2.0;
        let track_h = Self::track_height();
        let thumb_r = Self::thumb_radius();
        let norm = self.normalized();

        // Track background
        let track_rect = Rect::new(
            0.0,
            track_y - track_h / 2.0,
            size.width,
            track_y + track_h / 2.0,
        );
        painter.fill_rounded_rect(track_rect, theme.disabled, track_h / 2.0);

        // Active track
        let active_width = size.width * norm;
        if active_width > 0.0 {
            let active_rect = Rect::new(
                0.0,
                track_y - track_h / 2.0,
                active_width,
                track_y + track_h / 2.0,
            );
            painter.fill_rounded_rect(active_rect, theme.primary, track_h / 2.0);
        }

        // Thumb
        let thumb_x = active_width;
        let thumb_color = if self.dragging {
            Color::lerp(theme.primary, theme.on_primary, 0.3)
        } else if self.hovered {
            Color::lerp(theme.primary, theme.on_primary, 0.15)
        } else {
            theme.primary
        };
        let thumb_rect = Rect::new(
            thumb_x - thumb_r,
            track_y - thumb_r,
            thumb_x + thumb_r,
            track_y + thumb_r,
        );
        painter.fill_rounded_rect(thumb_rect, thumb_color, thumb_r);
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        let width = _ctx.state.size.width;
        match event {
            PointerEvent::Down(btn) if btn.button == PointerButton::Primary => {
                self.dragging = true;
                self.value = self.value_from_x(btn.pos.x, width);
                if let Some(on_change) = &mut self.on_change {
                    on_change(self.value);
                }
            }
            PointerEvent::Up(_) => {
                self.dragging = false;
            }
            PointerEvent::Move(move_evt) if self.dragging => {
                self.value = self.value_from_x(move_evt.pos.x, width);
                if let Some(on_change) = &mut self.on_change {
                    on_change(self.value);
                }
            }
            PointerEvent::Enter => {
                self.hovered = true;
            }
            PointerEvent::Leave => {
                self.hovered = false;
                self.dragging = false;
            }
            _ => {}
        }
    }
}
