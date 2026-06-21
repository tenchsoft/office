use super::super::*;

// ---------------------------------------------------------------------------
// Paste Special dialog
// ---------------------------------------------------------------------------

/// Paint the paste special dialog.
pub(crate) fn paint_paste_special_dialog(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    state: &SheetsState,
) {
    let w = 280.0;
    let h = 180.0;
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

    p.draw_text(
        "Paste Special",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 30.0;

    let modes = [
        ("All", state::PasteSpecialMode::All),
        ("Values Only", state::PasteSpecialMode::ValuesOnly),
        ("Formats Only", state::PasteSpecialMode::FormatsOnly),
        ("Formulas Only", state::PasteSpecialMode::FormulasOnly),
    ];

    for (label, mode) in &modes {
        let radio = if state.paste_special_mode == *mode {
            "(o)"
        } else {
            "( )"
        };
        p.draw_text(
            &format!("{radio} {label}"),
            x0 + 4.0,
            y,
            theme.on_surface,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
        y += 22.0;
    }

    y += 8.0;

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
