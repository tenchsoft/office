use super::*;

impl SheetsState {
    pub fn push_undo(&mut self) {
        let snapshot = GridSnapshot {
            grid: self.grid.clone(),
            selected_row: self.selected_row,
            selected_col: self.selected_col,
        };
        if self.undo_stack.len() >= UNDO_LIMIT {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        let Some(snapshot) = self.undo_stack.pop() else {
            return false;
        };
        let current = GridSnapshot {
            grid: self.grid.clone(),
            selected_row: self.selected_row,
            selected_col: self.selected_col,
        };
        self.redo_stack.push(current);
        self.grid = snapshot.grid;
        self.selected_row = snapshot.selected_row;
        self.selected_col = snapshot.selected_col;
        self.formula_draft = self
            .active_cell()
            .map(|c| c.value.clone())
            .unwrap_or_default();
        self.sync_content_from_grid();
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(snapshot) = self.redo_stack.pop() else {
            return false;
        };
        let current = GridSnapshot {
            grid: self.grid.clone(),
            selected_row: self.selected_row,
            selected_col: self.selected_col,
        };
        self.undo_stack.push(current);
        self.grid = snapshot.grid;
        self.selected_row = snapshot.selected_row;
        self.selected_col = snapshot.selected_col;
        self.formula_draft = self
            .active_cell()
            .map(|c| c.value.clone())
            .unwrap_or_default();
        self.sync_content_from_grid();
        true
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    #[allow(dead_code)] // public API — displayed in status bar
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}
