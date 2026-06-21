use super::*;

// Alignment helper
// ---------------------------------------------------------------------------

/// Compute the x position for a line of text based on alignment.
pub(crate) fn compute_aligned_x(
    left: f64,
    content_w: f64,
    line: &str,
    font_size: f32,
    alignment: Alignment,
) -> f64 {
    match alignment {
        Alignment::Left => left,
        Alignment::Center => {
            let text_w = estimate_text_width(line, font_size);
            left + (content_w - text_w) / 2.0
        }
        Alignment::Right => {
            let text_w = estimate_text_width(line, font_size);
            left + content_w - text_w
        }
        Alignment::Justify => left, // Justify handled differently in inline rendering
    }
}

// ---------------------------------------------------------------------------
// Inline formatting
// ---------------------------------------------------------------------------

/// Render a sequence of inline nodes with formatting and alignment.
/// Returns (new_y, flattened_text) for cursor/selection positioning.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_inline_nodes(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    nodes: &[InlineNode],
    left: f64,
    y: f64,
    content_w: f64,
    scale: f64,
    default_color: Color,
    alignment: Alignment,
) -> (f64, String) {
    let mut flat_text = String::new();
    let mut line_y = y;
    let mut line_buf = String::new();
    let base_font_size = (13.0 * scale) as f32;
    let chars_per_line = ((content_w / (7.0 * scale.max(0.6))) as usize).max(24);

    // Track last marks for trailing flush
    let mut last_color = default_color;
    let mut last_font_size = base_font_size;
    let mut last_weight = FontWeight::NORMAL;
    let mut last_italic = false;
    let mut last_underline = false;
    let mut last_strikethrough = false;
    let mut last_y_offset: f64 = 0.0;

    for node in nodes {
        match node {
            InlineNode::Text { text, marks } => {
                // Determine rendering properties from marks
                let weight = if marks.bold {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                };
                // Handle superscript/subscript: 70% font size + baseline shift
                let (script_scale, y_offset) = if marks.superscript {
                    (0.7, -4.0 * scale)
                } else if marks.subscript {
                    (0.7, 4.0 * scale)
                } else {
                    (1.0, 0.0)
                };
                let font_size = marks
                    .font_size
                    .map(|s| s * scale as f32 * script_scale as f32)
                    .unwrap_or(base_font_size * script_scale as f32);
                let color = marks
                    .text_color
                    .as_deref()
                    .and_then(parse_color)
                    .unwrap_or(default_color);

                // Save marks for trailing flush
                last_color = color;
                last_font_size = font_size;
                last_weight = weight;
                last_italic = marks.italic;
                last_underline = marks.underline;
                last_strikethrough = marks.strikethrough;
                last_y_offset = y_offset;

                // Draw background for code or background_color marks
                if marks.code || marks.background_color.is_some() {
                    let bg_color = marks
                        .background_color
                        .as_deref()
                        .and_then(parse_color)
                        .unwrap_or_else(|| Color::rgba8(0x30, 0x30, 0x40, 200));
                    let text_w = estimate_text_width(text, font_size);
                    let bg_rect = Rect::new(
                        left,
                        line_y - font_size as f64 + y_offset,
                        left + text_w,
                        line_y + 4.0 * scale + y_offset,
                    );
                    p.fill_rounded_rect(bg_rect, bg_color, 3.0);
                }

                // Render text with word wrapping
                let words: Vec<&str> = text.split_whitespace().collect();
                for word in words {
                    if line_buf.len() + word.len() + 1 > chars_per_line && !line_buf.is_empty() {
                        let line_x =
                            compute_aligned_x(left, content_w, &line_buf, font_size, alignment);
                        p.draw_text_cached(
                            cache,
                            &line_buf,
                            line_x,
                            line_y + y_offset,
                            color,
                            font_size,
                            weight,
                            false,
                            marks.italic,
                        );
                        // Underline
                        if marks.underline {
                            let text_w = estimate_text_width(&line_buf, font_size);
                            let line_x =
                                compute_aligned_x(left, content_w, &line_buf, font_size, alignment);
                            p.draw_line(
                                Point::new(line_x, line_y + 2.0 * scale + y_offset),
                                Point::new(line_x + text_w, line_y + 2.0 * scale + y_offset),
                                color,
                                1.0,
                            );
                        }
                        // Strikethrough
                        if marks.strikethrough {
                            let text_w = estimate_text_width(&line_buf, font_size);
                            let line_x =
                                compute_aligned_x(left, content_w, &line_buf, font_size, alignment);
                            let mid_y = line_y - font_size as f64 / 2.0 + y_offset;
                            p.draw_line(
                                Point::new(line_x, mid_y),
                                Point::new(line_x + text_w, mid_y),
                                color,
                                1.0,
                            );
                        }
                        flat_text.push_str(&line_buf);
                        flat_text.push('\n');
                        line_y += 20.0 * scale;
                        line_buf.clear();
                    }
                    if !line_buf.is_empty() {
                        line_buf.push(' ');
                    }
                    line_buf.push_str(word);
                }
            }
            InlineNode::Link {
                text, marks, href, ..
            } => {
                // Render links with accent color and underline, respecting marks
                let weight = if marks.bold {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                };
                let link_font_size = marks
                    .font_size
                    .map(|s| s * scale as f32)
                    .unwrap_or(base_font_size);
                let link_color = c_accent();

                let words: Vec<&str> = text.split_whitespace().collect();
                for word in words {
                    if line_buf.len() + word.len() + 1 > chars_per_line && !line_buf.is_empty() {
                        p.draw_text_cached(
                            cache,
                            &line_buf,
                            left,
                            line_y,
                            link_color,
                            link_font_size,
                            weight,
                            false,
                            false,
                        );
                        let text_w = estimate_text_width(&line_buf, link_font_size);
                        p.draw_line(
                            Point::new(left, line_y + 2.0 * scale),
                            Point::new(left + text_w, line_y + 2.0 * scale),
                            link_color,
                            1.0,
                        );
                        flat_text.push_str(&line_buf);
                        flat_text.push('\n');
                        line_y += 20.0 * scale;
                        line_buf.clear();
                    }
                    if !line_buf.is_empty() {
                        line_buf.push(' ');
                    }
                    line_buf.push_str(word);
                }
                // Show href as tooltip indicator on first word
                let _ = href; // tooltip rendering deferred to hover event handling
            }
            InlineNode::HardBreak => {
                if !line_buf.is_empty() {
                    p.draw_text_cached(
                        cache,
                        &line_buf,
                        left,
                        line_y,
                        default_color,
                        base_font_size,
                        FontWeight::NORMAL,
                        false,
                        false,
                    );
                    flat_text.push_str(&line_buf);
                    line_buf.clear();
                }
                flat_text.push('\n');
                line_y += 20.0 * scale;
            }
            InlineNode::InlineImage { alt, .. } => {
                let label = alt.as_deref().unwrap_or("[img]");
                flat_text.push_str(label);
                line_buf.push_str(label);
            }
        }
    }

    // Flush remaining text using last marks
    if !line_buf.is_empty() {
        let line_x = compute_aligned_x(left, content_w, &line_buf, last_font_size, alignment);
        p.draw_text_cached(
            cache,
            &line_buf,
            line_x,
            line_y + last_y_offset,
            last_color,
            last_font_size,
            last_weight,
            false,
            last_italic,
        );
        // Underline for trailing flush
        if last_underline {
            let text_w = estimate_text_width(&line_buf, last_font_size);
            p.draw_line(
                Point::new(line_x, line_y + 2.0 * scale + last_y_offset),
                Point::new(line_x + text_w, line_y + 2.0 * scale + last_y_offset),
                last_color,
                1.0,
            );
        }
        // Strikethrough for trailing flush
        if last_strikethrough {
            let text_w = estimate_text_width(&line_buf, last_font_size);
            let mid_y = line_y - last_font_size as f64 / 2.0 + last_y_offset;
            p.draw_line(
                Point::new(line_x, mid_y),
                Point::new(line_x + text_w, mid_y),
                last_color,
                1.0,
            );
        }
        flat_text.push_str(&line_buf);
        line_y += 20.0 * scale;
    }

    (line_y, flat_text)
}

