use super::state::{
    c_accent, c_btn_hover, c_canvas_bg, c_menu_bg, c_page_bg, c_separator, c_status_bg, c_text_dim,
    c_text_light, KodocsState, PAGE_H, PAGE_W,
};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

pub fn paint_thumbnails(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect) {
    p.fill_rect(rect, c_canvas_bg());
    p.draw_line(
        Point::new(rect.x1 - 1.0, rect.y0),
        Point::new(rect.x1 - 1.0, rect.y1),
        c_separator(),
        1.0,
    );

    let thumb_w = rect.width() - 24.0;
    let thumb_h = thumb_w * (PAGE_H / PAGE_W);
    let mut y = rect.y0 + 12.0;
    for idx in 0..3 {
        let x = rect.x0 + 12.0;
        let thumb = Rect::new(x, y, x + thumb_w, y + thumb_h);
        p.fill_rounded_rect(thumb, c_page_bg(), 2.0);
        p.stroke_rounded_rect(thumb, c_separator(), 1.0, 2.0);
        p.draw_text_cached(
            cache,
            &format!("{}", idx + 1),
            x + thumb_w / 2.0 - 4.0,
            y + thumb_h + 4.0,
            c_text_dim(),
            10.0,
            FontWeight::NORMAL,
            false,
            false,
        );
        y += thumb_h + 20.0;
    }
}

