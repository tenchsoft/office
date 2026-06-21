use super::*;

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
/// Paint a single block node and return the new y position.
pub(super) fn paint_block(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    block: &BlockNode,
    block_idx: usize,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    state: &DocsState,
    bottom: f64,
) -> f64 {
    match block {
        BlockNode::Paragraph { content, attrs } => paint_paragraph_block(
            p, cache, content, attrs, block_idx, left, y, content_w, scale, state,
        ),
        BlockNode::Heading {
            level,
            content,
            attrs,
        } => paint_heading(
            p, cache, *level, content, attrs, block_idx, left, y, content_w, scale, state,
        ),
        BlockNode::BulletList { items } => paint_bullet_list(
            p, cache, items, block_idx, left, y, content_w, scale, state, bottom,
        ),
        BlockNode::OrderedList { items, start } => paint_ordered_list(
            p, cache, items, *start, block_idx, left, y, content_w, scale, state, bottom,
        ),
        BlockNode::TaskList { items } => paint_task_list(
            p, cache, items, block_idx, left, y, content_w, scale, state, bottom,
        ),
        BlockNode::BlockQuote { content } => paint_blockquote(
            p, cache, content, block_idx, left, y, content_w, scale, state, bottom,
        ),
        BlockNode::CodeBlock { language, code } => paint_code_block(
            p, cache, language, code, block_idx, left, y, content_w, scale, state,
        ),
        BlockNode::HorizontalRule => paint_horizontal_rule(p, left, y, content_w, scale),
        BlockNode::PageBreak => {
            // Page break: draw a dashed line and advance y
            if y + 20.0 * scale <= bottom {
                let dash_y = y + 10.0 * scale;
                for x_off in (0..content_w as usize).step_by(12) {
                    let x0 = left + x_off as f64;
                    let x1 = (x0 + 6.0).min(left + content_w);
                    p.draw_line(
                        Point::new(x0, dash_y),
                        Point::new(x1, dash_y),
                        c_text_dim(),
                        1.0,
                    );
                }
            }
            y + 24.0 * scale
        }
        BlockNode::Table { rows } => paint_table(
            p, cache, rows, block_idx, left, y, content_w, scale, state, bottom,
        ),
        BlockNode::Image {
            source,
            alt,
            width,
            height,
        } => {
            let is_targeted = state.targeted_image_block == Some(block_idx)
                || state
                    .image_resize_drag
                    .as_ref()
                    .is_some_and(|d| d.block_idx == block_idx);
            paint_image_block(
                p,
                cache,
                source,
                alt.as_deref(),
                width,
                height,
                left,
                y,
                content_w,
                scale,
                is_targeted,
            )
        }
        BlockNode::Footnote { number, content } => {
            // Render footnote as a small paragraph with reference number
            let ref_text = format!("[{number}] ");
            let ref_w = ref_text.len() as f64 * 7.0 * scale;
            p.draw_text_cached(
                cache,
                &ref_text,
                left,
                y + 4.0,
                c_text_dim(),
                (11.0 * scale) as f32,
                FontWeight::NORMAL,
                false,
                false,
            );
            let mut fy = y;
            for inline in content {
                if let tench_document_core::InlineNode::Text { text, .. } = inline {
                    p.draw_text_cached(
                        cache,
                        text,
                        left + ref_w,
                        fy + 4.0,
                        c_text_dim(),
                        (11.0 * scale) as f32,
                        FontWeight::NORMAL,
                        false,
                        false,
                    );
                    fy += 16.0 * scale;
                }
            }
            fy.max(y + 16.0 * scale)
        }
    }
}