// ---------------------------------------------------------------------------
// Cursor rendering
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_cursor_for_block(
    p: &mut Painter<'_>,
    block_idx: usize,
    left: f64,
    block_start_y: f64,
    _content_w: f64,
    scale: f64,
    state: &KodocsState,
    block_text: &str,
) {
    let cursor = state.cursor();
    if cursor.block_idx != block_idx || !state.cursor_visible {
        return;
    }

    paint_cursor(p, left, block_start_y, scale, state, block_text);
}

pub(crate) fn paint_cursor(
    p: &mut Painter<'_>,
    left: f64,
    block_start_y: f64,
    scale: f64,
    state: &KodocsState,
    block_text: &str,
) {
    let cursor = state.cursor();
    let byte_offset = cursor.offset.min(block_text.len());

    // Convert byte offset to character offset for correct positioning
    let char_offset = block_text[..byte_offset].chars().count();

    let char_width = 7.0 * scale;
    let chars_per_line = ((content_width_estimate(scale)) / char_width) as usize;
    let line_offset = char_offset.checked_div(chars_per_line).unwrap_or(0);
    let col_offset = char_offset.saturating_sub(line_offset * chars_per_line);

    let cursor_x = left + col_offset as f64 * char_width;
    let cursor_y = block_start_y + line_offset as f64 * 20.0 * scale;
    let line_h = 18.0 * scale;

    // Draw cursor bar
    p.draw_line(
        Point::new(cursor_x, cursor_y - line_h * 0.8),
        Point::new(cursor_x, cursor_y + line_h * 0.2),
        c_accent(),
        1.5,
    );
}

