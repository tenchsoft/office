//! Toast notification widget.

use kurbo::{Axis, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::types::Color;
use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

pub struct Toast {
    message: String,
    kind: ToastKind,
}

impl Toast {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ToastKind::Info,
        }
    }

    pub fn kind(mut self, kind: ToastKind) -> Self {
        self.kind = kind;
        self
    }
}

impl Widget for Toast {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => (self.message.len() as f64 * 8.0 + 44.0).min(available),
            Axis::Vertical => 36.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut p = Painter::new(scene);
        let accent = match self.kind {
            ToastKind::Info => theme.primary,
            ToastKind::Success => Color::rgb8(0xA6, 0xE3, 0xA1),
            ToastKind::Warning => Color::rgb8(0xF9, 0xE2, 0xAF),
            ToastKind::Error => theme.error,
        };
        let rect = Rect::new(0.0, 0.0, size.width, size.height);
        p.fill_rounded_rect(rect, theme.surface, theme.border_radius);
        p.fill_rounded_rect(Rect::new(0.0, 0.0, 4.0, size.height), accent, 2.0);
        p.draw_text(
            &self.message,
            16.0,
            size.height / 2.0 + 5.0,
            theme.on_surface,
            theme.font_size,
            FontWeight::MEDIUM,
            false,
        );
    }
}
