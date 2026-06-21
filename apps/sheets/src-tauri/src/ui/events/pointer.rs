use super::super::*;
use tench_ui::core::events::{PointerButtonEvent, PointerMoveEvent, PointerScrollEvent};

impl SheetsApp {
    pub(crate) fn handle_pointer_down(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        let (grid_top, grid_bottom, grid_right) = self.grid_bounds(size);
        let formula_h = if self.state.show_formula_bar {
            FORMULA_H
        } else {
            0.0
        };

        // 10.2 Print preview overlay — intercept all clicks when visible
        if self.state.print_preview.visible {
            // Close button
            if hit_preview_close(e.pos.x, e.pos.y, size) {
                self.state.print_preview.visible = false;
                ctx.request_paint();
                return;
            }
            // Navigation bar
            if let Some(nav) =
                hit_preview_nav(e.pos.x, e.pos.y, size.width, &self.state.print_preview)
            {
                match nav {
                    PreviewNavAction::PrevPage => {
                        self.state.print_preview.prev_page();
                    }
                    PreviewNavAction::NextPage => {
                        self.state.print_preview.next_page();
                    }
                    PreviewNavAction::ZoomIn => {
                        self.state.print_preview.zoom_in();
                    }
                    PreviewNavAction::ZoomOut => {
                        self.state.print_preview.zoom_out();
                    }
                    PreviewNavAction::Print => {
                        self.state.toast = Some(("Sent to system printer".into(), Instant::now()));
                    }
                }
                ctx.request_paint();
                return;
            }
            // Click anywhere else in the preview overlay — ignore
            return;
        }

        // 10.4 Page setup dialog — intercept clicks when visible
        if self.state.show_page_setup {
            self.handle_page_setup_click(ctx, e);
            return;
        }

        // Find/Replace dialog
        if self.state.show_find_replace {
            self.handle_find_replace_click(ctx, e);
            return;
        }

        // Paste Special dialog
        if self.state.show_paste_special {
            self.handle_paste_special_click(ctx, e);
            return;
        }

        // Named Ranges dialog
        if self.state.show_named_ranges {
            self.handle_named_ranges_click(ctx, e);
            return;
        }

        // Insert Function dialog
        if self.state.show_insert_function {
            self.handle_insert_function_click(ctx, e);
            return;
        }

        // Sort dialog
        if self.state.show_sort_dialog {
            self.handle_sort_dialog_click(ctx, e);
            return;
        }

        // Settings dialog
        if self.state.show_settings {
            self.handle_settings_click(ctx, e);
            return;
        }

        // File dialog (Phase 4)
        if self.state.show_file_dialog {
            self.handle_file_dialog_click(ctx, e);
            return;
        }

        // Format cells dialog (Phase 5)
        if self.state.format_cells.visible {
            self.handle_format_cells_click(ctx, e);
            return;
        }

        // Conditional format dialog (Phase 5)
        if self.state.conditional_format_dialog.visible {
            self.handle_cond_format_click(ctx, e);
            return;
        }

        // Phase 6: Data Validation dialog
        if self.state.data_validation_dialog.visible {
            self.handle_data_validation_click(ctx, e);
            return;
        }

        // Phase 6: Pivot Table dialog
        if self.state.show_pivot_table {
            self.handle_pivot_table_click(ctx, e);
            return;
        }

        // Phase 7: Chart wizard dialog
        if self.state.show_chart_wizard {
            self.handle_chart_wizard_click(ctx, e);
            return;
        }

        // Phase 8: Tab color picker dialog
        if self.state.show_tab_color_picker {
            self.handle_tab_color_picker_click(ctx, e);
            return;
        }

        // Phase 8: Move sheet dialog
        if self.state.show_move_sheet_dialog {
            self.handle_move_sheet_dialog_click(ctx, e);
            return;
        }

        // Phase 6: Filter dropdown
        if self.state.show_filter_dropdown {
            if let Some(action) = hit_filter_dropdown(&self.state, e.pos.x, e.pos.y) {
                self.handle_filter_dropdown_action(ctx, action);
                return;
            }
            // Click outside filter dropdown closes it
            self.state.show_filter_dropdown = false;
            ctx.request_paint();
            return;
        }

        // Phase 6: Filter arrow click
        if let Some(col) = hit_filter_arrow(&self.state, e.pos.x, e.pos.y) {
            self.state.filter_dropdown_col = Some(col);
            self.state.show_filter_dropdown = true;
            // Initialize filter values if not already set for this column
            if self.state.filter_col != Some(col) {
                let all_vals = self.state.unique_values_for_col(col);
                self.state.filter_values = all_vals;
            }
            ctx.request_paint();
            return;
        }

        // Close context menu on any click outside it
        if self.state.context_menu.is_some() {
            if let Some(idx) = hit_context_menu(&self.state, e.pos.x, e.pos.y) {
                let target = self.state.context_menu.as_ref().map(|cm| cm.target.clone());
                let action = target.as_ref().map(|t| context_menu_action(t, idx));
                if let (Some(t), Some(a)) = (target, action) {
                    self.execute_context_menu_action(a, &t);
                }
            }
            self.state.context_menu = None;
            ctx.request_paint();
            return;
        }

        // Document tabs (topmost area)
        if e.pos.y < DOC_TAB_H {
            if let Some(tab_idx) = hit_doc_tab(e.pos.x, e.pos.y, self.state.doc_tabs.len()) {
                if hit_doc_tab_close(e.pos.x, e.pos.y, tab_idx) {
                    self.close_tab(tab_idx);
                } else if tab_idx != self.state.active_tab_idx {
                    self.state.switch_to_tab(tab_idx);
                }
                ctx.request_paint();
            }
            return;
        }

        // Menu bar
        if e.pos.y >= DOC_TAB_H && e.pos.y < DOC_TAB_H + MENU_H {
            // Check if a menu name was clicked
            if let Some(menu_idx) = hit_menu_bar(e.pos.x, e.pos.y) {
                if self.state.menu_state.open_menu == Some(menu_idx) {
                    self.state.menu_state.open_menu = None;
                    self.state.menu_state.hovered_submenu = None;
                } else {
                    self.state.menu_state.open_menu = Some(menu_idx);
                    self.state.menu_state.hovered_submenu = None;
                }
                ctx.request_paint();
                return;
            }

            // Click outside menu items closes dropdown
            if self.state.menu_state.open_menu.is_some() {
                self.state.menu_state.open_menu = None;
                ctx.request_paint();
            }
            return;
        }

        if self.state.menu_state.open_menu.is_some() {
            if let Some(action) = hit_dropdown(&self.state, e.pos.x, e.pos.y, DOC_TAB_H) {
                self.state.menu_state.open_menu = None;
                self.state.menu_state.hovered_submenu = None;
                self.execute_menu_action(action);
                ctx.request_paint();
                return;
            }
            if dropdown_contains(&self.state, e.pos.x, e.pos.y, DOC_TAB_H) {
                return;
            }
            self.state.menu_state.open_menu = None;
            self.state.menu_state.hovered_submenu = None;
            ctx.request_paint();
            return;
        }

        // Toolbar (Phase 5)
        if self.state.show_toolbar {
            let toolbar_y = DOC_TAB_H + MENU_H + formula_h;
            if e.pos.y >= toolbar_y && e.pos.y < toolbar_y + TOOLBAR_H {
                self.handle_toolbar_click(ctx, e, toolbar_y);
                return;
            }
        }

        // Grid area
        if e.pos.y >= grid_top && e.pos.y < grid_bottom && e.pos.x < grid_right {
            let zoom = self.state.zoom_percent as f64 / 100.0;
            let row_h = GRID_ROW_H * zoom;
            let col_w = grid::GRID_COL_W * zoom;

            // Check for right-click (context menu)
            if e.button == tench_ui::core::events::PointerButton::Secondary {
                let col = if col_w > 0.0 {
                    ((e.pos.x - ROW_HEADER_W) / col_w) as usize
                } else {
                    0
                };
                let row = if row_h > 0.0 {
                    ((e.pos.y - grid_top) / row_h) as usize
                } else {
                    0
                };

                // Determine context menu target
                let target = if e.pos.x < ROW_HEADER_W {
                    ContextMenuTarget::RowHeader { row }
                } else {
                    ContextMenuTarget::Cell { row, col }
                };

                self.state.context_menu = Some(state::ContextMenuState {
                    x: e.pos.x,
                    y: e.pos.y,
                    target,
                });
                self.state.select_cell(row, col);
                ctx.request_paint();
                return;
            }

            // Left click - check fill handle area first
            let sel_x = ROW_HEADER_W + (self.state.selected_col + 1) as f64 * col_w;
            let sel_y = grid_top + (self.state.selected_row + 1) as f64 * row_h;
            let handle_size = 6.0;
            if e.pos.x >= sel_x - handle_size * 2.0
                && e.pos.x <= sel_x
                && e.pos.y >= sel_y - handle_size * 2.0
                && e.pos.y <= sel_y
            {
                self.state.fill_handle_dragging = true;
                ctx.request_paint();
                return;
            }

            // Normal cell selection
            let col = if col_w > 0.0 {
                ((e.pos.x - ROW_HEADER_W) / col_w) as usize
            } else {
                0
            };
            let row = if row_h > 0.0 {
                ((e.pos.y - grid_top) / row_h) as usize
            } else {
                0
            };

            // Close dropdown menu when clicking in grid
            if self.state.menu_state.open_menu.is_some() {
                self.state.menu_state.open_menu = None;
            }

            // If editing a formula, clicking a cell inserts a reference
            if self
                .state
                .editing_cell
                .as_ref()
                .map(|e| e.is_formula_edit)
                .unwrap_or(false)
            {
                // Use select_cell which handles formula reference insertion
                self.state.select_cell(row, col);
                ctx.request_paint();
                return;
            }

            // If editing, commit on clicking elsewhere
            if self.state.editing_cell.is_some() {
                self.state.commit_edit();
            }

            // Single click on a different cell while editing commits the edit
            // (Double-click detection not available in PointerButtonEvent; F2 is the keyboard entry)

            // Begin range selection (mouse drag will extend)
            if self.state.begin_range_select(row, col) {
                ctx.request_paint();
            }
            return;
        }

        // Sheet tabs area
        if e.pos.y >= grid_bottom && e.pos.y < grid_bottom + TAB_H {
            let tabs_rect = Rect::new(0.0, grid_bottom, size.width, grid_bottom + TAB_H);

            // Check navigation buttons
            if let Some(nav) = hit_sheet_nav(e.pos.x, e.pos.y, tabs_rect) {
                match nav {
                    NavAction::First => {
                        self.state.select_sheet(0);
                    }
                    NavAction::Prev => {
                        let prev = self.state.active_sheet.saturating_sub(1);
                        self.state.select_sheet(prev);
                    }
                    NavAction::Next => {
                        let next =
                            (self.state.active_sheet + 1).min(self.state.sheet_names.len() - 1);
                        self.state.select_sheet(next);
                    }
                    NavAction::Last => {
                        let last = self.state.sheet_names.len().saturating_sub(1);
                        self.state.select_sheet(last);
                    }
                }
                ctx.request_paint();
                return;
            }

            // Check + button for adding a new sheet
            let nav_w = 112.0;
            let tab_start_x = tabs_rect.x0 + nav_w + 8.0;
            let mut tab_x = tab_start_x;
            for name in &self.state.sheet_names {
                let w = tabs::sheet_tab_width(name);
                tab_x += w;
            }
            let plus_btn_rect =
                Rect::new(tab_x, tabs_rect.y0 + 4.0, tab_x + 24.0, tabs_rect.y1 - 4.0);
            if plus_btn_rect.contains(e.pos) {
                self.state.add_sheet();
                ctx.request_paint();
                return;
            }

            // Check sheet tabs (right-click for context menu)
            if e.button == tench_ui::core::events::PointerButton::Secondary {
                // Fix underflow: check if e.pos.x >= tab_start_x before division
                if e.pos.x < tab_start_x {
                    return;
                }
                let mut tx = tab_start_x;
                let mut found_idx = None;
                for (i, name) in self.state.sheet_names.iter().enumerate() {
                    let w = tabs::sheet_tab_width(name);
                    if e.pos.x >= tx && e.pos.x < tx + w {
                        found_idx = Some(i);
                        break;
                    }
                    tx += w;
                }
                if let Some(idx) = found_idx {
                    self.state.context_menu = Some(state::ContextMenuState {
                        x: e.pos.x,
                        y: e.pos.y,
                        target: ContextMenuTarget::SheetTab { sheet_idx: idx },
                    });
                    ctx.request_paint();
                }
                return;
            }

            // Regular tab click — use dynamic widths
            if e.pos.x < tab_start_x {
                return;
            }
            let mut tx = tab_start_x;
            let mut found_idx = None;
            for (i, name) in self.state.sheet_names.iter().enumerate() {
                let w = tabs::sheet_tab_width(name);
                if e.pos.x >= tx && e.pos.x < tx + w {
                    found_idx = Some(i);
                    break;
                }
                tx += w;
            }
            if let Some(idx) = found_idx {
                if self.state.select_sheet(idx) {
                    ctx.request_paint();
                }
            }
            return;
        }

        // Status bar
        if e.pos.y >= size.height - STATUS_H {
            let status_rect = Rect::new(0.0, size.height - STATUS_H, size.width, size.height);

            // Check zoom slider first
            if let Some(ZoomAction::SliderDrag) =
                hit_zoom_slider(e.pos.x, e.pos.y, status_rect, self.state.zoom_percent)
            {
                self.zoom_slider_dragging = true;
                let new_zoom = zoom_from_slider_x(e.pos.x, status_rect);
                self.state.set_zoom(new_zoom);
                ctx.request_paint();
                return;
            }

            if let Some(zoom_action) = hit_zoom_controls(e.pos.x, e.pos.y, status_rect) {
                match zoom_action {
                    ZoomAction::ZoomIn => {
                        self.state.zoom_in();
                    }
                    ZoomAction::ZoomOut => {
                        self.state.zoom_out();
                    }
                    ZoomAction::ResetZoom => {
                        self.state.set_zoom(100);
                    }
                    ZoomAction::SliderDrag => {
                        // Handled above via hit_zoom_slider
                    }
                }
                ctx.request_paint();
            }
            return;
        }

        // Click on chart toggle
        if e.pos.y >= DOC_TAB_H + MENU_H && e.pos.y < DOC_TAB_H + MENU_H + formula_h {
            let chart_toggle = Rect::new(
                size.width - 36.0,
                DOC_TAB_H + MENU_H + 6.0,
                size.width - 8.0,
                DOC_TAB_H + MENU_H + 30.0,
            );
            if chart_toggle.contains(e.pos) {
                self.state.show_chart_panel = !self.state.show_chart_panel;
                ctx.request_paint();
            }
        }

        // Chart panel hit-testing
        if self.state.show_chart_panel {
            let chart_rect = Rect::new(
                size.width - self.state.chart_panel_width,
                grid_top,
                size.width,
                grid_bottom,
            );
            if chart_rect.contains(e.pos) {
                if let Some(action) = hit_chart_panel(&self.state, e.pos.x, e.pos.y, chart_rect) {
                    match action {
                        ChartPanelAction::PrevChart => {
                            self.state.prev_chart();
                        }
                        ChartPanelAction::NextChart => {
                            self.state.next_chart();
                        }
                        ChartPanelAction::DeleteChart => {
                            self.state.delete_current_chart();
                        }
                        ChartPanelAction::SwitchChartType(ct) => {
                            if !self.state.charts.is_empty() {
                                self.state.charts[self.state.active_chart_idx].chart_type = ct;
                            }
                        }
                        ChartPanelAction::ResizeStart => {
                            self.state.chart_panel_resizing = true;
                        }
                    }
                    ctx.request_paint();
                }
                return;
            }
        }

        // Click outside any interactive area closes dropdown
        if self.state.menu_state.open_menu.is_some() {
            self.state.menu_state.open_menu = None;
            ctx.request_paint();
        }
    }

