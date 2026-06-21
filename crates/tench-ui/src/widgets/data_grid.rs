//! DataGrid widget for large tabular product data.
//!
//! The widget exposes viewport math separately from painting so products can
//! test selection, keyboard navigation, and virtualization without a GPU.

use kurbo::{Axis, Point, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::{LogicalKey, NamedKey, PointerEvent, TextEvent};
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;

#[derive(Clone, Debug, PartialEq)]
pub struct DataGridColumn {
    pub id: String,
    pub title: String,
    pub width: f64,
    pub min_width: f64,
    pub sortable: bool,
    pub pinned: bool,
}

impl DataGridColumn {
    pub fn new(id: impl Into<String>, title: impl Into<String>, width: f64) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            width: width.max(48.0),
            min_width: 48.0,
            sortable: false,
            pinned: false,
        }
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataGridCell {
    pub text: String,
    pub aria_label: Option<String>,
}

impl DataGridCell {
    pub fn text(value: impl Into<String>) -> Self {
        Self {
            text: value.into(),
            aria_label: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataGridRow {
    pub id: String,
    pub cells: Vec<DataGridCell>,
    pub selected: bool,
}

impl DataGridRow {
    pub fn new(id: impl Into<String>, cells: Vec<DataGridCell>) -> Self {
        Self {
            id: id.into(),
            cells,
            selected: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataGridViewport {
    pub scroll_x: f64,
    pub scroll_y: f64,
    pub row_height: f64,
    pub header_height: f64,
    pub overscan_rows: usize,
}

impl Default for DataGridViewport {
    fn default() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            row_height: 32.0,
            header_height: 34.0,
            overscan_rows: 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataGridVisibleRange {
    pub first_row: usize,
    pub last_row: usize,
    pub first_column: usize,
    pub last_column: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataGridHitTarget {
    Header,
    Cell,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataGridHit {
    pub target: DataGridHitTarget,
    pub row_id: Option<String>,
    pub row_index: Option<usize>,
    pub column_id: String,
    pub column_index: usize,
}

pub fn data_grid_visible_range(
    columns: &[DataGridColumn],
    row_count: usize,
    viewport_size: Size,
    viewport: DataGridViewport,
) -> DataGridVisibleRange {
    let body_height = (viewport_size.height - viewport.header_height).max(0.0);
    let row_height = viewport.row_height.max(1.0);
    let first_row = ((viewport.scroll_y / row_height).floor() as usize)
        .saturating_sub(viewport.overscan_rows)
        .min(row_count);
    let visible_rows = (body_height / row_height).ceil().max(0.0) as usize
        + 1
        + viewport.overscan_rows.saturating_mul(2);
    let last_row = (first_row + visible_rows).min(row_count);

    let mut first_column = None;
    let mut last_column = None;
    let mut x = 0.0;
    for (index, column) in columns.iter().enumerate() {
        let width = column.width.max(column.min_width);
        let start = x - viewport.scroll_x;
        let end = start + width;
        if end >= 0.0 && start <= viewport_size.width {
            first_column.get_or_insert(index);
            last_column = Some(index + 1);
        }
        x += width;
    }

    DataGridVisibleRange {
        first_row,
        last_row,
        first_column: first_column.unwrap_or(0),
        last_column: last_column.unwrap_or(first_column.unwrap_or(0)),
    }
}

pub fn data_grid_hit_test(
    columns: &[DataGridColumn],
    rows: &[DataGridRow],
    viewport: DataGridViewport,
    point: Point,
) -> Option<DataGridHit> {
    let column_index = column_at_x(columns, point.x + viewport.scroll_x)?;
    let column_id = columns[column_index].id.clone();
    if point.y < viewport.header_height {
        return Some(DataGridHit {
            target: DataGridHitTarget::Header,
            row_id: None,
            row_index: None,
            column_id,
            column_index,
        });
    }

    let row_index = ((point.y - viewport.header_height + viewport.scroll_y)
        / viewport.row_height.max(1.0))
    .floor() as usize;
    rows.get(row_index).map(|row| DataGridHit {
        target: DataGridHitTarget::Cell,
        row_id: Some(row.id.clone()),
        row_index: Some(row_index),
        column_id,
        column_index,
    })
}

pub fn data_grid_keyboard_target_row(
    current: Option<usize>,
    row_count: usize,
    visible_rows: usize,
    key: NamedKey,
) -> Option<usize> {
    if row_count == 0 {
        return None;
    }
    let current = current.unwrap_or(0).min(row_count - 1);
    let page = visible_rows.max(1);
    match key {
        NamedKey::ArrowUp => Some(current.saturating_sub(1)),
        NamedKey::ArrowDown => Some((current + 1).min(row_count - 1)),
        NamedKey::Home => Some(0),
        NamedKey::End => Some(row_count - 1),
        NamedKey::PageUp => Some(current.saturating_sub(page)),
        NamedKey::PageDown => Some((current + page).min(row_count - 1)),
        _ => None,
    }
}

pub struct DataGrid {
    columns: Vec<DataGridColumn>,
    rows: Vec<DataGridRow>,
    viewport: DataGridViewport,
    // clippy: callback field type is idiomatic for this widget
    #[allow(clippy::type_complexity)]
    on_select: Option<Box<dyn Fn(&str) + Send>>,
    size: Size,
}

impl DataGrid {
    pub fn new(columns: Vec<DataGridColumn>, rows: Vec<DataGridRow>) -> Self {
        Self {
            columns,
            rows,
            viewport: DataGridViewport::default(),
            on_select: None,
            size: Size::ZERO,
        }
    }

    pub fn with_viewport(mut self, viewport: DataGridViewport) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn on_select(mut self, callback: impl Fn(&str) + Send + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    pub fn viewport(&self) -> DataGridViewport {
        self.viewport
    }

    pub fn visible_range(&self, size: Size) -> DataGridVisibleRange {
        data_grid_visible_range(&self.columns, self.rows.len(), size, self.viewport)
    }

    pub fn hit_test(&self, point: Point) -> Option<DataGridHit> {
        data_grid_hit_test(&self.columns, &self.rows, self.viewport, point)
    }

    fn max_scroll_y(&self, height: f64) -> f64 {
        let body_height = (height - self.viewport.header_height).max(0.0);
        (self.rows.len() as f64 * self.viewport.row_height - body_height).max(0.0)
    }

    fn selected_row_index(&self) -> Option<usize> {
        self.rows.iter().position(|row| row.selected)
    }

    fn visible_row_count(&self) -> usize {
        ((self.size.height - self.viewport.header_height).max(0.0)
            / self.viewport.row_height.max(1.0))
        .floor()
        .max(1.0) as usize
    }

    fn select_row_index(&mut self, row_index: usize) -> Option<String> {
        if row_index >= self.rows.len() {
            return None;
        }
        for row in &mut self.rows {
            row.selected = false;
        }
        let row = &mut self.rows[row_index];
        row.selected = true;
        Some(row.id.clone())
    }

    fn scroll_row_into_view(&mut self, row_index: usize) {
        let body_height = (self.size.height - self.viewport.header_height).max(0.0);
        let row_top = row_index as f64 * self.viewport.row_height;
        let row_bottom = row_top + self.viewport.row_height;
        if row_top < self.viewport.scroll_y {
            self.viewport.scroll_y = row_top;
        } else if row_bottom > self.viewport.scroll_y + body_height {
            self.viewport.scroll_y = (row_bottom - body_height).max(0.0);
        }
        self.viewport.scroll_y = self.viewport.scroll_y.clamp(
            0.0,
            self.max_scroll_y(self.size.height.max(self.viewport.header_height)),
        );
    }
}

impl Widget for DataGrid {
    fn measure(&mut self, _ctx: &mut MeasureCtx<'_>, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => available,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>, size: Size) {
        self.size = size;
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, scene: &mut Scene) {
        let theme = ctx.theme().clone();
        let size = ctx.size();
        let bounds = Rect::from_origin_size(Point::ZERO, size);
        let range = self.visible_range(size);
        let mut painter = Painter::new(scene);

        painter.fill_rect(bounds, theme.background);
        painter.fill_rect(
            Rect::new(0.0, 0.0, size.width, self.viewport.header_height),
            theme.surface,
        );

        let mut x = -self.viewport.scroll_x;
        for (column_index, column) in self.columns.iter().enumerate() {
            let width = column.width.max(column.min_width);
            if column_index >= range.first_column && column_index < range.last_column {
                painter.draw_text(
                    &column.title,
                    x + 8.0,
                    self.viewport.header_height / 2.0 + 4.0,
                    theme.secondary,
                    theme.font_size_small,
                    FontWeight::BOLD,
                    false,
                );
                painter.draw_line(
                    Point::new(x + width, 0.0),
                    Point::new(x + width, size.height),
                    theme.border,
                    0.5,
                );
            }
            x += width;
        }
        painter.draw_line(
            Point::new(0.0, self.viewport.header_height),
            Point::new(size.width, self.viewport.header_height),
            theme.border,
            1.0,
        );

        for row_index in range.first_row..range.last_row {
            let Some(row) = self.rows.get(row_index) else {
                continue;
            };
            let y = self.viewport.header_height + row_index as f64 * self.viewport.row_height
                - self.viewport.scroll_y;
            let row_rect = Rect::new(0.0, y, size.width, y + self.viewport.row_height);
            if row.selected {
                painter.fill_rect(row_rect, theme.primary);
            } else if row_index % 2 == 1 {
                painter.fill_rect(row_rect, Color::rgba8(255, 255, 255, 8));
            }
            painter.draw_line(
                Point::new(0.0, y + self.viewport.row_height),
                Point::new(size.width, y + self.viewport.row_height),
                theme.border,
                0.5,
            );

            let mut cell_x = -self.viewport.scroll_x;
            for (column_index, column) in self.columns.iter().enumerate() {
                let width = column.width.max(column.min_width);
                if column_index >= range.first_column && column_index < range.last_column {
                    if let Some(cell) = row.cells.get(column_index) {
                        painter.draw_text(
                            &cell.text,
                            cell_x + 8.0,
                            y + self.viewport.row_height / 2.0 + 5.0,
                            if row.selected {
                                theme.on_primary
                            } else {
                                theme.on_background
                            },
                            theme.font_size_small,
                            FontWeight::NORMAL,
                            false,
                        );
                    }
                }
                cell_x += width;
            }
        }
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx<'_>, event: &PointerEvent) {
        match event {
            PointerEvent::Down(event) => {
                if let Some(hit) = self.hit_test(event.pos) {
                    if let Some(row_index) = hit.row_index {
                        for row in &mut self.rows {
                            row.selected = false;
                        }
                        if let Some(row) = self.rows.get_mut(row_index) {
                            row.selected = true;
                            if let Some(callback) = &self.on_select {
                                callback(&row.id);
                            }
                            ctx.request_paint();
                        }
                    }
                }
            }
            PointerEvent::Scroll(event) => {
                self.viewport.scroll_y = (self.viewport.scroll_y + event.delta.y * 30.0)
                    .clamp(0.0, self.max_scroll_y(600.0));
                self.viewport.scroll_x = (self.viewport.scroll_x + event.delta.x * 30.0).max(0.0);
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn on_text_event(&mut self, ctx: &mut EventCtx<'_>, event: &TextEvent) {
        let TextEvent::Keyboard(event) = event else {
            return;
        };
        if !event.is_pressed {
            return;
        }
        let LogicalKey::Named(key) = &event.logical_key else {
            return;
        };
        let Some(target) = data_grid_keyboard_target_row(
            self.selected_row_index(),
            self.rows.len(),
            self.visible_row_count(),
            *key,
        ) else {
            return;
        };
        if let Some(row_id) = self.select_row_index(target) {
            self.scroll_row_into_view(target);
            if let Some(callback) = &self.on_select {
                callback(&row_id);
            }
            ctx.request_paint();
        }
    }

    fn accepts_focus(&self) -> bool {
        true
    }
}

fn column_at_x(columns: &[DataGridColumn], x: f64) -> Option<usize> {
    let mut current = 0.0;
    for (index, column) in columns.iter().enumerate() {
        let width = column.width.max(column.min_width);
        if x >= current && x <= current + width {
            return Some(index);
        }
        current += width;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn columns() -> Vec<DataGridColumn> {
        vec![
            DataGridColumn::new("title", "Title", 180.0),
            DataGridColumn::new("year", "Year", 80.0),
            DataGridColumn::new("status", "Status", 120.0),
        ]
    }

    #[test]
    fn visible_range_applies_vertical_overscan_and_horizontal_scroll() {
        let viewport = DataGridViewport {
            scroll_x: 190.0,
            scroll_y: 320.0,
            row_height: 32.0,
            header_height: 34.0,
            overscan_rows: 1,
        };

        let range = data_grid_visible_range(&columns(), 1_000, Size::new(200.0, 130.0), viewport);

        assert_eq!(range.first_row, 9);
        assert_eq!(range.last_row, 15);
        assert_eq!(range.first_column, 1);
        assert_eq!(range.last_column, 3);
    }

    #[test]
    fn hit_test_distinguishes_header_and_body_cell() {
        let rows = vec![DataGridRow::new(
            "row_1",
            vec![DataGridCell::text("Paper"), DataGridCell::text("2026")],
        )];

        let header = data_grid_hit_test(
            &columns(),
            &rows,
            DataGridViewport::default(),
            Point::new(190.0, 8.0),
        )
        .expect("header hit");
        let cell = data_grid_hit_test(
            &columns(),
            &rows,
            DataGridViewport::default(),
            Point::new(190.0, 48.0),
        )
        .expect("cell hit");

        assert_eq!(header.target, DataGridHitTarget::Header);
        assert_eq!(header.column_id, "year");
        assert_eq!(cell.target, DataGridHitTarget::Cell);
        assert_eq!(cell.row_id.as_deref(), Some("row_1"));
    }

    #[test]
    fn keyboard_target_row_supports_arrows_home_end_and_pages() {
        assert_eq!(
            data_grid_keyboard_target_row(Some(5), 20, 4, NamedKey::ArrowUp),
            Some(4)
        );
        assert_eq!(
            data_grid_keyboard_target_row(Some(5), 20, 4, NamedKey::ArrowDown),
            Some(6)
        );
        assert_eq!(
            data_grid_keyboard_target_row(Some(5), 20, 4, NamedKey::Home),
            Some(0)
        );
        assert_eq!(
            data_grid_keyboard_target_row(Some(5), 20, 4, NamedKey::End),
            Some(19)
        );
        assert_eq!(
            data_grid_keyboard_target_row(Some(5), 20, 4, NamedKey::PageUp),
            Some(1)
        );
        assert_eq!(
            data_grid_keyboard_target_row(Some(5), 20, 4, NamedKey::PageDown),
            Some(9)
        );
        assert_eq!(
            data_grid_keyboard_target_row(None, 20, 4, NamedKey::ArrowDown),
            Some(1)
        );
    }
}
