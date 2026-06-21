use super::super::*;

// ---------------------------------------------------------------------------
// Sort dialog painting
// ---------------------------------------------------------------------------

/// Paint the Sort dialog.
pub(crate) fn paint_sort_dialog(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    state: &SheetsState,
) {
    let w = 300.0;
    let h = 220.0;
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
        "Sort",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 28.0;

    // Sort by column
    let col_label = format!("Sort by: Column {}", state::col_letter(state.selected_col));
    p.draw_text(
        &col_label,
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 30.0;

    // Ascending / Descending radio
    let asc_marker = if state.sort_ascending { "(o)" } else { "( )" };
    let desc_marker = if state.sort_ascending { "( )" } else { "(o)" };
    p.draw_text(
        &format!("{asc_marker} Ascending"),
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    p.draw_text(
        &format!("{desc_marker} Descending"),
        x0 + 130.0,
        y,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 26.0;

    // Header row checkbox
    let header_marker = if state.sort_has_header { "[x]" } else { "[ ]" };
    p.draw_text(
        &format!("{header_marker} Data has header row"),
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
