use super::super::*;

// ---------------------------------------------------------------------------
// Insert Function dialog painting
// ---------------------------------------------------------------------------

/// Paint the Insert Function dialog.
pub(crate) fn paint_insert_function_dialog(
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

    p.fill_rounded_rect(modal, theme.surface, theme.border_radius);
    p.stroke_rounded_rect(modal, theme.border, 1.0, theme.border_radius);

    let x0 = modal.x0 + 16.0;
    let mut y = modal.y0 + 24.0;

    // Title
    p.draw_text(
        "Insert Function",
        x0,
        y,
        theme.on_surface,
        theme.font_size,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );
    y += 28.0;

    // Function list
    let catalog = state::function_catalog();
    let list_end_y = modal.y1 - 80.0;
    let row_h = 20.0;

    for (i, func) in catalog.iter().enumerate() {
        if y + row_h > list_end_y {
            // Show "..." to indicate more
            p.draw_text(
                "...",
                x0,
                y,
                theme.disabled,
                theme.font_size_small,
                tench_ui::parley::FontWeight::NORMAL,
                false,
            );
            break;
        }
        let is_selected = i == state.insert_function_selected;
        if is_selected {
            let sel_rect = Rect::new(x0 - 2.0, y - 14.0, modal.x1 - 16.0, y + 6.0);
            // Semi-transparent primary color for selection highlight
            let packed = theme.primary.to_u32();
            let r = ((packed >> 24) & 0xFF) as f32 / 255.0;
            let g = ((packed >> 16) & 0xFF) as f32 / 255.0;
            let b = ((packed >> 8) & 0xFF) as f32 / 255.0;
            let sel_color = Color::from_rgba(r, g, b, 0.15);
            p.fill_rounded_rect(sel_rect, sel_color, 3.0);
        }
        let color = if is_selected {
            theme.primary
        } else {
            theme.on_surface
        };
        p.draw_text(
            func.name,
            x0,
            y,
            color,
            theme.font_size_small,
            if is_selected {
                tench_ui::parley::FontWeight::BOLD
            } else {
                tench_ui::parley::FontWeight::NORMAL
            },
            false,
        );
        p.draw_text(
            func.category,
            x0 + 100.0,
            y,
            theme.secondary,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
        p.draw_text(
            func.description,
            x0 + 180.0,
            y,
            theme.secondary,
            theme.font_size_small,
            tench_ui::parley::FontWeight::NORMAL,
            false,
        );
        y += row_h;
    }

    // Signature of selected function
    let sig_y = list_end_y + 8.0;
    if let Some(func) = catalog.get(state.insert_function_selected) {
        p.draw_text(
            func.signature,
            x0,
            sig_y,
            theme.primary,
            theme.font_size_small,
            tench_ui::parley::FontWeight::BOLD,
            false,
        );
    }

    // Buttons
    let btn_y = modal.y1 - 40.0;
    let insert_btn = Rect::new(x0, btn_y, x0 + 70.0, btn_y + 24.0);
    p.fill_rounded_rect(insert_btn, theme.primary, 3.0);
    p.draw_text(
        "Insert",
        x0 + 10.0,
        btn_y + 17.0,
        Color::WHITE,
        theme.font_size_small,
        tench_ui::parley::FontWeight::BOLD,
        false,
    );

    let cancel_btn = Rect::new(x0 + 82.0, btn_y, x0 + 152.0, btn_y + 24.0);
    p.fill_rounded_rect(cancel_btn, theme.background, 3.0);
    p.stroke_rounded_rect(cancel_btn, theme.border, 0.5, 3.0);
    p.draw_text(
        "Cancel",
        x0 + 90.0,
        btn_y + 17.0,
        theme.on_surface,
        theme.font_size_small,
        tench_ui::parley::FontWeight::NORMAL,
        false,
    );
}
