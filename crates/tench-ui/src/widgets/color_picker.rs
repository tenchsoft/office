//! Color picker widget.

use kurbo::{Axis, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::{GradientDirection, Painter};
use crate::theme::Theme;

pub struct ColorPicker {
    color: Color,
    on_change: Option<Box<dyn FnMut(Color) + Send>>,
}

impl ColorPicker {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            on_change: None,
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn on_change(mut self, f: impl FnMut(Color) + Send + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl Widget for ColorPicker {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available.min(220.0),
            Axis::Vertical => 128.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut p = Painter::new(scene);
        let spectrum = Rect::new(0.0, 0.0, size.width, size.height - 32.0);
        p.fill_rect_linear_gradient(
            spectrum,
            Color::rgb8(0xF3, 0x8B, 0xA8),
            Color::rgb8(0x89, 0xB4, 0xFA),
            GradientDirection::Horizontal,
        );
        let swatch = Rect::new(0.0, size.height - 24.0, 24.0, size.height);
        p.fill_rounded_rect(swatch, self.color, 4.0);
        p.draw_text(
            "Color",
            34.0,
            size.height - 8.0,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::MEDIUM,
            false,
        );
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        if let PointerEvent::Down(e) = event {
            let size = ctx.state.size;
            let hue = (e.pos.x / size.width).clamp(0.0, 1.0);
            let value = (1.0 - e.pos.y / (size.height - 32.0).max(1.0)).clamp(0.0, 1.0);
            self.color = Color::rgb8(
                (255.0 * hue) as u8,
                (255.0 * value) as u8,
                (255.0 * (1.0 - hue)) as u8,
            );
            if let Some(on_change) = &mut self.on_change {
                on_change(self.color);
            }
        }
    }
}
