use super::super::*;

// ---------------------------------------------------------------------------
// Pivot Table placeholder dialog (Phase 6) painting
// ---------------------------------------------------------------------------

pub(crate) fn paint_pivot_table_dialog(p: &mut Painter<'_>, theme: &Theme, size: Size) {
    let w = 360.0;
    let h = 200.0;
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
        "Pivot Table",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 36.0;

    // Placeholder message
    p.draw_text(
        "Pivot Table is coming soon.",
        x0,
        y,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 22.0;
    p.draw_text(
        "This feature will allow you to summarize and",
        x0,
        y,
        theme.secondary,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 18.0;
    p.draw_text(
        "analyze data from your spreadsheet dynamically.",
        x0,
        y,
        theme.secondary,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );

    // OK button
    let ok_rect = Rect::new(x0, modal.y1 - 44.0, x0 + 60.0, modal.y1 - 20.0);
    p.fill_rounded_rect(ok_rect, theme.primary, 3.0);
    p.draw_text(
        "OK",
        x0 + 22.0,
        modal.y1 - 27.0,
        Color::WHITE,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
}
