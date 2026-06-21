use super::*;

pub(super) fn paint_title_row(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    rect: Rect,
    state: &KodocsState,
) {
    p.fill_rect(rect, c_canvas_bg());
    p.draw_text_cached(
        cache,
        state.title(),
        rect.x0 + 20.0,
        rect.y0 + rect.height() / 2.0,
        c_text_light(),
        18.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );
    p.draw_text_cached(
        cache,
        state.status_line(),
        rect.x0 + 190.0,
        rect.y0 + rect.height() / 2.0,
        c_text_dim(),
        12.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );
    p.draw_line(
        Point::new(rect.x0, rect.y1 - 1.0),
        Point::new(rect.x1, rect.y1 - 1.0),
        c_separator(),
        1.0,
    );
}

/// Korean menu bar items (must match menu_bar.rs MENU_NAMES).
const MENU_NAMES_KO: &[&str] = &["파일", "편집", "보기", "삽입", "서식", "도구", "도움말"];

/// Width for each menu item (must match menu_bar.rs menu_width).
fn menu_width(name: &str) -> f64 {
    match name {
        "삽입" | "서식" | "도움말" => 50.0,
        _ => 42.0,
    }
}

pub(super) fn menu_at(x: f64) -> Option<&'static str> {
    let mut left = 12.0;
    for name in MENU_NAMES_KO {
        let width = menu_width(name);
        if x >= left && x < left + width {
            return Some(*name);
        }
        left += width;
    }
    None
}

/// Return the menu items for a given Korean menu name.
pub(super) fn menu_items_for(menu_name: &str) -> Vec<&'static str> {
    match menu_name {
        "파일" => vec![
            "새 문서",
            "열기",
            "저장",
            "다른 이름으로 저장",
            "내보내기",
            "페이지 설정",
            "인쇄",
            "버전 기록",
            // HWP/HWPX support
            "HWP 가져오기",
            "HWPX 내보내기",
        ],
        "편집" => vec![
            "실행 취소",
            "다시 실행",
            "잘라내기",
            "복사",
            "붙여넣기",
            "모두 선택",
            "찾기",
            "바꾸기",
        ],
        "보기" => vec![
            "미리보기",
            "스타일 패널",
            "메모",
            "확대",
            "축소",
            "확대/축소 초기화",
            "세로쓰기",
        ],
        "삽입" => vec![
            "그림",
            "표",
            "링크",
            "가로줄",
            "페이지 나누기",
            "머리글",
            "바닥글",
            "수식",
        ],
        "서식" => vec![
            "굵게",
            "기울임",
            "밑줄",
            "취소선",
            "위 첨자",
            "아래 첨자",
            "서식 지우기",
            "인용",
            "한자 변환",
        ],
        "도구" => vec!["단어 수", "변경 내용 추적", "맞춤법 검사"],
        "도움말" => vec!["정보", "키보드 단축키"],
        _ => vec![],
    }
}

pub(super) fn paint_docs_modal(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    title: &str,
    hovered_item: Option<usize>,
) {
    let items = menu_items_for(title);
    let item_h = 26.0;
    let modal_w = 220.0;
    let modal_h = 8.0 + items.len() as f64 * item_h + 8.0;
    let modal_x = 12.0; // aligns with menu bar left padding
    let modal_y = MENU_BAR_H;

    let modal = Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
    p.fill_rounded_rect(modal, state::c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, state::c_separator(), 1.0, 6.0);

    for (idx, item) in items.iter().enumerate() {
        let item_y = modal_y + 8.0 + idx as f64 * item_h;
        let item_rect = Rect::new(
            modal_x + 4.0,
            item_y,
            modal_x + modal_w - 4.0,
            item_y + item_h,
        );

        // Hover highlight
        if hovered_item == Some(idx) {
            p.fill_rounded_rect(item_rect, state::c_accent(), 4.0);
        }

        let text_color = if hovered_item == Some(idx) {
            Color::WHITE
        } else {
            state::c_text_light()
        };
        p.draw_text_cached(
            cache,
            item,
            modal_x + 14.0,
            item_y + item_h / 2.0,
            text_color,
            12.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );

        // Render shortcut on the right side
        if let Some(shortcut) = menu_shortcut_ko(item) {
            p.draw_text_cached(
                cache,
                shortcut,
                modal_x + modal_w - 14.0,
                item_y + item_h / 2.0,
                state::c_text_dim(),
                10.0,
                tench_ui::parley::FontWeight::NORMAL,
                true,
                false,
            );
        }
    }
}

