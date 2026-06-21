//! Label widget — displays non-interactive text.

use kurbo::{Axis, Size};
use vello::Scene;

use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

/// A non-interactive text label.
pub struct Label {
    text: String,
    font_size: Option<f32>,
    color_override: Option<crate::core::types::Color>,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: None,
            color_override: None,
        }
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn with_color(mut self, color: crate::core::types::Color) -> Self {
        self.color_override = Some(color);
        self
    }
}

impl Widget for Label {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, _available: f64) -> f64 {
        let font_size = self.font_size.unwrap_or(Theme::default().font_size);
        // Estimate: ~0.6 * font_size per character width
        let char_width = font_size as f64 * 0.6;
        let text_width = self.text.len() as f64 * char_width + 4.0;
        let text_height = font_size as f64 * 1.4;

        match axis {
            Axis::Horizontal => text_width,
            Axis::Vertical => text_height,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {
        // No children
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let font_size = self.font_size.unwrap_or(theme.font_size);
        let color = self.color_override.unwrap_or(theme.on_background);

        let mut painter = Painter::new(scene);
        let y = size.height / 2.0;
        painter.draw_text(
            &self.text,
            0.0,
            y,
            color,
            font_size,
            parley::FontWeight::NORMAL,
            false,
        );
    }
}
