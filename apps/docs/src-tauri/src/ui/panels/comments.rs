use super::super::state::{
    c_accent, c_canvas_bg, c_menu_bg, c_separator, c_text_dim, c_text_light, DocsState,
};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

pub fn paint_comment_panel(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &DocsState,
) {
    p.fill_rect(rect, c_menu_bg());
    p.draw_line(
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x0, rect.y1),
        c_separator(),
        1.0,
    );

    let mut y = rect.y0 + 12.0;

    // --- Comments section header ---
    let comments_arrow = if state.comments_collapsed {
        "\u{25B8}"
    } else {
        "\u{25BE}"
    };
    let comments_header = format!("{} Comments", comments_arrow);
    p.draw_text_cached(
        cache,
        &comments_header,
        rect.x0 + 12.0,
        y + 10.0,
        c_text_light(),
        13.0,
        FontWeight::BOLD,
        false,
        false,
    );
    y += 30.0;

    // Comments content (if not collapsed)
    if !state.comments_collapsed {
        if state.comments.is_empty() {
            p.draw_text_cached(
                cache,
                "No comments yet",
                rect.x0 + 18.0,
                y + 4.0,
                c_text_dim(),
                11.0,
                FontWeight::NORMAL,
                false,
                false,
            );
            y += 24.0;
        } else {
            for comment in &state.comments {
                let bg_color = if comment.resolved {
                    Color::rgb8(0x1A, 0x2A, 0x1A)
                } else {
                    c_canvas_bg()
                };
                p.fill_rounded_rect(
                    Rect::new(rect.x0 + 10.0, y - 12.0, rect.x1 - 10.0, y + 40.0),
                    bg_color,
                    4.0,
                );
                // Author
                p.draw_text_cached(
                    cache,
                    &comment.author,
                    rect.x0 + 18.0,
                    y - 2.0,
                    c_accent(),
                    10.0,
                    FontWeight::BOLD,
                    false,
                    false,
                );
                // Comment text
                p.draw_text_cached(
                    cache,
                    &comment.text,
                    rect.x0 + 18.0,
                    y + 14.0,
                    c_text_light(),
                    11.0,
                    FontWeight::NORMAL,
                    false,
                    false,
                );
                // Resolved indicator
                if comment.resolved {
                    p.draw_text_cached(
                        cache,
                        "Resolved",
                        rect.x1 - 70.0,
                        y - 2.0,
                        Color::rgb8(0x66, 0xBB, 0x6A),
                        9.0,
                        FontWeight::BOLD,
                        false,
                        false,
                    );
                }
                y += 56.0;
            }
        }
    }

    // Separator between sections
    y += 4.0;
    p.draw_line(
        Point::new(rect.x0 + 12.0, y),
        Point::new(rect.x1 - 12.0, y),
        c_separator(),
        1.0,
    );
    y += 12.0;

    // --- Version History section header ---
    let vh_arrow = if state.version_history_collapsed {
        "\u{25B8}"
    } else {
        "\u{25BE}"
    };
    let vh_header = format!("{} Version History", vh_arrow);
    p.draw_text_cached(
        cache,
        &vh_header,
        rect.x0 + 12.0,
        y + 4.0,
        c_text_light(),
        12.0,
        FontWeight::BOLD,
        false,
        false,
    );
    y += 22.0;

    // Version History content (if not collapsed)
    if !state.version_history_collapsed {
        if state.version_history.is_empty() {
            p.draw_text_cached(
                cache,
                "No version history",
                rect.x0 + 18.0,
                y + 4.0,
                c_text_dim(),
                11.0,
                FontWeight::NORMAL,
                false,
                false,
            );
        } else {
            for entry in &state.version_history {
                if y > rect.y1 - 30.0 {
                    break;
                }
                let ts_display = if !entry.timestamp_label.is_empty() {
                    format!(" ({})", entry.timestamp_label)
                } else {
                    String::new()
                };
                let display_label = format!("{}{}", entry.label, ts_display);
                p.draw_text_cached(
                    cache,
                    &display_label,
                    rect.x0 + 18.0,
                    y,
                    c_text_light(),
                    10.0,
                    FontWeight::NORMAL,
                    false,
                    false,
                );
                // Show path and size on second line
                let detail = if entry.size_bytes > 0 {
                    format!("{} ({} bytes)", entry.path, entry.size_bytes)
                } else {
                    entry.path.clone()
                };
                p.draw_text_cached(
                    cache,
                    &detail,
                    rect.x0 + 18.0,
                    y + 12.0,
                    c_text_dim(),
                    9.0,
                    FontWeight::NORMAL,
                    false,
                    false,
                );
                y += 28.0;
            }
        }
    }

    // Track changes indicator at the bottom
    if state.track_changes {
        p.draw_text_cached(
            cache,
            "Track changes enabled",
            rect.x0 + 12.0,
            rect.y1 - 20.0,
            c_text_dim(),
            11.0,
            FontWeight::BOLD,
            false,
            false,
        );
    }
}