    pub(crate) fn handle_pointer_move(&mut self, ctx: &mut EventCtx, e: &PointerMoveEvent) {
        // Handle zoom slider drag
        if self.zoom_slider_dragging {
            let size = ctx.state.size;
            let status_rect = Rect::new(0.0, size.height - STATUS_H, size.width, size.height);
            let new_zoom = zoom_from_slider_x(e.pos.x, status_rect);
            if self.state.set_zoom(new_zoom) {
                ctx.request_paint();
            }
            return;
        }

        // Handle chart panel resize drag
        if self.state.chart_panel_resizing {
            let size = ctx.state.size;
            let new_width = (size.width - e.pos.x).clamp(CHART_W_MIN, CHART_W_MAX);
            if (new_width - self.state.chart_panel_width).abs() > 1.0 {
                self.state.chart_panel_width = new_width;
                ctx.request_paint();
            }
            return;
        }

        // Handle fill handle drag
        if self.state.fill_handle_dragging {
            let size = ctx.state.size;
            let (grid_top, _grid_bottom, _grid_right) = self.grid_bounds(size);
            let zoom = self.state.zoom_percent as f64 / 100.0;
            let row_h = GRID_ROW_H * zoom;
            let col_w = grid::GRID_COL_W * zoom;

            let target_col = if col_w > 0.0 {
                ((e.pos.x - ROW_HEADER_W) / col_w) as usize
            } else {
                self.state.selected_col
            };
            let target_row = if row_h > 0.0 {
                ((e.pos.y - grid_top) / row_h) as usize
            } else {
                self.state.selected_row
            };

            // Update drag state
            self.state.drag_state = Some(state::DragState {
                source_row: self.state.selected_row,
                source_col: self.state.selected_col,
                target_row,
                target_col,
                is_copy: false,
            });
            ctx.request_paint();
            return;
        }

        // Handle range selection drag
        if self.state.range_selecting {
            let size = ctx.state.size;
            let (grid_top, _grid_bottom, _grid_right) = self.grid_bounds(size);
            let zoom = self.state.zoom_percent as f64 / 100.0;
            let row_h = GRID_ROW_H * zoom;
            let col_w = grid::GRID_COL_W * zoom;

            let target_col = if col_w > 0.0 {
                ((e.pos.x - ROW_HEADER_W) / col_w) as usize
            } else {
                self.state.selected_col
            };
            let target_row = if row_h > 0.0 {
                ((e.pos.y - grid_top) / row_h) as usize
            } else {
                self.state.selected_row
            };

            if self.state.update_range_select(target_row, target_col) {
                ctx.request_paint();
            }
            return;
        }

        // Handle menu hover (for highlighting)
        if self.state.menu_state.open_menu.is_some() {
            let new_hover = hover_dropdown_item(&self.state, e.pos.x, e.pos.y, DOC_TAB_H);
            if new_hover != self.state.menu_state.hovered_submenu {
                self.state.menu_state.hovered_submenu = new_hover;
                ctx.request_paint();
            }
        }
    }

