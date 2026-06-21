use super::*;

pub(super) fn paint_title_row(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &DocsState,
) {
    p.fill_rect(rect, c_canvas_bg());
    p.draw_text_cached(
        cache,
        state.title(),
        rect.x0 + 20.0,
        rect.y0 + rect.height() / 2.0,
        c_text_light(),
        18.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );
    p.draw_text_cached(
        cache,
        state.status_line(),
        rect.x0 + 190.0,
        rect.y0 + rect.height() / 2.0,
        c_text_dim(),
        12.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );
    p.draw_line(
        Point::new(rect.x0, rect.y1 - 1.0),
        Point::new(rect.x1, rect.y1 - 1.0),
        c_separator(),
        1.0,
    );
}

pub(super) fn menu_at(x: f64) -> Option<&'static str> {
    let mut left = 12.0;
    for name in ["File", "Edit", "View", "Insert", "Format", "Tools", "Help"] {
        let width = match name {
            "Insert" | "Format" => 54.0,
            _ => 42.0,
        };
        if x >= left && x < left + width {
            return Some(name);
        }
        left += width;
    }
    None
}

/// Return the menu items for a given menu name.
pub(super) fn menu_items_for(menu_name: &str) -> Vec<&'static str> {
    match menu_name {
        "File" => vec![
            "New",
            "Open",
            "Save",
            "Save As",
            "Export As",
            "Page Setup",
            "Print",
            "Version History",
            "Recent Files",
        ],
        "Edit" => vec![
            "Undo",
            "Redo",
            "Cut",
            "Copy",
            "Paste",
            "Select All",
            "Find",
            "Replace",
            "Go To",
        ],
        "View" => vec![
            "Thumbnails",
            "Style Panel",
            "Comments",
            "Zoom In",
            "Zoom Out",
            "Reset Zoom",
        ],
        "Insert" => vec![
            "Image",
            "Table",
            "Link",
            "Horizontal Rule",
            "Page Break",
            "Header",
            "Footer",
            "Special Character",
            "Footnote",
        ],
        "Format" => vec![
            "Bold",
            "Italic",
            "Underline",
            "Strikethrough",
            "Superscript",
            "Subscript",
            "Clear Formatting",
            "Block Quote",
        ],
        "Tools" => vec!["Word Count", "Track Changes", "Spell Check"],
        "Help" => vec!["About", "Keyboard Shortcuts"],
        _ => vec![],
    }
}

fn menu_shortcut(item: &str) -> Option<&'static str> {
    match item {
        "New" => Some("Ctrl+N"),
        "Open" => Some("Ctrl+O"),
        "Save" => Some("Ctrl+S"),
        "Save As" => Some("Ctrl+Shift+S"),
        "Undo" => Some("Ctrl+Z"),
        "Redo" => Some("Ctrl+Y"),
        "Cut" => Some("Ctrl+X"),
        "Copy" => Some("Ctrl+C"),
        "Paste" => Some("Ctrl+V"),
        "Select All" => Some("Ctrl+A"),
        "Find" => Some("Ctrl+F"),
        "Replace" => Some("Ctrl+H"),
        "Go To" => Some("Ctrl+G"),
        "Bold" => Some("Ctrl+B"),
        "Italic" => Some("Ctrl+I"),
        "Underline" => Some("Ctrl+U"),
        "Zoom In" => Some("Ctrl++"),
        "Zoom Out" => Some("Ctrl+-"),
        "Reset Zoom" => Some("Ctrl+0"),
        "Word Count" => Some("Ctrl+Shift+C"),
        "Special Character" => Some("Ctrl+Shift+S"),
        _ => None,
    }
}

pub(super) fn is_info_modal(title: &str) -> bool {
    matches!(title, "About" | "Keyboard Shortcuts")
}

pub(super) fn info_modal_rect(size: Size) -> Rect {
    let modal_w = 420.0;
    let modal_h = 190.0;
    Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    )
}

pub(super) fn info_modal_close_rect(size: Size) -> Rect {
    let modal = info_modal_rect(size);
    Rect::new(
        modal.x1 - 92.0,
        modal.y1 - 42.0,
        modal.x1 - 16.0,
        modal.y1 - 14.0,
    )
}

