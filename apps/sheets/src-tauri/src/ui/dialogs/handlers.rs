use super::super::*;
use tench_ui::core::events::PointerButtonEvent;

// ---------------------------------------------------------------------------
// Dialog click/key handlers
// ---------------------------------------------------------------------------

impl SheetsApp {
    // -- Find/Replace dialog click handler --

    pub(crate) fn handle_find_replace_click(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        let w = 380.0;
        let h = 260.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            MENU_H + FORMULA_H + 10.0,
            size.width / 2.0 + w / 2.0,
            MENU_H + FORMULA_H + 10.0 + h,
        );
        let x0 = modal.x0 + 16.0;
        let mut y = modal.y0 + 24.0 + 28.0; // past title

        // Find input box
        let find_box = Rect::new(x0, y, modal.x1 - 16.0, y + 22.0);
        y += 28.0;

        // Replace input box
        let replace_box = Rect::new(x0, y, modal.x1 - 16.0, y + 22.0);
        y += 30.0;

        // Options row (Case, Regex, Formulas checkboxes)
        let options_y = y;
        let case_rect = Rect::new(x0, options_y - 12.0, x0 + 70.0, options_y + 12.0);
        let regex_rect = Rect::new(x0 + 80.0, options_y - 12.0, x0 + 155.0, options_y + 12.0);
        let formulas_rect = Rect::new(x0 + 165.0, options_y - 12.0, x0 + 260.0, options_y + 12.0);
        y += 22.0;

        // Scope row
        let scope_rect = Rect::new(x0, y - 12.0, x0 + 200.0, y + 12.0);
        y += 24.0;

        // Buttons row
        let buttons_y = y;
        let button_labels = ["Find", "Next", "Prev", "Replace", "Replace All", "Close"];
        let mut btn_rects = Vec::new();
        let mut bx = x0;
        for _ in &button_labels {
            btn_rects.push(Rect::new(bx, buttons_y, bx + 52.0, buttons_y + 22.0));
            bx += 58.0;
        }

        // Hit-test: click outside dialog closes it
        if !modal.contains(e.pos) {
            self.state.show_find_replace = false;
            self.state.find_replace.matches.clear();
            self.state.find_replace.current_match = None;
            ctx.request_paint();
            return;
        }

        // Find input
        if find_box.contains(e.pos) {
            self.state.find_replace_focused_field = FindReplaceFocusedField::Find;
            ctx.request_paint();
            return;
        }

        // Replace input
        if replace_box.contains(e.pos) {
            self.state.find_replace_focused_field = FindReplaceFocusedField::Replace;
            ctx.request_paint();
            return;
        }

        // Case sensitive checkbox
        if case_rect.contains(e.pos) {
            self.state.find_replace.case_sensitive = !self.state.find_replace.case_sensitive;
            ctx.request_paint();
            return;
        }

        // Regex checkbox
        if regex_rect.contains(e.pos) {
            self.state.find_replace.use_regex = !self.state.find_replace.use_regex;
            ctx.request_paint();
            return;
        }

        // Formulas checkbox
        if formulas_rect.contains(e.pos) {
            self.state.find_replace.search_in_formulas =
                !self.state.find_replace.search_in_formulas;
            ctx.request_paint();
            return;
        }

        // Scope toggle
        if scope_rect.contains(e.pos) {
            self.state.find_replace.scope = match self.state.find_replace.scope {
                state::SearchScope::CurrentSheet => state::SearchScope::EntireWorkbook,
                state::SearchScope::EntireWorkbook => state::SearchScope::Selection,
                state::SearchScope::Selection => state::SearchScope::CurrentSheet,
            };
            ctx.request_paint();
            return;
        }

