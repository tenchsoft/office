use super::*;

// Helpers
// ---------------------------------------------------------------------------

/// Extract plain text from inline nodes.
pub(crate) fn inline_nodes_to_text(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            InlineNode::Text { text, .. } => out.push_str(text),
            InlineNode::Link { text, .. } => out.push_str(text),
            InlineNode::InlineImage { alt, .. } => {
                if let Some(a) = alt {
                    out.push_str(a);
                }
            }
            InlineNode::HardBreak => out.push('\n'),
        }
    }
    out
}

/// Simple word-wrap: split text into lines that fit within `content_w`.
pub(crate) fn wrap_text(text: &str, content_w: f64, font_size: f32, scale: f64) -> Vec<String> {
    let chars_per_line = ((content_w / (font_size as f64 * 0.55).max(1.0)) as usize).max(16);
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut line = String::new();

    for word in words {
        if line.len() + word.len() + 1 > chars_per_line && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }
    let _ = scale; // scale used via font_size
    lines
}

/// Rough estimate of text width for positioning.
pub(crate) fn estimate_text_width(text: &str, font_size: f32) -> f64 {
    text.len() as f64 * font_size as f64 * 0.55
}

/// Parse a hex color string like "#FF0000" or "rgb(255,0,0)".
pub(crate) fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let hex = hex.trim();
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::rgb8(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::rgba8(r, g, b, a))
            }
            _ => None,
        }
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Multi-page helpers
// ---------------------------------------------------------------------------

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
/// Render the page header area. If a header template is configured, it is
/// rendered with auto-field substitution. Otherwise a subtle separator is drawn.
pub(crate) fn paint_page_header(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    left: f64,
    y: f64,
    content_w: f64,
    header_h: f64,
    scale: f64,
    page: usize,
    pages: usize,
    hf: &HeadersFooters,
    title: &str,
    editing: bool,
    editing_text: Option<&str>,
) {
    // Highlight header area when in editing mode
    if editing {
        let header_rect = Rect::new(left, y, left + content_w, y + header_h);
        p.fill_rect(header_rect, Color::rgba8(0xA7, 0x8B, 0xFA, 20));
        p.stroke_rounded_rect(header_rect, c_accent(), 1.0, 2.0);
    }

    // When actively editing, show the live buffer text
    if let Some(text) = editing_text {
        if !text.is_empty() {
            p.draw_text_cached(
                cache,
                text,
                left,
                y + header_h / 2.0,
                c_text_light(),
                (10.0 * scale) as f32,
                FontWeight::NORMAL,
                false,
                false,
            );
        } else {
            p.draw_text_cached(
                cache,
                "Type header...",
                left,
                y + header_h / 2.0,
                c_text_dim(),
                (10.0 * scale) as f32,
                FontWeight::NORMAL,
                false,
                false,
            );
        }
    } else if let Some(header_text) = hf.header_for_page(page, pages, title) {
        p.draw_text_cached(
            cache,
            &header_text,
            left,
            y + header_h / 2.0,
            if editing {
                c_text_light()
            } else {
                c_text_dim()
            },
            (10.0 * scale) as f32,
            FontWeight::NORMAL,
            false,
            false,
        );
    } else if editing {
        // Show placeholder when editing
        p.draw_text_cached(
            cache,
            "Type header...",
            left,
            y + header_h / 2.0,
            c_text_dim(),
            (10.0 * scale) as f32,
            FontWeight::NORMAL,
            false,
            false,
        );
    }
    // Separator line at bottom of header
    let line_y = y + header_h;
    p.draw_line(
        Point::new(left, line_y),
        Point::new(left + content_w, line_y),
        c_separator(),
        1.0,
    );
}

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
/// Render the page footer area. If a footer template is configured, it is
/// rendered with auto-field substitution. Otherwise a page number is shown.
pub(crate) fn paint_page_footer(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    left: f64,
    y: f64,
    content_w: f64,
    footer_h: f64,
    scale: f64,
    page: usize,
    pages: usize,
    hf: &HeadersFooters,
    title: &str,
    editing: bool,
    editing_text: Option<&str>,
) {
    // Highlight footer area when in editing mode
    if editing {
        let footer_rect = Rect::new(left, y, left + content_w, y + footer_h);
        p.fill_rect(footer_rect, Color::rgba8(0xA7, 0x8B, 0xFA, 20));
        p.stroke_rounded_rect(footer_rect, c_accent(), 1.0, 2.0);
    }

    // Separator line at top of footer
    p.draw_line(
        Point::new(left, y),
        Point::new(left + content_w, y),
        c_separator(),
        1.0,
    );
    // When actively editing, show the live buffer text
    if let Some(text) = editing_text {
        if !text.is_empty() {
            p.draw_text_cached(
                cache,
                text,
                left,
                y + footer_h / 2.0,
                c_text_light(),
                (10.0 * scale) as f32,
                FontWeight::NORMAL,
                false,
                false,
            );
        } else {
            p.draw_text_cached(
                cache,
                "Type footer...",
                left,
                y + footer_h / 2.0,
                c_text_dim(),
                (10.0 * scale) as f32,
                FontWeight::NORMAL,
                false,
                false,
            );
        }
    } else if let Some(footer_text) = hf.footer_for_page(page, pages, title) {
        p.draw_text_cached(
            cache,
            &footer_text,
            left,
            y + footer_h / 2.0,
            if editing {
                c_text_light()
            } else {
                c_text_dim()
            },
            (10.0 * scale) as f32,
            FontWeight::NORMAL,
            false,
            false,
        );
    } else if editing {
        // Show placeholder when editing
        p.draw_text_cached(
            cache,
            "Type footer...",
            left,
            y + footer_h / 2.0,
            c_text_dim(),
            (10.0 * scale) as f32,
            FontWeight::NORMAL,
            false,
            false,
        );
    } else {
        // Default: show page number centered
        let page_label = format!("{}", page);
        p.draw_text_cached(
            cache,
            &page_label,
            left + content_w / 2.0,
            y + footer_h / 2.0,
            c_text_dim(),
            (10.0 * scale) as f32,
            FontWeight::NORMAL,
            true,
            false,
        );
    }
}

