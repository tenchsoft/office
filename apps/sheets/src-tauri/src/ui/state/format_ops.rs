use super::*;

impl SheetsState {
    pub fn toggle_bold(&mut self) -> bool {
        self.push_undo();
        let (sr, sc, er, ec) = self.selection_range();
        for r in sr..=er {
            for c in sc..=ec {
                if let Some(cell) = self.grid.get_mut(r).and_then(|row| row.get_mut(c)) {
                    cell.format.bold = !cell.format.bold;
                }
            }
        }
        self.sync_content_from_grid();
        true
    }

    /// Toggle italic on the selected cell(s).
    pub fn toggle_italic(&mut self) -> bool {
        self.push_undo();
        let (sr, sc, er, ec) = self.selection_range();
        for r in sr..=er {
            for c in sc..=ec {
                if let Some(cell) = self.grid.get_mut(r).and_then(|row| row.get_mut(c)) {
                    cell.format.italic = !cell.format.italic;
                }
            }
        }
        self.sync_content_from_grid();
        true
    }

    /// Toggle underline on the selected cell(s).
    pub fn toggle_underline(&mut self) -> bool {
        self.push_undo();
        let (sr, sc, er, ec) = self.selection_range();
        for r in sr..=er {
            for c in sc..=ec {
                if let Some(cell) = self.grid.get_mut(r).and_then(|row| row.get_mut(c)) {
                    cell.format.underline = !cell.format.underline;
                }
            }
        }
        self.sync_content_from_grid();
        true
    }

    /// Set horizontal alignment on the selected cell(s).
    pub fn set_h_align(&mut self, align: HorizontalAlignment) -> bool {
        self.push_undo();
        let (sr, sc, er, ec) = self.selection_range();
        for r in sr..=er {
            for c in sc..=ec {
                if let Some(cell) = self.grid.get_mut(r).and_then(|row| row.get_mut(c)) {
                    cell.format.h_align = Some(align);
                }
            }
        }
        self.sync_content_from_grid();
        true
    }

    /// Set text color on the selected cell(s).
    pub fn set_text_color(&mut self, color: Color) -> bool {
        self.push_undo();
        let (sr, sc, er, ec) = self.selection_range();
        for r in sr..=er {
            for c in sc..=ec {
                if let Some(cell) = self.grid.get_mut(r).and_then(|row| row.get_mut(c)) {
                    cell.format.text_color = Some(color);
                }
            }
        }
        self.sync_content_from_grid();
        true
    }

    /// Set background color on the selected cell(s).
    pub fn set_bg_color(&mut self, color: Color) -> bool {
        self.push_undo();
        let (sr, sc, er, ec) = self.selection_range();
        for r in sr..=er {
            for c in sc..=ec {
                if let Some(cell) = self.grid.get_mut(r).and_then(|row| row.get_mut(c)) {
                    cell.format.bg_color = Some(color);
                }
            }
        }
        self.sync_content_from_grid();
        true
    }

    /// Set number format on the selected cell(s).
    pub fn set_number_format(&mut self, fmt: NumberFormat) -> bool {
        self.push_undo();
        let (sr, sc, er, ec) = self.selection_range();
        for r in sr..=er {
            for c in sc..=ec {
                if let Some(cell) = self.grid.get_mut(r).and_then(|row| row.get_mut(c)) {
                    cell.format.number_format = fmt.clone();
                }
            }
        }
        self.sync_content_from_grid();
        true
    }

    /// Apply a full format to the selected cell(s).
    pub fn apply_format(&mut self, fmt: &CellFormat) -> bool {
        self.push_undo();
        let (sr, sc, er, ec) = self.selection_range();
        for r in sr..=er {
            for c in sc..=ec {
                if let Some(cell) = self.grid.get_mut(r).and_then(|row| row.get_mut(c)) {
                    cell.format = fmt.clone();
                }
            }
        }
        self.sync_content_from_grid();
        true
    }

    /// Get the format of the active cell.
    pub fn active_cell_format(&self) -> CellFormat {
        self.active_cell()
            .map(|c| c.format.clone())
            .unwrap_or_default()
    }

