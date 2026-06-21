use super::*;

impl Widget for SheetsApp {
    fn measure(&mut self, _ctx: &mut MeasureCtx, _axis: Axis, available: f64) -> f64 {
        available
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, _size: Size) {}

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn paint(&mut self, ctx: &mut PaintCtx, scene: &mut Scene) {
        // Apply scroll inertia before painting
        if self.state.apply_scroll_inertia() {
            ctx.state.needs_paint = true;
        }

        // Auto-save check before painting
        if self.state.auto_save.should_save(self.state.is_dirty()) {
            self.auto_save();
        }

        let size = ctx.size();
        let theme = ctx.theme();
        // Sync maximized flag so the caption buttons paint the correct glyph.
        self.state.window_maximized = ctx.global.window_maximized;
        let mut p = Painter::new(scene);
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
        let grid_top = DOC_TAB_H + MENU_H + formula_h + toolbar_h + GRID_HEADER_H;
        let grid_bottom = size.height - STATUS_H - TAB_H;

        p.fill_background(size, theme.background);

        // 8.4 Document tabs (topmost)
        paint_doc_tabs(&self.state, &mut p, theme, size.width);

        // Menu bar
        paint_menu_bar(&self.state, &mut p, theme, size.width, DOC_TAB_H);

        // Caption buttons span the doc-tab + menu-bar header band.
        paint_window_controls(
            &mut p,
            size.width,
            DOC_TAB_H + MENU_H,
            self.state.window_maximized,
            self.state.window_control_hovered,
        );

        // Formula bar (conditional on toggle)
        if self.state.show_formula_bar {
            paint_formula_bar(
                &self.state,
                &mut p,
                theme,
                size.width,
                DOC_TAB_H + MENU_H,
                FORMULA_H,
            );
        }

        // Toolbar (Phase 5)
        if self.state.show_toolbar {
            toolbar::paint_toolbar(
                &self.state,
                &mut p,
                theme,
                size.width,
                DOC_TAB_H + MENU_H + formula_h,
            );
        }

        // Grid (clipped to prevent overflow into status bar / sheet tabs)
        p.push_clip(Rect::new(0.0, grid_top, size.width - chart_w, grid_bottom));
        paint_grid(
            &self.state,
            &mut p,
            theme,
            Rect::new(0.0, grid_top, size.width - chart_w, grid_bottom),
            &mut self.cell_cache,
            &mut self.text_cache,
        );
        p.pop_clip();

        if self.state.show_chart_panel {
            paint_chart_panel(
                &self.state,
                &mut p,
                theme,
                Rect::new(size.width - chart_w, grid_top, size.width, grid_bottom),
                &mut self.chart_cache,
            );
        }

        // Sheet tabs
        paint_sheet_tabs(
            &self.state,
            &mut p,
            theme,
            Rect::new(0.0, grid_bottom, size.width, grid_bottom + TAB_H),
        );

        // Status bar
        paint_status_bar(
            &self.state,
            &mut p,
            theme,
            Rect::new(0.0, size.height - STATUS_H, size.width, size.height),
        );

        // Active cell reference (in the grid header area)
        let active = self.state.active_cell_ref();
        p.draw_text(
            &active,
            ROW_HEADER_W + 6.0,
            grid_top - 8.0,
            theme.primary,
            theme.font_size_small,
            tench_ui::parley::FontWeight::BOLD,
            false,
        );

        // Dropdown menu (overlays on top)
        paint_dropdown_menu(&self.state, &mut p, theme, DOC_TAB_H);

        // Context menu (overlays on top)
        paint_context_menu(&self.state, &mut p, theme);

        // Dialogs
        if let Some(modal_type) = &self.state.active_modal {
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_sheets_modal(&mut p, theme, size, modal_type);
        }
        if self.state.show_welcome {
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_sheets_modal(&mut p, theme, size, &state::ModalType::Welcome);
        }
        if self.state.show_find_replace {
            // Dimming background
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_find_replace_dialog(&mut p, theme, size, &self.state);
        }
        if self.state.show_paste_special {
            // Dimming background
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_paste_special_dialog(&mut p, theme, size, &self.state);
        }
        if self.state.show_named_ranges {
            // Dimming background
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_named_ranges_dialog(&mut p, theme, size, &self.state);
        }
        if self.state.show_insert_function {
            // Dimming background
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_insert_function_dialog(&mut p, theme, size, &self.state);
        }
        if self.state.show_sort_dialog {
            // Dimming background
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_sort_dialog(&mut p, theme, size, &self.state);
        }
        if self.state.show_settings {
            // Dimming background
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_settings_dialog(&mut p, theme, size, &self.state);
        }
        // 10.2 Print preview (full-screen overlay)
        if self.state.print_preview.visible {
            paint_print_preview(&self.state, &mut p, theme, size);
        }
        // 10.4 Page setup dialog
        if self.state.show_page_setup {
            paint_page_setup_dialog(&self.state, &mut p, theme, size);
        }
        // File dialog (Phase 4)
        if self.state.show_file_dialog {
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_file_dialog(&mut p, theme, size, &self.state);
        }
        // Format cells dialog (Phase 5)
        if self.state.format_cells.visible {
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_format_cells_dialog(&mut p, theme, size, &self.state);
        }
        // Conditional format dialog (Phase 5)
        if self.state.conditional_format_dialog.visible {
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_cond_format_dialog(&mut p, theme, size, &self.state);
        }
        // Phase 6: Data Validation dialog
        if self.state.data_validation_dialog.visible {
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_data_validation_dialog(&mut p, theme, size, &self.state);
        }
        // Phase 6: Pivot Table placeholder dialog
        if self.state.show_pivot_table {
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_pivot_table_dialog(&mut p, theme, size);
        }
        // Phase 7: Chart wizard dialog
        if self.state.show_chart_wizard {
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_chart_wizard(&mut p, theme, size, &self.state);
        }
        // Phase 8: Tab color picker dialog
        if self.state.show_tab_color_picker {
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_tab_color_picker(&mut p, theme, size, &self.state);
        }
        // Phase 8: Move sheet dialog
        if self.state.show_move_sheet_dialog {
            p.fill_rect(
                Rect::new(0.0, 0.0, size.width, size.height),
                Color::rgba8(0, 0, 0, 120),
            );
            paint_move_sheet_dialog(&mut p, theme, size, &self.state);
        }
        // Toast auto-dismiss after 3 seconds
        if let Some((_, set_at)) = &self.state.toast {
            if set_at.elapsed().as_secs() >= 3 {
                self.state.toast = None;
            }
        }
        if let Some((msg, _)) = &self.state.toast {
            paint_toast(&mut p, theme, size, msg);
        }
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx, event: &PointerEvent) {
        match event {
            PointerEvent::Down(e) => {
                self.handle_pointer_down(ctx, e);
            }
            PointerEvent::Move(e) => {
                self.handle_pointer_move(ctx, e);
            }
            PointerEvent::Up(e) => {
                self.handle_pointer_up(ctx, e);
            }
            PointerEvent::Scroll(e) => {
                self.handle_scroll(ctx, e);
            }
            _ => {}
        }
    }

