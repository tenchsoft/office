use super::super::state::{
    c_accent, c_btn_hover, c_menu_bg, c_separator, c_text_dim, c_text_light, DocsState, SidebarTab,
};
use tench_document_core::BlockNode;
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

pub fn paint_style_panel(
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

    paint_sidebar_tabs(p, cache, rect, state.sidebar_tab);

    match state.sidebar_tab {
        SidebarTab::Style => paint_style_content(p, cache, rect, state),
        SidebarTab::Navigate => paint_navigate_content(p, cache, rect, state),
        SidebarTab::Ai => paint_ai_placeholder(p, cache, rect),
    }
}

fn paint_style_content(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect, state: &DocsState) {
    let mut y = rect.y0 + 56.0;
    paint_section_title(p, cache, rect.x0 + 16.0, y, "STATISTICS");
    y += 18.0;
    let card_w = (rect.width() - 40.0) / 2.0;
    let words = state.word_count.to_string();
    let characters = state.character_count().to_string();
    let paragraphs = state.paragraph_count().to_string();
    let headings = state
        .document_lines()
        .iter()
        .filter(|line| line.starts_with("# "))
        .count()
        .to_string();
    let lists = state
        .document_lines()
        .iter()
        .filter(|line| line.trim_start().starts_with("- "))
        .count()
        .to_string();
    let read_time = format!("{} min", state.read_time_minutes());
    let stats = [
        ("Words", words.as_str()),
        ("Characters", characters.as_str()),
        ("Paragraphs", paragraphs.as_str()),
        ("Headings", headings.as_str()),
        ("Lists", lists.as_str()),
        ("Read time", read_time.as_str()),
    ];
    for row in 0..3 {
        for col in 0..2 {
            let idx = row * 2 + col;
            let x = rect.x0 + 16.0 + col as f64 * (card_w + 8.0);
            let card = Rect::new(x, y, x + card_w, y + 48.0);
            p.fill_rounded_rect(card, c_btn_hover(), 6.0);
            p.stroke_rounded_rect(card, c_separator(), 1.0, 6.0);
            p.draw_text_cached(
                cache,
                stats[idx].0,
                card.x0 + 8.0,
                card.y0 + 14.0,
                c_text_dim(),
                10.0,
                FontWeight::NORMAL,
                false,
                false,
            );
            p.draw_text_cached(
                cache,
                stats[idx].1,
                card.x0 + 8.0,
                card.y0 + 33.0,
                c_text_light(),
                12.0,
                FontWeight::BOLD,
                false,
                false,
            );
        }
        y += 56.0;
    }

    y += 8.0;
    paint_section_title(p, cache, rect.x0 + 16.0, y, "TYPOGRAPHY");
    y += 24.0;
    paint_sidebar_field(
        p,
        cache,
        rect.x0 + 16.0,
        y,
        rect.width() - 32.0,
        "Font size",
        "16px",
    );
    y += 32.0;
    paint_sidebar_field(
        p,
        cache,
        rect.x0 + 16.0,
        y,
        rect.width() - 32.0,
        "Line height",
        "1.6",
    );
    y += 44.0;

    paint_section_title(p, cache, rect.x0 + 16.0, y, "EXPORT");
    y += 24.0;
    paint_sidebar_field(
        p,
        cache,
        rect.x0 + 16.0,
        y,
        rect.width() - 32.0,
        "Format",
        "Word document",
    );
    y += 44.0;

    paint_section_title(p, cache, rect.x0 + 16.0, y, "RECOVERY");
    y += 24.0;
    paint_side_action(
        p,
        cache,
        Rect::new(rect.x0 + 16.0, y, rect.x1 - 16.0, y + 32.0),
        "Save snapshot",
    );
    y += 52.0;

    paint_section_title(p, cache, rect.x0 + 16.0, y, "STATUS");
    y += 24.0;
    p.draw_text_cached(
        cache,
        state.status_line(),
        rect.x0 + 16.0,
        y,
        c_text_dim(),
        12.0,
        FontWeight::NORMAL,
        false,
        false,
    );
}

