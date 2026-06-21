// ---------------------------------------------------------------------------
// Popup automation nodes
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;
use tench_ui::UiAutomationNode;

use super::automation_helpers::{info_modal_rect_ko, push_kodocs_node};
use super::state::{self, KodocsState};

pub(super) fn push_popup_nodes(
    state: &KodocsState,
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    push_context_menu_nodes(state, nodes, next_id);
    push_comment_modal_nodes(state, size, nodes, next_id);
    push_hanja_popup_nodes(state, size, nodes, next_id);
    push_equation_editor_nodes(state, size, nodes, next_id);
}

fn push_context_menu_nodes(
    state: &KodocsState,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let Some(cm) = &state.context_menu else {
        return;
    };
    let items = state::context_menu_items(&cm.menu_type);
    for (idx, item) in items.iter().enumerate() {
        let item_key = item.to_lowercase().replace(' ', "_");
        push_kodocs_node(
            nodes,
            next_id,
            "menu_item",
            *item,
            format!("kodocs.context.{item_key}"),
            Rect::new(
                cm.x,
                cm.y + idx as f64 * 28.0,
                cm.x + 200.0,
                cm.y + (idx + 1) as f64 * 28.0,
            ),
        );
    }
}

fn push_comment_modal_nodes(
    state: &KodocsState,
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    if state.comment_modal.is_none() {
        return;
    }
    let m = info_modal_rect_ko(size);
    push_kodocs_node(nodes, next_id, "dialog", "메모", "kodocs.modal.comment", m);
    push_kodocs_node(
        nodes,
        next_id,
        "text_input",
        "메모",
        "kodocs.modal.comment.input",
        Rect::new(m.x0 + 20.0, m.y0 + 40.0, m.x1 - 20.0, m.y0 + 100.0),
    );
    for (label, id, x0, x1) in [
        ("확인", "kodocs.modal.comment.ok", m.x1 - 140.0, m.x1 - 80.0),
        (
            "취소",
            "kodocs.modal.comment.cancel",
            m.x1 - 70.0,
            m.x1 - 10.0,
        ),
    ] {
        push_kodocs_node(
            nodes,
            next_id,
            "button",
            label,
            id,
            Rect::new(x0, m.y1 - 44.0, x1, m.y1 - 16.0),
        );
    }
}

fn push_hanja_popup_nodes(
    state: &KodocsState,
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let Some(hanja) = &state.hanja_popup else {
        return;
    };
    let hp = info_modal_rect_ko(size);
    push_kodocs_node(
        nodes,
        next_id,
        "dialog",
        "한자 변환",
        "kodocs.hanja_popup",
        hp,
    );
    let row_h = 28.0;
    for (idx, cand) in hanja.candidates.iter().take(8).enumerate() {
        push_kodocs_node(
            nodes,
            next_id,
            "list_item",
            cand,
            format!("kodocs.hanja_popup.candidate.{idx}"),
            Rect::new(
                hp.x0 + 10.0,
                hp.y0 + 10.0 + idx as f64 * row_h,
                hp.x1 - 10.0,
                hp.y0 + 10.0 + (idx + 1) as f64 * row_h,
            ),
        );
    }
}

fn push_equation_editor_nodes(
    state: &KodocsState,
    size: Size,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    if state.equation_editor.is_none() {
        return;
    }
    let eq = info_modal_rect_ko(size);
    push_kodocs_node(
        nodes,
        next_id,
        "dialog",
        "수식 편집기",
        "kodocs.equation_editor",
        eq,
    );
    push_kodocs_node(
        nodes,
        next_id,
        "text_input",
        "수식 입력",
        "kodocs.equation_editor.input",
        Rect::new(eq.x0 + 20.0, eq.y0 + 20.0, eq.x1 - 20.0, eq.y0 + 52.0),
    );

    let symbols = [
        ("+", "kodocs.equation_editor.plus"),
        ("-", "kodocs.equation_editor.minus"),
        ("×", "kodocs.equation_editor.multiply"),
        ("÷", "kodocs.equation_editor.divide"),
        ("=", "kodocs.equation_editor.equals"),
        ("≠", "kodocs.equation_editor.not_equal"),
        ("<", "kodocs.equation_editor.less"),
        (">", "kodocs.equation_editor.greater"),
        ("≤", "kodocs.equation_editor.less_or_equal"),
        ("≥", "kodocs.equation_editor.greater_or_equal"),
        ("(", "kodocs.equation_editor.left_parenthesis"),
        (")", "kodocs.equation_editor.right_parenthesis"),
        ("[", "kodocs.equation_editor.left_bracket"),
        ("]", "kodocs.equation_editor.right_bracket"),
        ("{", "kodocs.equation_editor.left_brace"),
        ("}", "kodocs.equation_editor.right_brace"),
        ("π", "kodocs.equation_editor.pi"),
        ("√", "kodocs.equation_editor.square_root"),
        ("Σ", "kodocs.equation_editor.sum"),
        ("∫", "kodocs.equation_editor.integral"),
        ("x²", "kodocs.equation_editor.superscript_two"),
        ("x³", "kodocs.equation_editor.superscript_three"),
    ];
    let (btn_w, btn_h, cols) = (32.0, 28.0, 10);
    for (i, (label, id)) in symbols.iter().enumerate() {
        let bx = eq.x0 + 20.0 + (i % cols) as f64 * (btn_w + 2.0);
        let by = eq.y0 + 60.0 + (i / cols) as f64 * (btn_h + 2.0);
        push_kodocs_node(
            nodes,
            next_id,
            "button",
            *label,
            *id,
            Rect::new(bx, by, bx + btn_w, by + btn_h),
        );
    }
    for (label, id, rect) in [
        (
            "삽입",
            "kodocs.equation_editor.insert",
            Rect::new(eq.x1 - 140.0, eq.y1 - 44.0, eq.x1 - 80.0, eq.y1 - 16.0),
        ),
        (
            "취소",
            "kodocs.equation_editor.cancel",
            Rect::new(eq.x1 - 70.0, eq.y1 - 44.0, eq.x1 - 10.0, eq.y1 - 16.0),
        ),
        (
            "닫기",
            "kodocs.equation_editor.close",
            Rect::new(eq.x0 + 10.0, eq.y0 + 4.0, eq.x0 + 30.0, eq.y0 + 24.0),
        ),
    ] {
        push_kodocs_node(nodes, next_id, "button", label, id, rect);
    }
}
