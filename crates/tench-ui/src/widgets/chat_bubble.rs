//! ChatBubble — message bubble with streaming text support.

use crate::core::types::Color;
use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use kurbo::{Axis, Point, Rect, Size};

/// A chat message bubble.
pub struct ChatBubble {
    text: String,
    is_user: bool,
    streaming: bool,
}

impl ChatBubble {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_user: true,
            streaming: false,
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_user: false,
            streaming: false,
        }
    }

    pub fn streaming(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_user: false,
            streaming: true,
        }
    }

    pub fn append(&mut self, ch: char) {
        self.text.push(ch);
    }

    pub fn finish_streaming(&mut self) {
        self.streaming = false;
    }

    fn bubble_height(&self, width: f64) -> f64 {
        let chars_per_line = ((width - 32.0) / 7.0).max(1.0) as usize;
        let lines = (self.text.len() / chars_per_line).max(1) + 1;
        lines as f64 * 20.0 + 16.0
    }
}

impl Widget for ChatBubble {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available.min(400.0),
            Axis::Vertical => self.bubble_height(if available > 0.0 { available } else { 300.0 }),
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut vello::Scene) {
        let theme = ctx.theme().clone();
        let size = ctx.size();
        let mut painter = Painter::new(scene);

        let bg = if self.is_user {
            theme.primary
        } else {
            Color::rgb8(60, 60, 60)
        };
        let text_color = if self.is_user {
            Color::WHITE
        } else {
            theme.on_background
        };

        let _bubble_width = size.width;
        let bubble_rect = Rect::from_origin_size(Point::ZERO, size);

        painter.fill_rounded_rect(bubble_rect, bg, 12.0);

        painter.draw_text(
            &self.text,
            12.0,
            size.height / 2.0 + 4.0,
            text_color,
            14.0,
            parley::FontWeight::NORMAL,
            false,
        );

        // Streaming cursor
        if self.streaming {
            let cursor_x = 12.0 + (self.text.len() as f64 * 7.0).min(size.width - 24.0);
            painter.draw_text(
                "|",
                cursor_x,
                size.height / 2.0 + 4.0,
                text_color,
                14.0,
                parley::FontWeight::NORMAL,
                false,
            );
        }
    }
}
