//! Spreadsheet formula engine.
//!
//! Parses formulas like `=SUM(A1:B3)` into an AST, evaluates them against a
//! grid of cell data, and resolves cross-cell dependencies in topological
//! order.

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

mod deps;
mod eval;
mod grid;
mod model;
mod parse;

pub use deps::{build_eval_order, collect_dependencies};
pub use eval::{evaluate, CellProvider};
pub use grid::{evaluate_all_formulas, GridProvider};
pub use model::{BinOp, Expr, FormulaError, UnaryOp, Value};
pub use parse::{col_letters_to_index, parse_cell_ref, parse_formula};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
