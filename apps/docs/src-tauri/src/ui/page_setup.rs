use super::*;

// ---------------------------------------------------------------------------
// Page Setup Dialog
// ---------------------------------------------------------------------------

pub(super) fn paint_page_setup_dialog(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    size: Size,
    state: &DocsState,
) {
    let modal_w = 420.0;
    let modal_h = 380.0;
    let modal_x = size.width / 2.0 - modal_w / 2.0;
    let modal_y = size.height / 2.0 - modal_h / 2.0;

    // Semi-transparent backdrop
    let backdrop = Rect::new(0.0, 0.0, size.width, size.height);
    p.fill_rect(backdrop, Color::rgba8(0, 0, 0, 100));

    let modal = Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
    p.fill_rounded_rect(modal, state::c_menu_bg(), 8.0);
    p.stroke_rounded_rect(modal, state::c_separator(), 1.0, 8.0);

    // Title
    p.draw_text_cached(
        cache,
        "Page Setup",
        modal_x + 20.0,
        modal_y + 28.0,
        state::c_text_light(),
        16.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );

    let dialog = match &state.page_setup_dialog {
        Some(d) => d,
        None => return,
    };

    // Paper size label
    p.draw_text_cached(
        cache,
        "Paper Size:",
        modal_x + 20.0,
        modal_y + 62.0,
        state::c_text_dim(),
        12.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    // Paper size list
    let sizes = state::PAPER_SIZES;
    let mut size_y = modal_y + 90.0;
    for &paper_size in sizes {
        let label = state::paper_size_label(&paper_size);
        let is_selected = dialog.paper_size == paper_size;
        let item_rect = Rect::new(
            modal_x + 20.0,
            size_y - 2.0,
            modal_x + modal_w / 2.0 - 10.0,
            size_y + 18.0,
        );

        if is_selected {
            p.fill_rounded_rect(item_rect, Color::rgba8(0xA7, 0x8B, 0xFA, 40), 4.0);
        }
        p.draw_text_cached(
            cache,
            label,
            modal_x + 24.0,
            size_y + 12.0,
            if is_selected {
                state::c_accent()
            } else {
                state::c_text_light()
            },
            11.0,
            if is_selected {
                tench_ui::parley::FontWeight::BOLD
            } else {
                tench_ui::parley::FontWeight::NORMAL
            },
            false,
            false,
        );
        size_y += 24.0;
    }

    // Orientation section (right side)
    let right_x = modal_x + modal_w / 2.0 + 20.0;
    p.draw_text_cached(
        cache,
        "Orientation:",
        right_x,
        modal_y + 62.0,
        state::c_text_dim(),
        12.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    // Portrait button
    let portrait_rect = Rect::new(right_x, modal_y + 80.0, right_x + 80.0, modal_y + 104.0);
    let portrait_selected = dialog.orientation == Orientation::Portrait;
    if portrait_selected {
        p.fill_rounded_rect(portrait_rect, Color::rgba8(0xA7, 0x8B, 0xFA, 40), 4.0);
    }
    p.stroke_rounded_rect(portrait_rect, state::c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "Portrait",
        portrait_rect.x0 + portrait_rect.width() / 2.0,
        portrait_rect.y0 + portrait_rect.height() / 2.0,
        if portrait_selected {
            state::c_accent()
        } else {
            state::c_text_light()
        },
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        true,
        false,
    );

    // Landscape button
    let landscape_rect = Rect::new(
        right_x + 90.0,
        modal_y + 80.0,
        right_x + 190.0,
        modal_y + 104.0,
    );
    let landscape_selected = dialog.orientation == Orientation::Landscape;
    if landscape_selected {
        p.fill_rounded_rect(landscape_rect, Color::rgba8(0xA7, 0x8B, 0xFA, 40), 4.0);
    }
    p.stroke_rounded_rect(landscape_rect, state::c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "Landscape",
        landscape_rect.x0 + landscape_rect.width() / 2.0,
        landscape_rect.y0 + landscape_rect.height() / 2.0,
        if landscape_selected {
            state::c_accent()
        } else {
            state::c_text_light()
        },
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        true,
        false,
    );

    // Margins section
    p.draw_text_cached(
        cache,
        "Margins (mm):",
        right_x,
        modal_y + 130.0,
        state::c_text_dim(),
        12.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    let margin_labels = [
        ("Top", dialog.margin_top),
        ("Bottom", dialog.margin_bottom),
        ("Left", dialog.margin_left),
        ("Right", dialog.margin_right),
    ];
    let mut margin_y = modal_y + 156.0;
    for (idx, (label, value)) in margin_labels.iter().enumerate() {
        let is_editing = dialog.editing_margin_field == Some(idx);
        p.draw_text_cached(
            cache,
            label,
            right_x,
            margin_y + 10.0,
            state::c_text_light(),
            11.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
        let field_rect = Rect::new(right_x + 60.0, margin_y, right_x + 130.0, margin_y + 22.0);
        let bg_color = if is_editing {
            Color::rgba8(0x1A, 0x1A, 0x2E, 255)
        } else {
            Color::rgba8(0x0A, 0x0A, 0x0A, 255)
        };
        let border_color = if is_editing {
            state::c_accent()
        } else {
            state::c_separator()
        };
        p.fill_rounded_rect(field_rect, bg_color, 4.0);
        p.stroke_rounded_rect(
            field_rect,
            border_color,
            if is_editing { 2.0 } else { 1.0 },
            4.0,
        );
        let display_text = if is_editing {
            &dialog.margin_edit_buffer
        } else {
            // Use a temporary string for the formatted value
            &format!("{:.1}", value)
        };
        p.draw_text_cached(
            cache,
            display_text,
            field_rect.x0 + 8.0,
            field_rect.y0 + field_rect.height() / 2.0,
            if is_editing {
                state::c_text_light()
            } else {
                state::c_text_dim()
            },
            11.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
        margin_y += 28.0;
    }

    // Preview section (right side, below margins)
    let preview_y = margin_y + 8.0;
    p.draw_text_cached(
        cache,
        "Preview:",
        right_x,
        preview_y,
        state::c_text_dim(),
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );
    let preview_rect = Rect::new(
        right_x,
        preview_y + 16.0,
        right_x + 80.0,
        preview_y + 16.0 + 100.0,
    );
    p.fill_rounded_rect(preview_rect, Color::rgba8(0x25, 0x25, 0x25, 255), 2.0);
    p.stroke_rounded_rect(preview_rect, state::c_separator(), 1.0, 2.0);
    // Draw content area inside preview
    let preview_margin = 8.0;
    let content_preview = Rect::new(
        preview_rect.x0 + preview_margin,
        preview_rect.y0 + preview_margin,
        preview_rect.x1 - preview_margin,
        preview_rect.y1 - preview_margin,
    );
    p.fill_rect(content_preview, Color::rgba8(0x40, 0x40, 0x40, 255));

    // Page dimensions info
    let setup = dialog.to_page_setup();
    let (pw, ph) = setup.page_size_px();
    p.draw_text_cached(
        cache,
        &format!("{:.0} x {:.0} px", pw, ph),
        right_x + 90.0,
        preview_y + 40.0,
        state::c_text_dim(),
        10.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );
    let (w_mm, h_mm) = setup.paper_size.dimensions_mm();
    p.draw_text_cached(
        cache,
        &format!("{:.1} x {:.1} mm", w_mm, h_mm),
        right_x + 90.0,
        preview_y + 56.0,
        state::c_text_dim(),
        10.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    // OK button
    let ok_rect = Rect::new(
        modal_x + modal_w - 160.0,
        modal_y + modal_h - 44.0,
        modal_x + modal_w - 90.0,
        modal_y + modal_h - 16.0,
    );
    p.fill_rounded_rect(ok_rect, state::c_accent(), 4.0);
    p.draw_text_cached(
        cache,
        "OK",
        ok_rect.x0 + ok_rect.width() / 2.0,
        ok_rect.y0 + ok_rect.height() / 2.0,
        Color::WHITE,
        11.0,
        tench_ui::parley::FontWeight::BOLD,
        true,
        false,
    );

    // Cancel button
    let cancel_rect = Rect::new(
        modal_x + modal_w - 80.0,
        modal_y + modal_h - 44.0,
        modal_x + modal_w - 12.0,
        modal_y + modal_h - 16.0,
    );
    p.stroke_rounded_rect(cancel_rect, state::c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "Cancel",
        cancel_rect.x0 + cancel_rect.width() / 2.0,
        cancel_rect.y0 + cancel_rect.height() / 2.0,
        state::c_text_light(),
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        true,
        false,
    );
}
