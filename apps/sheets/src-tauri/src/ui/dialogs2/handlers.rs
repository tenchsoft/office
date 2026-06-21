use super::super::*;
use tench_ui::core::events::PointerButtonEvent;

// ---------------------------------------------------------------------------
// Dialog click/key handlers (Phase 4-8)
// ---------------------------------------------------------------------------

impl SheetsApp {
    // -- Phase 4: File dialog click handler --

    pub(crate) fn handle_file_dialog_click(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        let w = 420.0;
        let h = 160.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );
        let x0 = modal.x0 + 16.0;
        let y0 = modal.y0 + 40.0;

        // OK button
        let ok_rect = Rect::new(x0, y0 + 70.0, x0 + 60.0, y0 + 94.0);
        if ok_rect.contains(e.pos) {
            self.state.show_file_dialog = false;
            let path = self.state.file_dialog_path.clone();
            if !path.is_empty() {
                match self.state.file_dialog_mode {
                    FileDialogMode::Open => {
                        self.state.toast = Some((format!("Opened: {path}"), Instant::now()));
                    }
                    FileDialogMode::SaveAs => {
                        self.state.toast = Some((format!("Saved as: {path}"), Instant::now()));
                    }
                    FileDialogMode::ImportCsv => {
                        self.state.toast = Some((format!("Imported CSV: {path}"), Instant::now()));
                    }
                    FileDialogMode::ImportTsv => {
                        self.state.toast = Some((format!("Imported TSV: {path}"), Instant::now()));
                    }
                    FileDialogMode::ImportOds => {
                        self.state.toast = Some((format!("Imported ODS: {path}"), Instant::now()));
                    }
                    FileDialogMode::ExportXlsx => {
                        self.state.toast = Some((format!("Exported XLSX: {path}"), Instant::now()));
                    }
                    FileDialogMode::ExportPdf => {
                        self.state.toast = Some((format!("Exported PDF: {path}"), Instant::now()));
                    }
                    FileDialogMode::ExportHtml => {
                        self.state.toast = Some((format!("Exported HTML: {path}"), Instant::now()));
                    }
                    FileDialogMode::ExportCsv => {
                        self.state.toast = Some((format!("Exported CSV: {path}"), Instant::now()));
                    }
                }
            }
            ctx.request_paint();
            return;
        }

