use super::*;

/// Paint the tab bar for multi-document support.
pub(super) fn paint_tab_bar(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &KodocsState,
) {
    p.fill_rect(rect, c_canvas_bg());
    p.draw_line(
        Point::new(rect.x0, rect.y1 - 1.0),
        Point::new(rect.x1, rect.y1 - 1.0),
        c_separator(),
        1.0,
    );

    let tab_w: f64 = 160.0;
    let tab_h = rect.height();
    let close_btn_w = 20.0;
    let tab_x_start = 8.0;

    for (idx, tab) in state.open_tabs.iter().enumerate() {
        let tab_x = rect.x0 + tab_x_start + idx as f64 * (tab_w + 2.0);
        let tab_rect = Rect::new(tab_x, rect.y0, tab_x + tab_w, rect.y0 + tab_h);

        let is_active = idx == state.active_tab_idx;
        let bg = if is_active {
            Color::rgb8(0x25, 0x25, 0x25)
        } else {
            c_canvas_bg()
        };
        p.fill_rect(tab_rect, bg);
        if is_active {
            // Active tab indicator line at top
            p.draw_line(
                Point::new(tab_rect.x0, tab_rect.y0),
                Point::new(tab_rect.x1, tab_rect.y0),
                state::c_accent(),
                2.0,
            );
        } else {
            p.draw_line(
                Point::new(tab_rect.x0, tab_rect.y0),
                Point::new(tab_rect.x1, tab_rect.y0),
                c_separator(),
                1.0,
            );
        }

        // Tab title (truncate if needed, safe UTF-8 boundary)
        let title = if tab.title.chars().count() > 12 {
            let truncated: String = tab.title.chars().take(10).collect();
            if tab.dirty {
                format!("{}*", truncated)
            } else {
                format!("{}...", truncated)
            }
        } else if tab.dirty {
            format!("{} *", tab.title)
        } else {
            tab.title.clone()
        };
        let text_color = if is_active {
            c_text_light()
        } else {
            c_text_dim()
        };
        p.draw_text_cached(
            cache,
            &title,
            tab_rect.x0 + 8.0,
            tab_rect.y0 + tab_h / 2.0,
            text_color,
            11.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );

        // Close button (X) - only show for non-active tabs or if more than 1 tab
        if state.open_tabs.len() > 1 {
            let close_rect = Rect::new(
                tab_rect.x1 - close_btn_w - 4.0,
                tab_rect.y0 + (tab_h - 14.0) / 2.0,
                tab_rect.x1 - 4.0,
                tab_rect.y0 + (tab_h + 14.0) / 2.0,
            );
            p.draw_text_cached(
                cache,
                "x",
                close_rect.x0 + close_rect.width() / 2.0,
                close_rect.y0 + close_rect.height() / 2.0,
                c_text_dim(),
                10.0,
                tench_ui::parley::FontWeight::NORMAL,
                true,
                false,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Dropdown hit testing
// ---------------------------------------------------------------------------

pub(super) const FONT_FAMILY_SELECT_W: f64 = 120.0;
pub(super) const FONT_SIZE_SELECT_W: f64 = 62.0;
pub(super) const PARAGRAPH_SELECT_W: f64 = 112.0;
pub(super) const DROPDOWN_ITEM_H: f64 = 26.0;

/// Returns the x-position where the font family select starts in the toolbar.
pub(super) fn font_family_select_x() -> f64 {
    12.0 + 48.0 + 2.0 + 48.0 + 2.0 + 14.0
}

/// Returns the x-position where the font size select starts in the toolbar.
pub(super) fn font_size_select_x() -> f64 {
    font_family_select_x() + FONT_FAMILY_SELECT_W + 2.0 + 14.0
}

/// Returns the x-position where the paragraph style select starts in the toolbar.
pub(super) fn paragraph_style_select_x() -> f64 {
    font_size_select_x() + FONT_SIZE_SELECT_W + 2.0 + 14.0
}

/// If a font family dropdown item was clicked, return its index.
pub(super) fn font_family_dropdown_hit(x: f64, y: f64) -> Option<usize> {
    let ff_x = font_family_select_x();
    let dropdown_top = MENU_BAR_H + 8.0 + 32.0;
    if x < ff_x || x > ff_x + FONT_FAMILY_SELECT_W || y < dropdown_top {
        return None;
    }
    let rel_y = y - dropdown_top - 2.0;
    if rel_y < 0.0 {
        return None;
    }
    let idx = (rel_y / DROPDOWN_ITEM_H) as usize;
    if idx < FONT_FAMILIES.len() {
        Some(idx)
    } else {
        None
    }
}

/// If a font size dropdown item was clicked, return its index.
pub(super) fn font_size_dropdown_hit(x: f64, y: f64) -> Option<usize> {
    let fs_x = font_size_select_x();
    let dropdown_top = MENU_BAR_H + 8.0 + 32.0; // y of toolbar buttons + button height
    if x < fs_x || x > fs_x + FONT_SIZE_SELECT_W || y < dropdown_top {
        return None;
    }
    let rel_y = y - dropdown_top - 2.0;
    if rel_y < 0.0 {
        return None;
    }
    let idx = (rel_y / DROPDOWN_ITEM_H) as usize;
    if idx < FONT_SIZES.len() {
        Some(idx)
    } else {
        None
    }
}

/// If a paragraph style dropdown item was clicked, return its index.
pub(super) fn paragraph_style_dropdown_hit(x: f64, y: f64) -> Option<usize> {
    let ps_x = paragraph_style_select_x();
    let dropdown_top = MENU_BAR_H + 8.0 + 32.0;
    if x < ps_x || x > ps_x + PARAGRAPH_SELECT_W || y < dropdown_top {
        return None;
    }
    let rel_y = y - dropdown_top - 2.0;
    if rel_y < 0.0 {
        return None;
    }
    let idx = (rel_y / DROPDOWN_ITEM_H) as usize;
    if idx < ParagraphStyle::all().len() {
        Some(idx)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Table grid picker hit testing
// ---------------------------------------------------------------------------

pub(super) const TABLE_GRID_SIZE: usize = 5;
pub(super) const TABLE_GRID_CELL: f64 = 22.0;
pub(super) const TABLE_GRID_PAD: f64 = 4.0;

/// Returns the x-position where the Tbl button starts in the toolbar.
pub(super) fn table_grid_x() -> f64 {
    // Walk through all toolbar groups to find the Tbl button position
    let mut left = 12.0;
    // Group 1: Undo/Redo
    left += 48.0 + 2.0 + 48.0 + 2.0 + 14.0;
    // Group 2: Font family select
    left += FONT_FAMILY_SELECT_W + 2.0 + 14.0;
    // Group 3: Font size select
    left += FONT_SIZE_SELECT_W + 2.0 + 14.0;
    // Group 4: Paragraph style select
    left += PARAGRAPH_SELECT_W + 2.0 + 14.0;
    // Group 5: Format buttons (8 buttons)
    left += (32.0 + 2.0) * 7.0 + 36.0 + 2.0 + 14.0;
    // Group 6: List buttons (5 buttons)
    left += 28.0 + 2.0 + 28.0 + 2.0 + 28.0 + 2.0 + 32.0 + 2.0 + 32.0 + 2.0 + 14.0;
    // Group 7: Alignment buttons (4 buttons)
    left += (28.0 + 2.0) * 4.0 + 14.0;
    // Group 8: Link, Img, Tbl
    left += 42.0 + 2.0 + 38.0 + 2.0; // past Link and Img
    left
}

/// If a table grid cell was clicked/hovered, return (rows, cols) (1-indexed).
pub(super) fn table_grid_hit(x: f64, y: f64) -> Option<(usize, usize)> {
    let grid_x = table_grid_x();
    let dropdown_top = MENU_BAR_H + 8.0 + 32.0;
    let grid_y = dropdown_top + TABLE_GRID_PAD;

    let rel_x = x - grid_x - TABLE_GRID_PAD;
    let rel_y = y - grid_y;

    if rel_x < 0.0 || rel_y < 0.0 {
        return None;
    }

    let col = (rel_x / TABLE_GRID_CELL) as usize;
    let row = (rel_y / TABLE_GRID_CELL) as usize;

    if col < TABLE_GRID_SIZE && row < TABLE_GRID_SIZE {
        Some((row + 1, col + 1))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Color picker hit testing
// ---------------------------------------------------------------------------

pub(super) const COLOR_CELL_SIZE: f64 = 20.0;
pub(super) const COLOR_COLS: usize = 10;

/// Returns the x-position where the Color button starts in the toolbar.
pub(super) fn color_picker_x() -> f64 {
    let mut left = 12.0;
    // Group 1: Undo/Redo
    left += 48.0 + 2.0 + 48.0 + 2.0 + 14.0;
    // Group 2: Font family select
    left += FONT_FAMILY_SELECT_W + 2.0 + 14.0;
    // Group 3: Font size select
    left += FONT_SIZE_SELECT_W + 2.0 + 14.0;
    // Group 4: Paragraph style select
    left += PARAGRAPH_SELECT_W + 2.0 + 14.0;
    // Group 5: Format buttons (8 buttons)
    left += (32.0 + 2.0) * 7.0 + 36.0 + 2.0 + 14.0;
    // Group 6: List buttons (5 buttons)
    left += 28.0 + 2.0 + 28.0 + 2.0 + 28.0 + 2.0 + 32.0 + 2.0 + 32.0 + 2.0 + 14.0;
    // Group 7: Alignment buttons (4 buttons)
    left += (28.0 + 2.0) * 4.0 + 14.0;
    // Group 8: Insert buttons (5 buttons)
    left += 42.0 + 2.0 + 38.0 + 2.0 + 28.0 + 2.0 + 28.0 + 2.0 + 36.0 + 2.0 + 14.0;
    left
}

/// If a color picker cell was clicked, return the color string.
pub(super) fn color_picker_hit(x: f64, y: f64) -> Option<String> {
    let cp_x = color_picker_x();
    let dropdown_top = MENU_BAR_H + 8.0 + 32.0;
    let grid_y = dropdown_top + TABLE_GRID_PAD;

    let rel_x = x - cp_x - TABLE_GRID_PAD;
    let rel_y = y - grid_y;

    if rel_x < 0.0 || rel_y < 0.0 {
        return None;
    }

    let col = (rel_x / COLOR_CELL_SIZE) as usize;
    let row = (rel_y / COLOR_CELL_SIZE) as usize;

    let idx = row * COLOR_COLS + col;
    if idx < COLOR_PALETTE.len() {
        Some(COLOR_PALETTE[idx].to_string())
    } else {
        None
    }
}
