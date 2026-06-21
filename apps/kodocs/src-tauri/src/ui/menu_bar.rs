use super::state::{c_accent, c_menu_bg, c_separator, c_text_dim, c_text_light, KodocsState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

/// Korean menu bar items in order.
const MENU_NAMES: &[&str] = &["파일", "편집", "보기", "삽입", "서식", "도구", "도움말"];

/// Width for each menu item (Korean text is wider than English).
fn menu_width(name: &str) -> f64 {
    match name {
        "삽입" | "서식" | "도움말" => 50.0,
        _ => 42.0,
    }
}

pub fn paint_menu_bar(p: &mut Painter<'_>, cache: &mut TextCache, rect: Rect, state: &KodocsState) {
    p.fill_rect(rect, c_menu_bg());
    let mut x = rect.x0 + 12.0;
    let y = rect.y0 + rect.height() / 2.0;
    for name in MENU_NAMES {
        p.draw_text_cached(
            cache,
            name,
            x,
            y,
            c_text_light(),
            12.0,
            FontWeight::NORMAL,
            false,
            false,
        );
        x += menu_width(name);
    }
    // Right-side chrome content is shifted left of the caption button zone.
    let content_right = rect.x1 - WINDOW_CONTROLS_W;
    // Use real document title
    p.draw_text_cached(
        cache,
        state.title(),
        content_right - 280.0,
        y,
        c_text_dim(),
        12.0,
        FontWeight::NORMAL,
        false,
        false,
    );
    let pill = Rect::new(
        content_right - 80.0,
        rect.y0 + 8.0,
        content_right - 18.0,
        rect.y1 - 8.0,
    );
    p.stroke_rounded_rect(pill, c_separator(), 1.0, 999.0);
    // Korean save status
    let status_label = if state.is_dirty() {
        "저장 안 됨"
    } else {
        "저장됨"
    };
    p.draw_text_cached(
        cache,
        status_label,
        pill.x0 + 14.0,
        y,
        c_accent(),
        11.0,
        FontWeight::BOLD,
        false,
        false,
    );
    p.draw_line(
        Point::new(rect.x0, rect.y1 - 1.0),
        Point::new(rect.x1, rect.y1 - 1.0),
        c_separator(),
        1.0,
    );
    paint_window_controls(
        p,
        rect.x1,
        rect.height(),
        state.window_maximized,
        state.window_control_hovered,
    );
}
