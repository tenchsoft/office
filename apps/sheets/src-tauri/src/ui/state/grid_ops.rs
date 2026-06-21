use super::*;

impl SheetsState {
    pub fn insert_row(&mut self, at_row: usize) -> bool {
        if at_row > self.grid.len() {
            return false;
        }
        self.push_undo();
        let col_count = self.grid.first().map(|r| r.len()).unwrap_or(0);
        let new_row = vec![CellData::val(""); col_count];
        self.grid.insert(at_row, new_row);
        self.sync_content_from_grid();
        true
    }

    /// Insert a column at the given index.
    pub fn insert_col(&mut self, at_col: usize) -> bool {
        self.push_undo();
        for row in &mut self.grid {
            if at_col <= row.len() {
                row.insert(at_col, CellData::val(""));
            } else {
                row.push(CellData::val(""));
            }
        }
        self.sync_content_from_grid();
        true
    }

    /// Delete a row at the given index.
    pub fn delete_row(&mut self, row: usize) -> bool {
        if row >= self.grid.len() || self.grid.len() <= 1 {
            return false;
        }
        self.push_undo();
        self.grid.remove(row);
        if self.selected_row >= self.grid.len() {
            self.selected_row = self.grid.len() - 1;
        }
        self.sync_content_from_grid();
        true
    }

    /// Delete a column at the given index.
    pub fn delete_col(&mut self, col: usize) -> bool {
        self.push_undo();
        for row in &mut self.grid {
            if col < row.len() {
                row.remove(col);
            }
        }
        if self.selected_col >= self.grid.first().map(|r| r.len()).unwrap_or(0) {
            self.selected_col = self.selected_col.saturating_sub(1);
        }
        self.sync_content_from_grid();
        true
    }

    /// Select all cells.
    pub fn select_all(&mut self) -> bool {
        let max_row = self.grid.len().saturating_sub(1);
        let max_col = self
            .grid
            .first()
            .map(|r| r.len().saturating_sub(1))
            .unwrap_or(0);
        self.selection_anchor = (0, 0);
        self.selection_end = Some((max_row, max_col));
        self.select_all_active = true;
        self.selected_row = 0;
        self.selected_col = 0;
        self.recalculate_status();
        true
    }

    // ----- Phase 2: Range selection -----

    /// Start a range selection from the current cell to (row, col).
    pub fn extend_selection_to(&mut self, row: usize, col: usize) -> bool {
        if self.grid.get(row).and_then(|r| r.get(col)).is_none() {
            return false;
        }
        self.selection_end = Some((row, col));
        self.select_all_active = false;
        self.recalculate_status();
        true
    }

    /// Begin a mouse-drag range selection at (row, col).
    pub fn begin_range_select(&mut self, row: usize, col: usize) -> bool {
        if self.grid.get(row).and_then(|r| r.get(col)).is_none() {
            return false;
        }
        self.selected_row = row;
        self.selected_col = col;
        self.selection_anchor = (row, col);
        self.selection_end = None;
        self.range_selecting = true;
        self.select_all_active = false;
        self.formula_draft = self
            .active_cell()
            .map(|c| c.value.clone())
            .unwrap_or_default();
        true
    }

    /// Update range selection during mouse drag.
    pub fn update_range_select(&mut self, row: usize, col: usize) -> bool {
        if !self.range_selecting {
            return false;
        }
        let row = row.min(self.grid.len().saturating_sub(1));
        let col = col.min(
            self.grid
                .first()
                .map(|r| r.len().saturating_sub(1))
                .unwrap_or(0),
        );
        self.selection_end = Some((row, col));
        self.recalculate_status();
        true
    }

    /// End range selection (mouse up).
    pub fn end_range_select(&mut self) {
        self.range_selecting = false;
    }

    /// Clear range selection (return to single cell).
    pub fn clear_range_selection(&mut self) {
        self.selection_end = None;
        self.select_all_active = false;
    }

