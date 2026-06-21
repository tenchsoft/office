use super::super::state::{
    c_accent, c_btn_hover, c_canvas_bg, c_menu_bg, c_separator, c_text_dim, c_text_light,
    DocsState, GotoModalState, GotoMode, SpecialCharModalState, PAGE_H, PAGE_W,
    SPECIAL_CHAR_CATEGORIES,
};
use tench_document_core::BlockNode;
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

/// Paint the print preview modal.
pub fn paint_print_preview(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    win_size: Size,
    state: &DocsState,
) {
    use super::super::layout::modal::compute_print_preview;

    let layout = compute_print_preview(win_size);
    let modal = layout.modal;
    let modal_x = modal.x0;
    let modal_y = modal.y0;
    let modal_w = modal.width();
    let modal_h = modal.height();

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
    p.fill_rounded_rect(modal, c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, c_separator(), 1.0, 6.0);

    // Title
    p.draw_text_cached(
        cache,
        "Print Preview",
        modal_x + 16.0,
        modal_y + 20.0,
        c_text_light(),
        14.0,
        FontWeight::BOLD,
        false,
        false,
    );

    // Page preview area
    let preview_area = Rect::new(
        modal_x + 20.0,
        modal_y + 40.0,
        modal_x + modal_w - 20.0,
        modal_y + modal_h - 60.0,
    );
    p.fill_rounded_rect(preview_area, c_canvas_bg(), 4.0);

    // Draw miniature page
    if let Some(pp) = &state.print_preview {
        let num_pages = state.layout_cache.num_pages().max(1);
        let page_idx = pp.page_index.min(num_pages - 1);
        let preview_scale = (preview_area.height() - 20.0) / PAGE_H;
        let mini_page_w = PAGE_W * preview_scale;
        let mini_page_h = PAGE_H * preview_scale;
        let page_x = preview_area.x0 + (preview_area.width() - mini_page_w) / 2.0;
        let page_y = preview_area.y0 + 10.0;

        let page_rect = Rect::new(page_x, page_y, page_x + mini_page_w, page_y + mini_page_h);
        p.fill_rounded_rect(page_rect, Color::rgb8(0xFF, 0xFF, 0xFF), 2.0);
        p.stroke_rounded_rect(page_rect, c_separator(), 1.0, 2.0);

        // Draw content lines on the miniature page
        let line_h = 3.0;
        let pad = 20.0 * preview_scale;
        let mut line_y = page_y + pad;
        let page_map = state.layout_cache.page_map();
        let start_block = page_map.get(page_idx).map(|e| e.start_block).unwrap_or(0);
        let end_block = page_map
            .get(page_idx + 1)
            .map(|e| e.start_block)
            .unwrap_or(state.current_document().content.len());
        let doc = state.current_document();

        for block_idx in start_block..end_block {
            if line_y + line_h > page_y + mini_page_h - pad {
                break;
            }
            if let Some(block) = doc.content.get(block_idx) {
                let is_heading = matches!(block, BlockNode::Heading { level, .. } if *level <= 2);
                let line_w = if is_heading {
                    mini_page_w - pad * 2.0
                } else {
                    (mini_page_w - pad * 2.0) * 0.85
                };
                let color = Color::rgb8(0xCC, 0xCC, 0xCC);
                p.fill_rect(
                    Rect::new(page_x + pad, line_y, page_x + pad + line_w, line_y + line_h),
                    color,
                );
            }
            line_y += line_h + 2.0;
        }

        // Page indicator
        p.draw_text_cached(
            cache,
            &format!("Page {} of {}", page_idx + 1, num_pages),
            modal_x + modal_w / 2.0,
            modal_y + modal_h - 30.0,
            c_text_light(),
            12.0,
            FontWeight::NORMAL,
            true,
            false,
        );
    }

    // Navigation buttons
    let prev_btn = layout.prev_btn;
    p.fill_rounded_rect(prev_btn, c_btn_hover(), 4.0);
    p.stroke_rounded_rect(prev_btn, c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "Prev",
        prev_btn.x0 + prev_btn.width() / 2.0,
        prev_btn.y0 + prev_btn.height() / 2.0,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        true,
        false,
    );

    // Next button
    let next_btn = layout.next_btn;
    p.fill_rounded_rect(next_btn, c_btn_hover(), 4.0);
    p.stroke_rounded_rect(next_btn, c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "Next",
        next_btn.x0 + next_btn.width() / 2.0,
        next_btn.y0 + next_btn.height() / 2.0,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        true,
        false,
    );

    // Print button
    let print_btn = layout.print_btn;
    p.fill_rounded_rect(print_btn, c_accent(), 4.0);
    p.draw_text_cached(
        cache,
        "Print",
        print_btn.x0 + print_btn.width() / 2.0,
        print_btn.y0 + print_btn.height() / 2.0,
        Color::rgb8(0xFF, 0xFF, 0xFF),
        11.0,
        FontWeight::BOLD,
        true,
        false,
    );

    // Close button (X) at top right
    let close_rect = layout.close;
    p.draw_text_cached(
        cache,
        "X",
        close_rect.x0 + close_rect.width() / 2.0,
        close_rect.y0 + close_rect.height() / 2.0,
        c_text_dim(),
        14.0,
        FontWeight::NORMAL,
        false,
        false,
    );
}

