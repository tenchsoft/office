use super::render::*;
use super::*;

/// Render the full document area with multi-page support.
///
/// Pages are laid out vertically with gaps between them. Each page uses the
/// dimensions from the document's `PageSetup`. Content flows from page to page
/// automatically. Pages outside the visible area are skipped (virtualization).
///
/// When `state.vertical_writing` is enabled, text is rendered in top-to-bottom
/// columns that flow right-to-left, matching traditional Korean/Chinese layout.
pub fn paint_document_area(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &KodocsState,
) {
    if state.vertical_writing {
        paint_document_area_vertical(p, cache, rect, state);
        return;
    }

    paint_document_area_horizontal(p, cache, rect, state);
}

/// Horizontal (left-to-right) document layout – the default mode.
fn paint_document_area_horizontal(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &KodocsState,
) {
    let scale = state.zoom / 100.0;
    let doc = state.current_document();
    let setup = &doc.page_setup;
    let (page_w_raw, page_h_raw) = setup.page_size_px();
    let page_w = page_w_raw * scale;
    let page_h = page_h_raw * scale;
    let mm_to_px = 96.0 / 25.4;
    let margin_left = setup.margins.left as f64 * mm_to_px * scale;
    let margin_right = setup.margins.right as f64 * mm_to_px * scale;
    let margin_top = setup.margins.top as f64 * mm_to_px * scale;
    let margin_bottom = setup.margins.bottom as f64 * mm_to_px * scale;
    let _ = setup;

    let content_w = page_w - margin_left - margin_right;
    let header_h = HEADER_H * scale;
    let footer_h = FOOTER_H * scale;
    let page_content_h = page_h - margin_top - margin_bottom - header_h - footer_h;

    let num_pages;
    let page_map: Vec<super::super::state::PageMapEntry>;
    if state.layout_cache.is_valid(doc, state.zoom) {
        num_pages = state.layout_cache.num_pages();
        page_map = state.layout_cache.page_map().to_vec();
    } else {
        let total_content_h = estimate_total_content_height(doc, content_w, scale);
        num_pages = if total_content_h <= 0.0 {
            1
        } else {
            ((total_content_h / page_content_h).ceil() as usize).max(1)
        };
        page_map = compute_page_map(doc, content_w, page_content_h, scale);
        // Cache update happens in ensure_layout_cache() called from the paint loop
    }

    let page_x = rect.x0 + ((rect.width() - page_w) / 2.0).max(PAGE_MARGIN_X);

    // Virtualization: only render pages visible in the viewport
    let scroll_y = state.scroll_y;
    let viewport_top = rect.y0 + PAGE_MARGIN_Y - scroll_y;
    let viewport_bottom = rect.y1;

    // Pre-compute which blocks go on which page.
    // We walk through all blocks, accumulating their estimated heights,
    // and record the (start_block, start_y_offset_within_page) for each page.
    // page_map is already computed above (from cache or fresh).

    for page_idx in 0..num_pages {
        let page_y =
            rect.y0 + PAGE_MARGIN_Y + page_idx as f64 * (page_h + PAGE_GAP * scale) - scroll_y;

        // Virtualization: skip pages outside the visible area
        if page_y + page_h < viewport_top || page_y > viewport_bottom {
            continue;
        }

        // Page shadow
        let shadow = Rect::new(
            page_x + 3.0,
            page_y + 3.0,
            page_x + page_w + 3.0,
            page_y + page_h + 3.0,
        );
        p.fill_rounded_rect(shadow, Color::rgba8(0, 0, 0, 40), 2.0);

        // Page background
        let page_rect = Rect::new(page_x, page_y, page_x + page_w, page_y + page_h);
        p.fill_rounded_rect(page_rect, c_page_bg(), 4.0);
        p.stroke_rounded_rect(page_rect, c_separator(), 1.0, 4.0);

        // Header area
        let header_y = page_y + margin_top;
        paint_page_header(
            p,
            cache,
            page_x + margin_left,
            header_y,
            content_w,
            header_h,
            scale,
            page_idx + 1,
            num_pages,
            &doc.headers_footers,
            &doc.metadata.title,
            state.editing_header,
        );

        // Footer area
        let footer_y = page_y + page_h - margin_bottom - footer_h;
        paint_page_footer(
            p,
            cache,
            page_x + margin_left,
            footer_y,
            content_w,
            footer_h,
            scale,
            page_idx + 1,
            num_pages,
            &doc.headers_footers,
            &doc.metadata.title,
            state.editing_footer,
        );

        // Content area
        let content_top = header_y + header_h;
        let content_bottom = footer_y;
        let left = page_x + margin_left;

        let mut y = content_top;

        // Determine which blocks belong to this page
        if page_idx < page_map.len() {
            let entry = &page_map[page_idx];
            if entry.start_block == 0 && doc.content.is_empty() {
                // Empty document placeholder on first page
                p.draw_text_cached(
                    cache,
                    "내용을 입력하세요...",
                    left,
                    y,
                    c_text_dim(),
                    (14.0 * scale) as f32,
                    FontWeight::NORMAL,
                    false,
                    false,
                );
                paint_cursor(p, left, y, scale, state, "");
                continue;
            }

            let mut block_idx = entry.start_block;
            // Skip any initial y-offset for blocks that started on a previous page
            // (not applicable in the current simple model, but reserved for future)

            while block_idx < doc.content.len() {
                if y > content_bottom {
                    break;
                }
                let block = &doc.content[block_idx];
                y = paint_block(
                    p,
                    cache,
                    block,
                    block_idx,
                    left,
                    y,
                    content_w,
                    scale,
                    state,
                    content_bottom,
                );
                block_idx += 1;
            }
        }
    }
}

