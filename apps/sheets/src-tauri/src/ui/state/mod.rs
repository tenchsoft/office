// ---------------------------------------------------------------------------
// State module: re-exports and SheetsState definition
// ---------------------------------------------------------------------------

mod accessors;
mod cell;
mod chart;
mod chart_ops;
mod clipboard;
mod data_ops;
mod document;
mod document_state;
mod editing;
mod file_dialog;
mod file_ops;
mod format;
mod format_ops;
mod formula_editing;
mod grid_data;
mod grid_ops;
mod history;
mod init;
mod interaction;
mod menu;
mod print;
mod print_ops;
mod search;
mod search_ops;
mod selection;
mod sheet_tabs;
mod sync;
#[cfg(test)]
mod tests;
mod types;
mod validation;
mod validation_ops;
mod zoom;

pub use cell::*;
pub use chart::*;
pub use document::*;
pub use file_dialog::*;
pub use format::*;
pub use grid_data::col_letter;
#[allow(unused_imports)]
pub use grid_data::grid_to_csv_with_bom;
pub use menu::*;
pub use print::*;
pub use search::*;
pub use types::*;
pub use validation::*;

#[cfg(test)]
pub use grid_data::mock_grid;
use grid_data::{format_number, grid_to_csv, parse_cell_ref, parse_col_letters};

use std::time::Instant;

use tench_document_core::{OfficeArtifact, OfficeContent};
use tench_ui::prelude::*;

use crate::workbook_service;
