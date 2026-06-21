// ---------------------------------------------------------------------------
// Sidebar automation nodes
// ---------------------------------------------------------------------------

use tench_document_core::{BlockNode, InlineNode};
use tench_ui::prelude::*;
use tench_ui::UiAutomationNode;

use super::automation_helpers::push_docs_node;
use super::automation_version_history::push_version_history_nodes;
use super::layout::sidebar::compute_sidebar;
use super::state::{DocsState, SidebarTab};

pub(super) fn push_sidebar_nodes(
    state: &DocsState,
    content_w: f64,
    main_y: f64,
    status_y: f64,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    if !(state.show_style_panel || state.show_comments) {
        return;
    }

    let layout = compute_sidebar(content_w, main_y, status_y);
    push_sidebar_tabs(state, &layout.tabs, nodes, next_id);

    if state.sidebar_tab == SidebarTab::Style {
        push_docs_node(
            nodes,
            next_id,
            "button",
            "Save snapshot",
            "docs.sidebar.save_snapshot",
            layout.save_snapshot,
        );
    }
    if state.sidebar_tab == SidebarTab::Navigate {
        push_outline_nodes(state, layout.area, main_y, nodes, next_id);
    }
    if state.show_comments {
        push_comment_nodes(state, layout.area, main_y, nodes, next_id);
    }
    if state.show_style_panel && state.sidebar_tab == SidebarTab::Style {
        push_style_stats(state, layout.area, nodes, next_id);
    }
    push_version_history_nodes(state, layout.area, nodes, next_id);
}

fn push_sidebar_tabs(
    state: &DocsState,
    tab_rects: &[Rect; 3],
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let tabs = [
        ("Style", "docs.sidebar.style-tab", SidebarTab::Style),
        (
            "Navigate",
            "docs.sidebar.navigate-tab",
            SidebarTab::Navigate,
        ),
        ("AI", "docs.sidebar.ai-tab", SidebarTab::Ai),
    ];
    for (idx, (label, debug_id, tab)) in tabs.iter().enumerate() {
        push_docs_node(nodes, next_id, "tab", *label, *debug_id, tab_rects[idx]);
        if let Some(node) = nodes.last_mut() {
            node.value = Some(
                if state.sidebar_tab == *tab {
                    "selected"
                } else {
                    "unselected"
                }
                .to_string(),
            );
        }
    }
}

fn push_outline_nodes(
    state: &DocsState,
    area: Rect,
    main_y: f64,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let mut heading_y = main_y + 56.0;
    for (i, block) in state.current_document().content.iter().enumerate() {
        let BlockNode::Heading { content, level, .. } = block else {
            continue;
        };
        let text = content
            .iter()
            .filter_map(|node| match node {
                InlineNode::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<String>();
        if text.is_empty() {
            continue;
        }

        let indent = (*level as f64 - 1.0) * 16.0;
        let item_x = area.x0 + 16.0 + indent;
        let display: String = text.chars().take(30).collect();
        let truncated = if text.chars().count() > 30 {
            format!("{display}...")
        } else {
            display
        };
        push_docs_node(
            nodes,
            next_id,
            "list_item",
            truncated,
            format!("docs.outline.heading.{i}"),
            Rect::new(
                item_x,
                heading_y,
                item_x + area.width() - 32.0 - indent,
                heading_y + 22.0,
            ),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(format!("H{level}"));
        }
        heading_y += 26.0;
        if heading_y > area.y1 - 30.0 {
            break;
        }
    }
}

fn push_comment_nodes(
    state: &DocsState,
    area: Rect,
    main_y: f64,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    push_docs_node(
        nodes,
        next_id,
        "button",
        if state.comments_collapsed {
            "Comments (collapsed)"
        } else {
            "Comments"
        },
        "docs.comments.collapse",
        Rect::new(area.x0 + 12.0, main_y + 12.0, area.x1 - 12.0, main_y + 42.0),
    );
    if state.comments_collapsed {
        return;
    }
    if state.comments.is_empty() {
        push_docs_node(
            nodes,
            next_id,
            "text",
            "No comments yet",
            "docs.comments.empty",
            Rect::new(area.x0 + 18.0, main_y + 42.0, area.x1 - 18.0, main_y + 66.0),
        );
        return;
    }
    let mut comment_y = main_y + 42.0;
    for (i, _) in state.comments.iter().enumerate() {
        push_docs_node(
            nodes,
            next_id,
            "list_item",
            format!("Comment {}", i + 1),
            format!("docs.comments.row.{i}"),
            Rect::new(
                area.x0 + 10.0,
                comment_y - 12.0,
                area.x1 - 10.0,
                comment_y + 40.0,
            ),
        );
        comment_y += 56.0;
    }
}

fn push_style_stats(
    state: &DocsState,
    area: Rect,
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
) {
    let card_w = (area.width() - 40.0) / 2.0;
    let stats_y = area.y0 + 74.0;
    let stat_items = [
        ("docs.style_panel.words", state.word_count.to_string()),
        (
            "docs.style_panel.characters",
            state.character_count().to_string(),
        ),
        (
            "docs.style_panel.paragraphs",
            state.paragraph_count().to_string(),
        ),
        (
            "docs.style_panel.headings",
            count_heading_lines(state).to_string(),
        ),
        (
            "docs.style_panel.lists",
            count_list_lines(state).to_string(),
        ),
        (
            "docs.style_panel.read_time",
            format!("{} min", state.read_time_minutes()),
        ),
    ];
    for (idx, (debug_id, value)) in stat_items.iter().enumerate() {
        let row = idx / 2;
        let col = idx % 2;
        let x = area.x0 + 16.0 + col as f64 * (card_w + 8.0);
        push_docs_node(
            nodes,
            next_id,
            "text",
            value.clone(),
            *debug_id,
            Rect::new(
                x,
                stats_y + row as f64 * 56.0,
                x + card_w,
                stats_y + row as f64 * 56.0 + 48.0,
            ),
        );
        if let Some(node) = nodes.last_mut() {
            node.value = Some(value.clone());
        }
    }
}

fn count_heading_lines(state: &DocsState) -> usize {
    state
        .document_lines()
        .iter()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("# ")
                || trimmed.starts_with("## ")
                || trimmed.starts_with("### ")
                || trimmed.starts_with("#### ")
                || trimmed.starts_with("##### ")
                || trimmed.starts_with("###### ")
        })
        .count()
}

fn count_list_lines(state: &DocsState) -> usize {
    state
        .document_lines()
        .iter()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("- ")
                || trimmed.starts_with("* ")
                || trimmed.starts_with("+ ")
                || (trimmed.len() > 2
                    && trimmed
                        .as_bytes()
                        .first()
                        .is_some_and(|c| c.is_ascii_digit())
                    && trimmed.contains(". "))
        })
        .count()
}
