// ---------------------------------------------------------------------------
// Status automation nodes
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;
use tench_ui::UiAutomationNode;

use super::automation_helpers::push_docs_node;
use super::state::DocsState;

pub(super) fn push_status_nodes(
    state: &DocsState,
    width: f64,
    height: f64,
    status_y: f64,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    for (debug_id, value) in [
        ("docs.tabs.count", state.open_tabs.len().to_string()),
        ("docs.tabs.active_index", state.active_tab_idx.to_string()),
    ] {
        push_docs_node(
            nodes,
            next_id,
            "status",
            value.clone(),
            debug_id,
            Rect::new(0.0, 0.0, 0.0, 0.0),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(value);
        }
    }

    push_docs_node(
        nodes,
        next_id,
        "status_bar",
        state.status_line(),
        "docs.status_bar",
        Rect::new(0.0, status_y, width, height),
    );

    let page_count = state.layout_cache.num_pages().max(1);
    let page_text = format!("Page {}/{}", state.current_page, page_count);
    push_value_node(
        nodes,
        next_id,
        "text",
        page_text.clone(),
        "docs.status_bar.page",
        page_text,
        Rect::new(8.0, status_y, 120.0, height),
    );

    let words_text = format!("{} words", state.word_count);
    push_value_node(
        nodes,
        next_id,
        "text",
        words_text.clone(),
        "docs.status_bar.words",
        words_text,
        Rect::new(130.0, status_y, 250.0, height),
    );

    let chars_text = format!("{} chars", state.character_count());
    push_value_node(
        nodes,
        next_id,
        "text",
        chars_text.clone(),
        "docs.status_bar.chars",
        chars_text,
        Rect::new(260.0, status_y, 380.0, height),
    );

    let cursor_text = cursor_status_text(state);
    push_value_node(
        nodes,
        next_id,
        "text",
        cursor_text.clone(),
        "docs.status_bar.cursor",
        cursor_text,
        Rect::new(390.0, status_y, 520.0, height),
    );

    push_value_node(
        nodes,
        next_id,
        "text",
        "English",
        "docs.status_bar.language",
        "English",
        Rect::new(width - 240.0, status_y, width - 160.0, height),
    );

    let zoom_text = format!("{:.0}%", state.zoom);
    push_value_node(
        nodes,
        next_id,
        "text",
        zoom_text.clone(),
        "docs.status_bar.zoom",
        zoom_text,
        Rect::new(width - 120.0, status_y, width - 50.0, height),
    );

    let dirty_text = if state.is_dirty() {
        "Modified"
    } else {
        "Saved"
    };
    push_value_node(
        nodes,
        next_id,
        "text",
        dirty_text,
        "docs.status_bar.dirty",
        dirty_text,
        Rect::new(width - 50.0, status_y, width, height),
    );
}

fn push_value_node(
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
    role: &str,
    label: impl Into<String>,
    debug_id: &str,
    value: impl Into<String>,
    rect: Rect,
) {
    push_docs_node(nodes, next_id, role, label, debug_id, rect);
    if let Some(node) = nodes.last_mut() {
        node.value = Some(value.into());
    }
}

fn cursor_status_text(state: &DocsState) -> String {
    let cursor = state.cursor();
    let doc_text = state.document_text();
    let text_before_cursor = if cursor.offset > 0 && cursor.offset <= doc_text.len() {
        &doc_text[..cursor.offset]
    } else {
        ""
    };
    let line_num = text_before_cursor.lines().count().max(1);
    let col_num = text_before_cursor
        .lines()
        .last()
        .map(|l| l.chars().count() + 1)
        .unwrap_or(1);
    format!("Ln {line_num}, Col {col_num}")
}
