use super::super::dialogs::TAB_COLOR_PRESETS;
use super::super::*;

// ---------------------------------------------------------------------------
// Tab color picker dialog (Phase 8) painting
// ---------------------------------------------------------------------------

pub(crate) fn paint_tab_color_picker(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    _state: &SheetsState,
) {
    let w = 280.0;
    let h = 260.0;
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
        "Tab Color",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 26.0;

    // Color swatches
    let cols_per_row = 5;
    let swatch_size = 40.0;
    let gap = 8.0;

    for (i, (_label, color)) in TAB_COLOR_PRESETS.iter().enumerate() {
        let row = i / cols_per_row;
        let col = i % cols_per_row;
        let sx = x0 + col as f64 * (swatch_size + gap);
        let sy = y + row as f64 * (swatch_size + gap);
        let rect = Rect::new(sx, sy, sx + swatch_size, sy + swatch_size);
        p.fill_rounded_rect(rect, *color, 4.0);
    }

    // "No Color" button
    let no_color_y = y + (TAB_COLOR_PRESETS.len() / cols_per_row + 1) as f64 * (swatch_size + gap);
    let no_color_rect = Rect::new(x0, no_color_y, x0 + 120.0, no_color_y + 24.0);
    p.fill_rounded_rect(no_color_rect, theme.background, 3.0);
    p.stroke_rounded_rect(no_color_rect, theme.border, 0.5, 3.0);
    p.draw_text(
        "No Color",
        x0 + 8.0,
        no_color_y + 17.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
}
