use super::*;

pub(crate) fn sheets_automation_nodes(
    sheets: &SheetsState,
    size: Size,
    base_id: u64,
) -> Vec<UiAutomationNode> {
    let mut nodes = Vec::new();
    let mut next_id = base_id.saturating_mul(1000);
    let formula_h = if sheets.show_formula_bar {
        FORMULA_H
    } else {
        0.0
    };
    let toolbar_h = if sheets.show_toolbar { TOOLBAR_H } else { 0.0 };
    let grid_top = DOC_TAB_H + MENU_H + formula_h + toolbar_h + GRID_HEADER_H;
    let grid_bottom = size.height - STATUS_H - TAB_H;
    let grid_right = if sheets.show_chart_panel {
        size.width - sheets.chart_panel_width
    } else {
        size.width
    };

    push_sheets_node(
        &mut nodes,
        &mut next_id,
        "tab",
        sheets
            .doc_tabs
            .get(sheets.active_tab_idx)
            .map(|tab| tab.title.as_str())
            .unwrap_or("Workbook"),
        format!("sheets.doc_tab.{}", sheets.active_tab_idx),
        Rect::new(
            4.0 + sheets.active_tab_idx as f64 * 160.0,
            2.0,
            160.0 + sheets.active_tab_idx as f64 * 160.0,
            DOC_TAB_H - 2.0,
        ),
    );

    for (index, name) in MENU_NAMES.iter().enumerate() {
        let x = MENU_PAD_X + index as f64 * 54.0;
        push_sheets_node(
            &mut nodes,
            &mut next_id,
            "button",
            *name,
            format!("sheets.menu.{}", sheets_slug(name)),
            Rect::new(x - 4.0, DOC_TAB_H + 2.0, x + 54.0, DOC_TAB_H + MENU_H - 2.0),
        );
    }

    if let Some(menu_idx) = sheets.menu_state.open_menu {
        if let Some(items) = sheets.menus.get(menu_idx) {
            let menu_slug = MENU_NAMES
                .get(menu_idx)
                .map(|name| sheets_slug(name))
                .unwrap_or_else(|| format!("menu_{menu_idx}"));
            let menu_x = MENU_PAD_X + menu_idx as f64 * 54.0;
            let mut item_y = DOC_TAB_H + MENU_BAR_H + MENU_PAD_Y;
            for (item_idx, item) in items.iter().enumerate() {
                if item.is_separator() {
                    item_y += 9.0;
                    continue;
                }

                let item_slug = sheets_menu_item_slug(&item.label);
                push_sheets_node(
                    &mut nodes,
                    &mut next_id,
                    "menu_item",
                    &item.label,
                    format!("sheets.menu.{menu_slug}.item.{item_slug}"),
                    Rect::new(menu_x, item_y, menu_x + DROPDOWN_W, item_y + MENU_ITEM_H),
                );

                if sheets.menu_state.hovered_submenu == Some((menu_idx, item_idx)) {
                    let mut sub_y = item_y + MENU_PAD_Y;
                    for sub_item in &item.submenu {
                        if sub_item.is_separator() {
                            sub_y += 9.0;
                            continue;
                        }
                        push_sheets_node(
                            &mut nodes,
                            &mut next_id,
                            "menu_item",
                            &sub_item.label,
                            format!(
                                "sheets.menu.{menu_slug}.item.{item_slug}.{}",
                                sheets_menu_item_slug(&sub_item.label)
                            ),
                            Rect::new(
                                menu_x + DROPDOWN_W,
                                sub_y,
                                menu_x + DROPDOWN_W + 160.0,
                                sub_y + MENU_ITEM_H,
                            ),
                        );
                        sub_y += MENU_ITEM_H;
                    }
                }

                item_y += MENU_ITEM_H;
            }
        }
    }

    if sheets.show_formula_bar {
        let top = DOC_TAB_H + MENU_H;
        push_sheets_node(
            &mut nodes,
            &mut next_id,
            "text_input",
            sheets.active_cell_ref(),
            "sheets.formula.input",
            Rect::new(68.0, top + 6.0, size.width - 48.0, top + FORMULA_H - 6.0),
        );
    }

    if sheets.show_toolbar {
        let toolbar_y = DOC_TAB_H + MENU_H + formula_h;
        for (label, debug_id, rect) in sheets_toolbar_nodes(toolbar_y) {
            push_sheets_node(&mut nodes, &mut next_id, "button", label, debug_id, rect);
        }
    }

    if grid_bottom > grid_top && grid_right > ROW_HEADER_W {
        let zoom = sheets.zoom_percent as f64 / 100.0;
        let row_h = GRID_ROW_H * zoom;
        let col_w = grid::GRID_COL_W * zoom;
        let cell_x = ROW_HEADER_W + sheets.selected_col as f64 * col_w;
        let cell_y = grid_top + sheets.selected_row as f64 * row_h;
        push_sheets_node(
            &mut nodes,
            &mut next_id,
            "grid_cell",
            sheets.active_cell_ref(),
            format!(
                "sheets.grid.cell.{}.{}",
                sheets.selected_row, sheets.selected_col
            ),
            Rect::new(cell_x, cell_y, cell_x + col_w, cell_y + row_h),
        );
    }

    let tab_y = grid_bottom + 4.0;
    for (idx, (label, debug_id)) in [
        ("First sheet", "sheets.sheet_nav.first"),
        ("Previous sheet", "sheets.sheet_nav.previous"),
        ("Next sheet", "sheets.sheet_nav.next"),
        ("Last sheet", "sheets.sheet_nav.last"),
    ]
    .iter()
    .enumerate()
    {
        let x = 8.0 + idx as f64 * 24.0;
        push_sheets_node(
            &mut nodes,
            &mut next_id,
            "button",
            *label,
            *debug_id,
            Rect::new(x, tab_y, x + 20.0, grid_bottom + TAB_H - 4.0),
        );
    }

    let mut sheet_x = 112.0;
    for (idx, sheet_name) in sheets.sheet_names.iter().enumerate() {
        let w = tabs::sheet_tab_width(sheet_name);
        push_sheets_node(
            &mut nodes,
            &mut next_id,
            "tab",
            sheet_name,
            format!("sheets.sheet_tab.{idx}"),
            Rect::new(sheet_x, tab_y, sheet_x + w, grid_bottom + TAB_H - 4.0),
        );
        sheet_x += w;
    }
    push_sheets_node(
        &mut nodes,
        &mut next_id,
        "button",
        "Add sheet",
        "sheets.sheet.add",
        Rect::new(sheet_x, tab_y, sheet_x + 24.0, grid_bottom + TAB_H - 4.0),
    );

    let status_top = size.height - STATUS_H;
    let zoom_x = size.width - 160.0;
    push_sheets_node(
        &mut nodes,
        &mut next_id,
        "button",
        "Zoom out",
        "sheets.status.zoom_out",
        Rect::new(zoom_x, status_top + 4.0, zoom_x + 20.0, size.height - 4.0),
    );
    push_sheets_node(
        &mut nodes,
        &mut next_id,
        "button",
        "Reset zoom",
        "sheets.status.zoom_reset",
        Rect::new(
            zoom_x + 24.0,
            status_top + 4.0,
            zoom_x + 76.0,
            size.height - 4.0,
        ),
    );
    push_sheets_node(
        &mut nodes,
        &mut next_id,
        "button",
        "Zoom in",
        "sheets.status.zoom_in",
        Rect::new(
            zoom_x + 80.0,
            status_top + 4.0,
            zoom_x + 100.0,
            size.height - 4.0,
        ),
    );
    push_sheets_node(
        &mut nodes,
        &mut next_id,
        "slider",
        "Zoom",
        "sheets.status.zoom_slider",
        Rect::new(
            zoom_x + 106.0,
            status_top + 4.0,
            zoom_x + 150.0,
            size.height - 4.0,
        ),
    );

    if sheets.show_chart_panel {
        let chart_x = size.width - sheets.chart_panel_width;
        push_sheets_node(
            &mut nodes,
            &mut next_id,
            "panel",
            "Charts",
            "sheets.chart.panel",
            Rect::new(chart_x, grid_top, size.width, grid_bottom),
        );
    }

    push_sheets_overlay_nodes(sheets, size, &mut nodes, &mut next_id);

    nodes
}

