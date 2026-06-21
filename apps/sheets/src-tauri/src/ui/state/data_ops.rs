use super::*;

impl SheetsState {
    pub fn define_name(&mut self, name: String, sheet_idx: Option<usize>, range: CellRange) {
        // If name already exists, update it
        if let Some(existing) = self.named_ranges.iter_mut().find(|nr| nr.name == name) {
            existing.sheet_idx = sheet_idx;
            existing.range = range;
        } else {
            self.named_ranges.push(NamedRange {
                name,
                sheet_idx,
                range,
            });
        }
    }

    /// Resolve a named range to its cell range.
    #[allow(dead_code)] // public API — used by formula evaluator and named-ranges dialog
    pub fn resolve_name(&self, name: &str) -> Option<&CellRange> {
        self.named_ranges
            .iter()
            .find(|nr| nr.name.eq_ignore_ascii_case(name))
            .map(|nr| &nr.range)
    }

    /// Delete a named range by name.
    pub fn delete_named_range(&mut self, name: &str) -> bool {
        let before = self.named_ranges.len();
        self.named_ranges
            .retain(|nr| !nr.name.eq_ignore_ascii_case(name));
        self.named_ranges.len() != before
    }

    // ----- Sort -----

    /// Sort the grid rows by the selected column.
    /// `col` is the column index to sort by. `ascending` controls direction.
    /// `has_header` skips the first row if true.
    pub fn sort_grid_by_col(&mut self, col: usize, ascending: bool, has_header: bool) -> bool {
        if self.grid.is_empty() {
            return false;
        }
        let max_col = self
            .grid
            .first()
            .map(|r| r.len().saturating_sub(1))
            .unwrap_or(0);
        if col > max_col {
            return false;
        }
        self.push_undo();
        let start_row = if has_header { 1 } else { 0 };
        if start_row >= self.grid.len() {
            return false;
        }
        let mut rows: Vec<Vec<CellData>> = self.grid.split_off(start_row);
        rows.sort_by(|a, b| {
            let va = a.get(col).map(|c| c.display()).unwrap_or("");
            let vb = b.get(col).map(|c| c.display()).unwrap_or("");
            // Try numeric comparison first
            match (va.parse::<f64>(), vb.parse::<f64>()) {
                (Ok(na), Ok(nb)) => {
                    if ascending {
                        na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        nb.partial_cmp(&na).unwrap_or(std::cmp::Ordering::Equal)
                    }
                }
                _ => {
                    if ascending {
                        va.cmp(vb)
                    } else {
                        vb.cmp(va)
                    }
                }
            }
        });
        self.grid.append(&mut rows);
        self.sync_content_from_grid();
        true
    }

    // ----- Phase 6: Data operations -----

    /// Hide a row by adding it to `hidden_rows`.
    pub fn hide_row(&mut self, row: usize) {
        if !self.hidden_rows.contains(&row) {
            self.hidden_rows.push(row);
            self.hidden_rows.sort_unstable();
        }
    }

    /// Hide a column by adding it to `hidden_cols`.
    pub fn hide_col(&mut self, col: usize) {
        if !self.hidden_cols.contains(&col) {
            self.hidden_cols.push(col);
            self.hidden_cols.sort_unstable();
        }
    }

    /// Unhide all rows.
    pub fn unhide_all_rows(&mut self) {
        self.hidden_rows.clear();
    }

    /// Unhide all columns.
    pub fn unhide_all_cols(&mut self) {
        self.hidden_cols.clear();
    }

    /// Check if a row is hidden.
    pub fn is_row_hidden(&self, row: usize) -> bool {
        self.hidden_rows.contains(&row)
    }

    /// Check if a column is hidden.
    pub fn is_col_hidden(&self, col: usize) -> bool {
        self.hidden_cols.contains(&col)
    }

    /// Toggle filter mode on/off. When turning on, enables filter arrows on headers.
    pub fn toggle_filter(&mut self) {
        self.filter_active = !self.filter_active;
        if !self.filter_active {
            // Clear all filter state when turning off
            self.filter_col = None;
            self.filter_values.clear();
            self.filter_hidden_rows.clear();
            self.show_filter_dropdown = false;
            self.filter_dropdown_col = None;
        }
    }

    /// Get unique values in a column (excluding header row if filter is active).
    pub fn unique_values_for_col(&self, col: usize) -> Vec<String> {
        let mut values: Vec<String> = Vec::new();
        let start_row = if self.sort_has_header { 1 } else { 0 };
        for r in start_row..self.grid.len() {
            if let Some(row) = self.grid.get(r) {
                if let Some(cell) = row.get(col) {
                    let v = cell.display().to_string();
                    if !values.contains(&v) {
                        values.push(v);
                    }
                }
            }
        }
        values.sort();
        values
    }

    /// Apply a filter: hide rows where the specified column's value is NOT in `show_values`.
    pub fn apply_filter(&mut self, col: usize, show_values: &[String]) {
        self.filter_col = Some(col);
        self.filter_values = show_values.to_vec();
        self.filter_hidden_rows.clear();
        let start_row = if self.sort_has_header { 1 } else { 0 };
        for r in start_row..self.grid.len() {
            if let Some(row) = self.grid.get(r) {
                let cell_val = row.get(col).map(|c| c.display()).unwrap_or("");
                if !show_values.iter().any(|v| v == cell_val) {
                    self.filter_hidden_rows.push(r);
                }
            }
        }
    }

    /// Clear the current filter (show all rows again).
    pub fn clear_filter(&mut self) {
        self.filter_col = None;
        self.filter_values.clear();
        self.filter_hidden_rows.clear();
    }

    /// Check if a row is hidden by a filter.
    pub fn is_row_filtered(&self, row: usize) -> bool {
        self.filter_hidden_rows.contains(&row)
    }

    /// Check if a row should be visible (not hidden and not filtered).
    pub fn is_row_visible(&self, row: usize) -> bool {
        !self.is_row_hidden(row) && !self.is_row_filtered(row)
    }

    /// Check if a column should be visible (not hidden).
    pub fn is_col_visible(&self, col: usize) -> bool {
        !self.is_col_hidden(col)
    }
}