pub fn paint_style_panel(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &KodocsState,
) {
    p.fill_rect(rect, c_menu_bg());
    p.draw_line(
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x0, rect.y1),
        c_separator(),
        1.0,
    );

    paint_sidebar_tabs(p, cache, rect);

    let mut y = rect.y0 + 56.0;
    paint_section_title(p, cache, rect.x0 + 16.0, y, "통계");
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
    let read_time = format!("{}분", state.read_time_minutes());
    let stats = [
        ("단어", words.as_str()),
        ("글자", characters.as_str()),
        ("단락", paragraphs.as_str()),
        ("제목", headings.as_str()),
        ("목록", lists.as_str()),
        ("읽기 시간", read_time.as_str()),
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
    paint_section_title(p, cache, rect.x0 + 16.0, y, "글꼴");
    y += 24.0;
    paint_sidebar_field(
        p,
        cache,
        rect.x0 + 16.0,
        y,
        rect.width() - 32.0,
        "글꼴 크기",
        "16px",
    );
    y += 32.0;
    paint_sidebar_field(
        p,
        cache,
        rect.x0 + 16.0,
        y,
        rect.width() - 32.0,
        "줄 간격",
        "1.6",
    );
    y += 44.0;

    paint_section_title(p, cache, rect.x0 + 16.0, y, "내보내기");
    y += 24.0;
    paint_sidebar_field(
        p,
        cache,
        rect.x0 + 16.0,
        y,
        rect.width() - 32.0,
        "형식",
        "한글 문서",
    );
    y += 44.0;

    paint_section_title(p, cache, rect.x0 + 16.0, y, "복구");
    y += 24.0;
    paint_side_action(
        p,
        cache,
        Rect::new(rect.x0 + 16.0, y, rect.x1 - 16.0, y + 32.0),
        "스냅샷 저장",
    );
    y += 52.0;

    paint_section_title(p, cache, rect.x0 + 16.0, y, "상태");
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

fn paint_sidebar_tabs(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect) {
    let tab_w = rect.width() / 3.0;
    for (idx, label) in ["스타일", "탐색", "AI"].iter().enumerate() {
        let x = rect.x0 + idx as f64 * tab_w;
        let tab = Rect::new(x, rect.y0, x + tab_w, rect.y0 + 36.0);
        if idx == 0 {
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
            if idx == 0 { c_accent() } else { c_text_dim() },
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

pub fn paint_comment_panel(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &KodocsState,
) {
    p.fill_rect(rect, c_menu_bg());
    p.draw_line(
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x0, rect.y1),
        c_separator(),
        1.0,
    );
    p.draw_text_cached(
        cache,
        "메모",
        rect.x0 + 12.0,
        rect.y0 + 16.0,
        c_text_light(),
        13.0,
        FontWeight::BOLD,
        false,
        false,
    );
    let mut y = rect.y0 + 42.0;
    if state.comments.is_empty() {
        p.draw_text_cached(
            cache,
            "메모가 없습니다",
            rect.x0 + 18.0,
            y + 4.0,
            c_text_dim(),
            11.0,
            FontWeight::NORMAL,
            false,
            false,
        );
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
            // Resolved indicator (Korean)
            if comment.resolved {
                p.draw_text_cached(
                    cache,
                    "해결됨",
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
    if state.track_changes {
        p.draw_text_cached(
            cache,
            "변경 내용 추적 중",
            rect.x0 + 12.0,
            rect.y1 - 20.0,
            c_text_dim(),
            11.0,
            FontWeight::BOLD,
            false,
            false,
        );
    }

    // Version history section (Korean)
    if !state.version_history.is_empty() {
        let vh_y = if state.comments.is_empty() {
            rect.y0 + 56.0
        } else {
            y + 8.0
        };
        p.draw_line(
            Point::new(rect.x0 + 12.0, vh_y),
            Point::new(rect.x1 - 12.0, vh_y),
            c_separator(),
            1.0,
        );
        p.draw_text_cached(
            cache,
            "버전 기록",
            rect.x0 + 12.0,
            vh_y + 18.0,
            c_text_light(),
            12.0,
            FontWeight::BOLD,
            false,
            false,
        );
        let mut vh_entry_y = vh_y + 36.0;
        for entry in &state.version_history {
            if vh_entry_y > rect.y1 - 30.0 {
                break;
            }
            let ts_display = if entry.timestamp > 0 {
                format!(" ({})", entry.timestamp)
            } else {
                String::new()
            };
            let display_label = format!("{}{}", entry.label, ts_display);
            p.draw_text_cached(
                cache,
                &display_label,
                rect.x0 + 18.0,
                vh_entry_y,
                c_text_light(),
                10.0,
                FontWeight::NORMAL,
                false,
                false,
            );
            // Show path and size on second line
            let detail = if entry.size_bytes > 0 {
                format!("{} ({} 바이트)", entry.path, entry.size_bytes)
            } else {
                entry.path.clone()
            };
            p.draw_text_cached(
                cache,
                &detail,
                rect.x0 + 18.0,
                vh_entry_y + 12.0,
                c_text_dim(),
                9.0,
                FontWeight::NORMAL,
                false,
                false,
            );
            vh_entry_y += 28.0;
        }
    }
}

pub fn paint_status_bar(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &KodocsState,
) {
    p.fill_rect(rect, c_status_bg());
    p.draw_line(
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x1, rect.y0),
        c_separator(),
        1.0,
    );
    let y = rect.y0 + rect.height() / 2.0;
    p.draw_text_cached(
        cache,
        &format!("{} 단어", state.word_count),
        rect.x0 + 12.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    p.draw_text_cached(
        cache,
        &format!("{} 글자", state.character_count()),
        rect.x0 + 96.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    p.draw_text_cached(
        cache,
        &format!("{} 단락", state.paragraph_count()),
        rect.x0 + 220.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    p.draw_text_cached(
        cache,
        &format!("{}분 읽기", state.read_time_minutes()),
        rect.x0 + 320.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    p.draw_text_cached(
        cache,
        state.status_line(),
        rect.x0 + 430.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    paint_status_button(
        p,
        cache,
        Rect::new(
            rect.x1 - 252.0,
            rect.y0 + 4.0,
            rect.x1 - 174.0,
            rect.y1 - 4.0,
        ),
        &format!("페이지 {}/{}", state.current_page, state.page_count),
    );
    p.draw_text_cached(
        cache,
        &state.language,
        rect.x1 - 344.0,
        y,
        c_text_dim(),
        11.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    paint_status_button(
        p,
        cache,
        Rect::new(
            rect.x1 - 160.0,
            rect.y0 + 4.0,
            rect.x1 - 136.0,
            rect.y1 - 4.0,
        ),
        "-",
    );
    p.draw_text_cached(
        cache,
        &format!("확대/축소 {:.0}%", state.zoom),
        rect.x1 - 96.0,
        y,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        true,
        false,
    );
    paint_status_button(
        p,
        cache,
        Rect::new(rect.x1 - 52.0, rect.y0 + 4.0, rect.x1 - 28.0, rect.y1 - 4.0),
        "+",
    );
    if state.track_changes {
        p.draw_text_cached(
            cache,
            "변경 추적",
            rect.x1 - 170.0,
            y,
            c_text_light(),
            11.0,
            FontWeight::BOLD,
            false,
            false,
        );
    }
}

fn paint_status_button(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect, label: &str) {
    p.fill_rounded_rect(rect, c_toolbar_bg_compat(), 4.0);
    p.stroke_rounded_rect(rect, c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        label,
        rect.x0 + rect.width() / 2.0,
        rect.y0 + rect.height() / 2.0,
        c_text_light(),
        11.0,
        FontWeight::NORMAL,
        true,
        false,
    );
}

fn c_toolbar_bg_compat() -> Color {
    Color::rgb8(0x1A, 0x1A, 0x1A)
}
