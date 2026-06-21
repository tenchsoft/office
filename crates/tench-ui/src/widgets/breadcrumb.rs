//! Breadcrumb widget.

use kurbo::{Axis, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

pub struct Breadcrumb {
    items: Vec<String>,
    on_select: Option<Box<dyn FnMut(usize) + Send>>,
}

impl Breadcrumb {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            on_select: None,
        }
    }

    pub fn on_select(mut self, f: impl FnMut(usize) + Send + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }
}

impl Widget for Breadcrumb {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => 28.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, _ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let mut p = Painter::new(scene);
        let mut x = 0.0;
        for (idx, item) in self.items.iter().enumerate() {
            p.draw_text(
                item,
                x,
                18.0,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::MEDIUM,
                false,
            );
            x += item.len() as f64 * 7.0 + 8.0;
            if idx + 1 < self.items.len() {
                p.draw_text(
                    "/",
                    x,
                    18.0,
                    theme.disabled,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
                x += 12.0;
            }
        }
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        if let PointerEvent::Down(e) = event {
            let mut x = 0.0;
            for (idx, item) in self.items.iter().enumerate() {
                let width = item.len() as f64 * 7.0 + 8.0;
                if e.pos.x >= x && e.pos.x <= x + width {
                    if let Some(on_select) = &mut self.on_select {
                        on_select(idx);
                    }
                    break;
                }
                x += width + 12.0;
            }
        }
    }
}
