use super::*;

// ---------------------------------------------------------------------------
// Context menu painting
// ---------------------------------------------------------------------------

pub(super) fn paint_context_menu(p: &mut Painter, cache: &mut TextCache, cm: &ContextMenuState) {
    let item_h = 28.0;
    let menu_w = 200.0;
    let items = cm.items();
    let menu_h = items.len() as f64 * item_h;

    // Shadow
    p.fill_rect(
        Rect::new(
            cm.x + 2.0,
            cm.y + 2.0,
            cm.x + menu_w + 2.0,
            cm.y + menu_h + 2.0,
        ),
        Color::rgba8(0, 0, 0, 80),
    );

    // Background
    p.fill_rect(
        Rect::new(cm.x, cm.y, cm.x + menu_w, cm.y + menu_h),
        state::c_menu_bg(),
    );

    // Border
    p.stroke_rounded_rect(
        Rect::new(cm.x, cm.y, cm.x + menu_w, cm.y + menu_h),
        state::c_separator(),
        1.0,
        0.0,
    );

    // Items
    for (idx, &label) in items.iter().enumerate() {
        let item_y = cm.y + idx as f64 * item_h;
        let is_hovered = cm.hovered_item == Some(idx);

        if is_hovered {
            p.fill_rect(
                Rect::new(cm.x + 1.0, item_y, cm.x + menu_w - 1.0, item_y + item_h),
                state::c_btn_hover(),
            );
        }

        let text_color = if is_hovered {
            state::c_text_light()
        } else {
            state::c_text_dim()
        };
        p.draw_text_cached(
            cache,
            label,
            cm.x + 12.0,
            item_y + (item_h - 16.0) / 2.0,
            text_color,
            13.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
    }
}

// ---------------------------------------------------------------------------
// Comment modal painting
// ---------------------------------------------------------------------------

pub(super) fn paint_comment_modal(
    p: &mut Painter,
    cache: &mut TextCache,
    win_size: Size,
    modal: &CommentModalState,
    cursor_visible: bool,
) {
    let modal_w = 360.0;
    let modal_h = 140.0;
    let modal_x = win_size.width / 2.0 - modal_w / 2.0;
    let modal_y = win_size.height / 2.0 - modal_h / 2.0;

    // Dim overlay
    p.fill_rect(
        Rect::new(0.0, 0.0, win_size.width, win_size.height),
        Color::rgba8(0, 0, 0, 120),
    );

    // Shadow
    p.fill_rect(
        Rect::new(
            modal_x + 3.0,
            modal_y + 3.0,
            modal_x + modal_w + 3.0,
            modal_y + modal_h + 3.0,
        ),
        Color::rgba8(0, 0, 0, 60),
    );

    // Background
    p.fill_rect(
        Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h),
        state::c_menu_bg(),
    );

    // Border
    p.stroke_rounded_rect(
        Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h),
        state::c_separator(),
        1.0,
        0.0,
    );

    // Title
    p.draw_text_cached(
        cache,
        "Add Comment",
        modal_x + 16.0,
        modal_y + 14.0,
        state::c_text_light(),
        14.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );

    // Text input field
    let field_x = modal_x + 16.0;
    let field_y = modal_y + 38.0;
    let field_w = modal_w - 32.0;
    let field_h = 28.0;

    p.fill_rect(
        Rect::new(field_x, field_y, field_x + field_w, field_y + field_h),
        Color::rgb8(0x25, 0x25, 0x25),
    );
    p.stroke_rounded_rect(
        Rect::new(field_x, field_y, field_x + field_w, field_y + field_h),
        state::c_accent(),
        1.0,
        0.0,
    );

    // Draw text content
    p.draw_text_cached(
        cache,
        &modal.text,
        field_x + 8.0,
        field_y + 7.0,
        state::c_text_light(),
        13.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    // Cursor
    if cursor_visible {
        let text_before = &modal.text[..modal.cursor_pos];
        let cursor_x = field_x + 8.0 + text_before.len() as f64 * 7.5;
        p.fill_rect(
            Rect::new(
                cursor_x,
                field_y + 4.0,
                cursor_x + 1.5,
                field_y + field_h - 4.0,
            ),
            state::c_accent(),
        );
    }

    // Hint text
    p.draw_text_cached(
        cache,
        "Press Enter to submit, Esc to cancel",
        modal_x + 16.0,
        modal_y + modal_h - 28.0,
        state::c_text_dim(),
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );
}
