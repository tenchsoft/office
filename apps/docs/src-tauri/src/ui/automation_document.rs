// ---------------------------------------------------------------------------
// Document automation nodes
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;
use tench_ui::UiAutomationNode;

use super::automation_helpers::push_docs_node;
use super::document_text::extract_block_text;
use super::state::{DocsState, MENU_BAR_H, RULER_H, STATUS_BAR_H, TITLE_ROW_H, TOOLBAR_H};

pub(super) fn push_document_state_nodes(
    state: &DocsState,
    size: Size,
    doc_y: f64,
    content_w: f64,
    tab_bar_h: f64,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let zero = Rect::new(0.0, doc_y, 0.0, doc_y);
    let cursor = state.cursor();
    push_value_node(
        nodes,
        next_id,
        "cursor",
        "Cursor",
        "docs.document.cursor",
        format!("{}:{}", cursor.block_idx, cursor.offset),
        zero,
    );

    let selection = state
        .selection
        .as_ref()
        .map(|sel| {
            format!(
                "{}:{}-{}:{}",
                sel.start.block_idx, sel.start.offset, sel.end.block_idx, sel.end.offset
            )
        })
        .unwrap_or_else(|| "none".into());
    push_value_node(
        nodes,
        next_id,
        "selection",
        "Selection",
        "docs.document.selection",
        selection,
        zero,
    );

    let mut doc_text = state.current_document().to_plain_text();
    doc_text.truncate(2048);
    push_value_node(
        nodes,
        next_id,
        "text",
        "Document text",
        "docs.document.text",
        doc_text,
        zero,
    );

    for (debug_id, label, value) in [
        ("docs.document.dirty", "Dirty", bool_value(state.is_dirty())),
        (
            "docs.document.word_count",
            "Word count",
            state.word_count.to_string(),
        ),
        (
            "docs.document.undo_available",
            "Undo available",
            bool_value(state.undo_available),
        ),
        (
            "docs.document.redo_available",
            "Redo available",
            bool_value(state.redo_available),
        ),
        (
            "docs.document.cursor_visible",
            "Cursor visible",
            bool_value(state.cursor_visible),
        ),
        (
            "docs.document.paragraph_count",
            "Paragraph count",
            state.paragraph_count().to_string(),
        ),
        ("docs.document.zoom", "Zoom", format!("{:.0}", state.zoom)),
        (
            "docs.document.scroll_y",
            "Scroll Y",
            format!("{}", state.scroll_y as i64),
        ),
        (
            "docs.document.max_scroll_y",
            "Max scroll Y",
            max_scroll_value(state, size.height, tab_bar_h),
        ),
    ] {
        push_value_node(nodes, next_id, "status", label, debug_id, value, zero);
    }

    push_value_node(
        nodes,
        next_id,
        "text",
        "Selected text",
        "docs.document.selected_text",
        selected_text(state),
        zero,
    );
    push_value_node(
        nodes,
        next_id,
        "text",
        "Title",
        "docs.document.title",
        state.title().to_string(),
        zero,
    );

    let doc_plain = state.current_document().to_plain_text();
    let line_count = doc_plain.lines().count().max(1);
    for (debug_id, label, value) in [
        (
            "docs.document.line_count",
            "Line count",
            line_count.to_string(),
        ),
        (
            "docs.document.current_page",
            "Current page",
            state.current_page.to_string(),
        ),
        (
            "docs.document.page_count",
            "Page count",
            state.layout_cache.num_pages().max(1).to_string(),
        ),
        (
            "docs.clipboard.text",
            "Clipboard text",
            if state.clipboard_text.is_empty() {
                "none".into()
            } else {
                state.clipboard_text.clone()
            },
        ),
        (
            "docs.clipboard.node_count",
            "Clipboard node count",
            state.clipboard_node_count.to_string(),
        ),
    ] {
        push_value_node(nodes, next_id, "status", label, debug_id, value, zero);
    }

    let scale = state.zoom / 100.0;
    let setup = &state.current_document().page_setup;
    let (page_w_raw, page_h_raw) = setup.page_size_px();
    let page_w = page_w_raw * scale;
    let page_h = page_h_raw * scale;
    push_docs_node(
        nodes,
        next_id,
        "page",
        "Document page",
        "docs.document.page",
        Rect::new(
            (content_w - page_w) / 2.0,
            doc_y + 20.0,
            (content_w + page_w) / 2.0,
            doc_y + 20.0 + page_h,
        ),
    );
}

fn push_value_node(
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
    role: &str,
    label: &str,
    debug_id: &str,
    value: String,
    rect: Rect,
) {
    push_docs_node(nodes, next_id, role, label, debug_id, rect);
    if let Some(node) = nodes.last_mut() {
        node.value = Some(value);
    }
}

fn bool_value(value: bool) -> String {
    if value { "true" } else { "false" }.into()
}

fn max_scroll_value(state: &DocsState, height: f64, tab_bar_h: f64) -> String {
    let viewport_h =
        (height - MENU_BAR_H - TOOLBAR_H - TITLE_ROW_H - RULER_H - STATUS_BAR_H - tab_bar_h)
            .max(0.0);
    let max_scroll = (state.layout_cache.total_content_h() - viewport_h).max(0.0);
    format!("{}", max_scroll as i64)
}

fn selected_text(state: &DocsState) -> String {
    let Some(sel) = &state.selection else {
        return String::new();
    };
    let doc = state.current_document();
    if doc.content.is_empty() {
        return String::new();
    }

    let start_block = sel.start.block_idx.min(doc.content.len().saturating_sub(1));
    let end_block = sel.end.block_idx.min(doc.content.len().saturating_sub(1));
    if start_block == end_block {
        let block_text = extract_block_text(&doc.content[start_block]);
        let s = sel.start.offset.min(block_text.len());
        let e = sel.end.offset.min(block_text.len());
        let (lo, hi) = (s.min(e), s.max(e));
        return block_text[lo..hi].to_string();
    }

    let mut text = String::new();
    for bi in start_block..=end_block {
        if bi >= doc.content.len() {
            break;
        }
        if bi > start_block {
            text.push('\n');
        }
        let block_text = extract_block_text(&doc.content[bi]);
        if bi == start_block {
            let s = sel.start.offset.min(block_text.len());
            text.push_str(&block_text[s..]);
        } else if bi == end_block {
            let e = sel.end.offset.min(block_text.len());
            text.push_str(&block_text[..e]);
        } else {
            text.push_str(&block_text);
        }
    }
    text
}