/// Paint the word count dialog modal.
pub fn paint_word_count_modal(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    win_size: Size,
    state: &DocsState,
) {
    let modal_w = 360.0;
    let modal_h = 320.0;
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
    let modal = Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
    p.fill_rounded_rect(modal, c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, c_separator(), 1.0, 6.0);

    // Title
    p.draw_text_cached(
        cache,
        "Word Count",
        modal_x + 16.0,
        modal_y + 20.0,
        c_text_light(),
        14.0,
        FontWeight::BOLD,
        false,
        false,
    );

    // Statistics
    let plain_text = state.current_document().to_plain_text();
    let chars_with_spaces = plain_text.chars().count();
    let chars_without_spaces = plain_text.chars().filter(|c| !c.is_whitespace()).count();
    let stats = [
        ("Words", state.word_count.to_string()),
        ("Characters (with spaces)", chars_with_spaces.to_string()),
        (
            "Characters (without spaces)",
            chars_without_spaces.to_string(),
        ),
        ("Paragraphs", state.paragraph_count().to_string()),
        ("Pages", state.page_count.to_string()),
        ("Read time", format!("{} min", state.read_time_minutes())),
    ];

    let mut y = modal_y + 48.0;
    for (label, value) in &stats {
        p.draw_text_cached(
            cache,
            label,
            modal_x + 20.0,
            y,
            c_text_dim(),
            12.0,
            FontWeight::NORMAL,
            false,
            false,
        );
        p.draw_text_cached(
            cache,
            value,
            modal_x + modal_w - 20.0,
            y,
            c_text_light(),
            12.0,
            FontWeight::BOLD,
            false,
            false,
        );
        // Separator line
        y += 18.0;
        p.draw_line(
            Point::new(modal_x + 20.0, y),
            Point::new(modal_x + modal_w - 20.0, y),
            c_separator(),
            1.0,
        );
        y += 12.0;
    }

    // Close button
    let close_btn = Rect::new(
        modal_x + modal_w / 2.0 - 40.0,
        modal_y + modal_h - 44.0,
        modal_x + modal_w / 2.0 + 40.0,
        modal_y + modal_h - 16.0,
    );
    p.fill_rounded_rect(close_btn, c_btn_hover(), 4.0);
    p.stroke_rounded_rect(close_btn, c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "Close",
        close_btn.x0 + close_btn.width() / 2.0,
        close_btn.y0 + close_btn.height() / 2.0,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        true,
        false,
    );
}