/// Paint the Navigate sidebar tab showing document headings.
fn paint_navigate_content(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &DocsState,
) {
    let mut y = rect.y0 + 56.0;
    paint_section_title(p, cache, rect.x0 + 16.0, y, "OUTLINE");
    y += 22.0;

    let doc = state.current_document();
    let mut found_headings = false;
    for block in doc.content.iter() {
        if let BlockNode::Heading { content, level, .. } = block {
            found_headings = true;
            let mut text = String::new();
            for node in content {
                if let tench_document_core::InlineNode::Text { text: t, .. } = node {
                    text.push_str(t);
                }
            }
            if text.is_empty() {
                continue;
            }
            let indent = (*level as f64 - 1.0) * 16.0;
            let item_x = rect.x0 + 16.0 + indent;
            let max_w = rect.width() - 32.0 - indent;
            // Truncate text if too long
            let display: String = text.chars().take(30).collect();
            let truncated = if text.chars().count() > 30 {
                format!("{display}...")
            } else {
                display
            };
            let font_size = match level {
                1 => 13.0,
                2 => 12.0,
                _ => 11.0,
            };
            let weight = if *level <= 2 {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            };
            let color = if *level <= 2 {
                c_text_light()
            } else {
                c_text_dim()
            };

            let item_rect = Rect::new(item_x, y, item_x + max_w, y + 22.0);
            p.fill_rounded_rect(item_rect, c_btn_hover(), 4.0);

            p.draw_text_cached(
                cache,
                &truncated,
                item_x + 8.0,
                y + 12.0,
                color,
                font_size,
                weight,
                false,
                false,
            );
            y += 26.0;
            if y > rect.y1 - 30.0 {
                break;
            }
        }
    }
    if !found_headings {
        p.draw_text_cached(
            cache,
            "No headings in document",
            rect.x0 + 16.0,
            y + 8.0,
            c_text_dim(),
            11.0,
            FontWeight::NORMAL,
            false,
            false,
        );
    }
}

/// Paint the AI sidebar tab placeholder.
fn paint_ai_placeholder(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect) {
    let y = rect.y0 + 56.0;
    p.draw_text_cached(
        cache,
        "AI features coming soon",
        rect.x0 + 16.0,
        y + 8.0,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
}

fn paint_sidebar_tabs(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    active_tab: SidebarTab,
) {
    let tab_w = rect.width() / 3.0;
    for (idx, label) in ["Style", "Navigate", "AI"].iter().enumerate() {
        let x = rect.x0 + idx as f64 * tab_w;
        let tab = Rect::new(x, rect.y0, x + tab_w, rect.y0 + 36.0);
        let is_active = idx == active_tab as usize;
        if is_active {
            p.draw_line(
                Point::new(tab.x0 + 14.0, tab.y1 - 1.0),
                Point::new(tab.x1 - 14.0, tab.y1 - 1.0),
                c_accent(),
                2.0,
            );
        }
        p.draw_text_cached(
            cache,
            label,
            tab.x0 + tab.width() / 2.0,
            tab.y0 + tab.height() / 2.0,
            if is_active { c_accent() } else { c_text_dim() },
            11.0,
            FontWeight::BOLD,
            true,
            false,
        );
    }
    p.draw_line(
        Point::new(rect.x0, rect.y0 + 36.0),
        Point::new(rect.x1, rect.y0 + 36.0),
        c_separator(),
        1.0,
    );
}

fn paint_section_title(p: &mut Painter<'_>, cache: &mut TextCache, x: f64, y: f64, text: &str) {
    p.draw_text_cached(
        cache,
        text,
        x,
        y,
        c_text_light(),
        12.0,
        FontWeight::BOLD,
        false,
        false,
    );
}

fn paint_sidebar_field(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    x: f64,
    y: f64,
    width: f64,
    label: &str,
    value: &str,
) {
    p.draw_text_cached(
        cache,
        label,
        x,
        y + 16.0,
        c_text_dim(),
        12.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    let select = Rect::new(x + width - 128.0, y, x + width, y + 28.0);
    p.fill_rounded_rect(select, c_btn_hover(), 6.0);
    p.stroke_rounded_rect(select, c_separator(), 1.0, 6.0);
    p.draw_text_cached(
        cache,
        value,
        select.x0 + 8.0,
        select.y0 + 14.0,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
}

fn paint_side_action(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect, label: &str) {
    p.fill_rounded_rect(rect, c_btn_hover(), 6.0);
    p.stroke_rounded_rect(rect, c_separator(), 1.0, 6.0);
    p.draw_text_cached(
        cache,
        label,
        rect.x0 + rect.width() / 2.0,
        rect.y0 + rect.height() / 2.0,
        c_text_light(),
        12.0,
        FontWeight::BOLD,
        true,
        false,
    );
}
