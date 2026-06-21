// ---------------------------------------------------------------------------
// Modal automation nodes
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;
use tench_ui::UiAutomationNode;

use super::automation_helpers::{info_modal_rect_ko, push_kodocs_node};
use super::state::KodocsState;

pub(super) fn push_modal_nodes(
    state: &KodocsState,
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    if state.link_modal.is_some() {
        push_link_modal(size, nodes, next_id);
    } else if state.find_replace.is_some() {
        push_find_replace_modal(size, nodes, next_id);
    } else if state.page_setup_dialog.is_some() {
        push_page_setup_modal(size, nodes, next_id);
    } else if state.print_preview.is_some() {
        push_kodocs_node(
            nodes,
            next_id,
            "dialog",
            "인쇄 미리보기",
            "kodocs.modal.print_preview",
            info_modal_rect_ko(size),
        );
    } else if state.word_count_modal {
        push_simple_modal(
            size,
            "단어 수",
            "kodocs.modal.word_count",
            "kodocs.modal.word_count.close",
            nodes,
            next_id,
        );
    } else if state.goto_modal.is_some() {
        push_goto_modal(size, nodes, next_id);
    } else if state.special_char_modal.is_some() {
        push_simple_modal(
            size,
            "특수 문자",
            "kodocs.modal.special_char",
            "kodocs.modal.special_char.close",
            nodes,
            next_id,
        );
    }
}

fn push_link_modal(size: Size, nodes: &mut Vec<UiAutomationNode>, next_id: &mut u64) {
    let m = info_modal_rect_ko(size);
    push_kodocs_node(
        nodes,
        next_id,
        "dialog",
        "하이퍼링크 삽입",
        "kodocs.modal.link",
        m,
    );
    push_kodocs_node(
        nodes,
        next_id,
        "text_input",
        "URL",
        "kodocs.modal.link.url",
        Rect::new(m.x0 + 20.0, m.y0 + 40.0, m.x1 - 20.0, m.y0 + 68.0),
    );
    push_ok_cancel(
        nodes,
        next_id,
        m,
        "kodocs.modal.link.ok",
        "kodocs.modal.link.cancel",
    );
}

fn push_find_replace_modal(size: Size, nodes: &mut Vec<UiAutomationNode>, next_id: &mut u64) {
    let m = info_modal_rect_ko(size);
    push_kodocs_node(
        nodes,
        next_id,
        "dialog",
        "찾기 및 바꾸기",
        "kodocs.modal.find_replace",
        m,
    );
    for (label, id, y0, y1) in [
        ("찾기", "kodocs.modal.find_replace.query", 40.0, 68.0),
        ("바꾸기", "kodocs.modal.find_replace.replace", 76.0, 104.0),
    ] {
        push_kodocs_node(
            nodes,
            next_id,
            "text_input",
            label,
            id,
            Rect::new(m.x0 + 20.0, m.y0 + y0, m.x1 - 20.0, m.y0 + y1),
        );
    }
    for (label, id, rect) in [
        (
            "다음 찾기",
            "kodocs.modal.find_replace.next",
            Rect::new(m.x0 + 20.0, m.y1 - 44.0, m.x0 + 100.0, m.y1 - 16.0),
        ),
        (
            "이전 찾기",
            "kodocs.modal.find_replace.prev",
            Rect::new(m.x0 + 108.0, m.y1 - 44.0, m.x0 + 188.0, m.y1 - 16.0),
        ),
        (
            "바꾸기",
            "kodocs.modal.find_replace.replace_btn",
            Rect::new(m.x0 + 20.0, m.y1 - 78.0, m.x0 + 100.0, m.y1 - 50.0),
        ),
        (
            "모두 바꾸기",
            "kodocs.modal.find_replace.replace_all",
            Rect::new(m.x0 + 108.0, m.y1 - 78.0, m.x0 + 198.0, m.y1 - 50.0),
        ),
        (
            "닫기",
            "kodocs.modal.find_replace.close",
            Rect::new(m.x1 - 28.0, m.y0 + 4.0, m.x1 - 8.0, m.y0 + 24.0),
        ),
    ] {
        push_kodocs_node(nodes, next_id, "button", label, id, rect);
    }
    push_kodocs_node(
        nodes,
        next_id,
        "toggle",
        "대소문자 구분",
        "kodocs.modal.find_replace.case_sensitive",
        Rect::new(m.x0 + 196.0, m.y1 - 44.0, m.x0 + 220.0, m.y1 - 16.0),
    );
    push_kodocs_node(
        nodes,
        next_id,
        "toggle",
        "정규식",
        "kodocs.modal.find_replace.regex",
        Rect::new(m.x0 + 228.0, m.y1 - 44.0, m.x0 + 252.0, m.y1 - 16.0),
    );
}