pub(crate) fn content_width_estimate(scale: f64) -> f64 {
    let page_w = PAGE_W * scale;
    page_w - PAGE_PAD_X * 2.0 * scale
}

// ---------------------------------------------------------------------------
// Selection rendering
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_selection_for_block(
    p: &mut Painter<'_>,
    block_idx: usize,
    left: f64,
    block_start_y: f64,
    _content_w: f64,
    scale: f64,
    state: &KodocsState,
    block_text: &str,
) {
    let selection = match state.selection() {
        Some(sel) => sel,
        None => return,
    };

    let (start, end) = if selection.start <= selection.end {
        (&selection.start, &selection.end)
    } else {
        (&selection.end, &selection.start)
    };

    if block_idx < start.block_idx || block_idx > end.block_idx {
        return;
    }

    let char_width = 7.0 * scale;
    let chars_per_line = ((content_width_estimate(scale)) / char_width).max(1.0) as usize;
    let line_h = 20.0 * scale;
    let highlight_color = Color::rgba8(0xA7, 0x8B, 0xFA, 80);

    let sel_start = if block_idx == start.block_idx {
        start.offset
    } else {
        0
    };
    let sel_end = if block_idx == end.block_idx {
        end.offset.min(block_text.len())
    } else {
        block_text.len()
    };

    if sel_start >= sel_end {
        return;
    }

    // Convert byte offsets to char offsets for proper positioning
    let text_before_start = block_text.get(..sel_start).unwrap_or("");
    let text_selected = block_text.get(sel_start..sel_end).unwrap_or("");
    let char_start = text_before_start.chars().count();
    let char_count = text_selected.chars().count();

    let start_line = char_start / chars_per_line;
    let start_col = char_start % chars_per_line;
    let end_char = char_start + char_count;
    let end_line = end_char / chars_per_line;
    let end_col = end_char % chars_per_line;

    for line_idx in start_line..=end_line {
        let line_y = block_start_y + line_idx as f64 * line_h;
        let (col_start, col_end) = if line_idx == start_line && line_idx == end_line {
            (start_col, end_col)
        } else if line_idx == start_line {
            (start_col, chars_per_line)
        } else if line_idx == end_line {
            (0, end_col)
        } else {
            (0, chars_per_line)
        };

        if col_start < col_end {
            let rect = Rect::new(
                left + col_start as f64 * char_width,
                line_y - 14.0 * scale,
                left + col_end as f64 * char_width,
                line_y + 4.0 * scale,
            );
            p.fill_rect(rect, highlight_color);
        }
    }
}

// ---------------------------------------------------------------------------