    pub(crate) fn handle_pointer_up(&mut self, ctx: &mut EventCtx, _e: &PointerButtonEvent) {
        if self.zoom_slider_dragging {
            self.zoom_slider_dragging = false;
            ctx.request_paint();
        }
        if self.state.chart_panel_resizing {
            self.state.chart_panel_resizing = false;
            ctx.request_paint();
        }
        if self.state.fill_handle_dragging {
            self.state.fill_handle_dragging = false;
            if let Some(drag) = self.state.drag_state.take() {
                self.execute_fill_handle(
                    drag.source_row,
                    drag.source_col,
                    drag.target_row,
                    drag.target_col,
                );
            }
            ctx.request_paint();
        }
        if self.state.range_selecting {
            self.state.end_range_select();
            ctx.request_paint();
        }
    }

    pub(crate) fn handle_scroll(&mut self, ctx: &mut EventCtx, e: &PointerScrollEvent) {
        if e.modifiers.control {
            // Ctrl+scroll = zoom
            let new_zoom =
                (self.state.zoom_percent as i32 + (e.delta.y * 0.1) as i32).clamp(25, 400) as u32;
            if self.state.set_zoom(new_zoom) {
                ctx.request_paint();
            }
            return;
        }

        let dx = e.delta.x;
        let dy = e.delta.y;

        if dx.abs() > 0.0 || dy.abs() > 0.0 {
            self.state.scroll_x += dx;
            self.state.scroll_y += dy;
            self.state.scroll_velocity_x = dx;
            self.state.scroll_velocity_y = dy;
            ctx.request_paint();
        }
    }
}