// ---------------------------------------------------------------------------
// Block-level renderers
// ---------------------------------------------------------------------------

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_paragraph_block(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    content: &[InlineNode],
    attrs: &tench_document_core::ParagraphAttrs,
    block_idx: usize,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    state: &DocsState,
) -> f64 {
    // Apply indent from paragraph attrs
    let indent_left = attrs.indent_left as f64 * scale;
    let indent_right = attrs.indent_right as f64 * scale;
    let first_line_indent = attrs.indent_first_line as f64 * scale;
    let effective_left = left + indent_left + first_line_indent;
    let effective_w = content_w - indent_left - indent_right - first_line_indent;

    let (new_y, text) = paint_inline_nodes_with_alignment(
        p,
        cache,
        content,
        effective_left,
        y,
        effective_w,
        scale,
        c_text_dark(),
        attrs.alignment,
    );
    paint_search_highlights(
        p,
        block_idx,
        effective_left,
        y,
        effective_w,
        scale,
        state,
        &text,
    );
    paint_comment_highlights(
        p,
        block_idx,
        effective_left,
        y,
        effective_w,
        scale,
        state,
        &text,
    );
    paint_track_change_decorations(
        p,
        block_idx,
        effective_left,
        y,
        effective_w,
        scale,
        state,
        &text,
    );
    paint_cursor_for_block(
        p,
        block_idx,
        effective_left,
        y,
        effective_w,
        scale,
        state,
        &text,
    );
    paint_selection_for_block(
        p,
        block_idx,
        effective_left,
        y,
        effective_w,
        scale,
        state,
        &text,
    );
    new_y + 12.0 * scale
}

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_heading(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    level: u8,
    content: &[InlineNode],
    attrs: &tench_document_core::ParagraphAttrs,
    block_idx: usize,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    state: &DocsState,
) -> f64 {
    let (font_size, weight, spacing) = match level {
        1 => (28.0, FontWeight::BLACK, 36.0),
        2 => (24.0, FontWeight::EXTRA_BOLD, 32.0),
        3 => (20.0, FontWeight::BOLD, 28.0),
        4 => (17.0, FontWeight::BOLD, 24.0),
        5 => (15.0, FontWeight::BOLD, 22.0),
        _ => (13.0, FontWeight::BOLD, 20.0),
    };
    let scaled_size = (font_size * scale) as f32;

    // Apply indent from paragraph attrs
    let indent_left = attrs.indent_left as f64 * scale;
    let indent_right = attrs.indent_right as f64 * scale;
    let first_line_indent = attrs.indent_first_line as f64 * scale;
    let effective_left = left + indent_left + first_line_indent;
    let effective_w = content_w - indent_left - indent_right - first_line_indent;

    // Flatten inline content to plain text for heading rendering
    let text = inline_nodes_to_text(content);
    let lines = wrap_text(&text, effective_w, scaled_size, scale);

    let mut line_y = y;
    for line in &lines {
        let line_x = compute_aligned_x(
            effective_left,
            effective_w,
            line,
            scaled_size,
            attrs.alignment,
        );
        p.draw_text_cached(
            cache,
            line,
            line_x,
            line_y,
            c_text_dark(),
            scaled_size,
            weight,
            false,
            false,
        );
        line_y += spacing * scale;
    }

    paint_search_highlights(
        p,
        block_idx,
        effective_left,
        y,
        effective_w,
        scale,
        state,
        &text,
    );
    paint_comment_highlights(
        p,
        block_idx,
        effective_left,
        y,
        effective_w,
        scale,
        state,
        &text,
    );
    paint_track_change_decorations(
        p,
        block_idx,
        effective_left,
        y,
        effective_w,
        scale,
        state,
        &text,
    );
    paint_cursor_for_block(
        p,
        block_idx,
        effective_left,
        y,
        effective_w,
        scale,
        state,
        &text,
    );
    paint_selection_for_block(
        p,
        block_idx,
        effective_left,
        y,
        effective_w,
        scale,
        state,
        &text,
    );

    line_y + 8.0 * scale
}

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_bullet_list(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    items: &[tench_document_core::ListItem],
    block_idx: usize,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    state: &DocsState,
    bottom: f64,
) -> f64 {
    let bullet_indent = 24.0 * scale;
    let mut line_y = y;

    for item in items {
        if line_y > bottom {
            break;
        }
        // Draw bullet
        p.draw_text_cached(
            cache,
            "\u{2022}",
            left + 8.0 * scale,
            line_y,
            c_text_light(),
            (13.0 * scale) as f32,
            FontWeight::NORMAL,
            false,
            false,
        );
        let (new_y, text) = paint_inline_nodes(
            p,
            cache,
            &item.content,
            left + bullet_indent,
            line_y,
            content_w - bullet_indent,
            scale,
            c_text_dark(),
            Alignment::Left,
        );
        paint_cursor_for_block(p, block_idx, left, line_y, content_w, scale, state, &text);
        line_y = new_y + 4.0 * scale;
    }
    line_y + 8.0 * scale
}

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_ordered_list(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    items: &[tench_document_core::ListItem],
    start: u32,
    block_idx: usize,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    state: &DocsState,
    bottom: f64,
) -> f64 {
    let list_indent = 28.0 * scale;
    let mut line_y = y;

    for (i, item) in items.iter().enumerate() {
        if line_y > bottom {
            break;
        }
        let num = format!("{}.", start + i as u32);
        p.draw_text_cached(
            cache,
            &num,
            left + 4.0 * scale,
            line_y,
            c_text_light(),
            (13.0 * scale) as f32,
            FontWeight::NORMAL,
            false,
            false,
        );
        let (new_y, text) = paint_inline_nodes(
            p,
            cache,
            &item.content,
            left + list_indent,
            line_y,
            content_w - list_indent,
            scale,
            c_text_dark(),
            Alignment::Left,
        );
        paint_cursor_for_block(p, block_idx, left, line_y, content_w, scale, state, &text);
        line_y = new_y + 4.0 * scale;
    }
    line_y + 8.0 * scale
}

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_task_list(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    items: &[tench_document_core::TaskItem],
    block_idx: usize,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    state: &DocsState,
    bottom: f64,
) -> f64 {
    let check_indent = 28.0 * scale;
    let box_size = 12.0 * scale;
    let mut line_y = y;

    for item in items {
        if line_y > bottom {
            break;
        }
        // Draw checkbox
        let box_x = left + 6.0 * scale;
        let box_y = line_y - box_size + 2.0 * scale;
        let checkbox = Rect::new(box_x, box_y, box_x + box_size, box_y + box_size);
        p.stroke_rounded_rect(checkbox, c_text_light(), 1.0, 2.0);
        if item.checked {
            p.draw_text_cached(
                cache,
                "\u{2713}",
                box_x + 1.0 * scale,
                line_y,
                c_accent(),
                (12.0 * scale) as f32,
                FontWeight::BOLD,
                false,
                false,
            );
        }

        let color = if item.checked {
            c_text_dim()
        } else {
            c_text_dark()
        };
        let (new_y, text) = paint_inline_nodes(
            p,
            cache,
            &item.content,
            left + check_indent,
            line_y,
            content_w - check_indent,
            scale,
            color,
            Alignment::Left,
        );
        paint_cursor_for_block(p, block_idx, left, line_y, content_w, scale, state, &text);
        line_y = new_y + 4.0 * scale;
    }
    line_y + 8.0 * scale
}

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_blockquote(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    children: &[BlockNode],
    block_idx: usize,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    state: &DocsState,
    bottom: f64,
) -> f64 {
    let quote_indent = 20.0 * scale;
    let border_x = left + 8.0 * scale;

    // Draw left border
    p.draw_line(
        Point::new(border_x, y),
        Point::new(border_x, (y + 60.0 * scale).min(bottom)),
        c_accent(),
        3.0,
    );

    // Draw background
    let bg_rect = Rect::new(
        left + quote_indent,
        y,
        left + content_w,
        (y + 60.0 * scale).min(bottom),
    );
    p.fill_rect(bg_rect, Color::rgba8(0xA7, 0x8B, 0xFA, 15));

    let mut child_y = y;
    for child in children {
        if child_y > bottom {
            break;
        }
        child_y = paint_block(
            p,
            cache,
            child,
            block_idx,
            left + quote_indent,
            child_y,
            content_w - quote_indent,
            scale,
            state,
            bottom,
        );
    }
    child_y + 8.0 * scale
}

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_code_block(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    language: &Option<String>,
    code: &str,
    block_idx: usize,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    state: &DocsState,
) -> f64 {
    let padding = 12.0 * scale;
    let line_h = 18.0 * scale;

    // Count lines for height
    let line_count = code.lines().count().max(1);
    let block_h = line_count as f64 * line_h + padding * 2.0;

    // Background
    let bg = Rect::new(left, y, left + content_w, y + block_h);
    p.fill_rounded_rect(bg, Color::rgba8(0x1A, 0x1A, 0x2E, 200), 4.0);
    p.stroke_rounded_rect(bg, c_separator(), 1.0, 4.0);

    // Language label
    let mut text_y = y + padding;
    if let Some(lang) = language {
        if !lang.is_empty() {
            p.draw_text_cached(
                cache,
                lang,
                left + padding,
                text_y,
                c_accent(),
                (10.0 * scale) as f32,
                FontWeight::BOLD,
                false,
                false,
            );
            text_y += line_h;
        }
    }

    // Code lines
    let font_size = (12.0 * scale) as f32;
    for line in code.lines() {
        p.draw_text_cached(
            cache,
            line,
            left + padding,
            text_y,
            c_text_light(),
            font_size,
            FontWeight::NORMAL,
            false,
            false,
        );
        text_y += line_h;
    }

    paint_cursor_for_block(p, block_idx, left, y, content_w, scale, state, code);

    y + block_h + 12.0 * scale
}

