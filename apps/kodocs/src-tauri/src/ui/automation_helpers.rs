// ---------------------------------------------------------------------------
// Automation helpers
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;
use tench_ui::{UiAutomationNode, UiAutomationRect};

pub(super) fn push_kodocs_node(
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
    role: &str,
    label: impl Into<String>,
    debug_id: impl Into<String>,
    rect: Rect,
) {
    *next_id = next_id.saturating_add(1);
    nodes.push(UiAutomationNode {
        id: *next_id,
        debug_id: Some(debug_id.into()),
        role: role.to_string(),
        label: Some(label.into()),
        value: None,
        bounds: UiAutomationRect {
            x: rect.x0,
            y: rect.y0,
            width: rect.width(),
            height: rect.height(),
        },
        enabled: true,
        focused: false,
        hovered: false,
        children: Vec::new(),
    });
}

pub(super) fn info_modal_rect_ko(size: Size) -> Rect {
    let mw = 400.0;
    let mh = 300.0;
    Rect::new(
        (size.width - mw) / 2.0,
        (size.height - mh) / 2.0,
        (size.width + mw) / 2.0,
        (size.height + mh) / 2.0,
    )
}
