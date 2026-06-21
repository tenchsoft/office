//! ProgressBar — horizontal progress indicator.

use crate::core::types::Color;
use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use kurbo::{Axis, Rect, Size};

/// A horizontal progress bar.
pub struct ProgressBar {
    value: f64, // 0.0 to 1.0
    height: f64,
    show_label: bool,
}

impl ProgressBar {
    pub fn new(value: f64) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            height: 8.0,
            show_label: false,
        }
    }

    pub fn with_height(mut self, h: f64) -> Self {
        self.height = h;
        self
    }

    pub fn show_label(mut self) -> Self {
        self.show_label = true;
        self
    }

    pub fn set_value(&mut self, v: f64) {
        self.value = v.clamp(0.0, 1.0);
    }
}

impl Widget for ProgressBar {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => {
                if self.show_label {
                    self.height + 20.0
                } else {
                    self.height
                }
            }
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut vello::Scene) {
        let theme = ctx.theme().clone();
        let size = ctx.size();
        let mut painter = Painter::new(scene);

        let bar_y = if self.show_label { 16.0 } else { 0.0 };
        let bar_rect = Rect::new(0.0, bar_y, size.width, bar_y + self.height);

        // Track
        painter.fill_rounded_rect(bar_rect, Color::rgb8(50, 50, 50), self.height / 2.0);

        // Fill
        let fill_width = size.width * self.value;
        if fill_width > 0.0 {
            painter.fill_rounded_rect(
                Rect::new(0.0, bar_y, fill_width, bar_y + self.height),
                theme.primary,
                self.height / 2.0,
            );
        }

        // Label
        if self.show_label {
            painter.draw_text(
                &format!("{:.0}%", self.value * 100.0),
                0.0,
                12.0,
                theme.on_background,
                11.0,
                parley::FontWeight::NORMAL,
                false,
            );
        }
    }
}