pub(super) fn paint_horizontal_rule(
    p: &mut Painter<'_>,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
) -> f64 {
    let rule_y = y + 10.0 * scale;
    p.draw_line(
        Point::new(left, rule_y),
        Point::new(left + content_w, rule_y),
        c_separator(),
        1.0,
    );
    y + 24.0 * scale
}

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_table(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rows: &[TableRow],
    block_idx: usize,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    state: &DocsState,
    bottom: f64,
) -> f64 {
    if rows.is_empty() {
        return y + 20.0 * scale;
    }

    let col_count = rows[0].cells.len().max(1);
    let cell_w = content_w / col_count as f64;
    let cell_pad = 6.0 * scale;
    let min_cell_h = 24.0 * scale;
    let line_h = 20.0 * scale;
    let border_color = c_separator();
    let header_bg = Color::rgba8(0x2A, 0x2A, 0x2A, 255);

    // First pass: compute row heights
    let mut row_heights: Vec<f64> = Vec::with_capacity(rows.len());
    for row in rows {
        let max_lines = row
            .cells
            .iter()
            .map(|cell| {
                let text = cell
                    .content
                    .iter()
                    .map(|b| inline_nodes_to_text(extract_block_inline_content(b)))
                    .collect::<Vec<_>>()
                    .join(" ");
                let chars_per_line = if cell_w > 0.0 {
                    ((cell_w - cell_pad * 2.0) / (7.0 * scale)).max(1.0) as usize
                } else {
                    1
                };
                if text.is_empty() {
                    1
                } else {
                    text.len().div_ceil(chars_per_line)
                }
            })
            .max()
            .unwrap_or(1);
        row_heights.push((max_lines as f64 * line_h).max(min_cell_h));
    }

    let total_h: f64 = row_heights.iter().sum();
    if y + total_h > bottom {
        // Not enough space, draw placeholder
        p.draw_text_cached(
            cache,
            "[Table]",
            left,
            y,
            c_text_dim(),
            (13.0 * scale) as f32,
            FontWeight::NORMAL,
            false,
            false,
        );
        return y + 20.0 * scale;
    }

    let mut cell_y = y;

    for (ri, row) in rows.iter().enumerate() {
        let row_h = row_heights[ri];

        // Header row background
        if ri == 0 {
            let header_rect = Rect::new(left, cell_y, left + content_w, cell_y + row_h);
            p.fill_rect(header_rect, header_bg);
        }

        // Draw cell backgrounds and text
        let mut cell_x = left;
        for cell in row.cells.iter() {
            let _cell_rect = Rect::new(cell_x, cell_y, cell_x + cell_w, cell_y + row_h);

            // Cell border
            p.draw_line(
                Point::new(cell_x, cell_y),
                Point::new(cell_x, cell_y + row_h),
                border_color,
                1.0,
            );
            p.draw_line(
                Point::new(cell_x + cell_w, cell_y),
                Point::new(cell_x + cell_w, cell_y + row_h),
                border_color,
                1.0,
            );

            // Cell content
            let cell_text = cell
                .content
                .iter()
                .map(|b| inline_nodes_to_text(extract_block_inline_content(b)))
                .collect::<Vec<_>>()
                .join(" ");

            if !cell_text.is_empty() {
                let text_color = if ri == 0 {
                    c_text_light()
                } else {
                    c_text_dark()
                };
                let font_size = (12.0 * scale) as f32;
                let chars_per_line = if cell_w > cell_pad * 2.0 {
                    ((cell_w - cell_pad * 2.0) / (7.0 * scale)).max(1.0) as usize
                } else {
                    usize::MAX
                };

                let mut line_y = cell_y + cell_pad + 12.0 * scale;
                let mut remaining = cell_text.as_str();
                while !remaining.is_empty() && line_y + line_h <= cell_y + row_h {
                    let (line, rest) = if remaining.len() > chars_per_line {
                        let split_at = remaining[..chars_per_line]
                            .rfind(' ')
                            .unwrap_or(chars_per_line);
                        if split_at == 0 {
                            (
                                remaining
                                    .get(..chars_per_line.min(remaining.len()))
                                    .unwrap_or(""),
                                remaining
                                    .get(chars_per_line.min(remaining.len())..)
                                    .unwrap_or(""),
                            )
                        } else {
                            let end = split_at;
                            let r = remaining.get(end + 1..).unwrap_or("");
                            (remaining.get(..end).unwrap_or(""), r)
                        }
                    } else {
                        (remaining, "")
                    };
                    if !line.is_empty() {
                        p.draw_text_cached(
                            cache,
                            line,
                            cell_x + cell_pad,
                            line_y,
                            text_color,
                            font_size,
                            FontWeight::NORMAL,
                            false,
                            false,
                        );
                    }
                    line_y += line_h;
                    remaining = rest;
                }
            }

            cell_x += cell_w;
        }

        // Horizontal border at bottom of row
        p.draw_line(
            Point::new(left, cell_y + row_h),
            Point::new(left + content_w, cell_y + row_h),
            border_color,
            1.0,
        );

        // Top border (only for first row)
        if ri == 0 {
            p.draw_line(
                Point::new(left, cell_y),
                Point::new(left + content_w, cell_y),
                border_color,
                1.0,
            );
        }

        cell_y += row_h;
    }

    paint_cursor_for_block(p, block_idx, left, y, content_w, scale, state, "");

    cell_y + 12.0 * scale
}