    /// Check if the active cell is bold.
    pub fn is_active_bold(&self) -> bool {
        self.active_cell().map(|c| c.format.bold).unwrap_or(false)
    }

    /// Check if the active cell is italic.
    pub fn is_active_italic(&self) -> bool {
        self.active_cell().map(|c| c.format.italic).unwrap_or(false)
    }

    /// Check if the active cell is underlined.
    pub fn is_active_underline(&self) -> bool {
        self.active_cell()
            .map(|c| c.format.underline)
            .unwrap_or(false)
    }

    /// Get the effective row height for a given row index.
    pub fn row_height(&self, row: usize) -> f64 {
        self.row_heights.get(row).copied().unwrap_or(28.0)
    }

    /// Get the effective column width for a given column index.
    pub fn col_width(&self, col: usize) -> f64 {
        self.col_widths.get(col).copied().unwrap_or(100.0)
    }

    /// Set the height of a specific row.
    pub fn set_row_height(&mut self, row: usize, height: f64) {
        if self.row_heights.len() <= row {
            self.row_heights.resize(row + 1, 28.0);
        }
        self.row_heights[row] = height.max(12.0);
    }

    /// Set the width of a specific column.
    pub fn set_col_width(&mut self, col: usize, width: f64) {
        if self.col_widths.len() <= col {
            self.col_widths.resize(col + 1, 100.0);
        }
        self.col_widths[col] = width.max(20.0);
    }

    // ----- Cell merge -----

    /// Merge the selected cells into one.
    pub fn merge_selected(&mut self) -> bool {
        let (sr, sc, er, ec) = self.selection_range();
        if sr == er && sc == ec {
            return false; // Single cell, nothing to merge
        }
        // Check if already merged
        if self
            .merged_cells
            .iter()
            .any(|m| m.start_row == sr && m.start_col == sc && m.end_row == er && m.end_col == ec)
        {
            return false;
        }
        // Remove any overlapping merges
        self.merged_cells.retain(|m| {
            !(m.start_row <= er && m.end_row >= sr && m.start_col <= ec && m.end_col >= sc)
        });
        self.merged_cells.push(MergedCell {
            start_row: sr,
            start_col: sc,
            end_row: er,
            end_col: ec,
        });
        true
    }

    /// Unmerge the selected cells.
    pub fn unmerge_selected(&mut self) -> bool {
        let (sr, sc, er, ec) = self.selection_range();
        let before = self.merged_cells.len();
        self.merged_cells.retain(|m| {
            !(m.start_row >= sr && m.end_row <= er && m.start_col >= sc && m.end_col <= ec)
        });
        self.merged_cells.len() != before
    }

    /// Check if a cell is the top-left of a merged region.
    pub fn merged_region_start(&self, row: usize, col: usize) -> Option<&MergedCell> {
        self.merged_cells
            .iter()
            .find(|m| m.start_row == row && m.start_col == col)
    }

    /// Check if a cell is part of any merged region (but not the top-left).
    pub fn is_merged_hidden(&self, row: usize, col: usize) -> bool {
        self.merged_cells.iter().any(|m| {
            row >= m.start_row
                && row <= m.end_row
                && col >= m.start_col
                && col <= m.end_col
                && !(row == m.start_row && col == m.start_col)
        })
    }

    // ----- Format painter -----

    /// Start the format painter from the current cell.
    pub fn start_format_painter(&mut self) {
        self.format_painter_source = Some(self.active_cell_format());
        self.format_painter_active = true;
    }

    /// Apply the format painter to a target cell.
    pub fn apply_format_painter(&mut self, row: usize, col: usize) -> bool {
        let fmt = match &self.format_painter_source {
            Some(f) => f.clone(),
            None => return false,
        };
        self.push_undo();
        if let Some(cell) = self.grid.get_mut(row).and_then(|r| r.get_mut(col)) {
            cell.format = fmt;
        }
        self.format_painter_active = false;
        self.format_painter_source = None;
        self.sync_content_from_grid();
        true
    }

    /// Cancel the format painter.
    pub fn cancel_format_painter(&mut self) {
        self.format_painter_active = false;
        self.format_painter_source = None;
    }

