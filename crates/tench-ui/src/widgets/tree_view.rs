//! TreeView — collapsible tree with selection support.

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;
use crate::theme::Theme;
use kurbo::{Axis, Point, Rect, Size};

/// A single item in the tree.
#[derive(Clone, Debug)]
pub struct TreeItem {
    pub id: String,
    pub label: String,
    pub children: Vec<TreeItem>,
    pub expanded: bool,
    pub selected: bool,
}

impl TreeItem {
    pub fn leaf(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
            expanded: false,
            selected: false,
        }
    }

    pub fn branch(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<TreeItem>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children,
            expanded: false,
            selected: false,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

/// A collapsible tree view with selection.
pub struct TreeView {
    items: Vec<TreeItem>,
    row_height: f64,
    indent: f64,
    // clippy: callback field type is idiomatic for this widget
    #[allow(clippy::type_complexity)]
    on_select: Option<Box<dyn Fn(&str) + Send>>,
    hovered_row: Option<usize>,
    scroll_offset: f64,
}

impl TreeView {
    pub fn new(items: Vec<TreeItem>) -> Self {
        Self {
            items,
            row_height: 28.0,
            indent: 20.0,
            on_select: None,
            hovered_row: None,
            scroll_offset: 0.0,
        }
    }

    pub fn on_select(mut self, f: impl Fn(&str) + Send + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    fn count_visible(&self) -> usize {
        Self::count_visible_items(&self.items)
    }

    fn count_visible_items(items: &[TreeItem]) -> usize {
        let mut count = 0;
        for item in items {
            count += 1;
            if item.expanded {
                count += Self::count_visible_items(&item.children);
            }
        }
        count
    }

    #[allow(dead_code)]
    fn item_at_y(&self, y: f64) -> Option<(usize, String, bool)> {
        let adjusted_y = y + self.scroll_offset;
        let row = (adjusted_y / self.row_height) as usize;
        if row < self.count_visible() {
            Some((row, String::new(), false))
        } else {
            None
        }
    }

    /// Walk visible items and return the id of the item at the given visible row index.
    fn find_item_at_row(items: &[TreeItem], row: usize, counter: &mut usize) -> Option<String> {
        for item in items {
            if *counter == row {
                return Some(item.id.clone());
            }
            *counter += 1;
            if item.expanded {
                if let Some(id) = Self::find_item_at_row(&item.children, row, counter) {
                    return Some(id);
                }
            }
        }
        None
    }

    /// Toggle expand/collapse on the item with the given id.
    fn toggle_expand(items: &mut [TreeItem], target_id: &str) -> bool {
        for item in items.iter_mut() {
            if item.id == target_id {
                if !item.is_leaf() {
                    item.expanded = !item.expanded;
                    return true;
                }
                return false;
            }
            if Self::toggle_expand(&mut item.children, target_id) {
                return true;
            }
        }
        false
    }

    /// Set `selected = false` on all items.
    fn clear_selection(items: &mut [TreeItem]) {
        for item in items.iter_mut() {
            item.selected = false;
            Self::clear_selection(&mut item.children);
        }
    }

    /// Set `selected = true` on the item with the given id.
    fn select_item(items: &mut [TreeItem], target_id: &str) -> bool {
        for item in items.iter_mut() {
            if item.id == target_id {
                item.selected = true;
                return true;
            }
            if Self::select_item(&mut item.children, target_id) {
                return true;
            }
        }
        false
    }

    // clippy: recursive tree painting naturally requires many parameters
    #[allow(clippy::too_many_arguments)]
    fn paint_items(
        painter: &mut Painter,
        items: &[TreeItem],
        depth: usize,
        y_offset: &mut f64,
        row_height: f64,
        indent: f64,
        theme: &Theme,
        visible_rect: Rect,
    ) {
        for item in items {
            if *y_offset + row_height < visible_rect.y0 {
                // Above visible area — skip but advance offset
                *y_offset += row_height;
                if item.expanded {
                    Self::paint_items(
                        painter,
                        &item.children,
                        depth + 1,
                        y_offset,
                        row_height,
                        indent,
                        theme,
                        visible_rect,
                    );
                }
                continue;
            }
            if *y_offset > visible_rect.y1 {
                return;
            }

            let x = indent * depth as f64;
            let _row_rect = Rect::new(x, *y_offset, visible_rect.x1, *y_offset + row_height);

            // Selection highlight
            if item.selected {
                painter.fill_rect(
                    Rect::new(0.0, *y_offset, visible_rect.x1, *y_offset + row_height),
                    Color::rgb8(60, 100, 180),
                );
            }

            // Expand/collapse arrow
            if !item.is_leaf() {
                let arrow = if item.expanded { "v" } else { ">" };
                painter.draw_text(
                    arrow,
                    x + 4.0,
                    *y_offset + row_height / 2.0 + 4.0,
                    if item.selected {
                        Color::WHITE
                    } else {
                        Color::rgb8(180, 180, 180)
                    },
                    12.0,
                    parley::FontWeight::NORMAL,
                    false,
                );
            }

            // Label
            let label_x = if item.is_leaf() { x + 8.0 } else { x + 20.0 };
            painter.draw_text(
                &item.label,
                label_x,
                *y_offset + row_height / 2.0 + 4.0,
                if item.selected {
                    Color::WHITE
                } else {
                    theme.on_background
                },
                13.0,
                parley::FontWeight::NORMAL,
                false,
            );

            *y_offset += row_height;

            if item.expanded {
                Self::paint_items(
                    painter,
                    &item.children,
                    depth + 1,
                    y_offset,
                    row_height,
                    indent,
                    theme,
                    visible_rect,
                );
            }
        }
    }
}

impl Widget for TreeView {
    fn measure(&mut self, _ctx: &mut MeasureCtx, _axis: Axis, available: f64) -> f64 {
        let rows = self.count_visible() as f64;
        let preferred = rows * self.row_height;
        preferred.min(available)
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut vello::Scene) {
        let theme = ctx.theme().clone();
        let size = ctx.size();
        let mut painter = Painter::new(scene);
        let visible_rect = Rect::from_origin_size(Point::ZERO, size);

        let mut y = -self.scroll_offset;
        Self::paint_items(
            &mut painter,
            &self.items,
            0,
            &mut y,
            self.row_height,
            self.indent,
            &theme,
            visible_rect,
        );
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Down(e) => {
                let row = ((e.pos.y + self.scroll_offset) / self.row_height) as usize;
                let mut counter = 0;
                if let Some(item_id) = Self::find_item_at_row(&self.items, row, &mut counter) {
                    // Toggle expand/collapse for branches.
                    Self::toggle_expand(&mut self.items, &item_id);

                    // Update selection.
                    Self::clear_selection(&mut self.items);
                    Self::select_item(&mut self.items, &item_id);

                    // Fire on_select callback.
                    if let Some(ref on_select) = self.on_select {
                        on_select(&item_id);
                    }
                }
            }
            PointerEvent::Move(e) => {
                let row = ((e.pos.y + self.scroll_offset) / self.row_height) as usize;
                self.hovered_row = Some(row);
            }
            PointerEvent::Scroll(e) => {
                self.scroll_offset = (self.scroll_offset + e.delta.y * 30.0).max(0.0);
            }
            _ => {}
        }
    }
}
