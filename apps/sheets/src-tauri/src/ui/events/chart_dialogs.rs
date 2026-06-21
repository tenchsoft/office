use super::super::charts::{hit_chart_wizard, ChartWizardAction};
use super::super::dialogs::TAB_COLOR_PRESETS;
use super::super::*;
use tench_ui::core::events::PointerButtonEvent;

impl SheetsApp {
    // -- Phase 7: Chart wizard click handler --

    pub(crate) fn handle_chart_wizard_click(&mut self, ctx: &mut EventCtx, e: &PointerButtonEvent) {
        let size = ctx.state.size;
        if let Some(action) = hit_chart_wizard(&self.state, e.pos.x, e.pos.y, size) {
            match action {
                ChartWizardAction::Next => {
                    if self.state.chart_wizard_step < 2 {
                        self.state.chart_wizard_step += 1;
                    } else {
                        self.state.create_chart_from_wizard();
                    }
                }
                ChartWizardAction::Back => {
                    if self.state.chart_wizard_step > 0 {
                        self.state.chart_wizard_step -= 1;
                    }
                }
                ChartWizardAction::Cancel => {
                    self.state.show_chart_wizard = false;
                }
                ChartWizardAction::FocusDataRange => {
                    // Focus the data range field (handled by keyboard routing)
                }
                ChartWizardAction::FocusTitle => {
                    // Focus the title field (handled by keyboard routing)
                }
                ChartWizardAction::SelectChartType(ct) => {
                    self.state.chart_wizard_chart_type = ct;
                }
                ChartWizardAction::ToggleLegend => {
                    self.state.chart_wizard_show_legend = !self.state.chart_wizard_show_legend;
                }
                ChartWizardAction::ToggleAxisLabels => {
                    self.state.chart_wizard_show_axis_labels =
                        !self.state.chart_wizard_show_axis_labels;
                }
            }
            ctx.request_paint();
        }
    }

    // -- Phase 8: Tab color picker click handler --

    pub(crate) fn handle_tab_color_picker_click(
        &mut self,
        ctx: &mut EventCtx,
        e: &PointerButtonEvent,
    ) {
        let size = ctx.state.size;
        let w = 280.0;
        let h = 260.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );

        // Click outside closes
        if !modal.contains(e.pos) {
            self.state.show_tab_color_picker = false;
            ctx.request_paint();
            return;
        }

        let x0 = modal.x0 + 16.0;
        let y0 = modal.y0 + 50.0;
        let colors = TAB_COLOR_PRESETS;
        let cols_per_row = 5;
        let swatch_size = 40.0;
        let gap = 8.0;

        for (i, (_label, color)) in colors.iter().enumerate() {
            let row = i / cols_per_row;
            let col = i % cols_per_row;
            let sx = x0 + col as f64 * (swatch_size + gap);
            let sy = y0 + row as f64 * (swatch_size + gap);
            let rect = Rect::new(sx, sy, sx + swatch_size, sy + swatch_size);
            if rect.contains(e.pos) {
                if let Some(target_idx) = self.state.tab_color_target {
                    self.state.sheet_tab_colors.insert(target_idx, *color);
                }
                self.state.show_tab_color_picker = false;
                ctx.request_paint();
                return;
            }
        }

        // "No Color" button
        let no_color_y = y0 + (colors.len() / cols_per_row + 1) as f64 * (swatch_size + gap);
        let no_color_rect = Rect::new(x0, no_color_y, x0 + 120.0, no_color_y + 24.0);
        if no_color_rect.contains(e.pos) {
            if let Some(target_idx) = self.state.tab_color_target {
                self.state.sheet_tab_colors.remove(&target_idx);
            }
            self.state.show_tab_color_picker = false;
            ctx.request_paint();
        }
    }

    // -- Phase 8: Move sheet dialog click handler --

    pub(crate) fn handle_move_sheet_dialog_click(
        &mut self,
        ctx: &mut EventCtx,
        e: &PointerButtonEvent,
    ) {
        let size = ctx.state.size;
        let w = 300.0;
        let h = 200.0;
        let modal = Rect::new(
            size.width / 2.0 - w / 2.0,
            size.height / 2.0 - h / 2.0,
            size.width / 2.0 + w / 2.0,
            size.height / 2.0 + h / 2.0,
        );

        // Click outside closes
        if !modal.contains(e.pos) {
            self.state.show_move_sheet_dialog = false;
            ctx.request_paint();
            return;
        }

        let x0 = modal.x0 + 16.0;
        let y0 = modal.y0 + 50.0;
        let from = self.state.move_sheet_target;

        // Position list
        for (i, _name) in self.state.sheet_names.iter().enumerate() {
            let row_y = y0 + i as f64 * 24.0;
            let item_rect = Rect::new(x0, row_y, x0 + 200.0, row_y + 20.0);
            if item_rect.contains(e.pos) && i != from {
                self.state.move_sheet(from, i);
                self.state.show_move_sheet_dialog = false;
                ctx.request_paint();
                return;
            }
        }
    }
}
