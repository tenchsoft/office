//! Sheets UI: native spreadsheet editor widget.

mod actions;
mod automation;
mod charts;
mod dialogs;
mod dialogs2;
mod events;
mod formula_bar;
mod grid;
mod print_preview;
pub(crate) mod state;
mod tabs;
mod toolbar;
mod widget;

use charts::hit_chart_panel;
use charts::paint_chart_panel;
use charts::paint_chart_wizard;
use charts::ChartPanelAction;
use charts::ChartRenderCache;
use dialogs::paint_find_replace_dialog;
use dialogs::paint_insert_function_dialog;
use dialogs::paint_named_ranges_dialog;
use dialogs::paint_paste_special_dialog;
use dialogs::paint_settings_dialog;
use dialogs::paint_sheets_modal;
use dialogs::paint_sort_dialog;
use dialogs::paint_toast;
use dialogs2::paint_cond_format_dialog;
use dialogs2::paint_data_validation_dialog;
use dialogs2::paint_file_dialog;
use dialogs2::paint_format_cells_dialog;
use dialogs2::paint_move_sheet_dialog;
use dialogs2::paint_pivot_table_dialog;
use dialogs2::paint_tab_color_picker;
use formula_bar::{
    dropdown_contains, hit_dropdown, hit_menu_bar, hover_dropdown_item, paint_dropdown_menu,
    paint_formula_bar, paint_menu_bar, DROPDOWN_W, MENU_BAR_H, MENU_ITEM_H, MENU_NAMES, MENU_PAD_X,
    MENU_PAD_Y,
};
use grid::CellRenderCache;
use grid::{
    context_menu_action, hit_context_menu, paint_context_menu, paint_grid, GRID_HEADER_H,
    GRID_ROW_H, ROW_HEADER_W,
};
use grid::{hit_filter_arrow, hit_filter_dropdown, FilterDropdownAction};
use print_preview::{
    hit_preview_close, hit_preview_nav, paint_page_setup_dialog, paint_print_preview,
    PreviewNavAction,
};
use state::ContextMenuTarget;
use state::FindReplaceFocusedField;
use state::MenuAction;
use state::SheetsState;
use state::{ConditionOp, FileDialogMode, FormatCellsTab, HorizontalAlignment, NumberFormat};
use std::time::Instant;
use tabs::{
    hit_doc_tab, hit_doc_tab_close, hit_sheet_nav, hit_zoom_controls, hit_zoom_slider,
    paint_doc_tabs, paint_sheet_tabs, paint_status_bar, zoom_from_slider_x, NavAction, ZoomAction,
};

use tench_ui::prelude::*;
use tench_ui::render::painter::TextCache;
use tench_ui::render::ImageCache;

use crate::workbook_service;

const FORMULA_H: f64 = 28.0;
const MENU_H: f64 = 28.0;
const STATUS_H: f64 = 22.0;
const TAB_H: f64 = 28.0;
const CHART_W_MIN: f64 = 180.0;
const CHART_W_MAX: f64 = 600.0;
const DOC_TAB_H: f64 = 28.0;
const TOOLBAR_H: f64 = 32.0;

/// Root widget for the Sheets spreadsheet editor.
pub struct SheetsApp {
    state: SheetsState,
    cell_cache: CellRenderCache,
    chart_cache: ChartRenderCache,
    text_cache: TextCache,
    #[allow(dead_code)] // ImageCache will be used for cell inline images in a future phase.
    image_cache: ImageCache,
    zoom_slider_dragging: bool,
}

impl Default for SheetsApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SheetsApp {
    pub fn new() -> Self {
        Self {
            state: SheetsState::new(),
            cell_cache: CellRenderCache::new(),
            chart_cache: ChartRenderCache::new(),
            text_cache: TextCache::new(),
            image_cache: ImageCache::default_capacity(),
            zoom_slider_dragging: false,
        }
    }

    /// Access the internal state (read-only).
    pub fn state(&self) -> &SheetsState {
        &self.state
    }

    /// Access the internal state (mutable).
    pub fn state_mut(&mut self) -> &mut SheetsState {
        &mut self.state
    }

    fn grid_bounds(&self, size: Size) -> (f64, f64, f64) {
        let chart_w = if self.state.show_chart_panel {
            self.state.chart_panel_width
        } else {
            0.0
        };
        let formula_h = if self.state.show_formula_bar {
            FORMULA_H
        } else {
            0.0
        };
        let toolbar_h = if self.state.show_toolbar {
            TOOLBAR_H
        } else {
            0.0
        };
        let top = DOC_TAB_H + MENU_H + formula_h + toolbar_h + GRID_HEADER_H;
        let bottom = size.height - STATUS_H - TAB_H;
        let right = size.width - chart_w;
        (top, bottom, right)
    }
}