/// Estimate the total content height in pixels for all blocks in the document.
pub(crate) fn estimate_total_content_height(
    doc: &tench_document_core::TenchDocument,
    content_w: f64,
    scale: f64,
) -> f64 {
    let line_h = 20.0 * scale;
    let block_spacing = 12.0 * scale;
    let chars_per_line = if content_w > 0.0 {
        ((content_w / (7.0 * scale.max(0.6))) as usize).max(24)
    } else {
        80
    };

    let mut total = 0.0;
    for block in &doc.content {
        total += estimate_block_height(block, chars_per_line, line_h, block_spacing, scale);
    }
    total
}

/// Walk through all blocks, estimating heights, and produce a map that tells
/// each page which block index to start rendering from.
pub(crate) fn compute_page_map(
    doc: &tench_document_core::TenchDocument,
    content_w: f64,
    page_content_h: f64,
    scale: f64,
) -> Vec<super::super::super::state::PageMapEntry> {
    let mut entries = Vec::new();
    let line_h = 20.0 * scale;
    let block_spacing = 12.0 * scale;
    let chars_per_line = if content_w > 0.0 {
        ((content_w / (7.0 * scale.max(0.6))) as usize).max(24)
    } else {
        80
    };

    let mut accumulated_h: f64 = 0.0;

    entries.push(super::super::super::state::PageMapEntry { start_block: 0 });

    for (i, block) in doc.content.iter().enumerate() {
        let block_h = estimate_block_height(block, chars_per_line, line_h, block_spacing, scale);

        if accumulated_h + block_h > page_content_h && accumulated_h > 0.0 {
            // This block starts a new page
            entries.push(super::super::super::state::PageMapEntry { start_block: i });
            accumulated_h = block_h;
        } else {
            accumulated_h += block_h;
        }
    }

    entries
}