/// Return the Korean shortcut label for a menu item, if one exists.
fn menu_shortcut_ko(item: &str) -> Option<&'static str> {
    match item {
        "새 문서" => Some("Ctrl+N"),
        "열기" => Some("Ctrl+O"),
        "저장" => Some("Ctrl+S"),
        "다른 이름으로 저장" => Some("Ctrl+Shift+S"),
        "인쇄" => Some("Ctrl+P"),
        "실행 취소" => Some("Ctrl+Z"),
        "다시 실행" => Some("Ctrl+Y"),
        "잘라내기" => Some("Ctrl+X"),
        "복사" => Some("Ctrl+C"),
        "붙여넣기" => Some("Ctrl+V"),
        "모두 선택" => Some("Ctrl+A"),
        "찾기" => Some("Ctrl+F"),
        "바꾸기" => Some("Ctrl+H"),
        "굵게" => Some("Ctrl+B"),
        "기울임" => Some("Ctrl+I"),
        "밑줄" => Some("Ctrl+U"),
        "확대" => Some("Ctrl++"),
        "축소" => Some("Ctrl+-"),
        "확대/축소 초기화" => Some("Ctrl+0"),
        "링크" => Some("Ctrl+K"),
        "한자 변환" => Some("F9"),
        _ => None,
    }
}