fn push_sheets_overlay_nodes(
    sheets: &SheetsState,
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let modal_rect = Rect::new(
        (size.width - 420.0).max(0.0) / 2.0,
        (size.height - 300.0).max(0.0) / 2.0,
        (size.width + 420.0).min(size.width * 2.0) / 2.0,
        (size.height + 300.0).min(size.height * 2.0) / 2.0,
    );
    if sheets.show_welcome {
        push_sheets_dialog(
            nodes,
            next_id,
            "Welcome",
            "sheets.dialog.welcome",
            modal_rect,
        );
    }
    if sheets.show_find_replace {
        push_sheets_dialog(
            nodes,
            next_id,
            "Find and replace",
            "sheets.dialog.find_replace",
            modal_rect,
        );
        push_sheets_find_replace_nodes(size, nodes, next_id);
    }
    if sheets.show_paste_special {
        push_sheets_dialog(
            nodes,
            next_id,
            "Paste special",
            "sheets.dialog.paste_special",
            modal_rect,
        );
        push_sheets_paste_special_nodes(size, nodes, next_id);
    }
    if sheets.show_named_ranges {
        push_sheets_dialog(
            nodes,
            next_id,
            "Named ranges",
            "sheets.dialog.named_ranges",
            modal_rect,
        );
    }
    if sheets.show_insert_function {
        push_sheets_dialog(
            nodes,
            next_id,
            "Insert function",
            "sheets.dialog.insert_function",
            modal_rect,
        );
        push_sheets_insert_function_nodes(size, nodes, next_id);
    }
    if sheets.show_sort_dialog {
        push_sheets_dialog(nodes, next_id, "Sort", "sheets.dialog.sort", modal_rect);
        push_sheets_sort_nodes(size, nodes, next_id);
    }
    if sheets.show_settings {
        push_sheets_dialog(
            nodes,
            next_id,
            "Settings",
            "sheets.dialog.settings",
            modal_rect,
        );
    }
    if sheets.show_page_setup {
        push_sheets_dialog(
            nodes,
            next_id,
            "Page setup",
            "sheets.dialog.page_setup",
            modal_rect,
        );
        push_sheets_page_setup_nodes(size, nodes, next_id);
    }
    if sheets.show_file_dialog {
        push_sheets_dialog(
            nodes,
            next_id,
            "File dialog",
            "sheets.dialog.file",
            modal_rect,
        );
    }
    if sheets.format_cells.visible {
        push_sheets_dialog(
            nodes,
            next_id,
            "Format cells",
            "sheets.dialog.format_cells",
            modal_rect,
        );
        push_sheets_format_cells_nodes(size, nodes, next_id);
    }
    if sheets.conditional_format_dialog.visible {
        push_sheets_dialog(
            nodes,
            next_id,
            "Conditional format",
            "sheets.dialog.conditional_format",
            modal_rect,
        );
        push_sheets_conditional_format_nodes(size, nodes, next_id);
    }
    if sheets.data_validation_dialog.visible {
        push_sheets_dialog(
            nodes,
            next_id,
            "Data validation",
            "sheets.dialog.data_validation",
            modal_rect,
        );
        push_sheets_data_validation_nodes(size, nodes, next_id);
    }
    if sheets.show_pivot_table {
        push_sheets_dialog(
            nodes,
            next_id,
            "Pivot table",
            "sheets.dialog.pivot_table",
            modal_rect,
        );
    }
    if sheets.show_chart_wizard {
        push_sheets_dialog(
            nodes,
            next_id,
            "Chart wizard",
            "sheets.dialog.chart_wizard",
            modal_rect,
        );
        push_sheets_chart_wizard_nodes(sheets, size, nodes, next_id);
    }
    if sheets.print_preview.visible {
        push_sheets_node(
            nodes,
            next_id,
            "dialog",
            "Print preview",
            "sheets.print_preview",
            Rect::new(0.0, 0.0, size.width, size.height),
        );
    }
}

