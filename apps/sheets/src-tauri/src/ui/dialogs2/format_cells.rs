use super::super::*;

// ---------------------------------------------------------------------------
// Format Cells dialog (Phase 5) painting
// ---------------------------------------------------------------------------

/// Paint the Format Cells dialog (Phase 5).
pub(crate) fn paint_format_cells_dialog(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    state: &SheetsState,
) {
    let w = 400.0;
    let h = 380.0;
    let modal = Rect::new(
        size.width / 2.0 - w / 2.0,
        size.height / 2.0 - h / 2.0,
        size.width / 2.0 + w / 2.0,
        size.height / 2.0 + h / 2.0,
    );

    p.fill_rounded_rect(modal, theme.background, 6.0);
    p.stroke_rounded_rect(modal, theme.border, 1.0, 6.0);

    p.draw_text(
        "Format Cells",
        modal.x0 + 16.0,
        modal.y0 + 22.0,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );

    let x0 = modal.x0 + 16.0;
    let y0 = modal.y0 + 40.0;

    let tab_labels = ["Number", "Alignment", "Font", "Border", "Fill"];
    let tab_w = 70.0;
    for (i, label) in tab_labels.iter().enumerate() {
        let tab_x = x0 + i as f64 * (tab_w + 4.0);
        let tab_rect = Rect::new(tab_x, y0, tab_x + tab_w, y0 + 24.0);
        let is_active = match i {
            0 => state.format_cells.active_tab == FormatCellsTab::Number,
            1 => state.format_cells.active_tab == FormatCellsTab::Alignment,
            2 => state.format_cells.active_tab == FormatCellsTab::Font,
            3 => state.format_cells.active_tab == FormatCellsTab::Border,
            4 => state.format_cells.active_tab == FormatCellsTab::Fill,
            _ => false,
        };
        let bg = if is_active {
            theme.primary
        } else {
            theme.surface
        };
        let fg = if is_active {
            Color::WHITE
        } else {
            theme.on_surface
        };
        p.fill_rounded_rect(tab_rect, bg, 3.0);
        p.stroke_rounded_rect(tab_rect, theme.border, 0.5, 3.0);
        p.draw_text(
            label,
            tab_x + 6.0,
            y0 + 17.0,
            fg,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    }

    let content_y = y0 + 36.0;

    match state.format_cells.active_tab {
        FormatCellsTab::Number => {
            let formats = [
                "General",
                "Number",
                "Currency",
                "Percentage",
                "Date",
                "Text",
            ];
            let current = match &state.get_selected_cell_format().number_format {
                NumberFormat::General => 0,
                NumberFormat::Number { .. } => 1,
                NumberFormat::Currency { .. } => 2,
                NumberFormat::Percentage { .. } => 3,
                NumberFormat::Date => 4,
                NumberFormat::Text => 5,
            };
            for (i, label) in formats.iter().enumerate() {
                let row_y = content_y + i as f64 * 26.0;
                let item_rect = Rect::new(x0, row_y, x0 + 160.0, row_y + 22.0);
                let bg = if i == current {
                    theme.primary
                } else {
                    theme.background
                };
                let fg = if i == current {
                    Color::WHITE
                } else {
                    theme.on_surface
                };
                p.fill_rounded_rect(item_rect, bg, 3.0);
                p.stroke_rounded_rect(item_rect, theme.border, 0.5, 3.0);
                p.draw_text(
                    label,
                    x0 + 8.0,
                    row_y + 16.0,
                    fg,
                    theme.font_size_small,
                    tench_ui::parley::FontWeight::NORMAL,
                    false,
                );
            }
        }
        FormatCellsTab::Alignment => {
            let labels = ["Left", "Center", "Right"];
            let current = match state.get_selected_cell_format().h_align {
                Some(HorizontalAlignment::Left) => 0,
                Some(HorizontalAlignment::Center) => 1,
                Some(HorizontalAlignment::Right) => 2,
                None => 1,
            };
            for (i, label) in labels.iter().enumerate() {
                let row_y = content_y + i as f64 * 26.0;
                let item_rect = Rect::new(x0, row_y, x0 + 100.0, row_y + 22.0);
                let bg = if i == current {
                    theme.primary
                } else {
                    theme.background
                };
                let fg = if i == current {
                    Color::WHITE
                } else {
                    theme.on_surface
                };
                p.fill_rounded_rect(item_rect, bg, 3.0);
                p.stroke_rounded_rect(item_rect, theme.border, 0.5, 3.0);
                p.draw_text(
                    label,
                    x0 + 8.0,
                    row_y + 16.0,
                    fg,
                    theme.font_size_small,
                    tench_ui::parley::FontWeight::NORMAL,
                    false,
                );
            }
        }
        FormatCellsTab::Font => {
            let toggles = ["Bold", "Italic", "Underline"];
            let fmt = state.get_selected_cell_format();
            let states = [fmt.bold, fmt.italic, fmt.underline];
            for (i, label) in toggles.iter().enumerate() {
                let row_y = content_y + i as f64 * 26.0;
                let item_rect = Rect::new(x0, row_y, x0 + 100.0, row_y + 22.0);
                let bg = if states[i] {
                    theme.primary
                } else {
                    theme.background
                };
                let fg = if states[i] {
                    Color::WHITE
                } else {
                    theme.on_surface
                };
                p.fill_rounded_rect(item_rect, bg, 3.0);
                p.stroke_rounded_rect(item_rect, theme.border, 0.5, 3.0);
                p.draw_text(
                    label,
                    x0 + 8.0,
                    row_y + 16.0,
                    fg,
                    theme.font_size_small,
                    tench_ui::parley::FontWeight::NORMAL,
                    false,
                );
            }
        }
        FormatCellsTab::Border => {
            p.draw_text(
                "Border settings (coming soon)",
                x0,
                content_y + 16.0,
                theme.on_surface,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
        }
        FormatCellsTab::Fill => {
            let colors = [
                ("Yellow", Color::rgb8(255, 255, 0)),
                ("Light Green", Color::rgb8(144, 238, 144)),
                ("Light Blue", Color::rgb8(173, 216, 230)),
                ("Light Red", Color::rgb8(255, 182, 193)),
                ("Light Gray", Color::rgb8(211, 211, 211)),
                ("No Fill", Color::TRANSPARENT),
            ];
            for (i, (label, color)) in colors.iter().enumerate() {
                let row_y = content_y + i as f64 * 26.0;
                let item_rect = Rect::new(x0, row_y, x0 + 100.0, row_y + 22.0);
                p.fill_rounded_rect(item_rect, theme.background, 3.0);
                p.stroke_rounded_rect(item_rect, theme.border, 0.5, 3.0);
                p.fill_rounded_rect(
                    Rect::new(x0 + 4.0, row_y + 4.0, x0 + 20.0, row_y + 18.0),
                    *color,
                    2.0,
                );
                p.draw_text(
                    label,
                    x0 + 26.0,
                    row_y + 16.0,
                    theme.on_surface,
                    theme.font_size_small,
                    tench_ui::parley::FontWeight::NORMAL,
                    false,
                );
            }
        }
    }

    let btn_y = modal.y1 - 44.0;
    let ok_rect = Rect::new(x0, btn_y, x0 + 60.0, btn_y + 24.0);
    p.fill_rounded_rect(ok_rect, theme.primary, 3.0);
    p.draw_text(
        "OK",
        x0 + 22.0,
        btn_y + 17.0,
        Color::WHITE,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );

    let cancel_rect = Rect::new(x0 + 72.0, btn_y, x0 + 132.0, btn_y + 24.0);
    p.fill_rounded_rect(cancel_rect, theme.background, 3.0);
    p.stroke_rounded_rect(cancel_rect, theme.border, 0.5, 3.0);
    p.draw_text(
        "Cancel",
        x0 + 78.0,
        btn_y + 17.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
}
