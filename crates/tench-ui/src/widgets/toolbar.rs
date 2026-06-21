//! Toolbar widget for compact rows of command buttons.

use kurbo::{Axis, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::painter::Painter;
use crate::theme::Theme;

#[derive(Clone, Debug)]
pub struct ToolbarItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub active: bool,
    pub enabled: bool,
}

impl ToolbarItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            active: false,
            enabled: true,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

pub struct Toolbar {
    items: Vec<ToolbarItem>,
    hovered: Option<usize>,
    pressed: Option<usize>,
    // clippy: callback field type is idiomatic for this widget
    #[allow(clippy::type_complexity)]
    on_select: Option<Box<dyn FnMut(&str) + Send>>,
}

impl Toolbar {
    pub fn new(items: Vec<ToolbarItem>) -> Self {
        Self {
            items,
            hovered: None,
            pressed: None,
            on_select: None,
        }
    }

    pub fn on_select(mut self, f: impl FnMut(&str) + Send + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    fn item_rect(index: usize, height: f64) -> Rect {
        let x = 6.0 + index as f64 * 74.0;
        Rect::new(x, 4.0, x + 68.0, height - 4.0)
    }
}

impl Widget for Toolbar {
    fn measure(&mut self, _ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => (self.items.len() as f64 * 74.0 + 12.0).min(available),
            Axis::Vertical => 40.0,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        let theme = Theme::default();
        let size = ctx.state.size;
        let mut p = Painter::new(scene);
        p.fill_rect(Rect::new(0.0, 0.0, size.width, size.height), theme.surface);

        for (idx, item) in self.items.iter().enumerate() {
            let rect = Self::item_rect(idx, size.height);
            let bg = if item.active {
                theme.primary
            } else if self.pressed == Some(idx) {
                Color::lerp(theme.background, theme.primary, 0.25)
            } else if self.hovered == Some(idx) && item.enabled {
                theme.background
            } else {
                theme.surface
            };
            p.fill_rounded_rect(rect, bg, theme.border_radius);
            let text = item.icon.as_deref().unwrap_or(&item.label);
            p.draw_text(
                text,
                rect.x0 + 10.0,
                rect.y0 + rect.height() / 2.0 + 4.0,
                if item.enabled {
                    if item.active {
                        theme.on_primary
                    } else {
                        theme.on_surface
                    }
                } else {
                    theme.disabled
                },
                theme.font_size_small,
                FontWeight::MEDIUM,
                false,
            );
        }
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        let size = ctx.state.size;
        match event {
            PointerEvent::Move(e) => {
                self.hovered = self.items.iter().enumerate().find_map(|(idx, _)| {
                    Self::item_rect(idx, size.height)
                        .contains(e.pos)
                        .then_some(idx)
                });
            }
            PointerEvent::Leave => {
                self.hovered = None;
                self.pressed = None;
            }
            PointerEvent::Down(e) => {
                self.pressed = self.items.iter().enumerate().find_map(|(idx, item)| {
                    (item.enabled && Self::item_rect(idx, size.height).contains(e.pos))
                        .then_some(idx)
                });
            }
            PointerEvent::Up(_) => {
                if let Some(idx) = self.pressed.take() {
                    if let Some(item) = self.items.get(idx) {
                        if item.enabled {
                            if let Some(on_select) = &mut self.on_select {
                                on_select(&item.id);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