fn push_sheets_dialog(
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
    label: &str,
    debug_id: &str,
    rect: Rect,
) {
    push_sheets_node(nodes, next_id, "dialog", label, debug_id, rect);
}

fn sheets_centered_modal(size: Size, width: f64, height: f64) -> Rect {
    Rect::new(
        size.width / 2.0 - width / 2.0,
        size.height / 2.0 - height / 2.0,
        size.width / 2.0 + width / 2.0,
        size.height / 2.0 + height / 2.0,
    )
}

fn push_sheets_find_replace_nodes(
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let modal = Rect::new(
        size.width / 2.0 - 190.0,
        MENU_H + FORMULA_H + 10.0,
        size.width / 2.0 + 190.0,
        MENU_H + FORMULA_H + 270.0,
    );
    let x0 = modal.x0 + 16.0;
    let mut y = modal.y0 + 52.0;
    push_sheets_node(
        nodes,
        next_id,
        "text_input",
        "Find",
        "sheets.find.query",
        Rect::new(x0, y, modal.x1 - 16.0, y + 22.0),
    );
    y += 28.0;
    push_sheets_node(
        nodes,
        next_id,
        "text_input",
        "Replace",
        "sheets.find.replacement",
        Rect::new(x0, y, modal.x1 - 16.0, y + 22.0),
    );
    y += 30.0;
    for (label, debug_id, rect) in [
        (
            "Case sensitive",
            "sheets.find.case_sensitive",
            Rect::new(x0, y - 12.0, x0 + 70.0, y + 12.0),
        ),
        (
            "Regex",
            "sheets.find.regex",
            Rect::new(x0 + 80.0, y - 12.0, x0 + 155.0, y + 12.0),
        ),
        (
            "Formulas",
            "sheets.find.formulas",
            Rect::new(x0 + 165.0, y - 12.0, x0 + 260.0, y + 12.0),
        ),
    ] {
        push_sheets_node(nodes, next_id, "checkbox", label, debug_id, rect);
    }
    y += 22.0;
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Scope",
        "sheets.find.scope",
        Rect::new(x0, y - 12.0, x0 + 200.0, y + 12.0),
    );
    y += 24.0;
    let mut x = x0;
    for (label, debug_id) in [
        ("Find", "sheets.find.find"),
        ("Next", "sheets.find.next"),
        ("Prev", "sheets.find.previous"),
        ("Replace", "sheets.find.replace"),
        ("Replace All", "sheets.find.replace_all"),
        ("Close", "sheets.find.close"),
    ] {
        push_sheets_node(
            nodes,
            next_id,
            "button",
            label,
            debug_id,
            Rect::new(x, y, x + 52.0, y + 22.0),
        );
        x += 58.0;
    }
}

