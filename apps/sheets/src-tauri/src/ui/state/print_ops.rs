use super::*;

impl SheetsState {
    pub fn get_page_setup(&self) -> &PageSetup {
        &self.page_setup
    }

    /// Replace the entire page setup.
    pub fn set_page_setup(&mut self, setup: PageSetup) {
        self.page_setup = setup;
    }

    /// Set the print area for the current sheet.
    pub fn set_print_area(&mut self, range: CellRange) {
        self.page_setup.print_area = Some(range);
    }

    /// Clear the print area for the current sheet.
    pub fn clear_print_area(&mut self) {
        self.page_setup.print_area = None;
    }

    // ----- 10.2 Print Preview -----

    /// Compute print pages from the current grid and page setup.
    pub fn compute_print_pages(&mut self) {
        let (paper_w_mm, paper_h_mm) = self.page_setup.paper_size.dimensions_mm();
        let (usable_w, usable_h) = match self.page_setup.orientation {
            Orientation::Portrait => (paper_w_mm, paper_h_mm),
            Orientation::Landscape => (paper_h_mm, paper_w_mm),
        };

        let margins = &self.page_setup.margins;
        let content_w = usable_w - margins.left - margins.right;
        let content_h = usable_h - margins.top - margins.bottom;

        // Approximate cell dimensions in mm (assuming ~96 DPI: 1 mm ≈ 3.78 px)
        let cell_w_mm = 100.0 / 3.78; // GRID_COL_W in mm
        let cell_h_mm = 28.0 / 3.78; // GRID_ROW_H in mm

        let cols_per_page = if cell_w_mm > 0.0 {
            (content_w / cell_w_mm).max(1.0) as usize
        } else {
            1
        };
        let rows_per_page = if cell_h_mm > 0.0 {
            (content_h / cell_h_mm).max(1.0) as usize
        } else {
            1
        };

        // Determine the effective grid range
        let total_rows = self.grid.len();
        let total_cols = self.grid.first().map(|r| r.len()).unwrap_or(0);

        let (start_row, end_row, start_col, end_col) =
            if let Some(ref area) = self.page_setup.print_area {
                (
                    area.start_row,
                    area.end_row.min(total_rows.saturating_sub(1)),
                    area.start_col,
                    area.end_col.min(total_cols.saturating_sub(1)),
                )
            } else {
                (
                    0,
                    total_rows.saturating_sub(1),
                    0,
                    total_cols.saturating_sub(1),
                )
            };

        let row_count = end_row.saturating_sub(start_row) + 1;
        let col_count = end_col.saturating_sub(start_col) + 1;

        if row_count == 0 || col_count == 0 {
            self.print_preview.pages.clear();
            return;
        }

        let mut pages = Vec::new();
        let mut page_num = 1;
        let mut r = start_row;
        while r <= end_row {
            let page_end_row = (r + rows_per_page - 1).min(end_row);
            let mut c = start_col;
            while c <= end_col {
                let page_end_col = (c + cols_per_page - 1).min(end_col);
                pages.push(PrintPage {
                    rows: (r, page_end_row),
                    cols: (c, page_end_col),
                    page_number: page_num,
                });
                page_num += 1;
                c = page_end_col + 1;
            }
            r = page_end_row + 1;
        }

        self.print_preview.pages = pages;
        if self.print_preview.current_page >= self.print_preview.pages.len() {
            self.print_preview.current_page = 0;
        }
    }

    // ----- Phase 4: File dialog -----
}