        // Buttons
        for (i, rect) in btn_rects.iter().enumerate() {
            if rect.contains(e.pos) {
                match i {
                    0 => {
                        // Find
                        self.state.find();
                    }
                    1 => {
                        // Next
                        self.state.find_next();
                    }
                    2 => {
                        // Prev
                        self.state.find_prev();
                    }
                    3 => {
                        // Replace
                        self.state.replace_next();
                    }
                    4 => {
                        // Replace All
                        let count = self.state.replace_all();
                        self.state.toast =
                            Some((format!("Replaced {count} occurrences"), Instant::now()));
                    }
                    5 => {
                        // Close
                        self.state.show_find_replace = false;
                        self.state.find_replace.matches.clear();
                        self.state.find_replace.current_match = None;
                    }
                    _ => {}
                }
                ctx.request_paint();
                return;
            }
        }
    }

    // -- Find/Replace keyboard handler --

    pub(crate) fn handle_find_replace_key(
        &mut self,
        kb: &tench_ui::core::events::KeyboardEvent,
    ) -> bool {
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.show_find_replace = false;
                self.state.find_replace.matches.clear();
                self.state.find_replace.current_match = None;
                true
            }
            LogicalKey::Named(NamedKey::Tab) => {
                self.state.find_replace_focused_field = match self.state.find_replace_focused_field
                {
                    FindReplaceFocusedField::Find => FindReplaceFocusedField::Replace,
                    FindReplaceFocusedField::Replace => FindReplaceFocusedField::Find,
                };
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                self.state.find();
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                match self.state.find_replace_focused_field {
                    FindReplaceFocusedField::Find => {
                        self.state.find_replace.find_text.pop();
                    }
                    FindReplaceFocusedField::Replace => {
                        self.state.find_replace.replace_text.pop();
                    }
                }
                true
            }
            LogicalKey::Character(text) if !text.is_empty() && !kb.modifiers.control => {
                match self.state.find_replace_focused_field {
                    FindReplaceFocusedField::Find => {
                        self.state.find_replace.find_text.push_str(text);
                    }
                    FindReplaceFocusedField::Replace => {
                        self.state.find_replace.replace_text.push_str(text);
                    }
                }
                true
            }
            _ => false,
        }
    }

    // -- Paste Special dialog click handler --

    pub(crate) fn handle_paste_special_click(
        &mut self,
        ctx: &mut EventCtx,
        e: &PointerButtonEvent,
    ) {
        let size = ctx.state.size;
        let w = 280.0;
        let h = 180.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );
        let x0 = modal.x0 + 16.0;
        let mut y = modal.y0 + 24.0 + 30.0;

        // Radio button rows
        let modes = [
            state::PasteSpecialMode::All,
            state::PasteSpecialMode::ValuesOnly,
            state::PasteSpecialMode::FormatsOnly,
            state::PasteSpecialMode::FormulasOnly,
        ];
        let mut radio_rects = Vec::new();
        for _ in &modes {
            radio_rects.push(Rect::new(x0, y - 12.0, modal.x1 - 16.0, y + 12.0));
            y += 22.0;
        }
        y += 8.0;

        // OK button
        let ok_rect = Rect::new(x0, y, x0 + 60.0, y + 24.0);
        // Cancel button
        let cancel_rect = Rect::new(x0 + 72.0, y, x0 + 132.0, y + 24.0);

        // Click outside closes
        if !modal.contains(e.pos) {
            self.state.show_paste_special = false;
            ctx.request_paint();
            return;
        }

        // Radio buttons
        for (i, rect) in radio_rects.iter().enumerate() {
            if rect.contains(e.pos) {
                self.state.paste_special_mode = modes[i];
                ctx.request_paint();
                return;
            }
        }

        // OK button
        if ok_rect.contains(e.pos) {
            let mode = self.state.paste_special_mode;
            self.state.edit_paste_special(mode);
            self.state.show_paste_special = false;
            ctx.request_paint();
            return;
        }

        // Cancel button
        if cancel_rect.contains(e.pos) {
            self.state.show_paste_special = false;
            ctx.request_paint();
        }
    }

    // -- Named Ranges dialog click handler --

    pub(crate) fn handle_named_ranges_click(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        let w = 320.0;
        let h = 240.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );
        let x0 = modal.x0 + 16.0;
        let list_start_y = modal.y0 + 24.0 + 28.0 + 20.0; // past title + header
        let list_end_y = modal.y1 - 50.0;

        // Click outside closes
        if !modal.contains(e.pos) {
            self.state.show_named_ranges = false;
            ctx.request_paint();
            return;
        }

        // Check if clicking on a named range row
        if e.pos.y >= list_start_y && e.pos.y < list_end_y && e.pos.x >= x0 {
            let row_idx = ((e.pos.y - list_start_y) / 20.0) as usize;
            if row_idx < self.state.named_ranges.len() {
                self.state.named_ranges_selected = Some(row_idx);
                ctx.request_paint();
                return;
            }
        }

        // Buttons at the bottom
        let btn_y = modal.y1 - 40.0;
        let btn_labels = ["Add", "Edit", "Delete", "Close"];
        let mut bx = x0;
        for (i, _label) in btn_labels.iter().enumerate() {
            let btn_rect = Rect::new(bx, btn_y, bx + 56.0, btn_y + 24.0);
            if btn_rect.contains(e.pos) {
                match i {
                    0 => {
                        // Add: create a new named range with current selection
                        let (sr, sc, er, ec) = self.state.selection_range();
                        let range = state::CellRange {
                            start_row: sr,
                            start_col: sc,
                            end_row: er,
                            end_col: ec,
                        };
                        let name_idx = self.state.named_ranges.len() + 1;
                        let name = format!("Range{}", name_idx);
                        self.state.define_name(name, None, range);
                        self.state.named_ranges_selected =
                            Some(self.state.named_ranges.len().saturating_sub(1));
                    }
                    1 => {
                        // Edit: update the selected named range to current selection
                        if let Some(idx) = self.state.named_ranges_selected {
                            if idx < self.state.named_ranges.len() {
                                let (sr, sc, er, ec) = self.state.selection_range();
                                let range = state::CellRange {
                                    start_row: sr,
                                    start_col: sc,
                                    end_row: er,
                                    end_col: ec,
                                };
                                self.state.named_ranges[idx].range = range;
                            }
                        }
                    }
                    2 => {
                        // Delete: remove the selected named range
                        if let Some(idx) = self.state.named_ranges_selected {
                            if idx < self.state.named_ranges.len() {
                                let name = self.state.named_ranges[idx].name.clone();
                                self.state.delete_named_range(&name);
                                self.state.named_ranges_selected = None;
                            }
                        }
                    }
                    3 => {
                        // Close
                        self.state.show_named_ranges = false;
                    }
                    _ => {}
                }
                ctx.request_paint();
                return;
            }
            bx += 64.0;
        }
    }

    // -- Page Setup dialog click handler --

    pub(crate) fn handle_page_setup_click(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        // Match the dialog rect from paint_page_setup_dialog in print_preview.rs
        let modal_w = 400.0;
        let modal_h = 480.0;
        let modal = Rect::new(
            size.width / 2.0 - modal_w / 2.0,
            size.height / 2.0 - modal_h / 2.0,
            size.width / 2.0 + modal_w / 2.0,
            size.height / 2.0 + modal_h / 2.0,
        );

        // Click outside closes
        if !modal.contains(e.pos) {
            self.state.show_page_setup = false;
            ctx.request_paint();
            return;
        }

        let x0 = modal.x0 + 20.0;
        let mut y = modal.y0 + 28.0 + 32.0; // past title

        // Paper size row — cycle through sizes
        let paper_rect = Rect::new(x0, y - 12.0, modal.x1 - 20.0, y + 12.0);
        if paper_rect.contains(e.pos) {
            let sizes = [
                state::PaperSize::A4,
                state::PaperSize::Letter,
                state::PaperSize::Legal,
                state::PaperSize::Tabloid,
                state::PaperSize::A3,
                state::PaperSize::A5,
            ];
            let current = &self.state.page_setup.paper_size;
            let idx = sizes.iter().position(|s| s == current).unwrap_or(0);
            let next = sizes[(idx + 1) % sizes.len()];
            self.state.page_setup.paper_size = next;
            ctx.request_paint();
            return;
        }
        y += 24.0;

        // Orientation row — toggle
        let orient_rect = Rect::new(x0, y - 12.0, modal.x1 - 20.0, y + 12.0);
        if orient_rect.contains(e.pos) {
            self.state.page_setup.orientation = match self.state.page_setup.orientation {
                state::Orientation::Portrait => state::Orientation::Landscape,
                state::Orientation::Landscape => state::Orientation::Portrait,
            };
            ctx.request_paint();
            return;
        }
        y += 24.0;

        // Skip "Margins (mm):" label
        y += 20.0;
        // Skip 6 margin rows
        y += 18.0 * 6.0;
        y += 8.0;

        // Scaling row — toggle between Percentage and FitToPages
        let scaling_rect = Rect::new(x0, y - 12.0, modal.x1 - 20.0, y + 12.0);
        if scaling_rect.contains(e.pos) {
            self.state.page_setup.scaling = match self.state.page_setup.scaling {
                state::Scaling::Percentage(_) => state::Scaling::FitToPages {
                    width: Some(1),
                    height: None,
                },
                state::Scaling::FitToPages { .. } => state::Scaling::Percentage(100.0),
            };
            ctx.request_paint();
            return;
        }
        y += 24.0;

        // Skip "Options:" label
        y += 20.0;

        // Checkbox rows: Gridlines, Headers, Center H, Center V, Repeat Header
        let checkbox_toggles: [fn(&mut state::PageSetup); 5] = [
            |s| s.gridlines_print = !s.gridlines_print,
            |s| s.row_col_headers_print = !s.row_col_headers_print,
            |s| s.center_horizontally = !s.center_horizontally,
            |s| s.center_vertically = !s.center_vertically,
            |s| s.repeat_header = !s.repeat_header,
        ];
        for toggle in &checkbox_toggles {
            let cb_rect = Rect::new(x0, y - 12.0, modal.x1 - 20.0, y + 12.0);
            if cb_rect.contains(e.pos) {
                toggle(&mut self.state.page_setup);
                ctx.request_paint();
                return;
            }
            y += 20.0;
        }

        y += 8.0;

        // Skip print area row
        y += 32.0;

        // OK button
        let ok_rect = Rect::new(x0, y, x0 + 64.0, y + 24.0);
        if ok_rect.contains(e.pos) {
            self.state.show_page_setup = false;
            ctx.request_paint();
            return;
        }

        // Cancel button
        let cancel_rect = Rect::new(x0 + 72.0, y, x0 + 136.0, y + 24.0);
        if cancel_rect.contains(e.pos) {
            self.state.show_page_setup = false;
            ctx.request_paint();
        }
    }

    // -- Insert Function dialog click handler --

    pub(crate) fn handle_insert_function_click(
        &mut self,
        ctx: &mut EventCtx,
        e: &PointerButtonEvent,
    ) {
        let size = ctx.state.size;
        let w = 400.0;
        let h = 380.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );

        // Click outside closes
        if !modal.contains(e.pos) {
            self.state.show_insert_function = false;
            ctx.request_paint();
            return;
        }

        let x0 = modal.x0 + 16.0;
        let list_start_y = modal.y0 + 24.0 + 28.0;
        let list_end_y = modal.y1 - 80.0;
        let row_h = 20.0;

        // Function list
        if e.pos.x < modal.x1 - 16.0 && e.pos.y >= list_start_y && e.pos.y < list_end_y {
            let catalog = state::function_catalog();
            let idx = ((e.pos.y - list_start_y) / row_h) as usize;
            if idx < catalog.len() {
                self.state.insert_function_selected = idx;
                ctx.request_paint();
                return;
            }
        }

        // Insert button
        let insert_btn = Rect::new(x0, modal.y1 - 40.0, x0 + 70.0, modal.y1 - 16.0);
        if insert_btn.contains(e.pos) {
            let catalog = state::function_catalog();
            if let Some(func) = catalog.get(self.state.insert_function_selected) {
                // Insert function into the active cell
                let sig = func.name;
                self.state.begin_edit_replace();
                self.state.edit_insert_text(&format!("={sig}()"));
                // Move cursor before the closing paren
                if let Some(edit) = self.state.editing_cell.as_mut() {
                    edit.cursor_pos = edit.cursor_pos.saturating_sub(1);
                }
                self.state.show_insert_function = false;
            }
            ctx.request_paint();
            return;
        }

        // Cancel button
        let cancel_btn = Rect::new(x0 + 82.0, modal.y1 - 40.0, x0 + 152.0, modal.y1 - 16.0);
        if cancel_btn.contains(e.pos) {
            self.state.show_insert_function = false;
            ctx.request_paint();
        }
    }

    // -- Insert Function keyboard handler --

    pub(crate) fn handle_insert_function_key(
        &mut self,
        kb: &tench_ui::core::events::KeyboardEvent,
    ) -> bool {
        let catalog = state::function_catalog();
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.show_insert_function = false;
                true
            }
            LogicalKey::Named(NamedKey::ArrowDown) => {
                if self.state.insert_function_selected + 1 < catalog.len() {
                    self.state.insert_function_selected += 1;
                }
                true
            }
            LogicalKey::Named(NamedKey::ArrowUp) => {
                self.state.insert_function_selected =
                    self.state.insert_function_selected.saturating_sub(1);
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                if let Some(func) = catalog.get(self.state.insert_function_selected) {
                    let sig = func.name;
                    self.state.begin_edit_replace();
                    self.state.edit_insert_text(&format!("={sig}()"));
                    if let Some(edit) = self.state.editing_cell.as_mut() {
                        edit.cursor_pos = edit.cursor_pos.saturating_sub(1);
                    }
                    self.state.show_insert_function = false;
                }
                true
            }
            _ => false,
        }
    }

    // -- Sort dialog click handler --

    pub(crate) fn handle_sort_dialog_click(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        let w = 300.0;
        let h = 220.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );

        // Click outside closes
        if !modal.contains(e.pos) {
            self.state.show_sort_dialog = false;
            ctx.request_paint();
            return;
        }

        let x0 = modal.x0 + 16.0;
        let mut y = modal.y0 + 24.0 + 28.0;

        // Sort column dropdown area — cycle through columns
        let col_rect = Rect::new(x0, y, modal.x1 - 16.0, y + 22.0);
        if col_rect.contains(e.pos) {
            // Future: cycle the selected column for sorting
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // Ascending radio
        let asc_rect = Rect::new(x0, y - 12.0, x0 + 120.0, y + 12.0);
        if asc_rect.contains(e.pos) {
            self.state.sort_ascending = true;
            ctx.request_paint();
            return;
        }

        // Descending radio
        let desc_rect = Rect::new(x0 + 130.0, y - 12.0, x0 + 260.0, y + 12.0);
        if desc_rect.contains(e.pos) {
            self.state.sort_ascending = false;
            ctx.request_paint();
            return;
        }
        y += 26.0;

        // Header row checkbox
        let header_rect = Rect::new(x0, y - 12.0, x0 + 160.0, y + 12.0);
        if header_rect.contains(e.pos) {
            self.state.sort_has_header = !self.state.sort_has_header;
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // OK button
        let ok_rect = Rect::new(x0, y, x0 + 60.0, y + 24.0);
        if ok_rect.contains(e.pos) {
            self.state.sort_grid_by_col(
                self.state.selected_col,
                self.state.sort_ascending,
                self.state.sort_has_header,
            );
            self.state.show_sort_dialog = false;
            ctx.request_paint();
            return;
        }

        // Cancel button
        let cancel_rect = Rect::new(x0 + 72.0, y, x0 + 132.0, y + 24.0);
        if cancel_rect.contains(e.pos) {
            self.state.show_sort_dialog = false;
            ctx.request_paint();
        }
    }

    // -- Settings dialog click handler --

    pub(crate) fn handle_settings_click(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        let w = 320.0;
        let h = 260.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );

        // Click outside closes
        if !modal.contains(e.pos) {
            self.state.show_settings = false;
            ctx.request_paint();
            return;
        }

        let x0 = modal.x0 + 16.0;
        let mut y = modal.y0 + 24.0 + 28.0;

        // Auto-save interval buttons
        let minus_rect = Rect::new(x0 + 200.0, y - 12.0, x0 + 220.0, y + 12.0);
        let plus_rect = Rect::new(x0 + 260.0, y - 12.0, x0 + 280.0, y + 12.0);
        if minus_rect.contains(e.pos) {
            self.state.auto_save.interval_secs =
                self.state.auto_save.interval_secs.saturating_sub(5).max(5);
            ctx.request_paint();
            return;
        }
        if plus_rect.contains(e.pos) {
            self.state.auto_save.interval_secs = (self.state.auto_save.interval_secs + 5).min(300);
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // Auto-save enable toggle
        let toggle_rect = Rect::new(x0, y - 12.0, x0 + 200.0, y + 12.0);
        if toggle_rect.contains(e.pos) {
            self.state.auto_save.enabled = !self.state.auto_save.enabled;
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // Grid lines toggle
        let grid_rect = Rect::new(x0, y - 12.0, x0 + 200.0, y + 12.0);
        if grid_rect.contains(e.pos) {
            self.state.show_grid_lines = !self.state.show_grid_lines;
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // Headers toggle
        let headers_rect = Rect::new(x0, y - 12.0, x0 + 200.0, y + 12.0);
        if headers_rect.contains(e.pos) {
            self.state.show_headers = !self.state.show_headers;
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // Formula bar toggle
        let fbar_rect = Rect::new(x0, y - 12.0, x0 + 200.0, y + 12.0);
        if fbar_rect.contains(e.pos) {
            self.state.show_formula_bar = !self.state.show_formula_bar;
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // OK button
        let ok_rect = Rect::new(x0, y, x0 + 60.0, y + 24.0);
        if ok_rect.contains(e.pos) {
            self.state.show_settings = false;
            ctx.request_paint();
            return;
        }

        // Cancel button
        let cancel_rect = Rect::new(x0 + 72.0, y, x0 + 132.0, y + 24.0);
        if cancel_rect.contains(e.pos) {
            self.state.show_settings = false;
            ctx.request_paint();
        }
    }
}