        // Cancel button
        let cancel_rect = Rect::new(x0 + 72.0, y0 + 70.0, x0 + 132.0, y0 + 94.0);
        if cancel_rect.contains(e.pos) {
            self.state.show_file_dialog = false;
            ctx.request_paint();
        }
    }

    // -- Phase 4: File dialog key handler --

    pub(crate) fn handle_file_dialog_key(
        &mut self,
        kb: &tench_ui::core::events::KeyboardEvent,
    ) -> bool {
        use tench_ui::core::events::LogicalKey;
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.show_file_dialog = false;
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                self.state.show_file_dialog = false;
                let path = self.state.file_dialog_path.clone();
                if !path.is_empty() {
                    self.state.toast = Some((format!("File: {path}"), Instant::now()));
                }
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                self.state.file_dialog_path.pop();
                true
            }
            LogicalKey::Character(c) => {
                self.state.file_dialog_path.push_str(c);
                true
            }
            _ => false,
        }
    }

    // -- Phase 5: Toolbar click handler --

    pub(crate) fn handle_toolbar_click(
        &mut self,
        ctx: &mut EventCtx,
        e: &PointerButtonEvent,
        toolbar_y: f64,
    ) {
        let btn_w = 28.0;
        let gap = 2.0;
        let mut x = 8.0;

        // Bold
        if e.pos.x >= x
            && e.pos.x < x + btn_w
            && e.pos.y >= toolbar_y + 2.0
            && e.pos.y < toolbar_y + TOOLBAR_H - 2.0
        {
            self.state.toggle_format_bold();
            ctx.request_paint();
            return;
        }
        x += btn_w + gap;

        // Italic
        if e.pos.x >= x
            && e.pos.x < x + btn_w
            && e.pos.y >= toolbar_y + 2.0
            && e.pos.y < toolbar_y + TOOLBAR_H - 2.0
        {
            self.state.toggle_format_italic();
            ctx.request_paint();
            return;
        }
        x += btn_w + gap;

        // Underline
        if e.pos.x >= x
            && e.pos.x < x + btn_w
            && e.pos.y >= toolbar_y + 2.0
            && e.pos.y < toolbar_y + TOOLBAR_H - 2.0
        {
            self.state.toggle_format_underline();
            ctx.request_paint();
            return;
        }
        x += btn_w + gap;

        // Separator
        x += 8.0;

        // Align Left
        if e.pos.x >= x
            && e.pos.x < x + btn_w
            && e.pos.y >= toolbar_y + 2.0
            && e.pos.y < toolbar_y + TOOLBAR_H - 2.0
        {
            self.state.set_format_h_align(HorizontalAlignment::Left);
            ctx.request_paint();
            return;
        }
        x += btn_w + gap;

        // Align Center
        if e.pos.x >= x
            && e.pos.x < x + btn_w
            && e.pos.y >= toolbar_y + 2.0
            && e.pos.y < toolbar_y + TOOLBAR_H - 2.0
        {
            self.state.set_format_h_align(HorizontalAlignment::Center);
            ctx.request_paint();
            return;
        }
        x += btn_w + gap;

        // Align Right
        if e.pos.x >= x
            && e.pos.x < x + btn_w
            && e.pos.y >= toolbar_y + 2.0
            && e.pos.y < toolbar_y + TOOLBAR_H - 2.0
        {
            self.state.set_format_h_align(HorizontalAlignment::Right);
            ctx.request_paint();
            return;
        }
        x += btn_w + gap;

        // Separator
        x += 8.0;

        // Number format: General (60px wide)
        let nf_w = 70.0;
        if e.pos.x >= x
            && e.pos.x < x + nf_w
            && e.pos.y >= toolbar_y + 2.0
            && e.pos.y < toolbar_y + TOOLBAR_H - 2.0
        {
            // Cycle number format
            self.state.cycle_number_format();
            ctx.request_paint();
            return;
        }
        x += nf_w + gap;

        // Separator
        x += 8.0;

        // Format Painter
        if e.pos.x >= x
            && e.pos.x < x + btn_w + 20.0
            && e.pos.y >= toolbar_y + 2.0
            && e.pos.y < toolbar_y + TOOLBAR_H - 2.0
        {
            self.state.format_painter_active = !self.state.format_painter_active;
            if self.state.format_painter_active {
                self.state.format_painter_source = Some(self.state.get_selected_cell_format());
            } else {
                self.state.format_painter_source = None;
            }
            ctx.request_paint();
            return;
        }
        x += btn_w + 20.0 + gap;

        // Merge cells
        if e.pos.x >= x
            && e.pos.x < x + btn_w + 20.0
            && e.pos.y >= toolbar_y + 2.0
            && e.pos.y < toolbar_y + TOOLBAR_H - 2.0
        {
            self.state.toggle_merge_selection();
            ctx.request_paint();
        }
    }

    // -- Phase 5: Format Cells dialog click handler --

    pub(crate) fn handle_format_cells_click(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        let w = 400.0;
        let h = 380.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );
        let x0 = modal.x0 + 16.0;
        let y0 = modal.y0 + 40.0;

        // Tab buttons (Number, Alignment, Font, Border, Fill)
        let tab_w = 70.0;
        for i in 0..5usize {
            let tab_x = x0 + i as f64 * (tab_w + 4.0);
            let tab_rect = Rect::new(tab_x, y0, tab_x + tab_w, y0 + 24.0);
            if tab_rect.contains(e.pos) {
                self.state.format_cells.active_tab = match i {
                    0 => FormatCellsTab::Number,
                    1 => FormatCellsTab::Alignment,
                    2 => FormatCellsTab::Font,
                    3 => FormatCellsTab::Border,
                    4 => FormatCellsTab::Fill,
                    _ => FormatCellsTab::Number,
                };
                ctx.request_paint();
                return;
            }
        }

        // Content area y start
        let content_y = y0 + 36.0;

        match self.state.format_cells.active_tab {
            FormatCellsTab::Number => {
                // Number tab - format options
                let formats = [
                    "General",
                    "Number",
                    "Currency",
                    "Percentage",
                    "Date",
                    "Text",
                ];
                for (i, _label) in formats.iter().enumerate() {
                    let row_y = content_y + i as f64 * 26.0;
                    let item_rect = Rect::new(x0, row_y, x0 + 160.0, row_y + 22.0);
                    if item_rect.contains(e.pos) {
                        match i {
                            0 => self.state.set_number_format(NumberFormat::General),
                            1 => self.state.set_number_format(NumberFormat::Number {
                                decimals: 2,
                                thousands_sep: true,
                            }),
                            2 => self.state.set_number_format(NumberFormat::Currency {
                                symbol: "$".into(),
                                decimals: 2,
                            }),
                            3 => self
                                .state
                                .set_number_format(NumberFormat::Percentage { decimals: 0 }),
                            4 => self.state.set_number_format(NumberFormat::Date),
                            5 => self.state.set_number_format(NumberFormat::Text),
                            _ => false,
                        };
                        ctx.request_paint();
                        return;
                    }
                }
            }
            FormatCellsTab::Alignment => {
                // Alignment tab - horizontal alignment buttons
                let labels = ["Left", "Center", "Right"];
                for (i, _label) in labels.iter().enumerate() {
                    let row_y = content_y + i as f64 * 26.0;
                    let item_rect = Rect::new(x0, row_y, x0 + 100.0, row_y + 22.0);
                    if item_rect.contains(e.pos) {
                        match i {
                            0 => self.state.set_format_h_align(HorizontalAlignment::Left),
                            1 => self.state.set_format_h_align(HorizontalAlignment::Center),
                            2 => self.state.set_format_h_align(HorizontalAlignment::Right),
                            _ => false,
                        };
                        ctx.request_paint();
                        return;
                    }
                }
            }
            FormatCellsTab::Font => {
                // Font tab - Bold, Italic, Underline toggles
                let toggles = ["Bold", "Italic", "Underline"];
                for (i, _label) in toggles.iter().enumerate() {
                    let row_y = content_y + i as f64 * 26.0;
                    let item_rect = Rect::new(x0, row_y, x0 + 100.0, row_y + 22.0);
                    if item_rect.contains(e.pos) {
                        match i {
                            0 => self.state.toggle_format_bold(),
                            1 => self.state.toggle_format_italic(),
                            2 => self.state.toggle_format_underline(),
                            _ => false,
                        };
                        ctx.request_paint();
                        return;
                    }
                }
            }
            FormatCellsTab::Border => {
                // Border tab - no interactive elements yet
            }
            FormatCellsTab::Fill => {
                // Fill tab - color presets
                let colors = [
                    ("Yellow", Color::rgb8(255, 255, 0)),
                    ("Light Green", Color::rgb8(144, 238, 144)),
                    ("Light Blue", Color::rgb8(173, 216, 230)),
                    ("Light Red", Color::rgb8(255, 182, 193)),
                    ("Light Gray", Color::rgb8(211, 211, 211)),
                    ("No Fill", Color::TRANSPARENT),
                ];
                for (i, (_label, _color)) in colors.iter().enumerate() {
                    let row_y = content_y + i as f64 * 26.0;
                    let item_rect = Rect::new(x0, row_y, x0 + 100.0, row_y + 22.0);
                    if item_rect.contains(e.pos) {
                        self.state.set_format_bg_color(*_color);
                        ctx.request_paint();
                        return;
                    }
                }
            }
        }

        // OK button
        let ok_rect = Rect::new(x0, modal.y1 - 44.0, x0 + 60.0, modal.y1 - 20.0);
        if ok_rect.contains(e.pos) {
            self.state.format_cells.visible = false;
            ctx.request_paint();
            return;
        }

        // Cancel button
        let cancel_rect = Rect::new(x0 + 72.0, modal.y1 - 44.0, x0 + 132.0, modal.y1 - 20.0);
        if cancel_rect.contains(e.pos) {
            self.state.format_cells.visible = false;
            ctx.request_paint();
        }
    }

    // -- Phase 5: Format Cells dialog key handler --

    pub(crate) fn handle_format_cells_key(
        &mut self,
        kb: &tench_ui::core::events::KeyboardEvent,
    ) -> bool {
        use tench_ui::core::events::LogicalKey;
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.format_cells.visible = false;
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                self.state.format_cells.visible = false;
                true
            }
            _ => false,
        }
    }

    // -- Phase 5: Conditional format dialog click handler --

    pub(crate) fn handle_cond_format_click(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        let w = 380.0;
        let h = 300.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );
        let x0 = modal.x0 + 16.0;
        let y0 = modal.y0 + 40.0;

        // Condition operator buttons
        let ops = ["Greater Than", "Less Than", "Equal To", "Between"];
        for (i, _label) in ops.iter().enumerate() {
            let row_y = y0 + i as f64 * 26.0;
            let item_rect = Rect::new(x0, row_y, x0 + 140.0, row_y + 22.0);
            if item_rect.contains(e.pos) {
                self.state.conditional_format_dialog.condition = match i {
                    0 => ConditionOp::GreaterThan,
                    1 => ConditionOp::LessThan,
                    2 => ConditionOp::EqualTo,
                    3 => ConditionOp::Between,
                    _ => ConditionOp::GreaterThan,
                };
                ctx.request_paint();
                return;
            }
        }

        // Color presets for background
        let colors = [
            ("Red BG", Color::rgb8(255, 200, 200)),
            ("Green BG", Color::rgb8(200, 255, 200)),
            ("Blue BG", Color::rgb8(200, 200, 255)),
            ("Yellow BG", Color::rgb8(255, 255, 200)),
        ];
        for (i, (_label, color)) in colors.iter().enumerate() {
            let row_y = y0 + 120.0 + i as f64 * 26.0;
            let item_rect = Rect::new(x0, row_y, x0 + 100.0, row_y + 22.0);
            if item_rect.contains(e.pos) {
                self.state.conditional_format_dialog.bg_color = Some(*color);
                ctx.request_paint();
                return;
            }
        }

        // OK button
        let ok_rect = Rect::new(x0, modal.y1 - 44.0, x0 + 60.0, modal.y1 - 20.0);
        if ok_rect.contains(e.pos) {
            // Add the conditional format rule
            let value = self
                .state
                .conditional_format_dialog
                .value_text
                .parse::<f64>()
                .unwrap_or(0.0);
            let value2 = if self.state.conditional_format_dialog.condition == ConditionOp::Between {
                Some(
                    self.state
                        .conditional_format_dialog
                        .value2_text
                        .parse::<f64>()
                        .unwrap_or(0.0),
                )
            } else {
                None
            };
            let row = self.state.selected_row;
            let col = self.state.selected_col;
            let rule = state::ConditionalFormatRule {
                col,
                row_range: (row, row),
                condition: self.state.conditional_format_dialog.condition,
                value,
                value2,
                bg_color: self
                    .state
                    .conditional_format_dialog
                    .bg_color
                    .unwrap_or_else(|| Color::rgb8(255, 255, 255)),
                text_color: self
                    .state
                    .conditional_format_dialog
                    .text_color
                    .unwrap_or_else(|| Color::rgb8(0, 0, 0)),
            };
            self.state.conditional_formats.push(rule);
            self.state.conditional_format_dialog.visible = false;
            self.state.toast = Some(("Conditional format applied".into(), Instant::now()));
            ctx.request_paint();
            return;
        }

        // Cancel button
        let cancel_rect = Rect::new(x0 + 72.0, modal.y1 - 44.0, x0 + 132.0, modal.y1 - 20.0);
        if cancel_rect.contains(e.pos) {
            self.state.conditional_format_dialog.visible = false;
            ctx.request_paint();
        }
    }

    // -- Phase 5: Conditional format dialog key handler --

    pub(crate) fn handle_cond_format_key(
        &mut self,
        kb: &tench_ui::core::events::KeyboardEvent,
    ) -> bool {
        use tench_ui::core::events::LogicalKey;
        match &kb.logical_key {
            LogicalKey::Named(NamedKey::Escape) => {
                self.state.conditional_format_dialog.visible = false;
                true
            }
            LogicalKey::Named(NamedKey::Enter) => {
                self.state.conditional_format_dialog.visible = false;
                true
            }
            LogicalKey::Named(NamedKey::Backspace) => {
                self.state.conditional_format_dialog.value_text.pop();
                true
            }
            LogicalKey::Character(c) => {
                self.state.conditional_format_dialog.value_text.push_str(c);
                true
            }
            _ => false,
        }
    }

    // -- Phase 6: Data Validation dialog click handler --

    pub(crate) fn handle_data_validation_click(
        &mut self,
        ctx: &mut EventCtx,
        e: &PointerButtonEvent,
    ) {
        let size = ctx.state.size;
        let w = 400.0;
        let h = 340.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );

        // Click outside closes
        if !modal.contains(e.pos) {
            self.state.data_validation_dialog.visible = false;
            ctx.request_paint();
            return;
        }

        let x0 = modal.x0 + 16.0;
        let mut y = modal.y0 + 40.0;

        // Validation type dropdown area — cycle through types
        let type_rect = Rect::new(x0, y, modal.x1 - 16.0, y + 22.0);
        if type_rect.contains(e.pos) {
            let types = state::DataValidationType::ALL;
            let current = self.state.data_validation_dialog.draft.validation_type;
            let idx = types.iter().position(|t| *t == current).unwrap_or(0);
            let next = types[(idx + 1) % types.len()];
            self.state.data_validation_dialog.draft.validation_type = next;
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // Operator dropdown area — cycle through operators
        let op_rect = Rect::new(x0, y, modal.x1 - 16.0, y + 22.0);
        if op_rect.contains(e.pos) {
            let ops = state::DataValidationOperator::ALL;
            let current = self.state.data_validation_dialog.draft.operator;
            let idx = ops.iter().position(|o| *o == current).unwrap_or(0);
            let next = ops[(idx + 1) % ops.len()];
            self.state.data_validation_dialog.draft.operator = next;
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // Value1 field click
        let v1_rect = Rect::new(x0, y, modal.x1 - 16.0, y + 22.0);
        if v1_rect.contains(e.pos) {
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // Value2 field click
        let v2_rect = Rect::new(x0, y, modal.x1 - 16.0, y + 22.0);
        if v2_rect.contains(e.pos) {
            ctx.request_paint();
            return;
        }
        y += 30.0;

        // Error message field click
        let err_rect = Rect::new(x0, y, modal.x1 - 16.0, y + 22.0);
        if err_rect.contains(e.pos) {
            ctx.request_paint();
            return;
        }
        y += 40.0;

        // OK button
        let ok_rect = Rect::new(x0, y, x0 + 60.0, y + 24.0);
        if ok_rect.contains(e.pos) {
            let mut rule = self.state.data_validation_dialog.draft.clone();
            rule.range = state::CellRange {
                start_row: self.state.selection_anchor.0,
                start_col: self.state.selection_anchor.1,
                end_row: self
                    .state
                    .selection_end
                    .map(|(r, _)| r)
                    .unwrap_or(self.state.selected_row),
                end_col: self
                    .state
                    .selection_end
                    .map(|(_, c)| c)
                    .unwrap_or(self.state.selected_col),
            };
            self.state.add_data_validation_rule(rule);
            self.state.data_validation_dialog.visible = false;
            self.state.toast = Some(("Data validation rule added".into(), Instant::now()));
            ctx.request_paint();
            return;
        }

        // Cancel button
        let cancel_rect = Rect::new(x0 + 72.0, y, x0 + 132.0, y + 24.0);
        if cancel_rect.contains(e.pos) {
            self.state.data_validation_dialog.visible = false;
            ctx.request_paint();
        }
    }

    // -- Phase 6: Data Validation dialog key handler --

    pub(crate) fn handle_data_validation_key(
        &mut self,
        kb: &tench_ui::core::events::KeyboardEvent,
    ) -> bool {
        use tench_ui::core::events::{LogicalKey, NamedKey};

        if matches!(&kb.logical_key, LogicalKey::Named(NamedKey::Escape)) {
            self.state.data_validation_dialog.visible = false;
            return true;
        }

        if matches!(&kb.logical_key, LogicalKey::Named(NamedKey::Tab)) {
            // Cycle validation type on Tab (simplified)
            let types = state::DataValidationType::ALL;
            let current = self.state.data_validation_dialog.draft.validation_type;
            let idx = types.iter().position(|t| *t == current).unwrap_or(0);
            let next = types[(idx + 1) % types.len()];
            self.state.data_validation_dialog.draft.validation_type = next;
            return true;
        }

        false
    }

    // -- Phase 6: Pivot Table dialog click handler --

    pub(crate) fn handle_pivot_table_click(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        let w = 360.0;
        let h = 200.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );

        // Click outside closes
        if !modal.contains(e.pos) {
            self.state.show_pivot_table = false;
            ctx.request_paint();
            return;
        }

        let x0 = modal.x0 + 16.0;
        let ok_rect = Rect::new(x0, modal.y1 - 44.0, x0 + 60.0, modal.y1 - 20.0);
        if ok_rect.contains(e.pos) {
            self.state.show_pivot_table = false;
            ctx.request_paint();
        }
    }

    // -- Phase 6: Filter dropdown action handler --

    pub(crate) fn handle_filter_dropdown_action(
        &mut self,
        ctx: &mut EventCtx,
        action: FilterDropdownAction,
    ) {
        let filter_col = match self.state.filter_dropdown_col {
            Some(c) => c,
            None => return,
        };

        match action {
            FilterDropdownAction::ToggleSelectAll => {
                let all_vals = self.state.unique_values_for_col(filter_col);
                if self.state.filter_values.len() >= all_vals.len() {
                    self.state.filter_values.clear();
                } else {
                    self.state.filter_values = all_vals;
                }
            }
            FilterDropdownAction::ToggleItem(idx) => {
                let all_vals = self.state.unique_values_for_col(filter_col);
                if let Some(val) = all_vals.get(idx).cloned() {
                    if let Some(pos) = self.state.filter_values.iter().position(|v| *v == val) {
                        self.state.filter_values.remove(pos);
                    } else {
                        self.state.filter_values.push(val);
                    }
                }
            }
            FilterDropdownAction::Apply => {
                if self.state.filter_values.is_empty() {
                    self.state.clear_filter();
                } else {
                    self.state
                        .apply_filter(filter_col, &self.state.filter_values.clone());
                }
                self.state.show_filter_dropdown = false;
                self.state.toast = Some(if self.state.filter_hidden_rows.is_empty() {
                    ("Filter applied - all rows visible".into(), Instant::now())
                } else {
                    (
                        format!(
                            "Filter applied - {} rows hidden",
                            self.state.filter_hidden_rows.len()
                        ),
                        Instant::now(),
                    )
                });
            }
            FilterDropdownAction::Cancel => {
                self.state.show_filter_dropdown = false;
            }
        }
        ctx.request_paint();
    }
}
