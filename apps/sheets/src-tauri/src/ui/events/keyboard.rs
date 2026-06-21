use super::super::*;
use tench_ui::core::events::{LogicalKey, NamedKey};

impl SheetsApp {
    /// Handle keyboard input while in cell editing mode.
    pub(crate) fn handle_edit_key(&mut self, kb: &tench_ui::core::events::KeyboardEvent) -> bool {
        match &kb.logical_key {
            // Enter: commit edit + move down
            LogicalKey::Named(NamedKey::Enter) if !kb.modifiers.alt => {
                self.state.commit_edit();
                self.state.move_selection(1, 0);
                true
            }
            // Alt+Enter: insert line break in cell
            LogicalKey::Named(NamedKey::Enter) if kb.modifiers.alt => {
                self.state.edit_insert_text("\n");
                true
            }
            // Tab: commit edit + move right
            LogicalKey::Named(NamedKey::Tab) => {
                self.state.commit_edit();
                self.state.move_selection(0, 1);
                true
            }
            // Escape: cancel edit
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.cancel_edit();
                true
            }
            // Arrow keys: move cursor within edit draft
            LogicalKey::Named(NamedKey::ArrowLeft) => {
                self.state.dismiss_autocomplete();
                self.state.edit_move_cursor(-1)
            }
            LogicalKey::Named(NamedKey::ArrowRight) => {
                self.state.dismiss_autocomplete();
                self.state.edit_move_cursor(1)
            }
            LogicalKey::Named(NamedKey::ArrowUp) => {
                // Navigate autocomplete if visible, otherwise commit and move
                if self
                    .state
                    .editing_cell
                    .as_ref()
                    .and_then(|e| e.autocomplete.as_ref())
                    .is_some()
                {
                    self.state.autocomplete_navigate(-1)
                } else {
                    self.state.commit_edit();
                    self.state.move_selection(-1, 0);
                    true
                }
            }
            LogicalKey::Named(NamedKey::ArrowDown) => {
                // Navigate autocomplete if visible, otherwise commit and move
                if self
                    .state
                    .editing_cell
                    .as_ref()
                    .and_then(|e| e.autocomplete.as_ref())
                    .is_some()
                {
                    self.state.autocomplete_navigate(1)
                } else {
                    self.state.commit_edit();
                    self.state.move_selection(1, 0);
                    true
                }
            }
            // Home: cursor to start
            LogicalKey::Named(NamedKey::Home) => self.state.edit_cursor_home(),
            // End: cursor to end
            LogicalKey::Named(NamedKey::End) => self.state.edit_cursor_end(),
            // Backspace: delete char before cursor
            LogicalKey::Named(NamedKey::Backspace) => self.state.edit_backspace(),
            // Delete: delete char after cursor
            LogicalKey::Named(NamedKey::Delete) => self.state.edit_delete_forward(),
            // Tab or Enter with autocomplete: accept suggestion
            // (handled above for Tab/Enter, autocomplete is checked in accept_autocomplete)
            // Regular text: insert into draft
            LogicalKey::Character(text) if !text.is_empty() && !kb.modifiers.control => {
                self.state.edit_insert_text(text);
                true
            }
            _ => false,
        }
    }

    /// Handle keyboard input for the License activation modal.
    /// - Escape → close
    /// - Enter → trigger activation
    /// - Backspace → delete last char
    /// - Character (printable) → append (uppercased; license keys are case-insensitive)
    pub(crate) fn handle_license_modal_keyboard(
        &mut self,
        kb: &tench_ui::core::events::KeyboardEvent,
    ) -> bool {
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.license_modal = None;
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                if let Some(store) = self.license_store.clone() {
                    let key = self
                        .state
                        .license_modal
                        .as_ref()
                        .map(|m| m.license_key_input.clone())
                        .unwrap_or_default();
                    if !key.is_empty() {
                        match tench_update_client::activate_license(
                            &store,
                            None,
                            &key,
                            "sheets",
                            env!("CARGO_PKG_VERSION"),
                        ) {
                            Ok(()) => {
                                if let Some(m) = &mut self.state.license_modal {
                                    m.status_message = "Activated".into();
                                }
                            }
                            Err(err) => {
                                if let Some(m) = &mut self.state.license_modal {
                                    m.status_message = format!("Activation failed: {err}");
                                }
                            }
                        }
                    }
                }
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                if let Some(m) = &mut self.state.license_modal {
                    m.license_key_input.pop();
                }
                true
            }
            LogicalKey::Character(c) => {
                if let Some(m) = &mut self.state.license_modal {
                    m.license_key_input.push_str(&c.to_uppercase());
                }
                true
            }
            _ => false,
        }
    }

    /// Handle main keyboard input (not editing, not in any dialog).
    pub(crate) fn handle_main_key(
        &mut self,
        ctx: &mut EventCtx,
        kb: &tench_ui::core::events::KeyboardEvent,
    ) {
        // Phase 6: If pivot table dialog is open, close on Escape/Enter
        if self.state.show_pivot_table {
            if matches!(
                &kb.logical_key,
                LogicalKey::Named(NamedKey::Escape) | LogicalKey::Named(NamedKey::Enter)
            ) {
                self.state.show_pivot_table = false;
                ctx.request_paint();
            }
            return;
        }

        // Phase 7: If chart wizard is open, route keys to it
        if self.state.show_chart_wizard {
            let changed = super::super::charts::handle_chart_wizard_key(&mut self.state, kb);
            if changed {
                ctx.request_paint();
            }
            return;
        }

        // Phase 8: If renaming a sheet tab, route text to rename draft
        if self.state.renaming_sheet.is_some() {
            match &kb.logical_key {
                LogicalKey::Named(NamedKey::Escape) => {
                    self.state.renaming_sheet = None;
                    self.state.rename_draft.clear();
                    ctx.request_paint();
                }
                LogicalKey::Named(NamedKey::Enter) => {
                    if let Some(idx) = self.state.renaming_sheet.take() {
                        if !self.state.rename_draft.is_empty() && idx < self.state.sheet_names.len()
                        {
                            self.state.sheet_names[idx] =
                                std::mem::take(&mut self.state.rename_draft);
                        }
                    }
                    ctx.request_paint();
                }
                LogicalKey::Named(NamedKey::Backspace) => {
                    self.state.rename_draft.pop();
                    ctx.request_paint();
                }
                LogicalKey::Character(c) if !c.is_empty() && !kb.modifiers.control => {
                    self.state.rename_draft.push_str(c);
                    ctx.request_paint();
                }
                _ => {}
            }
            return;
        }

        let changed = match &kb.logical_key {
            // Ctrl+S: Save
            LogicalKey::Character(c)
                if kb.modifiers.control && c.eq_ignore_ascii_case("s") && !kb.modifiers.shift =>
            {
                self.save_current_workbook();
                true
            }
            // Ctrl+Shift+S: Save As
            LogicalKey::Character(c)
                if kb.modifiers.control && kb.modifiers.shift && c.eq_ignore_ascii_case("s") =>
            {
                self.state.file_dialog_mode = FileDialogMode::SaveAs;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            // Ctrl+N: New workbook
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("n") => {
                self.execute_menu_action(MenuAction::NewWorkbook)
            }
            // Ctrl+O: Open
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("o") => {
                self.state.file_dialog_mode = FileDialogMode::Open;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            // Ctrl+P: Print Preview
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("p") => {
                self.state.compute_print_pages();
                self.state.print_preview.visible = true;
                true
            }
            // Ctrl+H: Replace
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("h") => {
                self.state.show_find_replace = true;
                true
            }
            // Ctrl+A: Select all
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("a") => {
                self.state.select_all()
            }
            // Ctrl+1: Format cells
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("1") => {
                self.state.format_cells.visible = true;
                self.state.format_cells.active_tab = FormatCellsTab::Number;
                true
            }
            // Ctrl+F: Open find/replace
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("f") => {
                self.state.show_find_replace = true;
                true
            }
            // Ctrl+Z: Undo
            LogicalKey::Character(c)
                if kb.modifiers.control && !kb.modifiers.shift && c.eq_ignore_ascii_case("z") =>
            {
                self.state.undo()
            }
            // Ctrl+Y: Redo
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("y") => {
                self.state.redo()
            }
            // Ctrl+Shift+Z: Redo
            LogicalKey::Character(c)
                if kb.modifiers.control && kb.modifiers.shift && c.eq_ignore_ascii_case("z") =>
            {
                self.state.redo()
            }
            // Ctrl+C: Copy
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("c") => {
                self.state.edit_copy()
            }
            // Ctrl+X: Cut
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("x") => {
                self.state.edit_cut()
            }
            // Ctrl+Shift+V: Paste special
            LogicalKey::Character(c)
                if kb.modifiers.control && kb.modifiers.shift && c.eq_ignore_ascii_case("v") =>
            {
                self.state.show_paste_special = true;
                true
            }
            // Ctrl+V: Paste
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("v") => {
                self.state.edit_paste()
            }
            // Ctrl+W: Close current tab
            LogicalKey::Character(c) if kb.modifiers.control && c.eq_ignore_ascii_case("w") => {
                self.close_current_tab()
            }
            // F11: Toggle full screen
            LogicalKey::Named(NamedKey::F(11)) => {
                self.state.full_screen = !self.state.full_screen;
                true
            }
            // F2: Enter edit mode (append)
            LogicalKey::Named(NamedKey::F(2)) => {
                self.state.begin_edit_append();
                true
            }
            // Escape: Close dialogs / menus
            LogicalKey::Named(NamedKey::Escape) => {
                let mut changed = false;
                if self.state.menu_state.open_menu.is_some() {
                    self.state.menu_state.open_menu = None;
                    changed = true;
                }
                if self.state.context_menu.is_some() {
                    self.state.context_menu = None;
                    changed = true;
                }
                if self.state.show_find_replace {
                    self.state.show_find_replace = false;
                    self.state.find_replace.matches.clear();
                    self.state.find_replace.current_match = None;
                    changed = true;
                }
                if self.state.show_paste_special {
                    self.state.show_paste_special = false;
                    changed = true;
                }
                if self.state.show_named_ranges {
                    self.state.show_named_ranges = false;
                    changed = true;
                }
                if self.state.active_modal.is_some() {
                    self.state.active_modal = None;
                    changed = true;
                }
                if self.state.print_preview.visible {
                    self.state.print_preview.visible = false;
                    changed = true;
                }
                if self.state.show_page_setup {
                    self.state.show_page_setup = false;
                    changed = true;
                }
                if self.state.show_insert_function {
                    self.state.show_insert_function = false;
                    changed = true;
                }
                if self.state.show_sort_dialog {
                    self.state.show_sort_dialog = false;
                    changed = true;
                }
                if self.state.show_settings {
                    self.state.show_settings = false;
                    changed = true;
                }
                if self.state.show_chart_wizard {
                    self.state.show_chart_wizard = false;
                    changed = true;
                }
                if self.state.show_tab_color_picker {
                    self.state.show_tab_color_picker = false;
                    changed = true;
                }
                if self.state.show_move_sheet_dialog {
                    self.state.show_move_sheet_dialog = false;
                    changed = true;
                }
                changed
            }
            // PageDown: move selection down by visible rows
            LogicalKey::Named(NamedKey::PageDown) => {
                let size = ctx.state.size;
                let (_grid_top, grid_bottom, _grid_right) = self.grid_bounds(size);
                let grid_top = DOC_TAB_H
                    + MENU_H
                    + if self.state.show_formula_bar {
                        FORMULA_H
                    } else {
                        0.0
                    }
                    + if self.state.show_toolbar {
                        TOOLBAR_H
                    } else {
                        0.0
                    }
                    + GRID_HEADER_H;
                let grid_height = (grid_bottom - grid_top).max(0.0);
                let visible_rows = if grid_height > 0.0 {
                    (grid_height / GRID_ROW_H).max(1.0) as isize
                } else {
                    1
                };
                self.state
                    .move_selection_with_shift(visible_rows, 0, kb.modifiers.shift)
            }
            // PageUp: move selection up by visible rows
            LogicalKey::Named(NamedKey::PageUp) => {
                let size = ctx.state.size;
                let grid_bottom = size.height - STATUS_H - TAB_H;
                let grid_height = (grid_bottom
                    - (DOC_TAB_H
                        + MENU_H
                        + if self.state.show_formula_bar {
                            FORMULA_H
                        } else {
                            0.0
                        }
                        + if self.state.show_toolbar {
                            TOOLBAR_H
                        } else {
                            0.0
                        }
                        + GRID_HEADER_H))
                    .max(0.0);
                let visible_rows = if grid_height > 0.0 {
                    (grid_height / GRID_ROW_H).max(1.0) as isize
                } else {
                    1
                };
                self.state
                    .move_selection_with_shift(-visible_rows, 0, kb.modifiers.shift)
            }
            // Home: move to column 0 (or Ctrl+Home: jump to origin)
            LogicalKey::Named(NamedKey::Home) if kb.modifiers.control => {
                let target_row = if self.state.freeze_rows > 0 {
                    self.state.freeze_rows
                } else {
                    0
                };
                let target_col = if self.state.freeze_cols > 0 {
                    self.state.freeze_cols
                } else {
                    0
                };
                self.state.select_cell(target_row, target_col)
            }
            LogicalKey::Named(NamedKey::Home) => self.state.move_selection_with_shift(
                0,
                -(self.state.selected_col as isize),
                kb.modifiers.shift,
            ),
            // End: move to last column (or Ctrl+End: jump to last data cell)
            LogicalKey::Named(NamedKey::End) if kb.modifiers.control => {
                let max_row = self.state.grid.len().saturating_sub(1);
                let max_col = self
                    .state
                    .grid
                    .first()
                    .map(|r| r.len().saturating_sub(1))
                    .unwrap_or(0);
                let mut last_row = 0;
                let mut last_col = 0;
                for r in (0..=max_row).rev() {
                    for c in (0..=max_col).rev() {
                        let is_empty = self
                            .state
                            .grid
                            .get(r)
                            .and_then(|row| row.get(c))
                            .map(|cell| cell.value.is_empty())
                            .unwrap_or(true);
                        if !is_empty {
                            last_row = r;
                            last_col = c;
                            break;
                        }
                    }
                    if last_row != 0 || last_col != 0 {
                        break;
                    }
                }
                self.state.select_cell(last_row, last_col)
            }
            LogicalKey::Named(NamedKey::End) => {
                let max_col = self
                    .state
                    .grid
                    .get(self.state.selected_row)
                    .map(|row| row.len().saturating_sub(1))
                    .unwrap_or(0);
                let delta = max_col as isize - self.state.selected_col as isize;
                if delta != 0 {
                    self.state
                        .move_selection_with_shift(0, delta, kb.modifiers.shift)
                } else {
                    false
                }
            }
            // Arrow keys (not editing) — Shift extends range, Ctrl jumps data edge
            LogicalKey::Named(NamedKey::ArrowUp) => {
                if kb.modifiers.control {
                    let (r, c) = self.state.jump_data_edge(
                        self.state.selected_row,
                        self.state.selected_col,
                        -1,
                        0,
                    );
                    if kb.modifiers.shift {
                        self.state.selected_row = r;
                        self.state.selected_col = c;
                        self.state.selection_end = Some((r, c));
                        self.state.select_all_active = false;
                        self.state.recalculate_status();
                        true
                    } else {
                        self.state.select_cell(r, c)
                    }
                } else {
                    self.state
                        .move_selection_with_shift(-1, 0, kb.modifiers.shift)
                }
            }
            LogicalKey::Named(NamedKey::ArrowDown) => {
                if kb.modifiers.control {
                    let (r, c) = self.state.jump_data_edge(
                        self.state.selected_row,
                        self.state.selected_col,
                        1,
                        0,
                    );
                    if kb.modifiers.shift {
                        self.state.selected_row = r;
                        self.state.selected_col = c;
                        self.state.selection_end = Some((r, c));
                        self.state.select_all_active = false;
                        self.state.recalculate_status();
                        true
                    } else {
                        self.state.select_cell(r, c)
                    }
                } else {
                    self.state
                        .move_selection_with_shift(1, 0, kb.modifiers.shift)
                }
            }
            LogicalKey::Named(NamedKey::ArrowLeft) => {
                if kb.modifiers.control {
                    let (r, c) = self.state.jump_data_edge(
                        self.state.selected_row,
                        self.state.selected_col,
                        0,
                        -1,
                    );
                    if kb.modifiers.shift {
                        self.state.selected_row = r;
                        self.state.selected_col = c;
                        self.state.selection_end = Some((r, c));
                        self.state.select_all_active = false;
                        self.state.recalculate_status();
                        true
                    } else {
                        self.state.select_cell(r, c)
                    }
                } else {
                    self.state
                        .move_selection_with_shift(0, -1, kb.modifiers.shift)
                }
            }
            LogicalKey::Named(NamedKey::ArrowRight) => {
                if kb.modifiers.control {
                    let (r, c) = self.state.jump_data_edge(
                        self.state.selected_row,
                        self.state.selected_col,
                        0,
                        1,
                    );
                    if kb.modifiers.shift {
                        self.state.selected_row = r;
                        self.state.selected_col = c;
                        self.state.selection_end = Some((r, c));
                        self.state.select_all_active = false;
                        self.state.recalculate_status();
                        true
                    } else {
                        self.state.select_cell(r, c)
                    }
                } else {
                    self.state
                        .move_selection_with_shift(0, 1, kb.modifiers.shift)
                }
            }
            // Enter: move down
            LogicalKey::Named(NamedKey::Enter) => self.state.move_selection(1, 0),
            // Tab: move right
            LogicalKey::Named(NamedKey::Tab) => self.state.move_selection(0, 1),
            // Backspace: legacy single-char delete with undo (not editing)
            LogicalKey::Named(NamedKey::Backspace) => {
                self.state.delete_from_active_cell_with_undo()
            }
            // Delete: clear cell
            LogicalKey::Named(NamedKey::Delete) => {
                self.state.push_undo();
                if let Some(cell) = self.state.active_cell_mut() {
                    cell.value.clear();
                    cell.is_formula = false;
                }
                self.state.formula_draft.clear();
                self.state.sync_content_from_grid();
                true
            }
            // Regular text input: enter edit mode replacing cell value
            LogicalKey::Character(text) if !text.is_empty() && !kb.modifiers.control => {
                self.state.begin_edit_replace();
                self.state.edit_insert_text(text);
                true
            }
            _ => false,
        };

        if changed {
            ctx.request_paint();
        }
    }
}
