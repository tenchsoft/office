use super::state::{c_accent, c_menu_bg, c_separator, c_text_dim, c_text_light, KodocsState};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;
use tench_ui::render::TextCache;

/// Korean menu bar items in order.
const MENU_NAMES: &[&str] = &[
    "파일",
    "편집",
    "보기",
    "삽입",
    "서식",
    "도구",
    "라이선스",
    "도움말",
];

/// Width for each menu item (Korean text is wider than English).
fn menu_width(name: &str) -> f64 {
    match name {
        "삽입" | "서식" | "도움말" | "라이선스" => 50.0,
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
    // Update notification label — only painted when the license is not
    // active. Two messages cycle every 5 seconds. Click handling lives in
    // pointer_events.rs (looks up the same rect via notification_label_rect).
    if let Some(msg) = notification_label_message(state) {
        let label_rect = notification_label_rect(rect);
        p.draw_text_cached(
            cache,
            msg,
            label_rect.x0,
            y,
            c_accent(),
            11.0,
            FontWeight::BOLD,
            false,
            false,
        );
    }
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

/// Returns the message to display in the notification label, or None if the
/// label should be hidden (license is active).
///
/// Behavior matches the spec:
/// - license unauthenticated + no update available: fixed "$1/month" message
/// - license unauthenticated + update available: 2-message cycle, 5s each
/// - license authenticated: hidden
pub(super) fn notification_label_message(state: &KodocsState) -> Option<&'static str> {
    if state.license_active {
        return None;
    }
    if state.update_available {
        // Cycle every 5 seconds.
        let cycle = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
            / 5)
            % 2;
        if cycle == 0 {
            Some("신규 업데이트 있음")
        } else {
            Some("월 $1 로 라이선스 활성화 가능")
        }
    } else {
        Some("월 $1 로 라이선스 활성화 가능")
    }
}

/// Geometry of the notification label inside the menu bar rect. Used by both
/// the painter and the click handler so they stay in sync.
pub(super) fn notification_label_rect(menu_rect: Rect) -> Rect {
    // Park the label just to the right of the "도움말" menu entry.
    // Menu entries start at menu_rect.x0 + 12 and use 42-50px each. With the
    // new "라이선스" entry added (50px), 도움말 ends around x0 + 380.
    let label_x = menu_rect.x0 + 392.0;
    Rect::new(label_x, menu_rect.y0, label_x + 220.0, menu_rect.y1)
}