fn push_page_setup_modal(size: Size, nodes: &mut Vec<UiAutomationNode>, next_id: &mut u64) {
    let m = info_modal_rect_ko(size);
    push_kodocs_node(
        nodes,
        next_id,
        "dialog",
        "페이지 설정",
        "kodocs.modal.page_setup",
        m,
    );
    push_kodocs_node(
        nodes,
        next_id,
        "dropdown",
        "용지 크기",
        "kodocs.modal.page_setup.paper_size",
        Rect::new(m.x0 + 140.0, m.y0 + 20.0, m.x0 + 260.0, m.y0 + 48.0),
    );
    for (idx, (label, id)) in [
        ("A4", "kodocs.modal.page_setup.paper_size.a4"),
        ("Letter", "kodocs.modal.page_setup.paper_size.letter"),
        ("A3", "kodocs.modal.page_setup.paper_size.a3"),
        ("B5", "kodocs.modal.page_setup.paper_size.b5"),
        ("Legal", "kodocs.modal.page_setup.paper_size.legal"),
    ]
    .iter()
    .enumerate()
    {
        push_kodocs_node(
            nodes,
            next_id,
            "menu_item",
            *label,
            *id,
            Rect::new(
                m.x0 + 140.0,
                m.y0 + 52.0 + idx as f64 * 26.0,
                m.x0 + 260.0,
                m.y0 + 78.0 + idx as f64 * 26.0,
            ),
        );
    }
    for (label, id, x0, x1) in [
        ("세로", "kodocs.modal.page_setup.portrait", 140.0, 200.0),
        ("가로", "kodocs.modal.page_setup.landscape", 208.0, 268.0),
    ] {
        push_kodocs_node(
            nodes,
            next_id,
            "button",
            label,
            id,
            Rect::new(m.x0 + x0, m.y0 + 60.0, m.x0 + x1, m.y0 + 88.0),
        );
    }
    let right_x = m.x0 + 20.0;
    for (idx, (label, id)) in [
        ("위쪽", "kodocs.modal.page_setup.margin.top"),
        ("아래쪽", "kodocs.modal.page_setup.margin.bottom"),
        ("왼쪽", "kodocs.modal.page_setup.margin.left"),
        ("오른쪽", "kodocs.modal.page_setup.margin.right"),
    ]
    .iter()
    .enumerate()
    {
        let y = m.y0 + 110.0 + idx as f64 * 36.0;
        push_kodocs_node(
            nodes,
            next_id,
            "text_input",
            *label,
            *id,
            Rect::new(right_x + 80.0, y, right_x + 150.0, y + 28.0),
        );
    }
    push_ok_cancel(
        nodes,
        next_id,
        m,
        "kodocs.modal.page_setup.ok",
        "kodocs.modal.page_setup.cancel",
    );
}

fn push_goto_modal(size: Size, nodes: &mut Vec<UiAutomationNode>, next_id: &mut u64) {
    let m = info_modal_rect_ko(size);
    push_kodocs_node(nodes, next_id, "dialog", "이동", "kodocs.modal.goto", m);
    push_kodocs_node(
        nodes,
        next_id,
        "text_input",
        "이동",
        "kodocs.modal.goto.input",
        Rect::new(m.x0 + 20.0, m.y0 + 40.0, m.x1 - 20.0, m.y0 + 68.0),
    );
    push_ok_cancel(
        nodes,
        next_id,
        m,
        "kodocs.modal.goto.ok",
        "kodocs.modal.goto.cancel",
    );
}

fn push_simple_modal(
    size: Size,
    label: &str,
    debug_id: &str,
    close_id: &str,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let m = info_modal_rect_ko(size);
    push_kodocs_node(nodes, next_id, "dialog", label, debug_id, m);
    push_kodocs_node(
        nodes,
        next_id,
        "button",
        "닫기",
        close_id,
        Rect::new(m.x1 - 70.0, m.y1 - 44.0, m.x1 - 10.0, m.y1 - 16.0),
    );
}

fn push_ok_cancel(
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
    m: Rect,
    ok_id: &str,
    cancel_id: &str,
) {
    push_kodocs_node(
        nodes,
        next_id,
        "button",
        "확인",
        ok_id,
        Rect::new(m.x1 - 140.0, m.y1 - 44.0, m.x1 - 80.0, m.y1 - 16.0),
    );
    push_kodocs_node(
        nodes,
        next_id,
        "button",
        "취소",
        cancel_id,
        Rect::new(m.x1 - 70.0, m.y1 - 44.0, m.x1 - 10.0, m.y1 - 16.0),
    );
}
