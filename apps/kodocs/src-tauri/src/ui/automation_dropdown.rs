// ---------------------------------------------------------------------------
// Dropdown automation nodes
// ---------------------------------------------------------------------------

use tench_ui::prelude::*;
use tench_ui::UiAutomationNode;

use super::automation_helpers::push_kodocs_node;
use super::state::{self, KodocsState, ToolbarDropdown, MENU_BAR_H, TOOLBAR_H};

pub(super) fn push_dropdown_nodes(
    state: &KodocsState,
    width: f64,
    anchors: (f64, f64, f64),
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let Some(dd) = &state.open_dropdown else {
        return;
    };
    push_color_palette_nodes(dd, width, nodes, next_id);
    if matches!(dd, ToolbarDropdown::TableGrid) {
        push_table_grid_nodes(nodes, next_id);
    }
    match dd {
        ToolbarDropdown::FontFamily => {
            push_menu_items(
                anchors.0,
                120.0,
                state::FONT_FAMILIES,
                "font_family",
                nodes,
                next_id,
            );
        }
        ToolbarDropdown::FontSize => {
            let items: Vec<String> = state::FONT_SIZES
                .iter()
                .map(|size| format!("{}px", *size as i32))
                .collect();
            push_owned_menu_items(anchors.1, 62.0, &items, "font_size", nodes, next_id);
        }
        ToolbarDropdown::ParagraphStyle => {
            let items: Vec<(&str, &str)> = state::ParagraphStyle::all()
                .iter()
                .map(|style| match style {
                    state::ParagraphStyle::Paragraph => ("단락", "paragraph"),
                    state::ParagraphStyle::Heading1 => ("제목 1", "heading_1"),
                    state::ParagraphStyle::Heading2 => ("제목 2", "heading_2"),
                    state::ParagraphStyle::Heading3 => ("제목 3", "heading_3"),
                    state::ParagraphStyle::Heading4 => ("제목 4", "heading_4"),
                    state::ParagraphStyle::Heading5 => ("제목 5", "heading_5"),
                    state::ParagraphStyle::Heading6 => ("제목 6", "heading_6"),
                    state::ParagraphStyle::BlockQuote => ("인용", "block_quote"),
                    state::ParagraphStyle::CodeBlock => ("코드 블록", "code_block"),
                })
                .collect();
            push_style_items(anchors.2, 112.0, &items, nodes, next_id);
        }
        _ => {}
    }
}

fn push_color_palette_nodes(
    dd: &ToolbarDropdown,
    width: f64,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let cell_size = 20.0;
    let cols = 9;
    let palette_w = cols as f64 * cell_size;
    let prefix = match dd {
        ToolbarDropdown::ColorPicker => "kodocs.toolbar.text_color_palette",
        ToolbarDropdown::MarkPicker => "kodocs.toolbar.background_color_palette",
        _ => return,
    };
    let anchor_x = width - palette_w - 20.0;
    let palette_top = MENU_BAR_H + TOOLBAR_H;
    for (idx, color) in state::COLOR_PALETTE.iter().enumerate() {
        let row = idx / cols;
        let col = idx % cols;
        let cx = anchor_x + col as f64 * cell_size;
        let cy = palette_top + row as f64 * cell_size;
        let color_digits = color.trim_start_matches('#').to_ascii_lowercase();
        push_kodocs_node(
            nodes,
            next_id,
            "color_swatch",
            color.to_string(),
            format!("{prefix}.number{color_digits}"),
            Rect::new(cx, cy, cx + cell_size, cy + cell_size),
        );
    }
}

fn push_table_grid_nodes(nodes: &mut Vec<UiAutomationNode>, next_id: &mut u64) {
    let grid_top = MENU_BAR_H + TOOLBAR_H;
    let cell_sz = 20.0;
    let grid_anchor_x = 12.0;
    for r in 0..5 {
        for c in 0..5 {
            let gx = grid_anchor_x + c as f64 * cell_sz;
            let gy = grid_top + r as f64 * cell_sz;
            push_kodocs_node(
                nodes,
                next_id,
                "grid_cell",
                format!("{}x{}", r + 1, c + 1),
                format!("kodocs.toolbar.table_grid.cell.{}x{}", r + 1, c + 1),
                Rect::new(gx, gy, gx + cell_sz, gy + cell_sz),
            );
        }
    }
}

fn push_menu_items(
    x: f64,
    w: f64,
    items: &[&str],
    key: &str,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let dd_top = MENU_BAR_H + TOOLBAR_H;
    for (idx, &name) in items.iter().enumerate() {
        let id_key = name.to_lowercase().replace(' ', "_");
        push_kodocs_node(
            nodes,
            next_id,
            "menu_item",
            name,
            format!("kodocs.toolbar.{key}.item.{id_key}"),
            Rect::new(
                x,
                dd_top + idx as f64 * 26.0,
                x + w,
                dd_top + (idx + 1) as f64 * 26.0,
            ),
        );
    }
}

fn push_owned_menu_items(
    x: f64,
    w: f64,
    items: &[String],
    key: &str,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let refs: Vec<&str> = items.iter().map(String::as_str).collect();
    push_menu_items(x, w, &refs, key, nodes, next_id);
}

fn push_style_items(
    x: f64,
    w: f64,
    items: &[(&str, &str)],
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let dd_top = MENU_BAR_H + TOOLBAR_H;
    for (idx, (label, id_key)) in items.iter().enumerate() {
        push_kodocs_node(
            nodes,
            next_id,
            "menu_item",
            *label,
            format!("kodocs.toolbar.paragraph_style.item.{id_key}"),
            Rect::new(
                x,
                dd_top + idx as f64 * 26.0,
                x + w,
                dd_top + (idx + 1) as f64 * 26.0,
            ),
        );
    }
}
