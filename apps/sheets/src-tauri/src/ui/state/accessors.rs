use super::*;

impl SheetsState {
    pub fn current_artifact(&self) -> &OfficeArtifact {
        &self.artifact
    }

    pub fn current_content(&self) -> &OfficeContent {
        &self.content
    }

    pub fn is_dirty(&self) -> bool {
        self.artifact.dirty
    }

    pub fn status_line(&self) -> &str {
        &self.status
    }

    pub fn active_cell_ref(&self) -> String {
        format!("{}{}", col_letter(self.selected_col), self.selected_row + 1)
    }

    pub fn active_cell(&self) -> Option<&CellData> {
        self.grid
            .get(self.selected_row)
            .and_then(|row| row.get(self.selected_col))
    }

    pub fn active_cell_mut(&mut self) -> Option<&mut CellData> {
        self.grid
            .get_mut(self.selected_row)
            .and_then(|row| row.get_mut(self.selected_col))
    }
}