    // ----- Conditional formatting -----

    /// Evaluate conditional formats for a cell and return (bg_color, text_color) if any match.
    pub fn conditional_format_colors(
        &self,
        row: usize,
        col: usize,
    ) -> (Option<Color>, Option<Color>) {
        let cell_value = self
            .grid
            .get(row)
            .and_then(|r| r.get(col))
            .and_then(|c| c.display().parse::<f64>().ok());

        let mut bg = None;
        let mut fg = None;

        for rule in &self.conditional_formats {
            if col != rule.col {
                continue;
            }
            if row < rule.row_range.0 || row > rule.row_range.1 {
                continue;
            }
            let val = match cell_value {
                Some(v) => v,
                None => continue,
            };
            let matches = match rule.condition {
                ConditionOp::GreaterThan => val > rule.value,
                ConditionOp::LessThan => val < rule.value,
                ConditionOp::EqualTo => (val - rule.value).abs() < f64::EPSILON,
                ConditionOp::Between => val >= rule.value && rule.value2.is_none_or(|v2| val <= v2),
            };
            if matches {
                bg = Some(rule.bg_color);
                fg = Some(rule.text_color);
            }
        }

        (bg, fg)
    }

    /// Open the Format Cells dialog for the active cell.
    pub fn open_format_cells_dialog(&mut self) {
        self.format_cells.draft = self.active_cell_format();
        self.format_cells.active_tab = FormatCellsTab::default();
        self.format_cells.visible = true;
    }

    /// Apply the Format Cells dialog draft to the selection.
    pub fn apply_format_cells_dialog(&mut self) {
        let fmt = self.format_cells.draft.clone();
        self.apply_format(&fmt);
        self.format_cells.visible = false;
    }

    /// Convenience: toggle bold on selected cell(s) — wrapper for toolbar.
    pub fn toggle_format_bold(&mut self) -> bool {
        self.toggle_bold()
    }

    /// Convenience: toggle italic on selected cell(s) — wrapper for toolbar.
    pub fn toggle_format_italic(&mut self) -> bool {
        self.toggle_italic()
    }

    /// Convenience: toggle underline on selected cell(s) — wrapper for toolbar.
    pub fn toggle_format_underline(&mut self) -> bool {
        self.toggle_underline()
    }

    /// Convenience: set horizontal alignment — wrapper for toolbar.
    pub fn set_format_h_align(&mut self, align: HorizontalAlignment) -> bool {
        self.set_h_align(align)
    }

    /// Convenience: set background color — wrapper for dialogs.
    pub fn set_format_bg_color(&mut self, color: Color) -> bool {
        self.set_bg_color(color)
    }

    /// Get the format of the currently selected cell — wrapper for toolbar.
    pub fn get_selected_cell_format(&self) -> CellFormat {
        self.active_cell_format()
    }

    /// Cycle through number formats: General -> Number -> Currency -> Percentage -> Date -> Text -> General
    pub fn cycle_number_format(&mut self) -> bool {
        let current = &self.active_cell_format().number_format;
        let next = match current {
            NumberFormat::General => NumberFormat::Number {
                decimals: 2,
                thousands_sep: true,
            },
            NumberFormat::Number { .. } => NumberFormat::Currency {
                symbol: "$".into(),
                decimals: 2,
            },
            NumberFormat::Currency { .. } => NumberFormat::Percentage { decimals: 0 },
            NumberFormat::Percentage { .. } => NumberFormat::Date,
            NumberFormat::Date => NumberFormat::Text,
            NumberFormat::Text => NumberFormat::General,
        };
        self.set_number_format(next)
    }

    /// Toggle merge/unmerge on the current selection.
    pub fn toggle_merge_selection(&mut self) -> bool {
        // Check if already merged at selected cell
        if self.is_merged_hidden(self.selected_row, self.selected_col) {
            // Can't unmerge from a hidden cell, find the merge start
            return false;
        }
        if self
            .merged_region_start(self.selected_row, self.selected_col)
            .is_some()
        {
            self.unmerge_selected()
        } else {
            self.merge_selected()
        }
    }
}
