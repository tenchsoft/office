// ---------------------------------------------------------------------------
// Version history automation nodes
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;
use tench_ui::UiAutomationNode;

use super::automation_helpers::push_docs_node;
use super::state::DocsState;

pub(super) fn push_version_history_nodes(
    state: &DocsState,
    area: Rect,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let header_y = area.y0 + 12.0;
    push_docs_node(
        nodes,
        next_id,
        "button",
        if state.version_history_collapsed {
            "Version History (collapsed)"
        } else {
            "Version History"
        },
        "docs.version_history.header",
        Rect::new(area.x0 + 12.0, header_y, area.x1 - 12.0, header_y + 30.0),
    );
    if state.version_history_collapsed {
        return;
    }
    if state.version_history.is_empty() {
        push_docs_node(
            nodes,
            next_id,
            "text",
            "No version history",
            "docs.version_history.empty",
            Rect::new(
                area.x0 + 18.0,
                header_y + 30.0,
                area.x1 - 18.0,
                header_y + 54.0,
            ),
        );
        return;
    }
    let mut row_y = header_y + 30.0;
    for (i, entry) in state.version_history.iter().enumerate() {
        push_docs_node(
            nodes,
            next_id,
            "list_item",
            entry.label.clone(),
            format!("docs.version_history.row.{i}"),
            Rect::new(area.x0 + 10.0, row_y, area.x1 - 10.0, row_y + 40.0),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(format!(
                "{}|{}|{}",
                entry.timestamp_label, entry.path, entry.size_bytes
            ));
        }
        row_y += 44.0;
    }
}
