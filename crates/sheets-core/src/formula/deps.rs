use std::collections::{HashMap, HashSet, VecDeque};

use super::{Expr, FormulaError};

// ---------------------------------------------------------------------------
// Dependency tracking
// ---------------------------------------------------------------------------

/// Collect all cell references referenced by an expression.
pub fn collect_dependencies(expr: &Expr) -> HashSet<(usize, usize)> {
    let mut deps = HashSet::new();
    collect_deps_recursive(expr, &mut deps);
    deps
}

fn collect_deps_recursive(expr: &Expr, deps: &mut HashSet<(usize, usize)>) {
    match expr {
        Expr::CellRef { col, row } => {
            deps.insert((*row, *col));
        }
        Expr::Range {
            start_col,
            start_row,
            end_col,
            end_row,
        } => {
            for r in *start_row..=*end_row {
                for c in *start_col..=*end_col {
                    deps.insert((r, c));
                }
            }
        }
        Expr::BinOp { left, right, .. } => {
            collect_deps_recursive(left, deps);
            collect_deps_recursive(right, deps);
        }
        Expr::UnaryOp { operand, .. } => {
            collect_deps_recursive(operand, deps);
        }
        Expr::Function { args, .. } => {
            for arg in args {
                collect_deps_recursive(arg, deps);
            }
        }
        _ => {}
    }
}

/// Build a topological evaluation order for formula cells.
///
/// Returns cells in the order they should be evaluated, or an error if a
/// circular reference is detected.
pub fn build_eval_order(
    formula_cells: &HashSet<(usize, usize)>,
    parsed: &HashMap<(usize, usize), Expr>,
) -> Result<Vec<(usize, usize)>, FormulaError> {
    // Build adjacency: cell -> set of cells it depends on (that are also formulas)
    let mut in_degree: HashMap<(usize, usize), usize> = HashMap::new();
    let mut dependents: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

    for &cell in formula_cells {
        in_degree.entry(cell).or_insert(0);
        if let Some(expr) = parsed.get(&cell) {
            let deps = collect_dependencies(expr);
            for dep in deps {
                if formula_cells.contains(&dep) && dep != cell {
                    *in_degree.entry(cell).or_insert(0) += 1;
                    dependents.entry(dep).or_default().push(cell);
                }
                if dep == cell {
                    // Self-reference = circular
                    return Err(FormulaError::Circular);
                }
            }
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<(usize, usize)> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&cell, _)| cell)
        .collect();

    let mut order = Vec::new();
    while let Some(cell) = queue.pop_front() {
        order.push(cell);
        if let Some(deps) = dependents.get(&cell) {
            for &dep in deps {
                if let Some(deg) = in_degree.get_mut(&dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }
    }

    if order.len() != formula_cells.len() {
        return Err(FormulaError::Circular);
    }

    Ok(order)
}
