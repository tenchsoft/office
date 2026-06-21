// ---------------------------------------------------------------------------
// Dropdown automation nodes
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;
use tench_ui::UiAutomationNode;

use super::automation_helpers::push_docs_node;
use super::popovers;
use super::state::{DocsState, ParagraphStyle, ToolbarDropdown, FONT_FAMILIES, FONT_SIZES};
use super::toolbar_actions;
use super::{MENU_BAR_H, TOOLBAR_H};

pub(super) fn push_dropdown_nodes(
    state: &DocsState,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let Some(dropdown) = &state.open_dropdown else {
        return;
    };

    push_docs_node(
        nodes,
        next_id,
        "dropdown",
        format!("{dropdown:?}"),
        dropdown_debug_id(dropdown),
        Rect::new(
            0.0,
            MENU_BAR_H + TOOLBAR_H,
            220.0,
            MENU_BAR_H + TOOLBAR_H + 200.0,
        ),
    );

    let dropdown_top = MENU_BAR_H + 8.0 + 32.0;
    match dropdown {
        ToolbarDropdown::FontSize => push_font_size_items(state, dropdown_top, nodes, next_id),
        ToolbarDropdown::FontFamily => push_font_family_items(state, dropdown_top, nodes, next_id),
        ToolbarDropdown::ParagraphStyle => {
            push_paragraph_style_items(state, dropdown_top, nodes, next_id);
        }
        ToolbarDropdown::TableGrid => push_table_grid_items(state, nodes, next_id),
        ToolbarDropdown::ColorPicker | ToolbarDropdown::MarkPicker => {}
    }
}

fn dropdown_debug_id(dropdown: &ToolbarDropdown) -> &'static str {
    match dropdown {
        ToolbarDropdown::FontSize => "docs.dropdown.font_size",
        ToolbarDropdown::FontFamily => "docs.dropdown.font_family",
        ToolbarDropdown::ParagraphStyle => "docs.dropdown.paragraph_style",
        ToolbarDropdown::TableGrid => "docs.dropdown.table_grid",
        ToolbarDropdown::ColorPicker => "docs.dropdown.text_color",
        ToolbarDropdown::MarkPicker => "docs.dropdown.mark_color",
    }
}

fn push_font_size_items(
    state: &DocsState,
    top: f64,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let x = toolbar_actions::font_size_select_x();
    for (idx, &size) in FONT_SIZES.iter().enumerate() {
        let label = format!("{size:.0}");
        push_dropdown_item(
            state,
            DropdownItem {
                dropdown: ToolbarDropdown::FontSize,
                idx,
                label,
                debug_id: format!("docs.dropdown.font_size.{idx}"),
                rect: Rect::new(
                    x,
                    top + 2.0 + idx as f64 * popovers::DROPDOWN_ITEM_H,
                    x + toolbar_actions::FONT_SIZE_SELECT_W,
                    top + 2.0 + (idx + 1) as f64 * popovers::DROPDOWN_ITEM_H,
                ),
            },
            nodes,
            next_id,
        );
    }
}

fn push_font_family_items(
    state: &DocsState,
    top: f64,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let x = toolbar_actions::font_family_select_x();
    for (idx, &family) in FONT_FAMILIES.iter().enumerate() {
        push_dropdown_item(
            state,
            DropdownItem {
                dropdown: ToolbarDropdown::FontFamily,
                idx,
                label: family.to_string(),
                debug_id: format!("docs.dropdown.font_family.{idx}"),
                rect: Rect::new(
                    x,
                    top + 2.0 + idx as f64 * popovers::DROPDOWN_ITEM_H,
                    x + toolbar_actions::FONT_FAMILY_SELECT_W,
                    top + 2.0 + (idx + 1) as f64 * popovers::DROPDOWN_ITEM_H,
                ),
            },
            nodes,
            next_id,
        );
    }
}

fn push_paragraph_style_items(
    state: &DocsState,
    top: f64,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let x = toolbar_actions::paragraph_style_select_x();
    for (idx, style) in ParagraphStyle::all().iter().enumerate() {
        push_dropdown_item(
            state,
            DropdownItem {
                dropdown: ToolbarDropdown::ParagraphStyle,
                idx,
                label: style.label().to_string(),
                debug_id: format!("docs.dropdown.paragraph_style.{idx}"),
                rect: Rect::new(
                    x,
                    top + 2.0 + idx as f64 * popovers::DROPDOWN_ITEM_H,
                    x + toolbar_actions::PARAGRAPH_SELECT_W,
                    top + 2.0 + (idx + 1) as f64 * popovers::DROPDOWN_ITEM_H,
                ),
            },
            nodes,
            next_id,
        );
    }
}

struct DropdownItem {
    dropdown: ToolbarDropdown,
    idx: usize,
    label: String,
    debug_id: String,
    rect: Rect,
}

fn push_dropdown_item(
    state: &DocsState,
    item: DropdownItem,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let is_hovered = state.hovered_dropdown_item == Some((item.dropdown, item.idx));
    push_docs_node(
        nodes,
        next_id,
        "MenuItem",
        item.label,
        item.debug_id,
        item.rect,
    );
    if let Some(node) = nodes.last_mut() {
        node.value = Some(if is_hovered { "hovered" } else { "idle" }.to_string());
        node.hovered = is_hovered;
    }
}

fn push_table_grid_items(state: &DocsState, nodes: &mut Vec<UiAutomationNode>, next_id: &mut u64) {
    push_docs_node(
        nodes,
        next_id,
        "Label",
        "Table grid hover",
        "docs.table_grid.hover",
        Rect::new(0.0, 0.0, 0.0, 0.0),
    );
    if let Some(node) = nodes.last_mut() {
        node.value = Some(format!(
            "{}x{}",
            state.table_grid.hover_row, state.table_grid.hover_col
        ));
    }

    let cell_size = 22.0;
    for row in 0..5 {
        for col in 0..5 {
            let selected = row < state.table_grid.hover_row && col < state.table_grid.hover_col;
            push_docs_node(
                nodes,
                next_id,
                "TableCell",
                format!("{}x{}", row + 1, col + 1),
                format!("docs.table_grid.cell.{row}.{col}"),
                Rect::new(
                    col as f64 * cell_size,
                    row as f64 * cell_size,
                    (col + 1) as f64 * cell_size,
                    (row + 1) as f64 * cell_size,
                ),
            );
            if let Some(node) = nodes.last_mut() {
                node.value = Some(if selected { "hovered" } else { "idle" }.to_string());
                node.hovered = selected;
            }
        }
    }
}