    fn on_text_event(&mut self, ctx: &mut EventCtx, event: &TextEvent) {
        let TextEvent::Keyboard(kb) = event else {
            return;
        };
        if !kb.is_pressed {
            return;
        }

        // If editing a cell, route keys to the editing workflow
        if self.state.editing_cell.is_some() {
            let changed = self.handle_edit_key(kb);
            if changed {
                ctx.request_paint();
            }
            return;
        }

        // If find/replace dialog is open, route text input to the focused field
        if self.state.show_find_replace {
            let changed = self.handle_find_replace_key(kb);
            if changed {
                ctx.request_paint();
            }
            return;
        }

        // If insert function dialog is open, handle navigation
        if self.state.show_insert_function {
            let changed = self.handle_insert_function_key(kb);
            if changed {
                ctx.request_paint();
            }
            return;
        }

        // If file dialog is open, route text input
        if self.state.show_file_dialog {
            let changed = self.handle_file_dialog_key(kb);
            if changed {
                ctx.request_paint();
            }
            return;
        }

        // If format cells dialog is open, handle navigation
        if self.state.format_cells.visible {
            let changed = self.handle_format_cells_key(kb);
            if changed {
                ctx.request_paint();
            }
            return;
        }

        // If conditional format dialog is open, handle input
        if self.state.conditional_format_dialog.visible {
            let changed = self.handle_cond_format_key(kb);
            if changed {
                ctx.request_paint();
            }
            return;
        }

        // Phase 6: If data validation dialog is open, handle input
        if self.state.data_validation_dialog.visible {
            let changed = self.handle_data_validation_key(kb);
            if changed {
                ctx.request_paint();
            }
            return;
        }

        // Delegate remaining keyboard handling to events module
        self.handle_main_key(ctx, kb);
    }

    fn accessibility_tree(&self, state: &WidgetState) -> AccessibilityNode {
        AccessibilityNode {
            role: AccessRole::Window,
            label: Some("Tench Sheets".to_string()),
            value: Some(self.state.status_line().to_string()),
            focused: state.is_focused,
            disabled: state.is_disabled,
            children: Vec::new(),
        }
    }

    fn automation_children(&self, state: &WidgetState) -> Vec<UiAutomationNode> {
        automation::sheets_automation_nodes(&self.state, state.size, state.id.to_raw())
    }

    fn debug_id(&self) -> Option<&str> {
        Some("sheets.root")
    }
}
