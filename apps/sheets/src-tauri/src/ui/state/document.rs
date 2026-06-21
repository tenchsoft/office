// ---------------------------------------------------------------------------
// Document/session types
// ---------------------------------------------------------------------------

use std::collections::HashMap;

use tench_ui::prelude::*;

use super::cell::{CellData, GridSnapshot, NamedRange};
use super::chart::ChartDefinition;
use super::format::{ConditionalFormatRule, MergedCell};
use super::validation::DataValidationRule;
use super::SheetsState;

/// Document tab for multi-document support.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DocumentTab {
    pub session_id: String,
    pub title: String,
    pub dirty: bool,
}

/// Per-document session that holds all state for a single workbook.
/// Used for multi-workbook tab swap.
#[derive(Debug, Clone)]
pub struct DocumentSession {
    pub grid: Vec<Vec<CellData>>,
    pub selected_row: usize,
    pub selected_col: usize,
    pub formula_draft: String,
    pub undo_stack: Vec<GridSnapshot>,
    pub redo_stack: Vec<GridSnapshot>,
    pub named_ranges: Vec<NamedRange>,
    pub scroll_x: f64,
    pub scroll_y: f64,
    pub freeze_rows: usize,
    pub freeze_cols: usize,
    pub row_heights: Vec<f64>,
    pub col_widths: Vec<f64>,
    pub merged_cells: Vec<MergedCell>,
    pub conditional_formats: Vec<ConditionalFormatRule>,
    pub hidden_rows: Vec<usize>,
    pub hidden_cols: Vec<usize>,
    pub filter_active: bool,
    pub filter_col: Option<usize>,
    pub filter_values: Vec<String>,
    pub filter_hidden_rows: Vec<usize>,
    pub data_validation_rules: Vec<DataValidationRule>,
    pub charts: Vec<ChartDefinition>,
    pub active_chart_idx: usize,
    pub active_sheet: usize,
    pub sheet_names: Vec<String>,
    pub sheet_tab_colors: HashMap<usize, Color>,
}

impl DocumentSession {
    /// Capture the current session state from a SheetsState.
    pub fn capture(state: &SheetsState) -> Self {
        Self {
            grid: state.grid.clone(),
            selected_row: state.selected_row,
            selected_col: state.selected_col,
            formula_draft: state.formula_draft.clone(),
            undo_stack: state.undo_stack.clone(),
            redo_stack: state.redo_stack.clone(),
            named_ranges: state.named_ranges.clone(),
            scroll_x: state.scroll_x,
            scroll_y: state.scroll_y,
            freeze_rows: state.freeze_rows,
            freeze_cols: state.freeze_cols,
            row_heights: state.row_heights.clone(),
            col_widths: state.col_widths.clone(),
            merged_cells: state.merged_cells.clone(),
            conditional_formats: state.conditional_formats.clone(),
            hidden_rows: state.hidden_rows.clone(),
            hidden_cols: state.hidden_cols.clone(),
            filter_active: state.filter_active,
            filter_col: state.filter_col,
            filter_values: state.filter_values.clone(),
            filter_hidden_rows: state.filter_hidden_rows.clone(),
            data_validation_rules: state.data_validation_rules.clone(),
            charts: state.charts.clone(),
            active_chart_idx: state.active_chart_idx,
            active_sheet: state.active_sheet,
            sheet_names: state.sheet_names.clone(),
            sheet_tab_colors: state.sheet_tab_colors.clone(),
        }
    }

    /// Restore this session's state into a SheetsState.
    pub fn restore(&self, state: &mut SheetsState) {
        state.grid = self.grid.clone();
        state.selected_row = self.selected_row;
        state.selected_col = self.selected_col;
        state.formula_draft = self.formula_draft.clone();
        state.undo_stack = self.undo_stack.clone();
        state.redo_stack = self.redo_stack.clone();
        state.named_ranges = self.named_ranges.clone();
        state.scroll_x = self.scroll_x;
        state.scroll_y = self.scroll_y;
        state.freeze_rows = self.freeze_rows;
        state.freeze_cols = self.freeze_cols;
        state.row_heights = self.row_heights.clone();
        state.col_widths = self.col_widths.clone();
        state.merged_cells = self.merged_cells.clone();
        state.conditional_formats = self.conditional_formats.clone();
        state.hidden_rows = self.hidden_rows.clone();
        state.hidden_cols = self.hidden_cols.clone();
        state.filter_active = self.filter_active;
        state.filter_col = self.filter_col;
        state.filter_values = self.filter_values.clone();
        state.filter_hidden_rows = self.filter_hidden_rows.clone();
        state.data_validation_rules = self.data_validation_rules.clone();
        state.charts = self.charts.clone();
        state.active_chart_idx = self.active_chart_idx;
        state.active_sheet = self.active_sheet;
        state.sheet_names = self.sheet_names.clone();
        state.sheet_tab_colors = self.sheet_tab_colors.clone();
    }
}

/// Modal dialog type for the Sheets modal overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalType {
    About,
    Welcome,
    Shortcuts,
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FindReplaceFocusedField {
    #[default]
    Find,
    Replace,
}
