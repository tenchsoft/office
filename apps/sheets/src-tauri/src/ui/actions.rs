use super::*;

impl SheetsApp {
    pub(crate) fn execute_menu_action(&mut self, action: MenuAction) -> bool {
        match action {
            MenuAction::NewWorkbook => {
                let opened = workbook_service::create_workbook(None);
                let name = opened.artifact.title.clone();
                let session_id = format!(
                    "session-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                );
                self.state.doc_tabs.push(state::DocumentTab {
                    session_id,
                    title: name,
                    dirty: false,
                });
                self.state.active_tab_idx = self.state.doc_tabs.len() - 1;
                self.state.toast = Some(("New workbook created".into(), Instant::now()));
                true
            }
            MenuAction::OpenFile => {
                self.state.file_dialog_mode = FileDialogMode::Open;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            MenuAction::Save => {
                self.save_current_workbook();
                true
            }
            MenuAction::SaveAs => {
                self.state.file_dialog_mode = FileDialogMode::SaveAs;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            MenuAction::ImportCsv => {
                self.state.file_dialog_mode = FileDialogMode::ImportCsv;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            MenuAction::ImportTsv => {
                self.state.file_dialog_mode = FileDialogMode::ImportTsv;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            MenuAction::ImportOds => {
                self.state.file_dialog_mode = FileDialogMode::ImportOds;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            MenuAction::ExportXlsx => {
                self.state.file_dialog_mode = FileDialogMode::ExportXlsx;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            MenuAction::ExportPdf => {
                self.state.file_dialog_mode = FileDialogMode::ExportPdf;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            MenuAction::ExportHtml => {
                self.state.file_dialog_mode = FileDialogMode::ExportHtml;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            MenuAction::ExportCsv => {
                self.state.file_dialog_mode = FileDialogMode::ExportCsv;
                self.state.file_dialog_path.clear();
                self.state.show_file_dialog = true;
                true
            }
            MenuAction::Print => {
                self.state.compute_print_pages();
                self.state.print_preview.visible = true;
                true
            }
            MenuAction::PrintPreview => {
                self.state.compute_print_pages();
                self.state.print_preview.visible = true;
                true
            }
            MenuAction::PageSetup => {
                self.state.show_page_setup = true;
                true
            }
            MenuAction::Close => self.close_current_tab(),
            MenuAction::Undo => self.state.undo(),
            MenuAction::Redo => self.state.redo(),
            MenuAction::Cut => self.state.edit_cut(),
            MenuAction::Copy => self.state.edit_copy(),
            MenuAction::Paste => self.state.edit_paste(),
            MenuAction::PasteSpecial => {
                self.state.show_paste_special = true;
                true
            }
            MenuAction::Delete => self.state.delete_from_active_cell_with_undo(),
            MenuAction::SelectAll => self.state.select_all(),
            MenuAction::Find => {
                self.state.show_find_replace = true;
                true
            }
            MenuAction::Replace => {
                self.state.show_find_replace = true;
                true
            }
            MenuAction::ToggleFormulaBar => {
                self.state.show_formula_bar = !self.state.show_formula_bar;
                true
            }
            MenuAction::ToggleGridLines => {
                self.state.show_grid_lines = !self.state.show_grid_lines;
                true
            }
            MenuAction::ToggleHeaders => {
                self.state.show_headers = !self.state.show_headers;
                true
            }
            MenuAction::FreezePanes => self.state.freeze_at_cursor(),
            MenuAction::Zoom75 => self.state.set_zoom(75),
            MenuAction::Zoom100 => self.state.set_zoom(100),
            MenuAction::Zoom125 => self.state.set_zoom(125),
            MenuAction::Zoom150 => self.state.set_zoom(150),
            MenuAction::Zoom200 => self.state.set_zoom(200),
            MenuAction::ToggleFullScreen => {
                self.state.full_screen = !self.state.full_screen;
                true
            }
            MenuAction::InsertRowAbove => self.state.insert_row(self.state.selected_row),
            MenuAction::InsertRowBelow => self.state.insert_row(self.state.selected_row + 1),
            MenuAction::InsertColLeft => self.state.insert_col(self.state.selected_col),
            MenuAction::InsertColRight => self.state.insert_col(self.state.selected_col + 1),
            MenuAction::InsertSheet => self.state.add_sheet(),
            MenuAction::InsertChart => {
                self.state.open_chart_wizard();
                true
            }
            MenuAction::InsertFunction => {
                self.state.show_insert_function = true;
                self.state.insert_function_selected = 0;
                self.state.insert_function_scroll = 0;
                true
            }
            MenuAction::DefineName => {
                self.state.show_named_ranges = true;
                true
            }
            MenuAction::FormatCells => {
                self.state.format_cells.visible = true;
                self.state.format_cells.active_tab = FormatCellsTab::Number;
                true
            }
            MenuAction::RowHeight => {
                self.state.toast = Some(("Row height dialog".into(), Instant::now()));
                true
            }
            MenuAction::ColWidth => {
                self.state.toast = Some(("Column width dialog".into(), Instant::now()));
                true
            }
            MenuAction::RenameSheet => {
                self.state.toast = Some(("Rename sheet dialog".into(), Instant::now()));
                true
            }
            MenuAction::ConditionalFormat => {
                self.state.conditional_format_dialog.visible = true;
                true
            }
            MenuAction::Sort => {
                self.state.show_sort_dialog = true;
                true
            }
            MenuAction::FilterToggle => {
                self.state.toggle_filter();
                self.state.toast = Some(if self.state.filter_active {
                    ("Filter enabled".into(), Instant::now())
                } else {
                    ("Filter disabled".into(), Instant::now())
                });
                true
            }
            MenuAction::DataValidation => {
                self.state.data_validation_dialog.visible = true;
                true
            }
            MenuAction::PivotTable => {
                self.state.show_pivot_table = true;
                true
            }
            MenuAction::Settings => {
                self.state.show_settings = true;
                true
            }
            MenuAction::About => {
                self.state.open_modal(state::ModalType::About);
                true
            }
            MenuAction::Shortcuts => {
                self.state.open_modal(state::ModalType::Shortcuts);
                true
            }
            MenuAction::ActivateLicense => {
                self.state.license_modal = Some(state::LicenseModalState::default());
                true
            }
            MenuAction::GeneratePcCode => {
                if let Some(store) = &self.license_store {
                    let lic_state = store.state();
                    let meta = serde_json::json!({
                        "os": std::env::consts::OS,
                        "hostname": std::env::var("HOSTNAME")
                            .or_else(|_| std::env::var("COMPUTERNAME"))
                            .unwrap_or_else(|_| "unknown".into()),
                        "tench_app": "sheets",
                        "tench_ver": env!("CARGO_PKG_VERSION"),
                    });
                    match tench_license_store::encode_pc_request_code(&lic_state.device_id, meta) {
                        Ok(code) => self.state.toast = Some((code, Instant::now())),
                        Err(_) => {
                            self.state.toast = Some(("Failed to generate PC code".into(), Instant::now()))
                        }
                    }
                } else {
                    self.state.toast = Some(("License store unavailable".into(), Instant::now()));
                }
                true
            }
            MenuAction::ReleaseDevice => {
                if let Some(store) = &self.license_store {
                    match tench_update_client::release_license(store) {
                        Ok(()) => self.state.toast = Some(("Device released".into(), Instant::now())),
                        Err(e) => {
                            self.state.toast = Some((format!("Release failed: {e}"), Instant::now()))
                        }
                    }
                } else {
                    self.state.toast = Some(("License store unavailable".into(), Instant::now()));
                }
                true
            }
            MenuAction::None => false,
        }
    }

    pub(crate) fn execute_context_menu_action(
        &mut self,
        action: &str,
        target: &ContextMenuTarget,
    ) -> bool {
        match action {
            "Cut" => self.state.edit_cut(),
            "Copy" => self.state.edit_copy(),
            "Paste" => self.state.edit_paste(),
            "Format Cells" => {
                self.state.format_cells.visible = true;
                self.state.format_cells.active_tab = FormatCellsTab::Number;
                true
            }
            "Insert Row Above" => self.state.insert_row(self.state.selected_row),
            "Insert Row Below" => self.state.insert_row(self.state.selected_row + 1),
            "Insert Column Left" => self.state.insert_col(self.state.selected_col),
            "Insert Column Right" => self.state.insert_col(self.state.selected_col + 1),
            "Delete Row" => self.state.delete_row(self.state.selected_row),
            "Delete Column" => self.state.delete_col(self.state.selected_col),
            "Sort Ascending" => {
                self.state.sort_grid_by_col(
                    self.state.selected_col,
                    true,
                    self.state.sort_has_header,
                );
                self.state.toast = Some(("Sorted ascending".into(), Instant::now()));
                true
            }
            "Sort Descending" => {
                self.state.sort_grid_by_col(
                    self.state.selected_col,
                    false,
                    self.state.sort_has_header,
                );
                self.state.toast = Some(("Sorted descending".into(), Instant::now()));
                true
            }
            "Set Print Area" => {
                let (sr, sc, er, ec) = self.state.selection_range();
                self.state.page_setup.print_area = Some(state::CellRange {
                    start_row: sr,
                    start_col: sc,
                    end_row: er,
                    end_col: ec,
                });
                self.state.toast = Some(("Print area set".into(), Instant::now()));
                true
            }
            "Hide Row" => {
                match target {
                    ContextMenuTarget::RowHeader { row } => {
                        self.state.hide_row(*row);
                        self.state.toast =
                            Some((format!("Row {} hidden", row + 1), Instant::now()));
                    }
                    ContextMenuTarget::Cell { row, .. } => {
                        self.state.hide_row(*row);
                        self.state.toast =
                            Some((format!("Row {} hidden", row + 1), Instant::now()));
                    }
                    _ => {}
                }
                true
            }
            "Row Height" => {
                self.state.toast = Some(("Row height dialog".into(), Instant::now()));
                true
            }
            "Hide Column" => {
                match target {
                    ContextMenuTarget::ColHeader { col } => {
                        self.state.hide_col(*col);
                        self.state.toast = Some((
                            format!("Column {} hidden", state::col_letter(*col)),
                            Instant::now(),
                        ));
                    }
                    ContextMenuTarget::Cell { col, .. } => {
                        self.state.hide_col(*col);
                        self.state.toast = Some((
                            format!("Column {} hidden", state::col_letter(*col)),
                            Instant::now(),
                        ));
                    }
                    _ => {}
                }
                true
            }
            "Column Width" => {
                self.state.toast = Some(("Column width dialog".into(), Instant::now()));
                true
            }
            "Rename" => {
                if let ContextMenuTarget::SheetTab { sheet_idx } = target {
                    self.state.renaming_sheet = Some(*sheet_idx);
                    self.state.rename_draft = self
                        .state
                        .sheet_names
                        .get(*sheet_idx)
                        .cloned()
                        .unwrap_or_default();
                    return true;
                }
                false
            }
            "Duplicate" => {
                if let ContextMenuTarget::SheetTab { sheet_idx } = target {
                    return self.state.duplicate_sheet(*sheet_idx);
                }
                false
            }
            "Move" => {
                if let ContextMenuTarget::SheetTab { sheet_idx } = target {
                    self.state.move_sheet_target = *sheet_idx;
                    self.state.show_move_sheet_dialog = true;
                    return true;
                }
                false
            }
            "Delete" => {
                if let ContextMenuTarget::SheetTab { sheet_idx } = target {
                    if self.state.sheet_names.len() > 1 {
                        self.state.sheet_names.remove(*sheet_idx);
                        if self.state.active_sheet >= self.state.sheet_names.len() {
                            self.state.active_sheet = self.state.sheet_names.len() - 1;
                        }
                        return true;
                    }
                }
                false
            }
            "Tab Color" => {
                if let ContextMenuTarget::SheetTab { sheet_idx } = target {
                    self.state.tab_color_target = Some(*sheet_idx);
                    self.state.show_tab_color_picker = true;
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    /// Execute fill handle auto-fill from source to target.
    pub(crate) fn execute_fill_handle(
        &mut self,
        source_row: usize,
        source_col: usize,
        target_row: usize,
        target_col: usize,
    ) {
        if source_row == target_row && source_col == target_col {
            return;
        }
        self.state.push_undo();

        // Get source value
        let source_value = self
            .state
            .grid
            .get(source_row)
            .and_then(|r| r.get(source_col))
            .map(|c| c.value.clone())
            .unwrap_or_default();

        // Fill in the direction of the drag
        if target_col > source_col {
            // Fill right
            for c in (source_col + 1)..=target_col {
                let delta = (c - source_col) as i64;
                let filled = self.auto_fill_value(&source_value, delta);
                if let Some(row) = self.state.grid.get_mut(source_row) {
                    if let Some(cell) = row.get_mut(c) {
                        cell.value = filled;
                        cell.is_formula = cell.value.starts_with('=');
                    }
                }
            }
        } else if target_col < source_col {
            // Fill left
            for c in (target_col..source_col).rev() {
                let delta = -((source_col - c) as i64);
                let filled = self.auto_fill_value(&source_value, delta);
                if let Some(row) = self.state.grid.get_mut(source_row) {
                    if let Some(cell) = row.get_mut(c) {
                        cell.value = filled;
                        cell.is_formula = cell.value.starts_with('=');
                    }
                }
            }
        }

        if target_row > source_row {
            // Fill down
            for r in (source_row + 1)..=target_row {
                let delta = (r - source_row) as i64;
                let filled = self.auto_fill_value(&source_value, delta);
                if let Some(row) = self.state.grid.get_mut(r) {
                    if let Some(cell) = row.get_mut(source_col) {
                        cell.value = filled;
                        cell.is_formula = cell.value.starts_with('=');
                    }
                }
            }
        } else if target_row < source_row {
            // Fill up
            for r in (target_row..source_row).rev() {
                let delta = -((source_row - r) as i64);
                let filled = self.auto_fill_value(&source_value, delta);
                if let Some(row) = self.state.grid.get_mut(r) {
                    if let Some(cell) = row.get_mut(source_col) {
                        cell.value = filled;
                        cell.is_formula = cell.value.starts_with('=');
                    }
                }
            }
        }

        self.state.sync_content_from_grid();
    }

    /// Compute auto-fill value based on pattern detection.
    pub(crate) fn auto_fill_value(&self, source: &str, delta: i64) -> String {
        // Try numeric pattern
        if let Ok(num) = source.parse::<f64>() {
            // Increment by 1 for each step
            let result = num + delta as f64;
            if result.fract().abs() < f64::EPSILON {
                format!("{}", result as i64)
            } else {
                format!("{result:.2}")
            }
        } else {
            // Text: just copy
            source.to_string()
        }
    }

    pub(crate) fn close_current_tab(&mut self) -> bool {
        if self.state.doc_tabs.len() <= 1 {
            return false;
        }
        let idx = self.state.active_tab_idx;
        self.state.doc_tabs.remove(idx);
        self.state.active_tab_idx = if idx >= self.state.doc_tabs.len() {
            self.state.doc_tabs.len() - 1
        } else {
            idx
        };
        true
    }

    pub(crate) fn close_tab(&mut self, idx: usize) -> bool {
        if self.state.doc_tabs.len() <= 1 {
            return false;
        }
        self.state.doc_tabs.remove(idx);
        if self.state.active_tab_idx >= self.state.doc_tabs.len() {
            self.state.active_tab_idx = self.state.doc_tabs.len() - 1;
        } else if idx < self.state.active_tab_idx {
            self.state.active_tab_idx -= 1;
        }
        true
    }

    pub(crate) fn save_current_workbook(&mut self) {
        let artifact = self.state.current_artifact().clone();
        let content = self.state.current_content().clone();

        if artifact.path.is_some() {
            match workbook_service::save_workbook(artifact, content, None, None) {
                Ok(saved) => {
                    self.state.auto_save.mark_saved();
                    self.state.apply_saved_artifact(saved.artifact);
                }
                Err(error) => {
                    self.state.toast = Some(("Save failed".into(), Instant::now()));
                    self.state.set_toast(format!("Save failed: {error}"));
                }
            }
            return;
        }

        match workbook_service::save_recovery_snapshot(artifact, content) {
            Ok(_) => {
                self.state.auto_save.mark_saved();
                self.state.toast = Some(("Recovery snapshot saved".into(), Instant::now()));
            }
            Err(error) => {
                self.state.toast = Some(("Save failed".into(), Instant::now()));
                self.state.set_toast(format!("Recovery failed: {error}"));
            }
        }
    }

    pub(crate) fn auto_save(&mut self) {
        let artifact = self.state.current_artifact().clone();
        let content = self.state.current_content().clone();

        if artifact.path.is_some() {
            if let Ok(saved) = workbook_service::save_workbook(artifact, content, None, None) {
                self.state.apply_saved_artifact(saved.artifact);
            }
        } else if workbook_service::save_recovery_snapshot(artifact, content).is_ok() {
            self.state.toast = Some(("Auto-saved".into(), Instant::now()));
        }
        self.state.auto_save.mark_saved();
    }
}
