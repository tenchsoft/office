use super::*;

impl SheetsState {
    pub fn select_cell(&mut self, row: usize, col: usize) -> bool {
        if self.grid.get(row).and_then(|r| r.get(col)).is_none() {
            return false;
        }
        // If in formula edit mode, insert reference instead of selecting
        if let Some(ref edit) = self.editing_cell {
            if edit.is_formula_edit {
                self.insert_formula_reference(row, col);
                return true;
            }
        }
        self.selected_row = row;
        self.selected_col = col;
        self.selection_anchor = (row, col);
        self.selection_end = None;
        self.select_all_active = false;
        self.formula_draft = self
            .active_cell()
            .map(|cell| cell.value.clone())
            .unwrap_or_default();
        true
    }

    pub fn select_sheet(&mut self, index: usize) -> bool {
        if index >= self.sheet_names.len() {
            return false;
        }
        self.active_sheet = index;
        true
    }

    pub fn move_selection(&mut self, row_delta: isize, col_delta: isize) -> bool {
        let max_row = self.grid.len().saturating_sub(1) as isize;
        let max_col = self
            .grid
            .get(self.selected_row)
            .map(|row| row.len().saturating_sub(1))
            .unwrap_or(0) as isize;
        let next_row = (self.selected_row as isize + row_delta).clamp(0, max_row) as usize;
        let next_col = (self.selected_col as isize + col_delta).clamp(0, max_col) as usize;
        if next_row == self.selected_row && next_col == self.selected_col {
            return false;
        }
        self.select_cell(next_row, next_col)
    }
}