fn push_sheets_paste_special_nodes(
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let modal = sheets_centered_modal(size, 280.0, 180.0);
    let x0 = modal.x0 + 16.0;
    let mut y = modal.y0 + 54.0;
    for (label, debug_id) in [
        ("All", "sheets.paste_special.mode.all"),
        ("Values", "sheets.paste_special.mode.values"),
        ("Formats", "sheets.paste_special.mode.formats"),
        ("Formulas", "sheets.paste_special.mode.formulas"),
    ] {
        push_sheets_node(
            nodes,
            next_id,
            "radio",
            label,
            debug_id,
            Rect::new(x0, y - 12.0, modal.x1 - 16.0, y + 12.0),
        );
        y += 22.0;
    }
    y += 8.0;
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "OK",
        "sheets.paste_special.ok",
        Rect::new(x0, y, x0 + 60.0, y + 24.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Cancel",
        "sheets.paste_special.cancel",
        Rect::new(x0 + 72.0, y, x0 + 132.0, y + 24.0),
    );
}

fn push_sheets_insert_function_nodes(
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let modal = sheets_centered_modal(size, 400.0, 380.0);
    let x0 = modal.x0 + 16.0;
    let list_y = modal.y0 + 52.0;
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Function row",
        "sheets.insert_function.row.0",
        Rect::new(x0, list_y, modal.x1 - 16.0, list_y + 20.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Insert",
        "sheets.insert_function.insert",
        Rect::new(x0, modal.y1 - 40.0, x0 + 70.0, modal.y1 - 16.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Cancel",
        "sheets.insert_function.cancel",
        Rect::new(x0 + 82.0, modal.y1 - 40.0, x0 + 152.0, modal.y1 - 16.0),
    );
}

fn push_sheets_sort_nodes(size: Size, nodes: &mut Vec<UiAutomationNode>, next_id: &mut u64) {
    let modal = sheets_centered_modal(size, 300.0, 220.0);
    let x0 = modal.x0 + 16.0;
    let mut y = modal.y0 + 52.0;
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Column",
        "sheets.sort.column",
        Rect::new(x0, y, modal.x1 - 16.0, y + 22.0),
    );
    y += 30.0;
    push_sheets_node(
        nodes,
        next_id,
        "radio",
        "Ascending",
        "sheets.sort.ascending",
        Rect::new(x0, y - 12.0, x0 + 120.0, y + 12.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "radio",
        "Descending",
        "sheets.sort.descending",
        Rect::new(x0 + 130.0, y - 12.0, x0 + 260.0, y + 12.0),
    );
    y += 26.0;
    push_sheets_node(
        nodes,
        next_id,
        "checkbox",
        "Header row",
        "sheets.sort.header_row",
        Rect::new(x0, y - 12.0, x0 + 160.0, y + 12.0),
    );
    y += 30.0;
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "OK",
        "sheets.sort.ok",
        Rect::new(x0, y, x0 + 60.0, y + 24.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Cancel",
        "sheets.sort.cancel",
        Rect::new(x0 + 72.0, y, x0 + 132.0, y + 24.0),
    );
}

fn push_sheets_page_setup_nodes(size: Size, nodes: &mut Vec<UiAutomationNode>, next_id: &mut u64) {
    let modal = sheets_centered_modal(size, 400.0, 480.0);
    let x0 = modal.x0 + 20.0;
    let mut y = modal.y0 + 60.0;
    for (label, debug_id, dy) in [
        ("Paper size", "sheets.page_setup.paper_size", 24.0),
        ("Orientation", "sheets.page_setup.orientation", 24.0),
    ] {
        push_sheets_node(
            nodes,
            next_id,
            "button",
            label,
            debug_id,
            Rect::new(x0, y - 12.0, modal.x1 - 20.0, y + 12.0),
        );
        y += dy;
    }
    y += 20.0 + 18.0 * 6.0 + 8.0;
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Scaling",
        "sheets.page_setup.scaling",
        Rect::new(x0, y - 12.0, modal.x1 - 20.0, y + 12.0),
    );
    y += 44.0;
    for debug_id in [
        "sheets.page_setup.gridlines",
        "sheets.page_setup.headers",
        "sheets.page_setup.center_horizontally",
        "sheets.page_setup.center_vertically",
        "sheets.page_setup.repeat_header",
    ] {
        push_sheets_node(
            nodes,
            next_id,
            "checkbox",
            debug_id,
            debug_id,
            Rect::new(x0, y - 12.0, modal.x1 - 20.0, y + 12.0),
        );
        y += 20.0;
    }
    y += 40.0;
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "OK",
        "sheets.page_setup.ok",
        Rect::new(x0, y, x0 + 64.0, y + 24.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Cancel",
        "sheets.page_setup.cancel",
        Rect::new(x0 + 72.0, y, x0 + 136.0, y + 24.0),
    );
}

