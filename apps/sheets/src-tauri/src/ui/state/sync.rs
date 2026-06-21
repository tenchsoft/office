use super::*;
use std::collections::HashMap;

use tench_office_io::sheets::format as format_io;
use tench_sheets_core::formula::evaluate_all_formulas;

impl SheetsState {
    pub(super) fn with_recalculated_status(mut self) -> Self {
        self.recalculate_status();
        self
    }

    pub fn sync_content_from_grid(&mut self) {
        self.evaluate_formulas();
        let csv = grid_to_csv(&self.grid);
        self.content = format_io::csv_to_workbook_content(&csv, &self.workbook_name);
        self.artifact.dirty = csv != self.last_saved_csv;
        if self.artifact.dirty {
            self.status = "Unsaved changes".into();
        }
        self.recalculate_status();
    }

    /// Evaluate all formula cells and update their `display_value`.
    fn evaluate_formulas(&mut self) {
        // Build a simple string grid for the formula engine
        let rows = self.grid.len();
        let cols = self.grid.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut str_grid: Vec<Vec<String>> = Vec::with_capacity(rows);
        for r in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for c in 0..cols {
                let val = self
                    .grid
                    .get(r)
                    .and_then(|rr| rr.get(c))
                    .map(|cd| cd.value.as_str())
                    .unwrap_or("");
                row.push(val.into());
            }
            str_grid.push(row);
        }

        // Build named ranges map
        let names: HashMap<String, (usize, usize, usize, usize)> = self
            .named_ranges
            .iter()
            .map(|nr| {
                let key = nr.name.to_uppercase();
                (
                    key,
                    (
                        nr.range.start_row,
                        nr.range.start_col,
                        nr.range.end_row,
                        nr.range.end_col,
                    ),
                )
            })
            .collect();

        let results = evaluate_all_formulas(&str_grid, &names);
        self.formula_cache = results.clone();

        // Update display_value on formula cells
        for r in 0..rows {
            for c in 0..cols {
                if let Some(cell) = self.grid.get_mut(r).and_then(|rr| rr.get_mut(c)) {
                    if cell.is_formula {
                        if let Some(display) = results.get(&(r, c)) {
                            cell.display_value = display.clone();
                        }
                    }
                }
            }
        }
    }

    pub fn recalculate_status(&mut self) {
        let (sr, sc, er, ec) = self.selection_range();
        let values: Vec<f64> = self
            .grid
            .iter()
            .enumerate()
            .flat_map(|(r, row)| {
                row.iter().enumerate().filter_map(move |(c, cell)| {
                    if r < sr || r > er || c < sc || c > ec {
                        return None;
                    }
                    let s = cell.display();
                    if s.is_empty() {
                        return None;
                    }
                    s.parse::<f64>().ok()
                })
            })
            .collect();
        let sum: f64 = values.iter().sum();
        self.status_count = values.len();
        self.status_sum = format_number(sum);
        self.status_average = if values.is_empty() {
            "0".into()
        } else {
            format_number(sum / values.len() as f64)
        };
        let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
        self.status_min = if min_val == f64::INFINITY {
            String::new()
        } else {
            format_number(min_val)
        };
        let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        self.status_max = if max_val == f64::NEG_INFINITY {
            String::new()
        } else {
            format_number(max_val)
        };
    }
}
