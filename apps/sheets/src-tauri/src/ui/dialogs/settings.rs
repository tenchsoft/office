use super::super::*;

// ---------------------------------------------------------------------------
// Settings dialog painting
// ---------------------------------------------------------------------------

/// Paint the Settings dialog.
pub(crate) fn paint_settings_dialog(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    state: &SheetsState,
) {
    let w = 320.0;
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
        "Settings",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 28.0;

    // Auto-save interval
    let interval_label = format!("Auto-save interval: {}s", state.auto_save.interval_secs);
    p.draw_text(
        &interval_label,
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    // - / + buttons
    p.draw_text(
        "[-]",
        x0 + 200.0,
        y,
        theme.primary,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    p.draw_text(
        "[+]",
        x0 + 260.0,
        y,
        theme.primary,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 30.0;

    // Auto-save enabled
    let autosave_marker = if state.auto_save.enabled {
        "[x]"
    } else {
        "[ ]"
    };
    p.draw_text(
        &format!("{autosave_marker} Auto-save enabled"),
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 30.0;

    // Grid lines
    let grid_marker = if state.show_grid_lines { "[x]" } else { "[ ]" };
    p.draw_text(
        &format!("{grid_marker} Show grid lines"),
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 30.0;

    // Headers
    let headers_marker = if state.show_headers { "[x]" } else { "[ ]" };
    p.draw_text(
        &format!("{headers_marker} Show row/column headers"),
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 30.0;

    // Formula bar
    let fbar_marker = if state.show_formula_bar { "[x]" } else { "[ ]" };
    p.draw_text(
        &format!("{fbar_marker} Show formula bar"),
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 30.0;

    // OK / Cancel buttons
    let ok_rect = Rect::new(x0, y, x0 + 60.0, y + 24.0);
    p.fill_rounded_rect(ok_rect, theme.primary, 3.0);
    p.draw_text(
        "OK",
        x0 + 22.0,
        y + 17.0,
        Color::WHITE,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );

    let cancel_rect = Rect::new(x0 + 72.0, y, x0 + 132.0, y + 24.0);
    p.fill_rounded_rect(cancel_rect, theme.background, 3.0);
    p.stroke_rounded_rect(cancel_rect, theme.border, 0.5, 3.0);
    p.draw_text(
        "Cancel",
        x0 + 78.0,
        y + 17.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
}