fn push_sheets_format_cells_nodes(
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let modal = sheets_centered_modal(size, 400.0, 380.0);
    let x0 = modal.x0 + 16.0;
    let y0 = modal.y0 + 40.0;
    let tab_w = 70.0;
    for (idx, debug_id) in [
        "sheets.format_cells.tab.number",
        "sheets.format_cells.tab.alignment",
        "sheets.format_cells.tab.font",
        "sheets.format_cells.tab.border",
        "sheets.format_cells.tab.fill",
    ]
    .iter()
    .enumerate()
    {
        let x = x0 + idx as f64 * (tab_w + 4.0);
        push_sheets_node(
            nodes,
            next_id,
            "tab",
            *debug_id,
            *debug_id,
            Rect::new(x, y0, x + tab_w, y0 + 24.0),
        );
    }
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Number format row",
        "sheets.format_cells.number_format.0",
        Rect::new(x0, y0 + 36.0, x0 + 160.0, y0 + 58.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "OK",
        "sheets.format_cells.ok",
        Rect::new(x0, modal.y1 - 44.0, x0 + 60.0, modal.y1 - 20.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Cancel",
        "sheets.format_cells.cancel",
        Rect::new(x0 + 72.0, modal.y1 - 44.0, x0 + 132.0, modal.y1 - 20.0),
    );
}

fn push_sheets_conditional_format_nodes(
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let modal = sheets_centered_modal(size, 380.0, 300.0);
    let x0 = modal.x0 + 16.0;
    let y0 = modal.y0 + 40.0;
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Operator",
        "sheets.conditional_format.operator",
        Rect::new(x0, y0, x0 + 140.0, y0 + 22.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "text_input",
        "Value",
        "sheets.conditional_format.value",
        Rect::new(x0 + 160.0, y0, modal.x1 - 16.0, y0 + 22.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Color preset",
        "sheets.conditional_format.color.0",
        Rect::new(x0, y0 + 120.0, x0 + 100.0, y0 + 142.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "OK",
        "sheets.conditional_format.ok",
        Rect::new(x0, modal.y1 - 44.0, x0 + 60.0, modal.y1 - 20.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Cancel",
        "sheets.conditional_format.cancel",
        Rect::new(x0 + 72.0, modal.y1 - 44.0, x0 + 132.0, modal.y1 - 20.0),
    );
}

fn push_sheets_data_validation_nodes(
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let modal = sheets_centered_modal(size, 400.0, 340.0);
    let x0 = modal.x0 + 16.0;
    let mut y = modal.y0 + 40.0;
    for (role, label, debug_id) in [
        ("button", "Type", "sheets.data_validation.type"),
        ("button", "Operator", "sheets.data_validation.operator"),
        ("text_input", "Value", "sheets.data_validation.value"),
        (
            "text_input",
            "Second value",
            "sheets.data_validation.second_value",
        ),
        (
            "text_input",
            "Error message",
            "sheets.data_validation.error_message",
        ),
    ] {
        push_sheets_node(
            nodes,
            next_id,
            role,
            label,
            debug_id,
            Rect::new(x0, y, modal.x1 - 16.0, y + 22.0),
        );
        y += 30.0;
    }
    y += 10.0;
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "OK",
        "sheets.data_validation.ok",
        Rect::new(x0, y, x0 + 60.0, y + 24.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Cancel",
        "sheets.data_validation.cancel",
        Rect::new(x0 + 72.0, y, x0 + 132.0, y + 24.0),
    );
}

fn push_sheets_chart_wizard_nodes(
    sheets: &SheetsState,
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let modal = sheets_centered_modal(size, 420.0, 380.0);
    let x0 = modal.x0 + 16.0;
    let button_y = modal.y1 - 44.0;
    if sheets.chart_wizard_step > 0 {
        push_sheets_node(
            nodes,
            next_id,
            "button",
            "Back",
            "sheets.chart_wizard.back",
            Rect::new(x0, button_y, x0 + 60.0, button_y + 24.0),
        );
    }
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Next",
        "sheets.chart_wizard.next",
        Rect::new(x0 + 72.0, button_y, x0 + 136.0, button_y + 24.0),
    );
    push_sheets_node(
        nodes,
        next_id,
        "button",
        "Cancel",
        "sheets.chart_wizard.cancel",
        Rect::new(x0 + 148.0, button_y, x0 + 212.0, button_y + 24.0),
    );
    match sheets.chart_wizard_step {
        0 => push_sheets_node(
            nodes,
            next_id,
            "text_input",
            "Data range",
            "sheets.chart_wizard.data_range",
            Rect::new(x0, modal.y0 + 76.0, modal.x1 - 16.0, modal.y0 + 100.0),
        ),
        1 => {
            let mut y = modal.y0 + 80.0;
            for chart_type in state::ChartType::ALL {
                push_sheets_node(
                    nodes,
                    next_id,
                    "button",
                    chart_type.label(),
                    format!(
                        "sheets.chart_wizard.chart_type.{}",
                        sheets_slug(chart_type.label())
                    ),
                    Rect::new(x0, y - 2.0, x0 + 100.0, y + 20.0),
                );
                y += 28.0;
            }
        }
        _ => {
            push_sheets_node(
                nodes,
                next_id,
                "text_input",
                "Title",
                "sheets.chart_wizard.title",
                Rect::new(x0, modal.y0 + 76.0, modal.x1 - 16.0, modal.y0 + 100.0),
            );
            push_sheets_node(
                nodes,
                next_id,
                "checkbox",
                "Legend",
                "sheets.chart_wizard.legend",
                Rect::new(x0, modal.y0 + 136.0, x0 + 160.0, modal.y0 + 160.0),
            );
            push_sheets_node(
                nodes,
                next_id,
                "checkbox",
                "Axis labels",
                "sheets.chart_wizard.axis_labels",
                Rect::new(x0, modal.y0 + 160.0, x0 + 200.0, modal.y0 + 184.0),
            );
        }
    }
}

fn sheets_toolbar_nodes(toolbar_y: f64) -> Vec<(&'static str, &'static str, Rect)> {
    let btn_w = 28.0;
    let gap = 2.0;
    let btn_y = toolbar_y + 2.0;
    let btn_h = TOOLBAR_H - 4.0;
    let mut x = 8.0;
    let mut nodes = Vec::new();

    for (label, debug_id, width) in [
        ("Bold", "sheets.toolbar.bold", btn_w),
        ("Italic", "sheets.toolbar.italic", btn_w),
        ("Underline", "sheets.toolbar.underline", btn_w),
    ] {
        nodes.push((
            label,
            debug_id,
            Rect::new(x, btn_y, x + width, btn_y + btn_h),
        ));
        x += width + gap;
    }

    x += 8.0;
    for (label, debug_id, width) in [
        ("Align left", "sheets.toolbar.align_left", btn_w),
        ("Align center", "sheets.toolbar.align_center", btn_w),
        ("Align right", "sheets.toolbar.align_right", btn_w),
    ] {
        nodes.push((
            label,
            debug_id,
            Rect::new(x, btn_y, x + width, btn_y + btn_h),
        ));
        x += width + gap;
    }

    x += 8.0;
    nodes.push((
        "Number format",
        "sheets.toolbar.number_format",
        Rect::new(x, btn_y, x + 70.0, btn_y + btn_h),
    ));
    x += 70.0 + gap;
    x += 8.0;
    nodes.push((
        "Format painter",
        "sheets.toolbar.format_painter",
        Rect::new(x, btn_y, x + btn_w + 20.0, btn_y + btn_h),
    ));
    x += btn_w + 20.0 + gap;
    nodes.push((
        "Merge cells",
        "sheets.toolbar.merge_cells",
        Rect::new(x, btn_y, x + btn_w + 20.0, btn_y + btn_h),
    ));

    nodes
}

fn push_sheets_node(
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
    role: &str,
    label: impl Into<String>,
    debug_id: impl Into<String>,
    rect: Rect,
) {
    *next_id = next_id.saturating_add(1);
    nodes.push(UiAutomationNode {
        id: *next_id,
        debug_id: Some(debug_id.into()),
        role: role.to_string(),
        label: Some(label.into()),
        value: None,
        bounds: UiAutomationRect {
            x: rect.x0,
            y: rect.y0,
            width: rect.width(),
            height: rect.height(),
        },
        enabled: true,
        focused: false,
        hovered: false,
        children: Vec::new(),
    });
}

fn sheets_slug(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn sheets_menu_item_slug(label: &str) -> String {
    let slug = sheets_slug(label);
    match slug.as_str() {
        "" => "separator".to_string(),
        _ => slug,
    }
}