    /// Get the effective selection range as (start_row, start_col, end_row, end_col).
    /// Returns a normalized range where start <= end.
    pub fn selection_range(&self) -> (usize, usize, usize, usize) {
        if self.select_all_active {
            let max_row = self.grid.len().saturating_sub(1);
            let max_col = self
                .grid
                .first()
                .map(|r| r.len().saturating_sub(1))
                .unwrap_or(0);
            return (0, 0, max_row, max_col);
        }
        let (anchor_row, anchor_col) = self.selection_anchor;
        match self.selection_end {
            Some((end_row, end_col)) => {
                let min_row = anchor_row.min(end_row);
                let max_row = anchor_row.max(end_row);
                let min_col = anchor_col.min(end_col);
                let max_col = anchor_col.max(end_col);
                (min_row, min_col, max_row, max_col)
            }
            None => (
                self.selected_row,
                self.selected_col,
                self.selected_row,
                self.selected_col,
            ),
        }
    }

    /// Check if a cell is within the current selection range.
    pub fn is_cell_selected(&self, row: usize, col: usize) -> bool {
        let (sr, sc, er, ec) = self.selection_range();
        row >= sr && row <= er && col >= sc && col <= ec
    }

    /// Move selection with optional shift (for range extend).
    pub fn move_selection_with_shift(
        &mut self,
        row_delta: isize,
        col_delta: isize,
        shift: bool,
    ) -> bool {
        let max_row = self.grid.len().saturating_sub(1) as isize;
        let max_col = self
            .grid
            .get(self.selected_row)
            .map(|row| row.len().saturating_sub(1))
            .unwrap_or(0) as isize;
        let next_row = (self.selected_row as isize + row_delta).clamp(0, max_row) as usize;
        let next_col = (self.selected_col as isize + col_delta).clamp(0, max_col) as usize;

        if shift {
            // Extend the range selection
            self.selected_row = next_row;
            self.selected_col = next_col;
            self.selection_end = Some((next_row, next_col));
            self.select_all_active = false;
            self.recalculate_status();
            return true;
        }

        // Normal move: collapse range and move
        if next_row == self.selected_row && next_col == self.selected_col {
            return false;
        }
        self.selection_end = None;
        self.select_all_active = false;
        self.selection_anchor = (next_row, next_col);
        self.select_cell(next_row, next_col)
    }

    /// Jump to the edge of the data region in the given direction (like Excel Ctrl+Arrow).
    pub fn jump_data_edge(
        &self,
        row: usize,
        col: usize,
        drow: isize,
        dcol: isize,
    ) -> (usize, usize) {
        let max_row = self.grid.len().saturating_sub(1);
        let max_col = self
            .grid
            .first()
            .map(|r| r.len().saturating_sub(1))
            .unwrap_or(0);
        let mut r = row;
        let mut c = col;
        let is_empty = |row: usize, col: usize| -> bool {
            self.grid
                .get(row)
                .and_then(|rr| rr.get(col))
                .map(|cell| cell.value.is_empty())
                .unwrap_or(true)
        };
        let was_empty = is_empty(r, c);
        loop {
            let nr = (r as isize + drow).clamp(0, max_row as isize) as usize;
            let nc = (c as isize + dcol).clamp(0, max_col as isize) as usize;
            if nr == r && nc == c {
                break;
            }
            let next_empty = is_empty(nr, nc);
            if was_empty != next_empty {
                break;
            }
            r = nr;
            c = nc;
        }
        (r, c)
    }

    /// Apply inertial scroll decay. Returns true if scroll changed.
    pub fn apply_scroll_inertia(&mut self) -> bool {
        if self.scroll_velocity_x.abs() < 0.1 && self.scroll_velocity_y.abs() < 0.1 {
            return false;
        }
        self.scroll_x += self.scroll_velocity_x;
        self.scroll_y += self.scroll_velocity_y;
        self.scroll_velocity_x *= 0.9;
        self.scroll_velocity_y *= 0.9;
        if self.scroll_velocity_x.abs() < 0.1 {
            self.scroll_velocity_x = 0.0;
        }
        if self.scroll_velocity_y.abs() < 0.1 {
            self.scroll_velocity_y = 0.0;
        }
        true
    }

    /// Freeze panes at the current cursor position.
    pub fn freeze_at_cursor(&mut self) -> bool {
        let new_rows = self.selected_row;
        let new_cols = self.selected_col;
        if new_rows == self.freeze_rows && new_cols == self.freeze_cols {
            // Toggle off
            self.freeze_rows = 0;
            self.freeze_cols = 0;
        } else {
            self.freeze_rows = new_rows;
            self.freeze_cols = new_cols;
        }
        true
    }
}