/// Extract inline content from a block node (for table cell text extraction).
pub(super) fn extract_block_inline_content(block: &BlockNode) -> &[InlineNode] {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => content,
        _ => &[],
    }
}

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_image_block(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    source: &ImageSource,
    alt: Option<&str>,
    width: &Option<f32>,
    height: &Option<f32>,
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    is_targeted: bool,
) -> f64 {
    let max_img_w = content_w;
    let img_w = width
        .map(|w| (w as f64 * scale).min(max_img_w))
        .unwrap_or(max_img_w * 0.6);
    let img_h = height.map(|h| h as f64 * scale).unwrap_or(img_w * 0.75); // default 4:3 aspect ratio

    // Center the image horizontally
    let img_x = left + (content_w - img_w) / 2.0;
    let img_y = y + 8.0 * scale;

    let img_rect = Rect::new(img_x, img_y, img_x + img_w, img_y + img_h);

    // Draw image placeholder background
    let placeholder_bg = Color::rgba8(0x1A, 0x1A, 0x2E, 180);
    p.fill_rounded_rect(img_rect, placeholder_bg, 4.0);
    p.stroke_rounded_rect(img_rect, c_separator(), 1.0, 4.0);

    // Try to render embedded image data
    match source {
        ImageSource::Embedded { data } => {
            if !data.is_empty() {
                // For now, show the embedded indicator; actual pixel decoding
                // requires wgpu texture upload which is async. We show a label.
                let label = format!("{} bytes", data.len());
                p.draw_text_cached(
                    cache,
                    &label,
                    img_x + img_w / 2.0,
                    img_y + img_h / 2.0 - 8.0 * scale,
                    c_text_light(),
                    (11.0 * scale) as f32,
                    FontWeight::NORMAL,
                    true,
                    false,
                );
            } else {
                let label = alt.unwrap_or("[Image]");
                p.draw_text_cached(
                    cache,
                    label,
                    img_x + img_w / 2.0,
                    img_y + img_h / 2.0,
                    c_text_dim(),
                    (13.0 * scale) as f32,
                    FontWeight::NORMAL,
                    true,
                    false,
                );
            }
        }
        ImageSource::Referenced { path } => {
            // Show file path as label
            let display_path = if path.len() > 30 {
                format!("...{}", &path[path.len() - 27..])
            } else {
                path.clone()
            };
            p.draw_text_cached(
                cache,
                &display_path,
                img_x + img_w / 2.0,
                img_y + img_h / 2.0 - 6.0 * scale,
                c_text_light(),
                (12.0 * scale) as f32,
                FontWeight::NORMAL,
                true,
                false,
            );
            // Show alt text below
            if let Some(alt_text) = alt {
                p.draw_text_cached(
                    cache,
                    alt_text,
                    img_x + img_w / 2.0,
                    img_y + img_h / 2.0 + 10.0 * scale,
                    c_text_light(),
                    (11.0 * scale) as f32,
                    FontWeight::NORMAL,
                    true,
                    false,
                );
            }
        }
    }

    // Image selection handles — only for targeted images
    if is_targeted {
        let handle_size = 4.0 * scale;
        let handle_color = c_accent();
        let corners = [
            (img_x, img_y),
            (img_x + img_w, img_y),
            (img_x, img_y + img_h),
            (img_x + img_w, img_y + img_h),
        ];
        for (cx, cy) in corners {
            let handle = Rect::new(
                cx - handle_size / 2.0,
                cy - handle_size / 2.0,
                cx + handle_size / 2.0,
                cy + handle_size / 2.0,
            );
            p.fill_rect(handle, handle_color);
        }
    }

    img_y + img_h + 16.0 * scale
}

// ---------------------------------------------------------------------------

mod inline;
mod support;

pub(super) use inline::*;
pub(super) use support::*;
