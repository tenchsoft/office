//! VirtualTree widget for large hierarchical browsers.
//!
//! This is separate from `TreeView`: `TreeView` is a compact retained tree,
//! while `VirtualTree` exposes flattening, range, and hit-test contracts for
//! product-scale collections and curricula.

use kurbo::{Axis, Point, Rect, Size};
use parley::FontWeight;
use vello::Scene;

use crate::core::events::PointerEvent;
use crate::core::types::Color;
use crate::core::widget::{EventCtx, LayoutCtx, MeasureCtx, PaintCtx, Widget};
use crate::render::Painter;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualTreeNode {
    pub id: String,
    pub label: String,
    pub children: Vec<VirtualTreeNode>,
    pub expanded: bool,
    pub selected: bool,
    pub badge: Option<String>,
}

impl VirtualTreeNode {
    pub fn leaf(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
            expanded: false,
            selected: false,
            badge: None,
        }
    }

    pub fn branch(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<VirtualTreeNode>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children,
            expanded: false,
            selected: false,
            badge: None,
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualTreeRow {
    pub id: String,
    pub label: String,
    pub depth: usize,
    pub expanded: bool,
    pub selected: bool,
    pub has_children: bool,
    pub badge: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VirtualTreeViewport {
    pub scroll_y: f64,
    pub row_height: f64,
    pub indent: f64,
    pub overscan_rows: usize,
}

impl Default for VirtualTreeViewport {
    fn default() -> Self {
        Self {
            scroll_y: 0.0,
            row_height: 30.0,
            indent: 18.0,
            overscan_rows: 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualTreeVisibleRange {
    pub first_row: usize,
    pub last_row: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualTreeHit {
    pub row_index: usize,
    pub node_id: String,
    pub disclosure: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualTreeSearchMatch {
    pub node_id: String,
    pub label: String,
    pub path: Vec<String>,
    pub depth: usize,
}

pub fn flatten_visible_tree(nodes: &[VirtualTreeNode]) -> Vec<VirtualTreeRow> {
    let mut rows = Vec::new();
    append_visible_rows(nodes, 0, &mut rows);
    rows
}

pub fn virtual_tree_search(
    nodes: &[VirtualTreeNode],
    query: &str,
    limit: usize,
) -> Vec<VirtualTreeSearchMatch> {
    let query = normalize_search_text(query);
    if query.is_empty() || limit == 0 {
        return Vec::new();
    }

    let mut matches = Vec::new();
    let mut path = Vec::new();
    append_search_matches(nodes, &query, limit, &mut path, &mut matches);
    matches
}

pub fn virtual_tree_visible_range(
    row_count: usize,
    viewport_height: f64,
    viewport: VirtualTreeViewport,
) -> VirtualTreeVisibleRange {
    let row_height = viewport.row_height.max(1.0);
    let first_row = ((viewport.scroll_y / row_height).floor() as usize)
        .saturating_sub(viewport.overscan_rows)
        .min(row_count);
    let visible_rows = (viewport_height / row_height).ceil().max(0.0) as usize
        + 1
        + viewport.overscan_rows.saturating_mul(2);
    VirtualTreeVisibleRange {
        first_row,
        last_row: (first_row + visible_rows).min(row_count),
    }
}

pub fn virtual_tree_hit_test(
    rows: &[VirtualTreeRow],
    viewport: VirtualTreeViewport,
    point: Point,
) -> Option<VirtualTreeHit> {
    let row_index = ((point.y + viewport.scroll_y) / viewport.row_height.max(1.0)).floor() as usize;
    let row = rows.get(row_index)?;
    let disclosure_x0 = row.depth as f64 * viewport.indent;
    let disclosure = row.has_children
        && point.x >= disclosure_x0
        && point.x <= disclosure_x0 + viewport.indent + 8.0;
    Some(VirtualTreeHit {
        row_index,
        node_id: row.id.clone(),
        disclosure,
    })
}

pub struct VirtualTree {
    nodes: Vec<VirtualTreeNode>,
    viewport: VirtualTreeViewport,
    // clippy: callback field type is idiomatic for this widget
    #[allow(clippy::type_complexity)]
    on_select: Option<Box<dyn Fn(&str) + Send>>,
}

impl VirtualTree {
    pub fn new(nodes: Vec<VirtualTreeNode>) -> Self {
        Self {
            nodes,
            viewport: VirtualTreeViewport::default(),
            on_select: None,
        }
    }

    pub fn with_viewport(mut self, viewport: VirtualTreeViewport) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn on_select(mut self, callback: impl Fn(&str) + Send + 'static) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    pub fn rows(&self) -> Vec<VirtualTreeRow> {
        flatten_visible_tree(&self.nodes)
    }

    pub fn visible_range(&self, height: f64) -> VirtualTreeVisibleRange {
        virtual_tree_visible_range(self.rows().len(), height, self.viewport)
    }

    fn max_scroll_y(&self, height: f64) -> f64 {
        (self.rows().len() as f64 * self.viewport.row_height - height).max(0.0)
    }
}

impl Widget for VirtualTree {
    fn measure(&mut self, _ctx: &mut MeasureCtx<'_>, axis: Axis, available: f64) -> f64 {
        match axis {
            Axis::Horizontal => available,
            Axis::Vertical => available,
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx<'_>, _size: Size) {}

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, scene: &mut Scene) {
        let theme = ctx.theme().clone();
        let size = ctx.size();
        let rows = self.rows();
        let range = virtual_tree_visible_range(rows.len(), size.height, self.viewport);
        let mut painter = Painter::new(scene);

        painter.fill_rect(Rect::from_origin_size(Point::ZERO, size), theme.background);

        for row_index in range.first_row..range.last_row {
            let Some(row) = rows.get(row_index) else {
                continue;
            };
            let y = row_index as f64 * self.viewport.row_height - self.viewport.scroll_y;
            let row_rect = Rect::new(0.0, y, size.width, y + self.viewport.row_height);
            if row.selected {
                painter.fill_rect(row_rect, theme.primary);
            } else if row_index % 2 == 1 {
                painter.fill_rect(row_rect, Color::rgba8(255, 255, 255, 7));
            }

            let x = row.depth as f64 * self.viewport.indent;
            if row.has_children {
                painter.draw_text(
                    if row.expanded { "v" } else { ">" },
                    x + 4.0,
                    y + self.viewport.row_height / 2.0 + 4.0,
                    theme.secondary,
                    theme.font_size_small,
                    FontWeight::BOLD,
                    false,
                );
            }

            painter.draw_text(
                &row.label,
                x + self.viewport.indent + 6.0,
                y + self.viewport.row_height / 2.0 + 5.0,
                if row.selected {
                    theme.on_primary
                } else {
                    theme.on_background
                },
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );

            if let Some(badge) = &row.badge {
                painter.draw_text(
                    badge,
                    size.width - 48.0,
                    y + self.viewport.row_height / 2.0 + 5.0,
                    theme.secondary,
                    theme.font_size_small,
                    FontWeight::NORMAL,
                    false,
                );
            }
        }
    }

    fn on_pointer_event(&mut self, ctx: &mut EventCtx<'_>, event: &PointerEvent) {
        match event {
            PointerEvent::Down(event) => {
                let rows = self.rows();
                if let Some(hit) = virtual_tree_hit_test(&rows, self.viewport, event.pos) {
                    if hit.disclosure {
                        toggle_node(&mut self.nodes, &hit.node_id);
                    } else {
                        clear_tree_selection(&mut self.nodes);
                        select_node(&mut self.nodes, &hit.node_id);
                        if let Some(callback) = &self.on_select {
                            callback(&hit.node_id);
                        }
                    }
                    ctx.request_paint();
                }
            }
            PointerEvent::Scroll(event) => {
                self.viewport.scroll_y = (self.viewport.scroll_y + event.delta.y * 30.0)
                    .clamp(0.0, self.max_scroll_y(600.0));
                ctx.request_paint();
            }
            _ => {}
        }
    }
}

fn append_visible_rows(nodes: &[VirtualTreeNode], depth: usize, rows: &mut Vec<VirtualTreeRow>) {
    for node in nodes {
        rows.push(VirtualTreeRow {
            id: node.id.clone(),
            label: node.label.clone(),
            depth,
            expanded: node.expanded,
            selected: node.selected,
            has_children: node.has_children(),
            badge: node.badge.clone(),
        });
        if node.expanded {
            append_visible_rows(&node.children, depth + 1, rows);
        }
    }
}

fn append_search_matches(
    nodes: &[VirtualTreeNode],
    query: &str,
    limit: usize,
    path: &mut Vec<String>,
    matches: &mut Vec<VirtualTreeSearchMatch>,
) {
    if matches.len() >= limit {
        return;
    }

    for node in nodes {
        if matches.len() >= limit {
            return;
        }

        path.push(node.label.clone());
        let searchable = format!("{} {}", node.id, node.label);
        if normalize_search_text(&searchable).contains(query) {
            matches.push(VirtualTreeSearchMatch {
                node_id: node.id.clone(),
                label: node.label.clone(),
                path: path.clone(),
                depth: path.len().saturating_sub(1),
            });
        }
        append_search_matches(&node.children, query, limit, path, matches);
        path.pop();
    }
}

fn normalize_search_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn toggle_node(nodes: &mut [VirtualTreeNode], node_id: &str) -> bool {
    for node in nodes {
        if node.id == node_id {
            if node.has_children() {
                node.expanded = !node.expanded;
                return true;
            }
            return false;
        }
        if toggle_node(&mut node.children, node_id) {
            return true;
        }
    }
    false
}

fn clear_tree_selection(nodes: &mut [VirtualTreeNode]) {
    for node in nodes {
        node.selected = false;
        clear_tree_selection(&mut node.children);
    }
}

fn select_node(nodes: &mut [VirtualTreeNode], node_id: &str) -> bool {
    for node in nodes {
        if node.id == node_id {
            node.selected = true;
            return true;
        }
        if select_node(&mut node.children, node_id) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Vec<VirtualTreeNode> {
        vec![
            VirtualTreeNode::branch(
                "math",
                "Mathematics",
                vec![
                    VirtualTreeNode::leaf("algebra", "Algebra"),
                    VirtualTreeNode::leaf("calculus", "Calculus"),
                ],
            )
            .expanded(true)
            .badge("2"),
            VirtualTreeNode::branch(
                "science",
                "Science",
                vec![VirtualTreeNode::leaf("biology", "Biology")],
            ),
        ]
    }

    #[test]
    fn flatten_visible_tree_keeps_depth_and_collapsed_children_hidden() {
        let rows = flatten_visible_tree(&tree());

        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].id, "math");
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[3].id, "science");
    }

    #[test]
    fn virtual_range_applies_overscan() {
        let viewport = VirtualTreeViewport {
            scroll_y: 300.0,
            row_height: 30.0,
            indent: 18.0,
            overscan_rows: 2,
        };

        let range = virtual_tree_visible_range(1_000, 120.0, viewport);

        assert_eq!(range.first_row, 8);
        assert_eq!(range.last_row, 17);
    }

    #[test]
    fn hit_test_detects_disclosure_zone() {
        let rows = flatten_visible_tree(&tree());
        let hit =
            virtual_tree_hit_test(&rows, VirtualTreeViewport::default(), Point::new(8.0, 8.0))
                .expect("hit");

        assert_eq!(hit.node_id, "math");
        assert!(hit.disclosure);
    }

    #[test]
    fn search_returns_path_and_respects_limit() {
        let matches = virtual_tree_search(&tree(), "calculus", 5);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].node_id, "calculus");
        assert_eq!(
            matches[0].path,
            vec!["Mathematics".to_string(), "Calculus".to_string()]
        );

        let limited = virtual_tree_search(&tree(), "i", 1);
        assert_eq!(limited.len(), 1);
    }

    #[test]
    fn search_handles_large_curriculum_tree_without_flattening_visibility() {
        let nodes = (0..100_000)
            .map(|index| VirtualTreeNode::leaf(format!("node-{index}"), format!("Lesson {index}")))
            .collect::<Vec<_>>();

        let matches = virtual_tree_search(&nodes, "Lesson 99999", 10);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].node_id, "node-99999");
    }
}