/// Estimate the height of a single block in pixels.
pub(crate) fn estimate_block_height(
    block: &BlockNode,
    chars_per_line: usize,
    line_h: f64,
    block_spacing: f64,
    scale: f64,
) -> f64 {
    match block {
        BlockNode::Paragraph { content, .. } => {
            let text = inline_nodes_to_text(content);
            let lines = if text.is_empty() {
                1
            } else {
                text.chars().count().div_ceil(chars_per_line)
            };
            lines as f64 * line_h + block_spacing
        }
        BlockNode::Heading { content, level, .. } => {
            let text = inline_nodes_to_text(content);
            let heading_line_h = match level {
                1 => 36.0 * scale,
                2 => 32.0 * scale,
                3 => 28.0 * scale,
                _ => 20.0 * scale,
            };
            let lines = if text.is_empty() {
                1
            } else {
                text.chars().count().div_ceil(chars_per_line)
            };
            lines as f64 * heading_line_h + 8.0 * scale + block_spacing
        }
        BlockNode::CodeBlock { code, .. } => {
            let lines = code.lines().count().max(1);
            lines as f64 * 18.0 * scale + 24.0 * scale + block_spacing
        }
        BlockNode::HorizontalRule => 24.0 * scale + block_spacing,
        BlockNode::PageBreak => 24.0 * scale + block_spacing,
        BlockNode::Image { height, .. } => {
            height.unwrap_or(300.0) as f64 * scale + 24.0 * scale + block_spacing
        }
        BlockNode::BulletList { items } | BlockNode::OrderedList { items, .. } => {
            let total: f64 = items
                .iter()
                .map(|i| {
                    let text = inline_nodes_to_text(&i.content);
                    let lines = if text.is_empty() {
                        1
                    } else {
                        text.chars().count().div_ceil(chars_per_line)
                    };
                    lines as f64 * line_h + 4.0 * scale
                })
                .sum();
            total + 8.0 * scale + block_spacing
        }
        BlockNode::TaskList { items } => {
            let total: f64 = items
                .iter()
                .map(|i| {
                    let text = inline_nodes_to_text(&i.content);
                    let lines = if text.is_empty() {
                        1
                    } else {
                        text.chars().count().div_ceil(chars_per_line)
                    };
                    lines as f64 * line_h + 4.0 * scale
                })
                .sum();
            total + 8.0 * scale + block_spacing
        }
        BlockNode::BlockQuote { content } => {
            let inner: f64 = content
                .iter()
                .map(|b| estimate_block_height(b, chars_per_line, line_h, block_spacing, scale))
                .sum();
            inner + 8.0 * scale + block_spacing
        }
        BlockNode::Table { rows } => {
            let row_h = 24.0 * scale;
            rows.len().max(1) as f64 * row_h + 12.0 * scale + block_spacing
        }
        BlockNode::Footnote { content, .. } => {
            let text = inline_nodes_to_text(content);
            let lines = if text.is_empty() {
                1
            } else {
                text.chars().count().div_ceil(chars_per_line)
            };
            lines as f64 * line_h + block_spacing
        }
    }
}

// ---------------------------------------------------------------------------
// Search highlight rendering
// ---------------------------------------------------------------------------

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
/// Highlight all search matches in a block. The current match gets a brighter
/// colour; other matches get a subtle yellow background.
pub(crate) fn paint_search_highlights(
    p: &mut Painter<'_>,
    block_idx: usize,
    left: f64,
    block_start_y: f64,
    _content_w: f64,
    scale: f64,
    state: &DocsState,
    block_text: &str,
) {
    let fr = match &state.find_replace {
        Some(fr) if !fr.matches.is_empty() => fr,
        _ => return,
    };

    let char_width = 7.0 * scale;
    let chars_per_line = ((content_width_estimate(scale)) / char_width).max(1.0) as usize;
    let line_h = 20.0 * scale;

    for (mi, m) in fr.matches.iter().enumerate() {
        if m.block_idx != block_idx {
            continue;
        }
        let is_current = fr.current_match_idx == Some(mi);
        paint_range_highlight(
            p,
            left,
            block_start_y,
            scale,
            block_text,
            m.start_offset,
            m.end_offset,
            char_width,
            chars_per_line,
            line_h,
            if is_current {
                Color::rgba8(0xFF, 0xE0, 0x00, 120) // bright yellow for current match
            } else {
                Color::rgba8(0xFF, 0xE0, 0x00, 60) // subtle yellow for other matches
            },
        );
    }
}

