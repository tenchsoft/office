//! Grid widget — arranges children in a 2D grid.

use kurbo::{Axis, Point, Size};
use smallvec::SmallVec;
use vello::Scene;

use crate::core::widget::{LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetPod};

/// A grid entry: a widget placed at a specific row/column.
struct GridEntry {
    row: usize,
    col: usize,
    widget: WidgetPod,
}

/// A 2D grid layout.
pub struct Grid {
    entries: Vec<GridEntry>,
    columns: usize,
    gap: f64,
}

impl Grid {
    pub fn new(columns: usize) -> Self {
        Self {
            entries: Vec::new(),
            columns: columns.max(1),
            gap: 0.0,
        }
    }

    pub fn with_gap(mut self, gap: f64) -> Self {
        self.gap = gap;
        self
    }

    /// Add a child at the next available position (auto row/col).
    pub fn add_child(&mut self, child: impl Widget + 'static) {
        let index = self.entries.len();
        let row = index / self.columns;
        let col = index % self.columns;
        self.entries.push(GridEntry {
            row,
            col,
            widget: WidgetPod::new(child),
        });
    }

    /// Add a child at a specific row and column.
    pub fn add_child_at(&mut self, row: usize, col: usize, child: impl Widget + 'static) {
        self.entries.push(GridEntry {
            row,
            col,
            widget: WidgetPod::new(child),
        });
    }

    pub fn with_child(mut self, child: impl Widget + 'static) -> Self {
        self.add_child(child);
        self
    }

    fn rows(&self) -> usize {
        self.entries
            .iter()
            .map(|e| e.row)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0)
    }
}

impl Widget for Grid {
    fn measure(&mut self, ctx: &mut MeasureCtx, axis: Axis, available: f64) -> f64 {
        let gap = self.gap;
        let rows = self.rows();
        let cols = self.columns;

        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => {
                // Calculate cell height for each row
                let cell_width = (available - gap * (cols - 1) as f64) / cols as f64;

                let mut row_heights = vec![0.0_f64; rows];
                for entry in &mut self.entries {
                    let mut child_ctx = MeasureCtx {
                        state: &mut entry.widget.state,
                        global: ctx.global,
                    };
                    let h = entry
                        .widget
                        .widget
                        .measure(&mut child_ctx, Axis::Vertical, cell_width);
                    row_heights[entry.row] = row_heights[entry.row].max(h);
                }

                row_heights.iter().sum::<f64>() + gap * (rows.saturating_sub(1)) as f64
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, size: Size) {
        let gap = self.gap;
        let cols = self.columns;
        let rows = self.rows();
        if rows == 0 || cols == 0 {
            return;
        }

        let cell_width = (size.width - gap * (cols - 1) as f64) / cols as f64;

        // First pass: measure row heights
        let mut row_heights = vec![0.0_f64; rows];
        for entry in &mut self.entries {
            let mut child_ctx = MeasureCtx {
                state: &mut entry.widget.state,
                global: ctx.global,
            };
            let h = entry
                .widget
                .widget
                .measure(&mut child_ctx, Axis::Vertical, cell_width);
            row_heights[entry.row] = row_heights[entry.row].max(h);
        }

        // Calculate y offsets for each row
        let mut row_y: SmallVec<[f64; 16]> = SmallVec::with_capacity(rows);
        let mut y = 0.0;
        for row_height in row_heights.iter().take(rows) {
            row_y.push(y);
            y += row_height + gap;
        }

        // Second pass: position each child
        for entry in &mut self.entries {
            let x = entry.col as f64 * (cell_width + gap);
            let y = row_y[entry.row];
            let cell_size = Size::new(cell_width, row_heights[entry.row]);

            entry.widget.state.position = Point::new(x, y);
            entry.widget.state.size = cell_size;

            let mut child_ctx = LayoutCtx {
                state: &mut entry.widget.state,
                global: ctx.global,
            };
            entry.widget.widget.layout(&mut child_ctx, cell_size);
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        for entry in &mut self.entries {
            let theme = ctx.theme().clone();
            let mut child_ctx = PaintCtx {
                state: &mut entry.widget.state,
                global: ctx.global,
                theme,
            };
            entry.widget.widget.paint(&mut child_ctx, scene);
        }
    }

    fn children(&self) -> Vec<crate::core::types::WidgetId> {
        self.entries.iter().map(|e| e.widget.state.id).collect()
    }

    fn child_mut(&mut self, id: crate::core::types::WidgetId) -> Option<&mut WidgetPod> {
        self.entries
            .iter_mut()
            .find(|e| e.widget.state.id == id)
            .map(|e| &mut e.widget)
    }
}