/// Vertical (top-to-bottom) document layout for traditional Korean/Chinese writing.
///
/// Text flows in columns from top to bottom, with columns arranged right-to-left.
/// Each column has a fixed width (equivalent to line height in horizontal mode),
/// and characters are stacked vertically within each column.
fn paint_document_area_vertical(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &KodocsState,
) {
    let scale = state.zoom / 100.0;
    let doc = state.current_document();
    let setup = &doc.page_setup;
    let (page_w_raw, page_h_raw) = setup.page_size_px();
    let page_w = page_w_raw * scale;
    let page_h = page_h_raw * scale;
    let mm_to_px = 96.0 / 25.4;
    let margin_left = setup.margins.left as f64 * mm_to_px * scale;
    let margin_right = setup.margins.right as f64 * mm_to_px * scale;
    let margin_top = setup.margins.top as f64 * mm_to_px * scale;
    let margin_bottom = setup.margins.bottom as f64 * mm_to_px * scale;

    let content_h = page_h - margin_top - margin_bottom;
    let header_h = HEADER_H * scale;
    let footer_h = FOOTER_H * scale;
    let _ = (header_h, footer_h);

    let page_x = rect.x0 + ((rect.width() - page_w) / 2.0).max(PAGE_MARGIN_X);

    // Single page for now (vertical mode)
    let page_y = rect.y0 + PAGE_MARGIN_Y - state.scroll_y;

    // Page shadow
    let shadow = Rect::new(
        page_x + 3.0,
        page_y + 3.0,
        page_x + page_w + 3.0,
        page_y + page_h + 3.0,
    );
    p.fill_rounded_rect(shadow, Color::rgba8(0, 0, 0, 40), 2.0);

    // Page background
    let page_rect = Rect::new(page_x, page_y, page_x + page_w, page_y + page_h);
    p.fill_rounded_rect(page_rect, c_page_bg(), 4.0);
    p.stroke_rounded_rect(page_rect, c_separator(), 1.0, 4.0);

    // Vertical mode indicator
    p.draw_text_cached(
        cache,
        "세로쓰기",
        page_x + margin_left,
        page_y + margin_top - 16.0 * scale,
        c_text_dim(),
        (9.0 * scale) as f32,
        FontWeight::NORMAL,
        false,
        false,
    );

    // Column parameters: each column is one character wide
    let col_w = 20.0 * scale; // column width = line height equivalent
    let chars_per_col = (content_h / col_w).max(1.0) as usize;
    let content_w = page_w - margin_left - margin_right;
    let num_cols = (content_w / col_w).max(1.0) as usize;

    // Collect all text from all blocks
    let all_text: String = doc
        .content
        .iter()
        .map(|b| inline_nodes_to_text(extract_block_inline_content(b)))
        .collect::<Vec<_>>()
        .join("\n");

    if all_text.is_empty() {
        p.draw_text_cached(
            cache,
            "내용을 입력하세요...",
            page_x + page_w - margin_right - 100.0 * scale,
            page_y + margin_top,
            c_text_dim(),
            (14.0 * scale) as f32,
            FontWeight::NORMAL,
            false,
            true,
        );
        return;
    }

    // Render text in vertical columns, right-to-left
    let chars: Vec<char> = all_text.chars().collect();
    let mut char_idx = 0;
    let font_size = (13.0 * scale) as f32;

    for col in 0..num_cols {
        if char_idx >= chars.len() {
            break;
        }

        // Right-to-left: column 0 is at the rightmost position
        let col_x = page_x + page_w - margin_right - (col as f64 + 1.0) * col_w;

        for row in 0..chars_per_col {
            if char_idx >= chars.len() {
                break;
            }
            let ch = chars[char_idx];
            char_idx += 1;

            let char_y = page_y + margin_top + row as f64 * col_w;

            // Render the character rotated (in vertical mode, CJK characters
            // are naturally upright; Latin characters are rotated 90 degrees)
            if is_cjk_char(ch) {
                // CJK characters render upright in vertical mode
                p.draw_text_cached(
                    cache,
                    &ch.to_string(),
                    col_x + col_w / 2.0,
                    char_y + col_w * 0.8,
                    c_text_dark(),
                    font_size,
                    FontWeight::NORMAL,
                    true, // center horizontally
                    false,
                );
            } else if ch == '\n' {
                // Newline moves to the next column
                break;
            } else {
                // Non-CJK characters render rotated (we show them upright
                // for simplicity; proper rotation would need glyph transforms)
                p.draw_text_cached(
                    cache,
                    &ch.to_string(),
                    col_x + col_w / 2.0,
                    char_y + col_w * 0.8,
                    c_text_dark(),
                    font_size,
                    FontWeight::NORMAL,
                    true,
                    false,
                );
            }
        }
    }
}
