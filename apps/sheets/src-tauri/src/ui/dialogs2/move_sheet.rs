use super::super::*;

// ---------------------------------------------------------------------------
// Move sheet dialog (Phase 8) painting
// ---------------------------------------------------------------------------

pub(crate) fn paint_move_sheet_dialog(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    state: &SheetsState,
) {
    let w = 300.0;
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
        "Move Sheet",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 26.0;

    p.draw_text(
        "Select target position:",
        x0,
        y,
        theme.secondary,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
    y += 20.0;

    let from = state.move_sheet_target;
    for (i, name) in state.sheet_names.iter().enumerate() {
        let is_target = i == from;
        let label = if is_target {
            format!("{} (current)", name)
        } else {
            format!("before {}", name)
        };
        let item_rect = Rect::new(x0, y - 2.0, x0 + 200.0, y + 16.0);
        if is_target {
            p.fill_rounded_rect(item_rect, theme.background, 3.0);
            p.draw_text(
                &label,
                x0 + 8.0,
                y + 12.0,
                theme.disabled,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
        } else {
            p.fill_rounded_rect(item_rect, theme.surface, 3.0);
            p.stroke_rounded_rect(item_rect, theme.border, 0.5, 3.0);
            p.draw_text(
                &label,
                x0 + 8.0,
                y + 12.0,
                theme.on_surface,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
        }
        y += 24.0;
    }
}
