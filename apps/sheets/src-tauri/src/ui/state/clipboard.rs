use super::*;

impl SheetsState {
    pub fn edit_copy(&mut self) -> bool {
        let (sr, sc, er, ec) = self.selection_range();
        let mut cells: Vec<Vec<CellData>> = Vec::new();
        for r in sr..=er {
            let mut row_cells = Vec::new();
            for c in sc..=ec {
                if let Some(row) = self.grid.get(r) {
                    if let Some(cell) = row.get(c) {
                        row_cells.push(cell.clone());
                    } else {
                        row_cells.push(CellData::val(""));
                    }
                }
            }
            if !row_cells.is_empty() {
                cells.push(row_cells);
            }
        }
        if cells.is_empty() {
            return false;
        }
        self.clipboard.cells = cells;
        self.clipboard.source_row = sr;
        self.clipboard.source_col = sc;
        self.clipboard.is_cut = false;
        self.toast = Some(("Copied".into(), Instant::now()));
        true
    }

    /// Cut the selected range (or active cell).
    pub fn edit_cut(&mut self) -> bool {
        if self.edit_copy() {
            self.push_undo();
            self.clipboard.is_cut = true;
            let (sr, sc, er, ec) = self.selection_range();
            for r in sr..=er {
                for c in sc..=ec {
                    if let Some(row) = self.grid.get_mut(r) {
                        if let Some(cell) = row.get_mut(c) {
                            cell.value.clear();
                            cell.is_formula = false;
                        }
                    }
                }
            }
            self.formula_draft.clear();
            self.sync_content_from_grid();
            self.toast = Some(("Cut".into(), Instant::now()));
            return true;
        }
        false
    }

    /// Paste from the internal clipboard.
    pub fn edit_paste(&mut self) -> bool {
        if self.clipboard.cells.is_empty() {
            return false;
        }
        self.push_undo();
        for (dr, src_row) in self.clipboard.cells.iter().enumerate() {
            for (dc, src_cell) in src_row.iter().enumerate() {
                let target_row = self.selected_row + dr;
                let target_col = self.selected_col + dc;
                if let Some(row) = self.grid.get_mut(target_row) {
                    if let Some(cell) = row.get_mut(target_col) {
                        cell.value = src_cell.value.clone();
                        cell.is_formula = src_cell.is_formula;
                    }
                }
            }
        }

        // If this was a cut, clear the source
        if self.clipboard.is_cut {
            let sr = self.clipboard.source_row;
            let sc = self.clipboard.source_col;
            if let Some(row) = self.grid.get_mut(sr) {
                if let Some(cell) = row.get_mut(sc) {
                    cell.value.clear();
                    cell.is_formula = false;
                }
            }
            self.clipboard.is_cut = false;
        }

        self.formula_draft = self
            .active_cell()
            .map(|c| c.value.clone())
            .unwrap_or_default();
        self.sync_content_from_grid();
        self.toast = Some(("Pasted".into(), Instant::now()));
        true
    }

    /// Paste with a special mode (values only, formulas only, etc.).
    pub fn edit_paste_special(&mut self, mode: PasteSpecialMode) -> bool {
        if self.clipboard.cells.is_empty() {
            return false;
        }
        self.push_undo();
        for (dr, src_row) in self.clipboard.cells.iter().enumerate() {
            for (dc, src_cell) in src_row.iter().enumerate() {
                let target_row = self.selected_row + dr;
                let target_col = self.selected_col + dc;
                if let Some(row) = self.grid.get_mut(target_row) {
                    if let Some(cell) = row.get_mut(target_col) {
                        match mode {
                            PasteSpecialMode::All => {
                                cell.value = src_cell.value.clone();
                                cell.is_formula = src_cell.is_formula;
                            }
                            PasteSpecialMode::ValuesOnly => {
                                if !src_cell.is_formula {
                                    cell.value = src_cell.value.clone();
                                }
                                // If source is a formula, we'd evaluate it — for now, skip
                            }
                            PasteSpecialMode::FormatsOnly => {
                                // Formats are not yet stored per-cell; placeholder
                            }
                            PasteSpecialMode::FormulasOnly => {
                                if src_cell.is_formula {
                                    cell.value = src_cell.value.clone();
                                    cell.is_formula = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.formula_draft = self
            .active_cell()
            .map(|c| c.value.clone())
            .unwrap_or_default();
        self.sync_content_from_grid();
        self.toast = Some(("Paste special".into(), Instant::now()));
        true
    }

    /// Push undo and delete from active cell (for undo support).
    pub fn delete_from_active_cell_with_undo(&mut self) -> bool {
        let has_value = self
            .active_cell()
            .map(|c| !c.value.is_empty())
            .unwrap_or(false);
        if !has_value {
            return false;
        }
        self.push_undo();
        let Some(cell) = self.active_cell_mut() else {
            return false;
        };
        cell.value.pop();
        cell.is_formula = cell.value.starts_with('=');
        self.formula_draft = cell.value.clone();
        self.sync_content_from_grid();
        true
    }

    /// Push undo and push text to active cell (for undo support).
    pub fn push_text_to_active_cell_with_undo(&mut self, text: &str) -> bool {
        if self.active_cell().is_none() {
            return false;
        }
        self.push_undo();
        let Some(cell) = self.active_cell_mut() else {
            return false;
        };
        cell.value.push_str(text);
        cell.is_formula = cell.value.starts_with('=');
        self.formula_draft = cell.value.clone();
        self.sync_content_from_grid();
        true
    }
}
