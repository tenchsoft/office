//! Menu bar widget.

use kurbo::{Axis, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

pub struct MenuBar {
    menus: Vec<String>,
    active: Option<usize>,
    on_select: Option<Box<dyn FnMut(usize) + Send>>,
}

impl MenuBar {
    pub fn new(menus: Vec<String>) -> Self {
        Self {
            menus,
            active: None,
            on_select: None,
        }
    }

    pub fn on_select(mut self, f: impl FnMut(usize) + Send + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    fn menu_rect(idx: usize) -> Rect {
        let x = 8.0 + idx as f64 * 58.0;
        Rect::new(x, 4.0, x + 54.0, 26.0)
    }
}

impl Widget for MenuBar {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => 30.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let mut p = Painter::new(scene);
        p.fill_rect(
            Rect::new(0.0, 0.0, ctx.state.size.width, ctx.state.size.height),
            theme.surface,
        );
        for (idx, menu) in self.menus.iter().enumerate() {
            let rect = Self::menu_rect(idx);
            if self.active == Some(idx) {
                p.fill_rounded_rect(rect, theme.background, 3.0);
            }
            p.draw_text(
                menu,
                rect.x0 + 8.0,
                rect.y0 + 15.0,
                theme.on_surface,
                12.0,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        if let PointerEvent::Down(e) = event {
            for idx in 0..self.menus.len() {
                if Self::menu_rect(idx).contains(e.pos) {
                    self.active = Some(idx);
                    if let Some(on_select) = &mut self.on_select {
                        on_select(idx);
                    }
                    break;
                }
            }
        }
    }
}