/// Paint the goto page/line modal.
pub fn paint_goto_modal(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    win_size: Size,
    goto_state: &GotoModalState,
    cursor_visible: bool,
) {
    use super::super::layout::modal::compute_goto;

    let layout = compute_goto(win_size);
    let modal = layout.modal;
    let modal_x = modal.x0;
    let modal_y = modal.y0;
    let _modal_w = modal.width();
    let modal_h = modal.height();

    // Dim overlay
    p.fill_rect(
        Rect::new(0.0, 0.0, win_size.width, win_size.height),
        Color::rgba8(0, 0, 0, 120),
    );

    p.fill_rounded_rect(modal, c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, c_separator(), 1.0, 6.0);

    // Title
    let title = match goto_state.mode {
        GotoMode::Page => "Go to Page",
        GotoMode::Line => "Go to Line",
    };
    p.draw_text_cached(
        cache,
        title,
        modal_x + 16.0,
        modal_y + 20.0,
        c_text_light(),
        14.0,
        FontWeight::BOLD,
        false,
        false,
    );

    // Mode toggle buttons
    let page_btn = layout.page_mode;
    let line_btn = layout.line_mode;

    let is_page = goto_state.mode == GotoMode::Page;
    p.fill_rounded_rect(
        page_btn,
        if is_page { c_accent() } else { c_btn_hover() },
        4.0,
    );
    p.draw_text_cached(
        cache,
        "Page",
        page_btn.x0 + page_btn.width() / 2.0,
        page_btn.y0 + page_btn.height() / 2.0,
        if is_page {
            Color::rgb8(0xFF, 0xFF, 0xFF)
        } else {
            c_text_light()
        },
        11.0,
        FontWeight::BOLD,
        true,
        false,
    );
    p.fill_rounded_rect(
        line_btn,
        if !is_page { c_accent() } else { c_btn_hover() },
        4.0,
    );
    p.draw_text_cached(
        cache,
        "Line",
        line_btn.x0 + line_btn.width() / 2.0,
        line_btn.y0 + line_btn.height() / 2.0,
        if !is_page {
            Color::rgb8(0xFF, 0xFF, 0xFF)
        } else {
            c_text_light()
        },
        11.0,
        FontWeight::BOLD,
        true,
        false,
    );

    // Input field
    let input_field = layout.input_field;
    let field_x = input_field.x0;
    let field_y = input_field.y0;
    let _field_w = input_field.width();
    let field_h = input_field.height();
    p.fill_rect(input_field, Color::rgb8(0x25, 0x25, 0x25));
    p.stroke_rounded_rect(input_field, c_accent(), 1.0, 0.0);

    let placeholder = match goto_state.mode {
        GotoMode::Page => "Enter page number...",
        GotoMode::Line => "Enter line number...",
    };
    let display_text = if goto_state.input.is_empty() {
        placeholder.to_string()
    } else {
        goto_state.input.clone()
    };
    let text_color = if goto_state.input.is_empty() {
        c_text_dim()
    } else {
        c_text_light()
    };
    p.draw_text_cached(
        cache,
        &display_text,
        field_x + 8.0,
        field_y + 7.0,
        text_color,
        13.0,
        FontWeight::NORMAL,
        false,
        false,
    );

    if cursor_visible && !goto_state.input.is_empty() {
        let text_before = &goto_state.input[..goto_state.cursor_pos];
        let cursor_x = field_x + 8.0 + text_before.len() as f64 * 7.5;
        p.fill_rect(
            Rect::new(
                cursor_x,
                field_y + 4.0,
                cursor_x + 1.5,
                field_y + field_h - 4.0,
            ),
            c_accent(),
        );
    }

    // Hint
    p.draw_text_cached(
        cache,
        "Press Enter to go, Esc to cancel",
        modal_x + 16.0,
        modal_y + modal_h - 28.0,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
}

/// Paint the special character modal.
pub fn paint_special_char_modal(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    win_size: Size,
    modal_state: &SpecialCharModalState,
) {
    let modal_w = 420.0;
    let modal_h = 380.0;
    let modal_x = win_size.width / 2.0 - modal_w / 2.0;
    let modal_y = win_size.height / 2.0 - modal_h / 2.0;

    // Dim overlay
    p.fill_rect(
        Rect::new(0.0, 0.0, win_size.width, win_size.height),
        Color::rgba8(0, 0, 0, 120),
    );

    let modal = Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
    p.fill_rounded_rect(modal, c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, c_separator(), 1.0, 6.0);

    // Title
    p.draw_text_cached(
        cache,
        "Insert Special Character",
        modal_x + 16.0,
        modal_y + 20.0,
        c_text_light(),
        14.0,
        FontWeight::BOLD,
        false,
        false,
    );

    // Category tabs
    let tab_y = modal_y + 36.0;
    let mut tab_x = modal_x + 12.0;
    for (idx, (cat_name, _)) in SPECIAL_CHAR_CATEGORIES.iter().enumerate() {
        let is_active = idx == modal_state.category_idx;
        let tab_w = cat_name.len() as f64 * 7.0 + 16.0;
        let tab_rect = Rect::new(tab_x, tab_y, tab_x + tab_w, tab_y + 22.0);
        p.fill_rounded_rect(
            tab_rect,
            if is_active { c_accent() } else { c_btn_hover() },
            4.0,
        );
        p.draw_text_cached(
            cache,
            cat_name,
            tab_rect.x0 + 8.0,
            tab_rect.y0 + 12.0,
            if is_active {
                Color::rgb8(0xFF, 0xFF, 0xFF)
            } else {
                c_text_dim()
            },
            10.0,
            FontWeight::NORMAL,
            false,
            false,
        );
        tab_x += tab_w + 4.0;
    }

    // Character grid
    if let Some((_cat_name, chars)) = SPECIAL_CHAR_CATEGORIES.get(modal_state.category_idx) {
        let grid_x = modal_x + 16.0;
        let grid_y = tab_y + 30.0;
        let cell_size = 32.0;
        let cols = ((modal_w - 32.0) / cell_size) as usize;

        for (i, &ch) in chars.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let cell_x = grid_x + col as f64 * cell_size;
            let cell_y = grid_y + row as f64 * cell_size;
            if cell_y + cell_size > modal_y + modal_h - 40.0 {
                break;
            }
            let cell = Rect::new(cell_x, cell_y, cell_x + cell_size, cell_y + cell_size);
            p.fill_rounded_rect(cell, c_btn_hover(), 2.0);
            p.stroke_rounded_rect(cell, c_separator(), 1.0, 2.0);

            let ch_str = ch.to_string();
            p.draw_text_cached(
                cache,
                &ch_str,
                cell.x0 + cell_size / 2.0,
                cell.y0 + cell_size / 2.0,
                c_text_light(),
                14.0,
                FontWeight::NORMAL,
                true,
                false,
            );
        }
    }

    // Close button (bottom-right)
    let close_btn = Rect::new(
        modal_x + modal_w - 92.0,
        modal_y + modal_h - 42.0,
        modal_x + modal_w - 16.0,
        modal_y + modal_h - 14.0,
    );
    p.fill_rounded_rect(close_btn, c_accent(), 4.0);
    p.draw_text_cached(
        cache,
        "Close",
        close_btn.x0 + close_btn.width() / 2.0,
        close_btn.y0 + close_btn.height() / 2.0,
        Color::rgb8(0xFF, 0xFF, 0xFF),
        11.0,
        FontWeight::NORMAL,
        true,
        false,
    );

    // Close hint
    p.draw_text_cached(
        cache,
        "Click a character to insert, Esc to cancel",
        modal_x + 16.0,
        modal_y + modal_h - 20.0,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
}
