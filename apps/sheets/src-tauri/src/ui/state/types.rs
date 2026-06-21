use std::collections::HashMap;
use std::time::Instant;

use tench_document_core::{OfficeArtifact, OfficeContent};
use tench_ui::prelude::Color;

use super::*;

/// Drag-and-drop state.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DragState {
    pub source_row: usize,
    pub source_col: usize,
    pub target_row: usize,
    pub target_col: usize,
    pub is_copy: bool,
}

/// Resize target for row/column drag-resize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeTarget {
    Row { row: usize },
    Col { col: usize },
}

#[derive(Debug, Clone)]
pub struct SheetsState {
    pub(super) artifact: OfficeArtifact,
    pub(super) content: OfficeContent,
    pub(super) last_saved_csv: String,
    pub(super) status: String,
    pub workbook_name: String,
    pub show_welcome: bool,
    pub active_modal: Option<ModalType>,
    pub toast: Option<(String, Instant)>,
    pub grid: Vec<Vec<CellData>>,
    /// Cached formula evaluation results: (row, col) → display string.
    pub(super) formula_cache: HashMap<(usize, usize), String>,
    pub selected_col: usize,
    pub selected_row: usize,
    pub active_sheet: usize,
    pub sheet_names: Vec<String>,
    pub show_chart_panel: bool,
    pub formula_draft: String,
    pub status_sum: String,
    pub status_count: usize,
    pub status_average: String,
    pub status_min: String,
    pub status_max: String,
    pub find_replace: FindReplaceState,
    pub show_find_replace: bool,
    pub auto_save: AutoSaveState,
    pub clipboard: GridClipboard,
    pub paste_special_mode: PasteSpecialMode,
    pub show_paste_special: bool,
    pub undo_stack: Vec<GridSnapshot>,
    pub redo_stack: Vec<GridSnapshot>,
    pub named_ranges: Vec<NamedRange>,
    pub show_named_ranges: bool,
    // 8.1 Menu system
    pub menu_state: MenuState,
    pub menus: Vec<Vec<MenuItem>>,
    // 8.2 Context menu
    pub context_menu: Option<ContextMenuState>,
    // 8.3 Drag and drop / fill handle
    pub drag_state: Option<DragState>,
    pub fill_handle_dragging: bool,
    // 8.4 Document tabs
    pub doc_tabs: Vec<DocumentTab>,
    pub active_tab_idx: usize,
    // 8.5 Zoom
    pub zoom_percent: u32,
    pub scroll_x: f64,
    pub scroll_y: f64,
    pub scroll_velocity_x: f64,
    pub scroll_velocity_y: f64,
    // 8.6 Freeze panes
    pub freeze_rows: usize,
    pub freeze_cols: usize,
    // 8.7 View toggles
    pub show_formula_bar: bool,
    pub show_grid_lines: bool,
    pub show_headers: bool,
    pub full_screen: bool,
    // 10.1 Page setup
    pub page_setup: PageSetup,
    // 10.2 Print preview
    pub print_preview: PrintPreviewState,
    pub show_page_setup: bool,
    // Phase 3: Dialog states
    pub show_insert_function: bool,
    pub show_sort_dialog: bool,
    pub show_settings: bool,
    pub find_replace_focused_field: FindReplaceFocusedField,
    pub sort_ascending: bool,
    pub sort_has_header: bool,
    pub insert_function_selected: usize,
    pub insert_function_scroll: usize,
    pub named_ranges_selected: Option<usize>,
    // Phase 1: Cell editing
    pub editing_cell: Option<EditingCell>,
    /// Parsed formula references for color highlighting during edit.
    pub formula_refs: Vec<FormulaRef>,
    // Phase 2: Range selection
    /// Anchor cell for range selection (where Shift+click/drag started).
    pub selection_anchor: (usize, usize),
    /// End cell of the current range selection (None = single cell selected).
    pub selection_end: Option<(usize, usize)>,
    /// Whether the user is currently dragging to select a range.
    pub range_selecting: bool,
    /// Whether all cells are selected (Ctrl+A).
    pub select_all_active: bool,
    // Phase 4: File dialog
    pub show_file_dialog: bool,
    pub file_dialog_mode: FileDialogMode,
    pub file_dialog_path: String,
    /// Recent files list.
    pub recent_files: Vec<String>,
    /// Dropped file path (stub for Tauri integration).
    pub dropped_file: Option<String>,
    // Phase 5: Cell formatting
    pub show_toolbar: bool,
    pub format_cells: FormatCellsState,
    pub conditional_format_dialog: ConditionalFormatDialogState,
    pub conditional_formats: Vec<ConditionalFormatRule>,
    pub merged_cells: Vec<MergedCell>,
    pub format_painter_active: bool,
    pub format_painter_source: Option<CellFormat>,
    /// Per-row heights (index → height in px). Empty = use default GRID_ROW_H.
    pub row_heights: Vec<f64>,
    /// Per-column widths (index → width in px). Empty = use default GRID_COL_W.
    pub col_widths: Vec<f64>,
    /// Whether the user is currently dragging a row/column resize handle.
    pub resizing_row_col: Option<ResizeTarget>,
    /// Row height / column width dialog state.
    pub show_row_height_dialog: bool,
    pub show_col_width_dialog: bool,
    pub row_height_input: String,
    pub col_width_input: String,
    // Phase 6: Data operations
    /// Filter state
    pub filter_active: bool,
    pub filter_col: Option<usize>,
    pub filter_values: Vec<String>,
    pub filter_hidden_rows: Vec<usize>,
    pub show_filter_dropdown: bool,
    pub filter_dropdown_col: Option<usize>,
    /// Hide rows/columns
    pub hidden_rows: Vec<usize>,
    pub hidden_cols: Vec<usize>,
    /// Data validation
    pub data_validation_dialog: DataValidationDialogState,
    pub data_validation_rules: Vec<DataValidationRule>,
    /// Pivot table placeholder
    pub show_pivot_table: bool,
    // Phase 7: Charts
    pub show_chart_wizard: bool,
    pub chart_wizard_step: usize,
    pub chart_wizard_data_range: String,
    pub chart_wizard_chart_type: ChartType,
    pub chart_wizard_title: String,
    pub chart_wizard_show_legend: bool,
    pub chart_wizard_show_axis_labels: bool,
    pub charts: Vec<ChartDefinition>,
    pub active_chart_idx: usize,
    pub chart_panel_width: f64,
    pub chart_panel_resizing: bool,
    // Phase 8: Sheet tab management
    pub renaming_sheet: Option<usize>,
    pub rename_draft: String,
    pub show_tab_color_picker: bool,
    pub tab_color_target: Option<usize>,
    pub sheet_tab_colors: HashMap<usize, Color>,
    pub show_move_sheet_dialog: bool,
    pub move_sheet_target: usize,
    // Phase 8: Document sessions
    pub sessions: HashMap<String, DocumentSession>,
    /// Whether the platform window is currently maximized (caption glyph).
    pub window_maximized: bool,
    /// Caption button currently under the pointer, if any (hover feedback).
    pub window_control_hovered: Option<tench_ui::WindowControl>,
}