/// Paint the right-click context menu.
pub(super) fn paint_context_menu(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    _size: Size,
    ctx_menu: &state::ContextMenuState,
) {
    let items = state::context_menu_items(&ctx_menu.menu_type);
    let item_h = 28.0;
    let menu_w = 200.0;
    let menu_h = items.len() as f64 * item_h + 8.0;

    let menu_rect = Rect::new(
        ctx_menu.x,
        ctx_menu.y,
        ctx_menu.x + menu_w,
        ctx_menu.y + menu_h,
    );
    p.fill_rounded_rect(menu_rect, state::c_menu_bg(), 6.0);
    p.stroke_rounded_rect(menu_rect, state::c_separator(), 1.0, 6.0);

    for (idx, item) in items.iter().enumerate() {
        let item_y = ctx_menu.y + 4.0 + idx as f64 * item_h;
        let item_rect = Rect::new(
            ctx_menu.x + 4.0,
            item_y,
            ctx_menu.x + menu_w - 4.0,
            item_y + item_h,
        );

        if ctx_menu.hovered_item == Some(idx) {
            p.fill_rounded_rect(item_rect, state::c_accent(), 4.0);
        }

        let text_color = if ctx_menu.hovered_item == Some(idx) {
            Color::WHITE
        } else {
            state::c_text_light()
        };
        p.draw_text_cached(
            cache,
            item,
            ctx_menu.x + 14.0,
            item_y + item_h / 2.0,
            text_color,
            12.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
    }
}

pub(super) fn paint_link_modal(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    size: Size,
    link_state: &LinkModalState,
) {
    let modal_w = 380.0;
    let modal_h = 160.0;
    let modal = Rect::new(
        size.width / 2.0 - modal_w / 2.0,
        size.height / 2.0 - modal_h / 2.0,
        size.width / 2.0 + modal_w / 2.0,
        size.height / 2.0 + modal_h / 2.0,
    );

    // Semi-transparent backdrop
    let backdrop = Rect::new(0.0, 0.0, size.width, size.height);
    p.fill_rect(backdrop, Color::rgba8(0, 0, 0, 100));

    p.fill_rounded_rect(modal, state::c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, state::c_separator(), 1.0, 6.0);

    // Title (Korean)
    p.draw_text_cached(
        cache,
        "하이퍼링크 삽입",
        modal.x0 + 16.0,
        modal.y0 + 28.0,
        state::c_text_light(),
        14.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );

    // URL label
    p.draw_text_cached(
        cache,
        "주소:",
        modal.x0 + 16.0,
        modal.y0 + 58.0,
        state::c_text_dim(),
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    // URL input field
    let input_rect = Rect::new(
        modal.x0 + 16.0,
        modal.y0 + 68.0,
        modal.x1 - 16.0,
        modal.y0 + 94.0,
    );
    p.fill_rounded_rect(input_rect, Color::rgba8(0x0A, 0x0A, 0x0A, 255), 4.0);
    p.stroke_rounded_rect(input_rect, state::c_separator(), 1.0, 4.0);

    // URL text with cursor
    let display_text = if link_state.url.is_empty() {
        "https://".to_string()
    } else {
        link_state.url.clone()
    };
    let text_color = if link_state.url.is_empty() {
        state::c_text_dim()
    } else {
        state::c_text_light()
    };
    p.draw_text_cached(
        cache,
        &display_text,
        input_rect.x0 + 8.0,
        input_rect.y0 + input_rect.height() / 2.0,
        text_color,
        12.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    // OK button (Korean: 확인)
    let ok_rect = Rect::new(
        modal.x1 - 160.0,
        modal.y1 - 40.0,
        modal.x1 - 90.0,
        modal.y1 - 12.0,
    );
    p.fill_rounded_rect(ok_rect, state::c_accent(), 4.0);
    p.draw_text_cached(
        cache,
        "확인",
        ok_rect.x0 + ok_rect.width() / 2.0,
        ok_rect.y0 + ok_rect.height() / 2.0,
        Color::WHITE,
        11.0,
        tench_ui::parley::FontWeight::BOLD,
        true,
        false,
    );

    // Cancel button (Korean: 취소)
    let cancel_rect = Rect::new(
        modal.x1 - 80.0,
        modal.y1 - 40.0,
        modal.x1 - 12.0,
        modal.y1 - 12.0,
    );
    p.stroke_rounded_rect(cancel_rect, state::c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "취소",
        cancel_rect.x0 + cancel_rect.width() / 2.0,
        cancel_rect.y0 + cancel_rect.height() / 2.0,
        state::c_text_light(),
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        true,
        false,
    );
}

pub(super) fn paint_find_replace_modal(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    size: Size,
    fr_state: &FindReplaceState,
) {
    let modal_w = 400.0;
    let modal_h = if fr_state.show_replace { 200.0 } else { 160.0 };
    let modal_x = size.width / 2.0 - modal_w / 2.0;
    let modal_y = size.height / 2.0 - modal_h / 2.0;

    // Semi-transparent backdrop
    let backdrop = Rect::new(0.0, 0.0, size.width, size.height);
    p.fill_rect(backdrop, Color::rgba8(0, 0, 0, 100));

    let modal = Rect::new(modal_x, modal_y, modal_x + modal_w, modal_y + modal_h);
    p.fill_rounded_rect(modal, state::c_menu_bg(), 6.0);
    p.stroke_rounded_rect(modal, state::c_separator(), 1.0, 6.0);

    // Title (Korean)
    let title = if fr_state.show_replace {
        "찾기 및 바꾸기"
    } else {
        "찾기"
    };
    p.draw_text_cached(
        cache,
        title,
        modal_x + 16.0,
        modal_y + 22.0,
        state::c_text_light(),
        14.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );

    // Match count (Korean)
    let match_info = if fr_state.matches.is_empty() {
        "결과 없음".to_string()
    } else {
        let current = fr_state.current_match_idx.map(|i| i + 1).unwrap_or(0);
        format!("{}/{} 일치", current, fr_state.matches.len())
    };
    p.draw_text_cached(
        cache,
        &match_info,
        modal_x + modal_w - 16.0,
        modal_y + 22.0,
        state::c_text_dim(),
        10.0,
        tench_ui::parley::FontWeight::NORMAL,
        true,
        false,
    );

    // Search input (Korean)
    p.draw_text_cached(
        cache,
        "찾기:",
        modal_x + 16.0,
        modal_y + 48.0,
        state::c_text_dim(),
        11.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );
    let search_input = Rect::new(
        modal_x + 60.0,
        modal_y + 38.0,
        modal_x + modal_w - 16.0,
        modal_y + 60.0,
    );
    p.fill_rounded_rect(search_input, Color::rgba8(0x0A, 0x0A, 0x0A, 255), 4.0);
    p.stroke_rounded_rect(search_input, state::c_separator(), 1.0, 4.0);
    let query_display = if fr_state.query.is_empty() {
        "검색어를 입력하세요..."
    } else {
        &fr_state.query
    };
    let query_color = if fr_state.query.is_empty() {
        state::c_text_dim()
    } else {
        state::c_text_light()
    };
    p.draw_text_cached(
        cache,
        query_display,
        search_input.x0 + 8.0,
        search_input.y0 + search_input.height() / 2.0,
        query_color,
        12.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );

    // Replace input (Korean)
    if fr_state.show_replace {
        p.draw_text_cached(
            cache,
            "바꾸기:",
            modal_x + 16.0,
            modal_y + 78.0,
            state::c_text_dim(),
            11.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
        let replace_input = Rect::new(
            modal_x + 60.0,
            modal_y + 68.0,
            modal_x + modal_w - 16.0,
            modal_y + 90.0,
        );
        p.fill_rounded_rect(replace_input, Color::rgba8(0x0A, 0x0A, 0x0A, 255), 4.0);
        p.stroke_rounded_rect(replace_input, state::c_separator(), 1.0, 4.0);
        let repl_display = if fr_state.replacement.is_empty() {
            "바꿀 내용..."
        } else {
            &fr_state.replacement
        };
        let repl_color = if fr_state.replacement.is_empty() {
            state::c_text_dim()
        } else {
            state::c_text_light()
        };
        p.draw_text_cached(
            cache,
            repl_display,
            replace_input.x0 + 8.0,
            replace_input.y0 + replace_input.height() / 2.0,
            repl_color,
            12.0,
            tench_ui::parley::FontWeight::NORMAL,
            false,
            false,
        );
    }

    // Buttons (Korean)
    let btn_y = if fr_state.show_replace {
        modal_y + 104.0
    } else {
        modal_y + 72.0
    };
    let btn_h = 28.0;
    let btn_gap = 8.0;
    let mut btn_x = modal_x + 16.0;

    // Find Next button (다음 찾기)
    let find_next_rect = Rect::new(btn_x, btn_y, btn_x + 80.0, btn_y + btn_h);
    p.fill_rounded_rect(find_next_rect, state::c_accent(), 4.0);
    p.draw_text_cached(
        cache,
        "다음 찾기",
        find_next_rect.x0 + find_next_rect.width() / 2.0,
        find_next_rect.y0 + find_next_rect.height() / 2.0,
        Color::WHITE,
        10.0,
        tench_ui::parley::FontWeight::BOLD,
        true,
        false,
    );
    btn_x += 80.0 + btn_gap;

    // Find Prev button (이전 찾기)
    let find_prev_rect = Rect::new(btn_x, btn_y, btn_x + 80.0, btn_y + btn_h);
    p.stroke_rounded_rect(find_prev_rect, state::c_separator(), 1.0, 4.0);
    p.draw_text_cached(
        cache,
        "이전 찾기",
        find_prev_rect.x0 + find_prev_rect.width() / 2.0,
        find_prev_rect.y0 + find_prev_rect.height() / 2.0,
        state::c_text_light(),
        10.0,
        tench_ui::parley::FontWeight::NORMAL,
        true,
        false,
    );
    btn_x += 80.0 + btn_gap;

    if fr_state.show_replace {
        // Replace button (바꾸기)
        let replace_rect = Rect::new(btn_x, btn_y, btn_x + 64.0, btn_y + btn_h);
        p.stroke_rounded_rect(replace_rect, state::c_separator(), 1.0, 4.0);
        p.draw_text_cached(
            cache,
            "바꾸기",
            replace_rect.x0 + replace_rect.width() / 2.0,
            replace_rect.y0 + replace_rect.height() / 2.0,
            state::c_text_light(),
            10.0,
            tench_ui::parley::FontWeight::NORMAL,
            true,
            false,
        );
        btn_x += 64.0 + btn_gap;

        // Replace All button (모두 바꾸기)
        let replace_all_rect = Rect::new(btn_x, btn_y, btn_x + 80.0, btn_y + btn_h);
        p.stroke_rounded_rect(replace_all_rect, state::c_separator(), 1.0, 4.0);
        p.draw_text_cached(
            cache,
            "모두 바꾸기",
            replace_all_rect.x0 + replace_all_rect.width() / 2.0,
            replace_all_rect.y0 + replace_all_rect.height() / 2.0,
            state::c_text_light(),
            10.0,
            tench_ui::parley::FontWeight::NORMAL,
            true,
            false,
        );
    }

    // Close button (X) at top right
    let close_rect = Rect::new(
        modal_x + modal_w - 28.0,
        modal_y + 6.0,
        modal_x + modal_w - 10.0,
        modal_y + 24.0,
    );
    p.draw_text_cached(
        cache,
        "X",
        close_rect.x0 + close_rect.width() / 2.0,
        close_rect.y0 + close_rect.height() / 2.0,
        state::c_text_dim(),
        11.0,
        tench_ui::parley::FontWeight::BOLD,
        true,
        false,
    );

    // Keyboard hints (Korean)
    let hint_y = modal_y + modal_h - 20.0;
    p.draw_text_cached(
        cache,
        "Enter: 다음 찾기 | Esc: 닫기",
        modal_x + 16.0,
        hint_y,
        state::c_text_dim(),
        9.0,
        tench_ui::parley::FontWeight::NORMAL,
        false,
        false,
    );
}

pub(super) fn paint_docs_toast(
    p: &mut Painter<'_>,
    cache: &mut TextCache,
    size: Size,
    message: &str,
) {
    let rect = Rect::new(
        size.width - 260.0,
        MENU_BAR_H + TOOLBAR_H + 12.0,
        size.width - 16.0,
        MENU_BAR_H + TOOLBAR_H + 48.0,
    );
    p.fill_rounded_rect(rect, state::c_menu_bg(), 6.0);
    p.stroke_rounded_rect(rect, state::c_separator(), 1.0, 6.0);
    p.draw_text_cached(
        cache,
        message,
        rect.x0 + 12.0,
        rect.y0 + 23.0,
        state::c_text_light(),
        11.0,
        tench_ui::parley::FontWeight::BOLD,
        false,
        false,
    );
}