// ---------------------------------------------------------------------------
// Comment highlight rendering
// ---------------------------------------------------------------------------

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
/// Highlight ranges that have comments attached.
pub(crate) fn paint_comment_highlights(
    p: &mut Painter<'_>,
    block_idx: usize,
    left: f64,
    block_start_y: f64,
    _content_w: f64,
    scale: f64,
    state: &DocsState,
    block_text: &str,
) {
    if state.comments.is_empty() {
        return;
    }

    let char_width = 7.0 * scale;
    let chars_per_line = ((content_width_estimate(scale)) / char_width).max(1.0) as usize;
    let line_h = 20.0 * scale;

    for comment in &state.comments {
        if comment.range.block_idx != block_idx {
            continue;
        }
        paint_range_highlight(
            p,
            left,
            block_start_y,
            scale,
            block_text,
            comment.range.start_offset,
            comment.range.end_offset,
            char_width,
            chars_per_line,
            line_h,
            Color::rgba8(0xFF, 0xEB, 0x3B, 50), // light yellow for comment ranges
        );
    }
}

// ---------------------------------------------------------------------------
// Track changes decoration rendering
// ---------------------------------------------------------------------------

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
/// Draw decorations for tracked changes: green underline for inserts,
/// red strikethrough for deletes.
pub(crate) fn paint_track_change_decorations(
    p: &mut Painter<'_>,
    _block_idx: usize,
    left: f64,
    block_start_y: f64,
    _content_w: f64,
    scale: f64,
    state: &DocsState,
    block_text: &str,
) {
    if !state.track_changes {
        return;
    }

    let char_width = 7.0 * scale;
    let chars_per_line = ((content_width_estimate(scale)) / char_width).max(1.0) as usize;
    let line_h = 20.0 * scale;

    // Get tracked changes from the engine — they're stored in state
    for change in &state.tracked_changes {
        if change.block_idx != _block_idx {
            continue;
        }

        let so = change.start_offset.min(block_text.len());
        let eo = change.end_offset.min(block_text.len());
        if so >= eo {
            continue;
        }

        // Convert byte offsets to char offsets
        let text_before = block_text.get(..so).unwrap_or("");
        let text_selected = block_text.get(so..eo).unwrap_or("");
        let char_start = text_before.chars().count();
        let char_count = text_selected.chars().count();

        let start_line = char_start / chars_per_line;
        let start_col = char_start % chars_per_line;
        let end_char = char_start + char_count;
        let end_line = end_char / chars_per_line;
        let end_col = end_char % chars_per_line;

        let (decoration_color, is_underline) = match change.change_type {
            ChangeType::Insert => (Color::rgba8(0x66, 0xBB, 0x6A, 200), true), // green underline
            ChangeType::Delete => (Color::rgba8(0xEF, 0x53, 0x50, 200), false), // red strikethrough
        };

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
                let x0 = left + col_start as f64 * char_width;
                let x1 = left + col_end as f64 * char_width;
                if is_underline {
                    // Green underline for inserts
                    p.draw_line(
                        Point::new(x0, line_y + 3.0 * scale),
                        Point::new(x1, line_y + 3.0 * scale),
                        decoration_color,
                        2.0,
                    );
                } else {
                    // Red strikethrough for deletes
                    let mid_y = line_y - 6.0 * scale;
                    p.draw_line(
                        Point::new(x0, mid_y),
                        Point::new(x1, mid_y),
                        decoration_color,
                        2.0,
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared range highlight helper
// ---------------------------------------------------------------------------

// UI painting functions need many layout parameters
#[allow(clippy::too_many_arguments)]
/// Paint a highlighted range within a block, converting byte offsets to
/// character positions and wrapping across lines as needed.
pub(crate) fn paint_range_highlight(
    p: &mut Painter<'_>,
    left: f64,
    block_start_y: f64,
    scale: f64,
    block_text: &str,
    start_offset: usize,
    end_offset: usize,
    char_width: f64,
    chars_per_line: usize,
    line_h: f64,
    color: Color,
) {
    let so = start_offset.min(block_text.len());
    let eo = end_offset.min(block_text.len());
    if so >= eo {
        return;
    }

    // Convert byte offsets to char offsets
    let text_before = block_text.get(..so).unwrap_or("");
    let text_selected = block_text.get(so..eo).unwrap_or("");
    let char_start = text_before.chars().count();
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
            p.fill_rect(rect, color);
        }
    }
}
