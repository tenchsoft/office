// ---------------------------------------------------------------------------
// Automation helpers
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;
use tench_ui::{UiAutomationNode, UiAutomationRect};

use super::toolbar_actions::ToolbarAction;

pub(super) fn toolbar_action_debug_id(action: &ToolbarAction) -> Option<&'static str> {
    match action {
        ToolbarAction::Undo => Some("docs.toolbar.undo"),
        ToolbarAction::Redo => Some("docs.toolbar.redo"),
        ToolbarAction::FormatButton(0) => Some("docs.toolbar.bold"),
        ToolbarAction::FormatButton(1) => Some("docs.toolbar.italic"),
        ToolbarAction::FormatButton(2) => Some("docs.toolbar.underline"),
        ToolbarAction::FormatButton(3) => Some("docs.toolbar.strikethrough"),
        ToolbarAction::FormatButton(4) => Some("docs.toolbar.code"),
        ToolbarAction::FormatButton(5) => Some("docs.toolbar.superscript"),
        ToolbarAction::FormatButton(6) => Some("docs.toolbar.subscript"),
        ToolbarAction::FormatButton(7) => Some("docs.toolbar.highlight"),
        ToolbarAction::BulletList => Some("docs.toolbar.bullet_list"),
        ToolbarAction::NumberedList => Some("docs.toolbar.numbered_list"),
        ToolbarAction::Checklist => Some("docs.toolbar.checklist"),
        ToolbarAction::Outdent => Some("docs.toolbar.outdent"),
        ToolbarAction::Indent => Some("docs.toolbar.indent"),
        ToolbarAction::AlignLeft => Some("docs.toolbar.align_left"),
        ToolbarAction::AlignCenter => Some("docs.toolbar.align_center"),
        ToolbarAction::AlignRight => Some("docs.toolbar.align_right"),
        ToolbarAction::AlignJustify => Some("docs.toolbar.align_justify"),
        ToolbarAction::HorizontalRule => Some("docs.toolbar.horizontal_rule"),
        ToolbarAction::BlockQuote => Some("docs.toolbar.block_quote"),
        ToolbarAction::InsertLink => Some("docs.toolbar.insert_link"),
        ToolbarAction::InsertImage => Some("docs.toolbar.insert_image"),
        ToolbarAction::InsertTable => Some("docs.toolbar.insert_table"),
        ToolbarAction::TextColor => Some("docs.toolbar.text_color"),
        ToolbarAction::MarkColor => Some("docs.toolbar.highlight_color"),
        _ => None,
    }
}

pub(super) fn push_docs_node(
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

pub(super) fn simple_docs_id(label: &str) -> String {
    label.to_lowercase().replace(' ', "_")
}
