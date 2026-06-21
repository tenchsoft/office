use super::super::state::{ChartType, SheetsState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

// ---------------------------------------------------------------------------
// Chart wizard painting
// ---------------------------------------------------------------------------

/// Paint the chart creation wizard dialog.
pub fn paint_chart_wizard(p: &mut Painter<'_>, theme: &Theme, size: Size, state: &SheetsState) {
    let w = 420.0;
    let h = 380.0;
    let modal = Rect::new(
        size.width / 2.0 - w / 2.0,
        size.height / 2.0 - h / 2.0,
        size.width / 2.0 + w / 2.0,
        size.height / 2.0 + h / 2.0,
    );

    p.fill_rounded_rect(modal, theme.surface, theme.border_radius);
    p.stroke_rounded_rect(modal, theme.border, 1.0, theme.border_radius);

    let x0 = modal.x0 + 16.0;
    let mut y = modal.y0 + 24.0;

    // Title
    p.draw_text(
        "Insert Chart",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );

    // Step indicator
    let step_labels = ["1. Data Range", "2. Chart Type", "3. Customize"];
    let mut step_x = modal.x1 - 180.0;
    for (i, label) in step_labels.iter().enumerate() {
        let is_active = i == state.chart_wizard_step;
        let color = if is_active {
            theme.primary
        } else {
            theme.disabled
        };
        p.draw_text(
            label,
            step_x,
            y,
            color,
            theme.font_size_small,
            if is_active {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            },
            false,
        );
        step_x += 60.0;
    }
    y += 32.0;

    match state.chart_wizard_step {
        0 => {
            // Step 1: Data Range
            p.draw_text(
                "Data Range:",
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::BOLD,
                false,
            );
            y += 20.0;

            let input_rect = Rect::new(x0, y, modal.x1 - 16.0, y + 24.0);
            p.fill_rounded_rect(input_rect, Color::WHITE, 3.0);
            p.stroke_rounded_rect(input_rect, theme.primary, 1.0, 3.0);
            let display = if state.chart_wizard_data_range.is_empty() {
                "e.g., A1:D5"
            } else {
                &state.chart_wizard_data_range
            };
            p.draw_text(
                display,
                x0 + 8.0,
                y + 17.0,
                if state.chart_wizard_data_range.is_empty() {
                    theme.disabled
                } else {
                    theme.on_surface
                },
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
            y += 40.0;

            p.draw_text(
                "Select a range in the grid or type a reference above.",
                x0,
                y,
                theme.secondary,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
        1 => {
            // Step 2: Chart Type
            p.draw_text(
                "Select Chart Type:",
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::BOLD,
                false,
            );
            y += 24.0;

            for ct in ChartType::ALL {
                let is_selected = ct == state.chart_wizard_chart_type;
                let label = ct.label();
                let btn_rect = Rect::new(x0, y - 2.0, x0 + 100.0, y + 20.0);
                if is_selected {
                    p.fill_rounded_rect(btn_rect, theme.primary, 3.0);
                    p.draw_text(
                        label,
                        x0 + 8.0,
                        y + 14.0,
                        Color::WHITE,
                        theme.font_size_small,
                        FontWeight::BOLD,
                        false,
                    );
                } else {
                    p.fill_rounded_rect(btn_rect, theme.background, 3.0);
                    p.stroke_rounded_rect(btn_rect, theme.border, 0.5, 3.0);
                    p.draw_text(
                        label,
                        x0 + 8.0,
                        y + 14.0,
                        theme.on_surface,
                        theme.font_size_small,
                        FontWeight::NORMAL,
                        false,
                    );
                }
                y += 28.0;
            }
        }
        _ => {
            // Step 3: Customize
            p.draw_text(
                "Chart Title:",
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::BOLD,
                false,
            );
            y += 20.0;

            let title_rect = Rect::new(x0, y, modal.x1 - 16.0, y + 24.0);
            p.fill_rounded_rect(title_rect, Color::WHITE, 3.0);
            p.stroke_rounded_rect(title_rect, theme.border, 0.5, 3.0);
            let title_display = if state.chart_wizard_title.is_empty() {
                "Chart Title"
            } else {
                &state.chart_wizard_title
            };
            p.draw_text(
                title_display,
                x0 + 8.0,
                y + 17.0,
                if state.chart_wizard_title.is_empty() {
                    theme.disabled
                } else {
                    theme.on_surface
                },
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
            y += 36.0;

            // Show Legend checkbox
            let legend_marker = if state.chart_wizard_show_legend {
                "[x]"
            } else {
                "[ ]"
            };
            p.draw_text(
                &format!("{legend_marker} Show Legend"),
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
            y += 24.0;

            // Show Axis Labels checkbox
            let axis_marker = if state.chart_wizard_show_axis_labels {
                "[x]"
            } else {
                "[ ]"
            };
            p.draw_text(
                &format!("{axis_marker} Show Axis Labels"),
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    // Navigation buttons at the bottom
    let btn_y = modal.y1 - 44.0;

    // Back button (if not step 0)
    if state.chart_wizard_step > 0 {
        let back_rect = Rect::new(x0, btn_y, x0 + 60.0, btn_y + 24.0);
        p.fill_rounded_rect(back_rect, theme.background, 3.0);
        p.stroke_rounded_rect(back_rect, theme.border, 0.5, 3.0);
        p.draw_text(
            "Back",
            x0 + 14.0,
            btn_y + 17.0,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
    }

    // Next / Create button
    let next_x = x0 + 72.0;
    let next_label = if state.chart_wizard_step >= 2 {
        "Create"
    } else {
        "Next"
    };
    let next_rect = Rect::new(next_x, btn_y, next_x + 64.0, btn_y + 24.0);
    p.fill_rounded_rect(next_rect, theme.primary, 3.0);
    p.draw_text(
        next_label,
        next_x + 10.0,
        btn_y + 17.0,
        Color::WHITE,
        theme.font_size_small,
        FontWeight::BOLD,
        false,
    );

    // Cancel button
    let cancel_rect = Rect::new(next_x + 76.0, btn_y, next_x + 140.0, btn_y + 24.0);
    p.fill_rounded_rect(cancel_rect, theme.background, 3.0);
    p.stroke_rounded_rect(cancel_rect, theme.border, 0.5, 3.0);
    p.draw_text(
        "Cancel",
        next_x + 84.0,
        btn_y + 17.0,
        theme.on_surface,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
}

/// Hit-test the chart wizard dialog.
pub fn hit_chart_wizard(
    state: &SheetsState,
    x: f64,
    y: f64,
    size: Size,
) -> Option<ChartWizardAction> {
    let w = 420.0;
    let h = 380.0;
    let modal = Rect::new(
        size.width / 2.0 - w / 2.0,
        size.height / 2.0 - h / 2.0,
        size.width / 2.0 + w / 2.0,
        size.height / 2.0 + h / 2.0,
    );

    if !modal.contains(Point::new(x, y)) {
        return Some(ChartWizardAction::Cancel);
    }

    let x0 = modal.x0 + 16.0;
    let btn_y = modal.y1 - 44.0;

    // Back button
    if state.chart_wizard_step > 0 {
        let back_rect = Rect::new(x0, btn_y, x0 + 60.0, btn_y + 24.0);
        if back_rect.contains(Point::new(x, y)) {
            return Some(ChartWizardAction::Back);
        }
    }

    // Next / Create button
    let next_x = x0 + 72.0;
    let next_rect = Rect::new(next_x, btn_y, next_x + 64.0, btn_y + 24.0);
    if next_rect.contains(Point::new(x, y)) {
        return Some(ChartWizardAction::Next);
    }

    // Cancel button
    let cancel_rect = Rect::new(next_x + 76.0, btn_y, next_x + 140.0, btn_y + 24.0);
    if cancel_rect.contains(Point::new(x, y)) {
        return Some(ChartWizardAction::Cancel);
    }

    // Step-specific hit testing
    match state.chart_wizard_step {
        0 => {
            // Data range input field
            let field_y = modal.y0 + 24.0 + 32.0 + 20.0;
            let input_rect = Rect::new(x0, field_y, modal.x1 - 16.0, field_y + 24.0);
            if input_rect.contains(Point::new(x, y)) {
                return Some(ChartWizardAction::FocusDataRange);
            }
        }
        1 => {
            // Chart type selection
            let mut type_y = modal.y0 + 24.0 + 32.0 + 24.0;
            for ct in ChartType::ALL {
                let btn_rect = Rect::new(x0, type_y - 2.0, x0 + 100.0, type_y + 20.0);
                if btn_rect.contains(Point::new(x, y)) {
                    return Some(ChartWizardAction::SelectChartType(ct));
                }
                type_y += 28.0;
            }
        }
        _ => {
            // Customize step
            let mut field_y = modal.y0 + 24.0 + 32.0 + 20.0;
            // Title input
            let title_rect = Rect::new(x0, field_y, modal.x1 - 16.0, field_y + 24.0);
            if title_rect.contains(Point::new(x, y)) {
                return Some(ChartWizardAction::FocusTitle);
            }
            field_y += 36.0 + 24.0;
            // Legend checkbox
            let legend_rect = Rect::new(x0, field_y - 12.0, x0 + 160.0, field_y + 12.0);
            if legend_rect.contains(Point::new(x, y)) {
                return Some(ChartWizardAction::ToggleLegend);
            }
            field_y += 24.0;
            // Axis labels checkbox
            let axis_rect = Rect::new(x0, field_y - 12.0, x0 + 200.0, field_y + 12.0);
            if axis_rect.contains(Point::new(x, y)) {
                return Some(ChartWizardAction::ToggleAxisLabels);
            }
        }
    }

    None
}

/// Actions from the chart wizard.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartWizardAction {
    Next,
    Back,
    Cancel,
    FocusDataRange,
    FocusTitle,
    SelectChartType(ChartType),
    ToggleLegend,
    ToggleAxisLabels,
}

/// Which field is focused in the chart wizard.
#[allow(dead_code)] // will be used for tab-focus cycling in chart wizard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartWizardFocus {
    #[default]
    DataRange,
    Title,
}

/// Handle keyboard input for the chart wizard.
pub fn handle_chart_wizard_key(
    state: &mut SheetsState,
    kb: &tench_ui::core::events::KeyboardEvent,
) -> bool {
    use tench_ui::core::events::{LogicalKey, NamedKey};

    if !kb.is_pressed {
        return false;
    }

    match &kb.logical_key {
        LogicalKey::Named(NamedKey::Escape) => {
            state.show_chart_wizard = false;
            true
        }
        LogicalKey::Named(NamedKey::Enter) => {
            if state.chart_wizard_step < 2 {
                state.chart_wizard_step += 1;
            } else {
                state.create_chart_from_wizard();
            }
            true
        }
        LogicalKey::Named(NamedKey::Backspace) => {
            match state.chart_wizard_step {
                0 => {
                    state.chart_wizard_data_range.pop();
                }
                2 => {
                    state.chart_wizard_title.pop();
                }
                _ => {}
            }
            true
        }
        LogicalKey::Character(c) if !c.is_empty() && !kb.modifiers.control => {
            match state.chart_wizard_step {
                0 => {
                    state.chart_wizard_data_range.push_str(c);
                }
                2 => {
                    state.chart_wizard_title.push_str(c);
                }
                _ => {}
            }
            true
        }
        _ => false,
    }
}
