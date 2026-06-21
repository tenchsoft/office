//! Context menu widget.

use kurbo::{Axis, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

#[derive(Clone, Debug)]
pub struct ContextMenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

impl ContextMenuItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled: true,
        }
    }
}

pub struct ContextMenu {
    items: Vec<ContextMenuItem>,
    // clippy: callback field type is idiomatic for this widget
    #[allow(clippy::type_complexity)]
    on_select: Option<Box<dyn FnMut(&str) + Send>>,
}

impl ContextMenu {
    pub fn new(items: Vec<ContextMenuItem>) -> Self {
        Self {
            items,
            on_select: None,
        }
    }

    pub fn on_select(mut self, f: impl FnMut(&str) + Send + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }
}

impl Widget for ContextMenu {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, _available: f64) -> f64 {
        match axis {
            Axis::Horizontal => 180.0,
            Axis::Vertical => self.items.len() as f64 * 28.0 + 8.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut p = Painter::new(scene);
        p.fill_rounded_rect(
            Rect::new(0.0, 0.0, size.width, size.height),
            theme.surface,
            theme.border_radius,
        );
        for (idx, item) in self.items.iter().enumerate() {
            let y = 8.0 + idx as f64 * 28.0;
            p.draw_text(
                &item.label,
                12.0,
                y + 18.0,
                if item.enabled {
                    theme.on_surface
                } else {
                    theme.disabled
                },
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        if let PointerEvent::Down(e) = event {
            let idx = ((e.pos.y - 8.0) / 28.0) as usize;
            if let Some(item) = self.items.get(idx) {
                if item.enabled {
                    if let Some(on_select) = &mut self.on_select {
                        on_select(&item.id);
                    }
                }
            }
        }
    }
}
