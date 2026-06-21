use super::super::*;

// ---------------------------------------------------------------------------
// Named Ranges dialog
// ---------------------------------------------------------------------------

/// Paint the named ranges management dialog.
pub(crate) fn paint_named_ranges_dialog(
    p: &mut Painter<'_>,
    theme: &Theme,
    size: Size,
    state: &SheetsState,
) {
    let w = 320.0;
    let h = 240.0;
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
        "Named Ranges",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 28.0;

    if state.named_ranges.is_empty() {
        p.draw_text(
            "No named ranges defined",
            x0,
            y,
            theme.disabled,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
    } else {
        // Header
        p.draw_text(
            "Name",
            x0,
            y,
            theme.secondary,
            theme.font_size_small,
            tench_ui::parley::FontWeight::BOLD,
            false,
        );
        p.draw_text(
            "Scope",
            x0 + 100.0,
            y,
            theme.secondary,
            theme.font_size_small,
            tench_ui::parley::FontWeight::BOLD,
            false,
        );
        p.draw_text(
            "Refers To",
            x0 + 170.0,
            y,
            theme.secondary,
            theme.font_size_small,
            tench_ui::parley::FontWeight::BOLD,
            false,
        );
        y += 20.0;

        for nr in &state.named_ranges {
            if y > modal.y1 - 50.0 {
                break;
            }
            p.draw_text(
                &nr.name,
                x0,
                y,
                theme.on_surface,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
            let scope = match nr.sheet_idx {
                Some(idx) => format!("Sheet{}", idx + 1),
                None => "Global".to_string(),
            };
            p.draw_text(
                &scope,
                x0 + 100.0,
                y,
                theme.secondary,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
            p.draw_text(
                &nr.range.to_address(),
                x0 + 170.0,
                y,
                theme.primary,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
            y += 20.0;
        }
    }

    // Buttons at the bottom
    y = modal.y1 - 40.0;
    let btn_labels = ["Add", "Edit", "Delete", "Close"];
    let mut bx = x0;
    for label in &btn_labels {
        let btn_rect = Rect::new(bx, y, bx + 56.0, y + 24.0);
        p.fill_rounded_rect(btn_rect, theme.primary, 3.0);
        p.draw_text(
            label,
            bx + 8.0,
            y + 17.0,
            Color::WHITE,
            theme.font_size_small,
            tench_ui::parley::FontWeight::BOLD,
            false,
        );
        bx += 64.0;
    }
}
