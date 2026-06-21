use super::*;
use std::collections::HashMap;
use std::time::Instant;

use tench_office_io::sheets::format as format_io;

use crate::workbook_service;

impl SheetsState {
    pub fn new() -> Self {
        let grid = vec![vec![CellData::val(""); 26]; 100];
        Self::new_with_grid(grid)
    }

    /// Create a new state with a custom grid (used by tests).
    pub fn new_with_grid(grid: Vec<Vec<CellData>>) -> Self {
        let opened = workbook_service::create_workbook(Some("Quarterly Plan".into()));
        let workbook_name = opened.artifact.title.clone();
        let csv = grid_to_csv(&grid);
        let content = format_io::csv_to_workbook_content(&csv, &workbook_name);
        let selected_row = 1;
        let selected_col = 1;
        let formula_draft = grid[selected_row][selected_col].value.clone();
        let session_id = format!(
            "session-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );
        Self {
            artifact: opened.artifact,
            content,
            last_saved_csv: csv,
            status: "Workbook loaded locally".into(),
            grid: grid.clone(),
            formula_cache: HashMap::new(),
            workbook_name: workbook_name.clone(),
            show_welcome: false,
            active_modal: None,
            toast: Some(("Workbook loaded locally".into(), Instant::now())),
            selected_col,
            selected_row,
            active_sheet: 0,
            sheet_names: vec!["Sheet1".into(), "Sheet2".into(), "Sheet3".into()],
            show_chart_panel: true,
            formula_draft,
            status_sum: String::new(),
            status_count: 0,
            status_average: String::new(),
            status_min: String::new(),
            status_max: String::new(),
            find_replace: FindReplaceState::default(),
            show_find_replace: false,
            auto_save: AutoSaveState::new(30),
            clipboard: GridClipboard::default(),
            paste_special_mode: PasteSpecialMode::All,
            show_paste_special: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            named_ranges: Vec::new(),
            show_named_ranges: false,
            // 8.1 Menu system
            menu_state: MenuState::default(),
            menus: build_menus(),
            // 8.2 Context menu
            context_menu: None,
            // 8.3 Drag and drop
            drag_state: None,
            fill_handle_dragging: false,
            // 8.4 Document tabs
            doc_tabs: vec![DocumentTab {
                session_id,
                title: workbook_name,
                dirty: false,
            }],
            active_tab_idx: 0,
            // 8.5 Zoom
            zoom_percent: 100,
            scroll_x: 0.0,
            scroll_y: 0.0,
            scroll_velocity_x: 0.0,
            scroll_velocity_y: 0.0,
            // 8.6 Freeze panes
            freeze_rows: 0,
            freeze_cols: 0,
            // 8.7 View toggles
            show_formula_bar: true,
            show_grid_lines: true,
            show_headers: true,
            full_screen: false,
            // 10.1 Page setup
            page_setup: PageSetup::default(),
            // 10.2 Print preview
            print_preview: PrintPreviewState::default(),
            show_page_setup: false,
            // Phase 3: Dialog states
            show_insert_function: false,
            show_sort_dialog: false,
            show_settings: false,
            find_replace_focused_field: FindReplaceFocusedField::default(),
            sort_ascending: true,
            sort_has_header: true,
            insert_function_selected: 0,
            insert_function_scroll: 0,
            named_ranges_selected: None,
            // Phase 1: Cell editing
            editing_cell: None,
            formula_refs: Vec::new(),
            // Phase 2: Range selection
            selection_anchor: (selected_row, selected_col),
            selection_end: None,
            range_selecting: false,
            select_all_active: false,
            // Phase 4: File dialog
            show_file_dialog: false,
            file_dialog_mode: FileDialogMode::Open,
            file_dialog_path: String::new(),
            recent_files: Vec::new(),
            dropped_file: None,
            // Phase 5: Cell formatting
            show_toolbar: true,
            format_cells: FormatCellsState::default(),
            conditional_format_dialog: ConditionalFormatDialogState::default(),
            conditional_formats: Vec::new(),
            merged_cells: Vec::new(),
            format_painter_active: false,
            format_painter_source: None,
            row_heights: Vec::new(),
            col_widths: Vec::new(),
            resizing_row_col: None,
            show_row_height_dialog: false,
            show_col_width_dialog: false,
            row_height_input: String::new(),
            col_width_input: String::new(),
            // Phase 6: Data operations
            filter_active: false,
            filter_col: None,
            filter_values: Vec::new(),
            filter_hidden_rows: Vec::new(),
            show_filter_dropdown: false,
            filter_dropdown_col: None,
            hidden_rows: Vec::new(),
            hidden_cols: Vec::new(),
            data_validation_dialog: DataValidationDialogState::default(),
            data_validation_rules: Vec::new(),
            show_pivot_table: false,
            // Phase 7: Charts
            show_chart_wizard: false,
            chart_wizard_step: 0,
            chart_wizard_data_range: String::new(),
            chart_wizard_chart_type: ChartType::Bar,
            chart_wizard_title: String::new(),
            chart_wizard_show_legend: true,
            chart_wizard_show_axis_labels: true,
            charts: Vec::new(),
            active_chart_idx: 0,
            chart_panel_width: 260.0,
            chart_panel_resizing: false,
            // Phase 8: Sheet tab management
            renaming_sheet: None,
            rename_draft: String::new(),
            show_tab_color_picker: false,
            tab_color_target: None,
            sheet_tab_colors: HashMap::new(),
            show_move_sheet_dialog: false,
            move_sheet_target: 0,
            // Phase 8: Document sessions
            sessions: HashMap::new(),
            window_maximized: false,
            window_control_hovered: None,
        }
        .with_recalculated_status()
    }
}
