use std::collections::{HashMap, HashSet};

use super::{build_eval_order, evaluate, parse_formula, CellProvider, Expr};

// ---------------------------------------------------------------------------
// Grid-based evaluator (convenience)
// ---------------------------------------------------------------------------

/// A simple cell provider backed by a `Vec<Vec<String>>` grid.
pub struct GridProvider<'a> {
    /// Raw cell values (formulas stored as `=...`).
    pub raw: &'a Vec<Vec<String>>,
    /// Evaluated display values (formula results).
    pub display: &'a HashMap<(usize, usize), String>,
    /// Named ranges: name → (start_row, start_col, end_row, end_col).
    pub names: &'a HashMap<String, (usize, usize, usize, usize)>,
}

impl<'a> CellProvider for GridProvider<'a> {
    #[allow(dead_code)]
    fn get_cell_value(&self, row: usize, col: usize) -> Option<&str> {
        self.raw
            .get(row)
            .and_then(|r| r.get(col))
            .map(|s| s.as_str())
    }

    fn get_cell_display(&self, row: usize, col: usize) -> Option<&str> {
        if let Some(s) = self.display.get(&(row, col)) {
            Some(s.as_str())
        } else {
            self.raw
                .get(row)
                .and_then(|r| r.get(col))
                .map(|s| s.as_str())
        }
    }

    #[allow(dead_code)]
    fn resolve_name(&self, name: &str) -> Option<(usize, usize, usize, usize)> {
        self.names.get(name).copied()
    }
}

/// Evaluate all formula cells in a grid.
///
/// Returns a map of (row, col) → display string for all formula cells.
pub fn evaluate_all_formulas(
    grid: &[Vec<String>],
    names: &HashMap<String, (usize, usize, usize, usize)>,
) -> HashMap<(usize, usize), String> {
    let mut formula_cells = HashSet::new();
    let mut parsed: HashMap<(usize, usize), Expr> = HashMap::new();

    for (r, row) in grid.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            if let Some(stripped) = cell.strip_prefix('=') {
                formula_cells.insert((r, c));
                if let Ok(expr) = parse_formula(stripped) {
                    parsed.insert((r, c), expr);
                }
            }
        }
    }

    // Build evaluation order
    let eval_order = match build_eval_order(&formula_cells, &parsed) {
        Ok(order) => order,
        Err(_) => {
            // Circular reference — mark all formula cells as error
            let mut result = HashMap::new();
            for &cell in &formula_cells {
                result.insert(cell, "#CIRC!".into());
            }
            return result;
        }
    };

    let mut display: HashMap<(usize, usize), String> = HashMap::new();

    for cell in eval_order {
        if let Some(expr) = parsed.get(&cell) {
            let provider = GridProvider {
                raw: &grid.to_vec(), // temporary — we need the original grid
                display: &display,
                names,
            };
            let val = evaluate(expr, &provider);
            display.insert(cell, val.as_str());
        }
    }

    display
}