pub(super) fn paint_docs_modal(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    size: Size,
    title: &str,
    hovered_item: Option<usize>,
) {
    if is_info_modal(title) {
        paint_info_modal(p, cache, size, title);
        return;
    }

    let items = menu_items_for(title);
    let item_h = 26.0;
    let modal_w = 220.0;
    let modal_h = 8.0 + items.len() as f64 * item_h + 8.0;
    let modal_x = 12.0; // aligns with menu bar left padding
    let modal_y = MENU_BAR_H;

    let modal = Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
    p.fill_rounded_rect(modal, state::c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, state::c_separator(), 1.0, 6.0);

    for (idx, item) in items.iter().enumerate() {
        let is_hovered = hovered_item == Some(idx);
        if is_hovered {
            let item_rect = Rect::new(
                modal_x + 2.0,
                modal_y + 8.0 + idx as f64 * item_h,
                modal_x + modal_w - 2.0,
                modal_y + 8.0 + (idx as f64 + 1.0) * item_h,
            );
            p.fill_rounded_rect(item_rect, state::c_btn_hover(), 4.0);
        }
        let text_color = if is_hovered {
            state::c_text_light()
        } else {
            state::c_text_dim()
        };
        p.draw_text_cached(
            cache,
            item,
            modal_x + 14.0,
            modal_y + 8.0 + idx as f64 * item_h + item_h / 2.0,
            text_color,
            12.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
        // Submenu arrow for items with submenus
        if item == &"Recent Files" {
            p.draw_text_cached(
                cache,
                "\u{25B6}",
                modal_x + modal_w - 14.0,
                modal_y + 8.0 + idx as f64 * item_h + item_h / 2.0,
                state::c_text_dim(),
                10.0,
                tench_ui::parley::FontWeight::NORMAL,
                true,
                false,
            );
        } else if let Some(shortcut) = menu_shortcut(item) {
            p.draw_text_cached(
                cache,
                shortcut,
                modal_x + modal_w - 14.0,
                modal_y + 8.0 + idx as f64 * item_h + item_h / 2.0,
                state::c_text_dim(),
                10.0,
                tench_ui::parley::FontWeight::NORMAL,
                true,
                false,
            );
        }
    }
}

fn paint_info_modal(p: &mut Painter<'_>, cache: &mut TextCache, size: Size, title: &str) {
    let modal = info_modal_rect(size);
    let backdrop = Rect::new(0.0, 0.0, size.width, size.height);
    p.fill_rect(backdrop, Color::rgba8(0, 0, 0, 100));
    p.fill_rounded_rect(modal, state::c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, state::c_separator(), 1.0, 6.0);

    let (heading, lines): (&str, &[&str]) = match title {
        "Keyboard Shortcuts" => (
            "Keyboard Shortcuts",
            &[
                "Ctrl+B / Ctrl+I / Ctrl+U: text formatting",
                "Ctrl+S: save document",
                "Ctrl+F / Ctrl+H: find and replace",
            ],
        ),
        _ => (
            "About Tench Docs",
            &[
                "Tench Docs v0.1.0",
                "Native Rust document editor",
                "Local-first document workspace",
            ],
        ),
    };

    p.draw_text_cached(
        cache,
        heading,
        modal.x0 + 22.0,
        modal.y0 + 34.0,
        state::c_text_light(),
        15.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );

    for (idx, line) in lines.iter().enumerate() {
        p.draw_text_cached(
            cache,
            line,
            modal.x0 + 22.0,
            modal.y0 + 72.0 + idx as f64 * 24.0,
            state::c_text_dim(),
            12.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
    }

    let close = info_modal_close_rect(size);
    p.fill_rounded_rect(close, state::c_accent(), 4.0);
    p.draw_text_cached(
        cache,
        "Close",
        close.x0 + close.width() / 2.0,
        close.y0 + close.height() / 2.0,
        Color::WHITE,
        11.0,
        tench_ui::parley::FontWeight::BOLD,
        true,
        false,
    );
}

pub(super) fn paint_link_modal(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    size: Size,
    link_state: &LinkModalState,
) {
    let modal_w = 380.0;
    let modal_h = 160.0;
    let modal = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );

    // Semi-transparent backdrop
    let backdrop = Rect::new(0.0, 0.0, size.width, size.height);
    p.fill_rect(backdrop, Color::rgba8(0, 0, 0, 100));

    p.fill_rounded_rect(modal, state::c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, state::c_separator(), 1.0, 6.0);

    // Title
    p.draw_text_cached(
        cache,
        "Insert Hyperlink",
        modal.x0 + 16.0,
        modal.y0 + 28.0,
        state::c_text_light(),
        14.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );

    // URL label
    p.draw_text_cached(
        cache,
        "URL:",
        modal.x0 + 16.0,
        modal.y0 + 58.0,
        state::c_text_dim(),
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    // URL input field
    let input_rect = Rect::new(
        modal.x0 + 16.0,
        modal.y0 + 68.0,
        modal.x1 - 16.0,
        modal.y0 + 94.0,
    );
    p.fill_rounded_rect(input_rect, Color::rgba8(0x0A, 0x0A, 0x0A, 255), 4.0);
    p.stroke_rounded_rect(input_rect, state::c_separator(), 1.0, 4.0);

    // URL text with cursor
    let display_text = if link_state.url.is_empty() {
        "https://".to_string()
    } else {
        link_state.url.clone()
    };
    let text_color = if link_state.url.is_empty() {
        state::c_text_dim()
    } else {
        state::c_text_light()
    };
    p.draw_text_cached(
        cache,
        &display_text,
        input_rect.x0 + 8.0,
        input_rect.y0 + input_rect.height() / 2.0,
        text_color,
        12.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    // OK button
    let ok_rect = Rect::new(
        modal.x1 - 160.0,
        modal.y1 - 40.0,
        modal.x1 - 90.0,
        modal.y1 - 12.0,
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
        modal.x1 - 80.0,
        modal.y1 - 40.0,
        modal.x1 - 12.0,
        modal.y1 - 12.0,
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

pub(super) fn paint_find_replace_modal(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    size: Size,
    fr_state: &FindReplaceState,
) {
    use super::layout::modal::compute_find_replace;

    let layout = compute_find_replace(size, fr_state.show_replace);
    let modal = layout.modal;
    let modal_x = modal.x0;
    let modal_y = modal.y0;
    let modal_w = modal.width();
    let modal_h = modal.height();

    // Semi-transparent backdrop
    let backdrop = Rect::new(0.0, 0.0, size.width, size.height);
    p.fill_rect(backdrop, Color::rgba8(0, 0, 0, 100));

    p.fill_rounded_rect(modal, state::c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, state::c_separator(), 1.0, 6.0);

    // Title
    let title = if fr_state.show_replace {
        "Find and Replace"
    } else {
        "Find"
    };
    p.draw_text_cached(
        cache,
        title,
        modal_x + 16.0,
        modal_y + 22.0,
        state::c_text_light(),
        14.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );

    // Match count
    let match_info = if fr_state.matches.is_empty() {
        "No results".to_string()
    } else {
        let current = fr_state.current_match_idx.map(|i| i + 1).unwrap_or(0);
        format!("{}/{} matches", current, fr_state.matches.len())
    };
    p.draw_text_cached(
        cache,
        &match_info,
        modal_x + modal_w - 16.0,
        modal_y + 22.0,
        state::c_text_dim(),
        10.0,
        tench_ui::parley::FontWeight::NORMAL,
        true,
        false,
    );

    // Search input
    p.draw_text_cached(
        cache,
        "Find:",
        modal_x + 16.0,
        modal_y + 48.0,
        state::c_text_dim(),
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );
    let search_input = Rect::new(
        modal_x + 60.0,
        modal_y + 38.0,
        modal_x + modal_w - 16.0,
        modal_y + 60.0,
    );
    p.fill_rounded_rect(search_input, Color::rgba8(0x0A, 0x0A, 0x0A, 255), 4.0);
    p.stroke_rounded_rect(search_input, state::c_separator(), 1.0, 4.0);
    let query_display = if fr_state.query.is_empty() {
        "Type to search..."
    } else {
        &fr_state.query
    };
    let query_color = if fr_state.query.is_empty() {
        state::c_text_dim()
    } else {
        state::c_text_light()
    };
    p.draw_text_cached(
        cache,
        query_display,
        search_input.x0 + 8.0,
        search_input.y0 + search_input.height() / 2.0,
        query_color,
        12.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    // Replace input (only if show_replace)
    if fr_state.show_replace {
        p.draw_text_cached(
            cache,
            "Replace:",
            modal_x + 16.0,
            modal_y + 78.0,
            state::c_text_dim(),
            11.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
        let replace_input = Rect::new(
            modal_x + 60.0,
            modal_y + 68.0,
            modal_x + modal_w - 16.0,
            modal_y + 90.0,
        );
        p.fill_rounded_rect(replace_input, Color::rgba8(0x0A, 0x0A, 0x0A, 255), 4.0);
        p.stroke_rounded_rect(replace_input, state::c_separator(), 1.0, 4.0);
        let repl_display = if fr_state.replacement.is_empty() {
            "Replacement text..."
        } else {
            &fr_state.replacement
        };
        let repl_color = if fr_state.replacement.is_empty() {
            state::c_text_dim()
        } else {
            state::c_text_light()
        };
        p.draw_text_cached(
            cache,
            repl_display,
            replace_input.x0 + 8.0,
            replace_input.y0 + replace_input.height() / 2.0,
            repl_color,
            12.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
    }

    // Buttons
    let _btn_y = layout.btn_row_y;
    let _btn_h = layout.btn_h;

    // Find Next button
    let find_next_rect = layout.find_next;
    p.fill_rounded_rect(find_next_rect, state::c_accent(), 4.0);
    p.draw_text_cached(
        cache,
        "Find Next",
        find_next_rect.x0 + find_next_rect.width() / 2.0,
        find_next_rect.y0 + find_next_rect.height() / 2.0,
        Color::WHITE,
        10.0,
        tench_ui::parley::FontWeight::BOLD,
        true,
        false,
    );

    // Find Prev button
    let find_prev_rect = layout.find_prev;
    p.stroke_rounded_rect(find_prev_rect, state::c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "Find Prev",
        find_prev_rect.x0 + find_prev_rect.width() / 2.0,
        find_prev_rect.y0 + find_prev_rect.height() / 2.0,
        state::c_text_light(),
        10.0,
        tench_ui::parley::FontWeight::NORMAL,
        true,
        false,
    );

    if let Some(replace_rect) = layout.replace {
        // Replace button
        p.stroke_rounded_rect(replace_rect, state::c_separator(), 1.0, 4.0);
        p.draw_text_cached(
            cache,
            "Replace",
            replace_rect.x0 + replace_rect.width() / 2.0,
            replace_rect.y0 + replace_rect.height() / 2.0,
            state::c_text_light(),
            10.0,
            tench_ui::parley::FontWeight::NORMAL,
            true,
            false,
        );

        // Replace All button
        if let Some(replace_all_rect) = layout.replace_all {
            p.stroke_rounded_rect(replace_all_rect, state::c_separator(), 1.0, 4.0);
            p.draw_text_cached(
                cache,
                "Replace All",
                replace_all_rect.x0 + replace_all_rect.width() / 2.0,
                replace_all_rect.y0 + replace_all_rect.height() / 2.0,
                state::c_text_light(),
                10.0,
                tench_ui::parley::FontWeight::NORMAL,
                true,
                false,
            );
        }
    }

    // Aa (case sensitive) toggle - right side of button row
    let aa_rect = layout.case_sensitive;
    let aa_bg = if fr_state.case_sensitive {
        state::c_accent()
    } else {
        Color::TRANSPARENT
    };
    if fr_state.case_sensitive {
        p.fill_rounded_rect(aa_rect, aa_bg, 4.0);
    }
    p.stroke_rounded_rect(aa_rect, state::c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "Aa",
        aa_rect.x0 + aa_rect.width() / 2.0,
        aa_rect.y0 + aa_rect.height() / 2.0,
        if fr_state.case_sensitive {
            Color::WHITE
        } else {
            state::c_text_dim()
        },
        10.0,
        tench_ui::parley::FontWeight::BOLD,
        true,
        false,
    );

    // .* (regex) toggle
    let regex_rect = layout.regex;
    let regex_bg = if fr_state.use_regex {
        state::c_accent()
    } else {
        Color::TRANSPARENT
    };
    if fr_state.use_regex {
        p.fill_rounded_rect(regex_rect, regex_bg, 4.0);
    }
    p.stroke_rounded_rect(regex_rect, state::c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        ".*",
        regex_rect.x0 + regex_rect.width() / 2.0,
        regex_rect.y0 + regex_rect.height() / 2.0,
        if fr_state.use_regex {
            Color::WHITE
        } else {
            state::c_text_dim()
        },
        10.0,
        tench_ui::parley::FontWeight::BOLD,
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
        state::c_text_dim(),
        11.0,
        tench_ui::parley::FontWeight::BOLD,
        true,
        false,
    );

    // Keyboard hints
    let hint_y = modal_y + modal_h - 20.0;
    p.draw_text_cached(
        cache,
        "Enter: Find Next | Esc: Close",
        modal_x + 16.0,
        hint_y,
        state::c_text_dim(),
        9.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );
}

pub(super) fn paint_docs_toast(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    size: Size,
    message: &str,
) {
    let rect = Rect::new(
        size.width - 260.0,
        MENU_BAR_H + TOOLBAR_H + 12.0,
        size.width - 16.0,
        MENU_BAR_H + TOOLBAR_H + 48.0,
    );
    p.fill_rounded_rect(rect, state::c_menu_bg(), 6.0);
    p.stroke_rounded_rect(rect, state::c_separator(), 1.0, 6.0);
    p.draw_text_cached(
        cache,
        message,
        rect.x0 + 12.0,
        rect.y0 + 23.0,
        state::c_text_light(),
        11.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );
}
