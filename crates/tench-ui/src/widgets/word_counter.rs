//! WordCounter — live word/character count display.

use crate::core::types::Color;
use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use kurbo::{Axis, Size};

/// Displays word and character counts.
pub struct WordCounter {
    text: String,
}

impl Default for WordCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl WordCounter {
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    pub fn update(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    fn char_count(&self) -> usize {
        self.text.len()
    }
}

impl Widget for WordCounter {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => 200.0_f64.min(available),
            Axis::Vertical => 16.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut vello::Scene) {
        let _theme = ctx.theme().clone();
        let _size = ctx.size();
        let mut painter = Painter::new(scene);

        let label = format!(
            "{} words · {} characters",
            self.word_count(),
            self.char_count()
        );

        painter.draw_text(
            &label,
            0.0,
            12.0,
            Color::rgb8(120, 120, 120),
            11.0,
            parley::FontWeight::NORMAL,
            false,
        );
    }
}
