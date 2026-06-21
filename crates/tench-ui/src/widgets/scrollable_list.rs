//! ScrollableList — virtual scrolling list widget.

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use kurbo::{Axis, Point, Rect, Size};

/// A virtual-scrolling list that renders only visible items.
pub struct ScrollableList {
    items: Vec<String>,
    item_height: f64,
    selected_index: Option<usize>,
    scroll_offset: f64,
    on_select: Option<Box<dyn Fn(usize) + Send>>,
}

impl ScrollableList {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            item_height: 36.0,
            selected_index: None,
            scroll_offset: 0.0,
            on_select: None,
        }
    }

    pub fn with_item_height(mut self, h: f64) -> Self {
        self.item_height = h;
        self
    }

    pub fn on_select(mut self, f: impl Fn(usize) + Send + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.scroll_offset = 0.0;
        self.selected_index = None;
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    #[allow(dead_code)]
    fn max_scroll(&self, view_height: f64) -> f64 {
        let total = self.items.len() as f64 * self.item_height;
        (total - view_height).max(0.0)
    }
}

impl Widget for ScrollableList {
    fn measure(&mut self, _ctx: &mut MeasureCtx, _axis: Axis, available: f64) -> f64 {
        let total = self.items.len() as f64 * self.item_height;
        total.min(available)
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut vello::Scene) {
        let theme = ctx.theme().clone();
        let size = ctx.size();
        let mut painter = Painter::new(scene);

        // Background
        painter.fill_rect(Rect::from_origin_size(Point::ZERO, size), theme.background);

        // Visible range
        let first = (self.scroll_offset / self.item_height) as usize;
        let visible_count = (size.height / self.item_height).ceil() as usize + 1;
        let last = (first + visible_count).min(self.items.len());

        for i in first..last {
            let y = i as f64 * self.item_height - self.scroll_offset;
            let row_rect = Rect::new(0.0, y, size.width, y + self.item_height);

            // Hover/selection highlight
            if self.selected_index == Some(i) {
                painter.fill_rect(row_rect, theme.primary);
            }

            // Item text
            let text_color = if self.selected_index == Some(i) {
                Color::WHITE
            } else {
                theme.on_background
            };
            painter.draw_text(
                &self.items[i],
                12.0,
                y + self.item_height / 2.0 + 4.0,
                text_color,
                13.0,
                parley::FontWeight::NORMAL,
                false,
            );
        }

        // Scrollbar
        let total = self.items.len() as f64 * self.item_height;
        if total > size.height {
            let scrollbar_height = (size.height / total * size.height).max(20.0);
            let scrollbar_y = (self.scroll_offset / total) * size.height;
            painter.fill_rounded_rect(
                Rect::new(
                    size.width - 6.0,
                    scrollbar_y,
                    size.width - 2.0,
                    scrollbar_y + scrollbar_height,
                ),
                Color::rgb8(100, 100, 100),
                2.0,
            );
        }
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Down(e) => {
                let index = ((e.pos.y + self.scroll_offset) / self.item_height) as usize;
                if index < self.items.len() {
                    self.selected_index = Some(index);
                }
            }
            PointerEvent::Scroll(e) => {
                self.scroll_offset = (self.scroll_offset + e.delta.y * 30.0).max(0.0);
            }
            _ => {}
        }
    }
}
