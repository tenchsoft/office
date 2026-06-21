use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAutomationCaptureRequest {
    #[serde(default = "default_true")]
    pub include_png: bool,
    #[serde(default = "default_true")]
    pub include_tree: bool,
}

impl Default for UiAutomationCaptureRequest {
    fn default() -> Self {
        Self {
            include_png: true,
            include_tree: true,
        }
    }
}

impl UiAutomationCaptureRequest {
    pub const fn png_only() -> Self {
        Self {
            include_png: true,
            include_tree: false,
        }
    }

    pub const fn tree_only() -> Self {
        Self {
            include_png: false,
            include_tree: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAutomationCapture {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub png_bytes: Vec<u8>,
    pub ui_tree: Option<UiAutomationNode>,
    pub focused_node: Option<u64>,
    pub hovered_node: Option<u64>,
}

impl UiAutomationCapture {
    pub fn tree(&self) -> Option<&UiAutomationNode> {
        self.ui_tree.as_ref()
    }

    pub fn find_node(&self, selector: &UiAutomationSelector) -> Option<&UiAutomationNode> {
        self.ui_tree
            .as_ref()
            .and_then(|root| find_node(root, selector))
    }

    pub fn node_inventory(&self) -> Vec<UiAutomationNodeSummary> {
        self.ui_tree
            .as_ref()
            .map(node_inventory)
            .unwrap_or_default()
    }

    pub fn nodes_at_point(&self, point: UiAutomationPoint) -> Vec<&UiAutomationNode> {
        self.ui_tree
            .as_ref()
            .map(|root| nodes_at_point(root, point))
            .unwrap_or_default()
    }

    pub fn report(&self) -> String {
        format_capture_report(self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAutomationNode {
    pub id: u64,
    pub debug_id: Option<String>,
    pub role: String,
    pub label: Option<String>,
    pub value: Option<String>,
    pub bounds: UiAutomationRect,
    pub enabled: bool,
    pub focused: bool,
    pub hovered: bool,
    #[serde(default)]
    pub children: Vec<UiAutomationNode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAutomationNodeSummary {
    pub id: u64,
    pub depth: usize,
    pub debug_id: Option<String>,
    pub role: String,
    pub label: Option<String>,
    pub value: Option<String>,
    pub bounds: UiAutomationRect,
    pub center: UiAutomationPoint,
    pub enabled: bool,
    pub focused: bool,
    pub hovered: bool,
    pub interactive: bool,
    pub child_count: usize,
    pub selector_hint: Option<UiAutomationSelector>,
}

impl UiAutomationNode {
    pub fn center(&self) -> UiAutomationPoint {
        self.bounds.center()
    }

    pub fn node_count(&self) -> usize {
        walk_nodes(self).count()
    }

    pub fn walk(&self) -> UiAutomationNodeIter<'_> {
        walk_nodes(self)
    }

    pub fn find(&self, selector: &UiAutomationSelector) -> Option<&UiAutomationNode> {
        find_node(self, selector)
    }

    pub fn find_by_debug_id(&self, debug_id: &str) -> Option<&UiAutomationNode> {
        walk_nodes(self).find(|node| node.debug_id.as_deref() == Some(debug_id))
    }

    pub fn nodes_by_role<'a>(&'a self, role: &str) -> Vec<&'a UiAutomationNode> {
        walk_nodes(self)
            .filter(|node| node.role.eq_ignore_ascii_case(role))
            .collect()
    }

    pub fn display_text(&self) -> Option<&str> {
        self.label.as_deref().or(self.value.as_deref())
    }

    pub fn selector_hint(&self) -> Option<UiAutomationSelector> {
        if let Some(debug_id) = &self.debug_id {
            return Some(UiAutomationSelector::debug_id(debug_id.clone()));
        }
        if let Some(label) = &self.label {
            if !self.role.is_empty() && self.role != "unknown" {
                return Some(UiAutomationSelector::role_and_text(
                    self.role.clone(),
                    label.clone(),
                ));
            }
            return Some(UiAutomationSelector::text(label.clone()));
        }
        self.value
            .as_ref()
            .map(|value| UiAutomationSelector::text(value.clone()))
    }

    pub fn is_interactive(&self) -> bool {
        self.enabled && !self.bounds.is_empty() && is_interactive_role(&self.role)
    }

    pub fn summary(&self, depth: usize) -> UiAutomationNodeSummary {
        UiAutomationNodeSummary {
            id: self.id,
            depth,
            debug_id: self.debug_id.clone(),
            role: self.role.clone(),
            label: self.label.clone(),
            value: self.value.clone(),
            bounds: self.bounds,
            center: self.center(),
            enabled: self.enabled,
            focused: self.focused,
            hovered: self.hovered,
            interactive: self.is_interactive(),
            child_count: self.children.len(),
            selector_hint: self.selector_hint(),
        }
    }
}

pub struct UiAutomationNodeIter<'a> {
    stack: Vec<&'a UiAutomationNode>,
}

impl<'a> Iterator for UiAutomationNodeIter<'a> {
    type Item = &'a UiAutomationNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        self.stack.extend(node.children.iter().rev());
        Some(node)
    }
}

impl UiAutomationRect {
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    pub fn center(&self) -> UiAutomationPoint {
        UiAutomationPoint {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    pub fn area(&self) -> f64 {
        self.width.max(0.0) * self.height.max(0.0)
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn contains_point(&self, point: UiAutomationPoint) -> bool {
        point.x >= self.x
            && point.x <= self.right()
            && point.y >= self.y
            && point.y <= self.bottom()
    }

    pub fn contains_rect(&self, rect: UiAutomationRect) -> bool {
        rect.x >= self.x
            && rect.right() <= self.right()
            && rect.y >= self.y
            && rect.bottom() <= self.bottom()
    }

    pub fn intersects(&self, rect: UiAutomationRect) -> bool {
        self.x < rect.right()
            && self.right() > rect.x
            && self.y < rect.bottom()
            && self.bottom() > rect.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAutomationRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAutomationPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiAutomationSelector {
    ByDebugId { debug_id: String },
    ByRoleAndText { role: String, text: String },
    ByText { text: String },
    ByPoint { point: UiAutomationPoint },
}

impl UiAutomationSelector {
    pub fn debug_id(debug_id: impl Into<String>) -> Self {
        Self::ByDebugId {
            debug_id: debug_id.into(),
        }
    }

    pub fn role_and_text(role: impl Into<String>, text: impl Into<String>) -> Self {
        Self::ByRoleAndText {
            role: role.into(),
            text: text.into(),
        }
    }

    pub fn text(text: impl Into<String>) -> Self {
        Self::ByText { text: text.into() }
    }

    pub fn point(x: f64, y: f64) -> Self {
        Self::ByPoint {
            point: UiAutomationPoint { x, y },
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::ByDebugId { debug_id } => format!("debug_id={debug_id}"),
            Self::ByRoleAndText { role, text } => format!("role={role}, text={text}"),
            Self::ByText { text } => format!("text={text}"),
            Self::ByPoint { point } => format!("point=({}, {})", point.x, point.y),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiAutomationAction {
    Capture {
        #[serde(default)]
        request: UiAutomationCaptureRequest,
    },
    Click {
        selector: UiAutomationSelector,
        #[serde(default)]
        modifiers: UiAutomationModifiers,
    },
    RightClick {
        selector: UiAutomationSelector,
        #[serde(default)]
        modifiers: UiAutomationModifiers,
    },
    DoubleClick {
        selector: UiAutomationSelector,
        #[serde(default)]
        modifiers: UiAutomationModifiers,
    },
    TypeText {
        selector: UiAutomationSelector,
        text: String,
    },
    KeyPress {
        key: UiAutomationKey,
        #[serde(default)]
        modifiers: UiAutomationModifiers,
    },
    Scroll {
        selector: UiAutomationSelector,
        delta_x: f64,
        delta_y: f64,
    },
    Drag {
        start: UiAutomationPoint,
        end: UiAutomationPoint,
        #[serde(default = "default_drag_steps")]
        steps: usize,
    },
    DragFromTo {
        from_selector: UiAutomationSelector,
        to_selector: UiAutomationSelector,
        #[serde(default = "default_drag_steps")]
        steps: usize,
    },
    Hover {
        selector: UiAutomationSelector,
    },
    WaitFor {
        selector: UiAutomationSelector,
        timeout_ms: u64,
    },
    AnimFrame {
        timestamp_ms: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum UiAutomationKey {
    Character(String),
    Enter,
    Tab,
    Escape,
    Backspace,
    Delete,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    PageUp,
    PageDown,
    F2,
    F12,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAutomationModifiers {
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub control: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub meta: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum UiAutomationError {
    Disabled,
    ReleaseBuild,
    BackendUnavailable,
    CaptureUnavailable,
    SelectorNotFound,
    Timeout,
    InvalidAction,
    ClickBlocked {
        target_selector: String,
        blocked_by: Option<String>,
        point: UiAutomationPoint,
    },
    Internal(String),
}

pub fn find_node<'a>(
    root: &'a UiAutomationNode,
    selector: &UiAutomationSelector,
) -> Option<&'a UiAutomationNode> {
    match selector {
        UiAutomationSelector::ByPoint { .. } => None,
        UiAutomationSelector::ByDebugId { debug_id } => find_node_by(root, &|node| {
            node.debug_id.as_deref() == Some(debug_id.as_str())
        }),
        UiAutomationSelector::ByRoleAndText { role, text } => {
            let role = role.to_ascii_lowercase();
            find_node_by(root, &|node| {
                node.role.eq_ignore_ascii_case(&role)
                    && node
                        .label
                        .as_deref()
                        .is_some_and(|label| text_matches(label, text))
            })
        }
        UiAutomationSelector::ByText { text } => find_node_by(root, &|node| {
            node.label
                .as_deref()
                .is_some_and(|label| text_matches(label, text))
                || node
                    .value
                    .as_deref()
                    .is_some_and(|value| text_matches(value, text))
        }),
    }
}

pub fn find_nodes<'a>(
    root: &'a UiAutomationNode,
    selector: &UiAutomationSelector,
) -> Vec<&'a UiAutomationNode> {
    if matches!(selector, UiAutomationSelector::ByPoint { .. }) {
        return Vec::new();
    }

    walk_nodes(root)
        .filter(|node| selector_matches(node, selector))
        .collect()
}

pub fn node_inventory(root: &UiAutomationNode) -> Vec<UiAutomationNodeSummary> {
    let mut summaries = Vec::with_capacity(root.node_count());
    collect_node_inventory(root, 0, &mut summaries);
    summaries
}

pub fn interactive_node_inventory(root: &UiAutomationNode) -> Vec<UiAutomationNodeSummary> {
    node_inventory(root)
        .into_iter()
        .filter(|summary| summary.interactive)
        .collect()
}

pub fn nodes_at_point(root: &UiAutomationNode, point: UiAutomationPoint) -> Vec<&UiAutomationNode> {
    walk_nodes(root)
        .filter(|node| node.bounds.contains_point(point))
        .collect()
}

/// Returns `true` if `ancestor_id` is an ancestor of `descendant_id` in the
/// tree rooted at `root`.
pub fn is_ancestor_of(root: &UiAutomationNode, ancestor_id: u64, descendant_id: u64) -> bool {
    if ancestor_id == descendant_id {
        return false;
    }
    find_ancestor_recursive(root, ancestor_id, descendant_id)
}

fn find_ancestor_recursive(node: &UiAutomationNode, ancestor_id: u64, descendant_id: u64) -> bool {
    for child in &node.children {
        if child.id == ancestor_id {
            // Check if descendant_id is in this child's subtree.
            return contains_id(child, descendant_id);
        }
        if find_ancestor_recursive(child, ancestor_id, descendant_id) {
            return true;
        }
    }
    false
}

fn contains_id(node: &UiAutomationNode, id: u64) -> bool {
    if node.id == id {
        return true;
    }
    node.children.iter().any(|child| contains_id(child, id))
}

pub fn walk_nodes(root: &UiAutomationNode) -> UiAutomationNodeIter<'_> {
    UiAutomationNodeIter { stack: vec![root] }
}

pub fn describe_selector(selector: &UiAutomationSelector) -> String {
    selector.description()
}

pub fn format_capture_report(capture: &UiAutomationCapture) -> String {
    let mut report = String::new();
    let _ = writeln!(
        report,
        "viewport={}x{} png_bytes={} focused={:?} hovered={:?}",
        capture.width,
        capture.height,
        capture.png_bytes.len(),
        capture.focused_node,
        capture.hovered_node
    );

    let Some(root) = &capture.ui_tree else {
        let _ = writeln!(report, "ui_tree=<not captured>");
        return report;
    };

    let inventory = node_inventory(root);
    let interactive_count = inventory
        .iter()
        .filter(|summary| summary.interactive)
        .count();
    let _ = writeln!(
        report,
        "nodes={} interactive={}",
        inventory.len(),
        interactive_count
    );
    let _ = writeln!(report, "depth id role selector center bounds state text");

    for summary in inventory {
        let selector = summary
            .selector_hint
            .as_ref()
            .map(UiAutomationSelector::description)
            .unwrap_or_else(|| "none".to_string());
        let state = format_node_state(&summary);
        let text = summary
            .label
            .as_deref()
            .or(summary.value.as_deref())
            .map(compact_text)
            .unwrap_or_default();
        let _ = writeln!(
            report,
            "{} {} {} {} ({:.1},{:.1}) [{:.1},{:.1} {:.1}x{:.1}] {} {}",
            summary.depth,
            summary.id,
            summary.role,
            selector,
            summary.center.x,
            summary.center.y,
            summary.bounds.x,
            summary.bounds.y,
            summary.bounds.width,
            summary.bounds.height,
            state,
            text
        );
    }

    report
}

pub fn format_tree_report(root: &UiAutomationNode) -> String {
    format_capture_report(&UiAutomationCapture {
        width: root.bounds.width.max(0.0).round() as u32,
        height: root.bounds.height.max(0.0).round() as u32,
        png_bytes: Vec::new(),
        ui_tree: Some(root.clone()),
        focused_node: walk_nodes(root)
            .find(|node| node.focused)
            .map(|node| node.id),
        hovered_node: walk_nodes(root)
            .find(|node| node.hovered)
            .map(|node| node.id),
    })
}

fn collect_node_inventory(
    node: &UiAutomationNode,
    depth: usize,
    summaries: &mut Vec<UiAutomationNodeSummary>,
) {
    summaries.push(node.summary(depth));
    for child in &node.children {
        collect_node_inventory(child, depth + 1, summaries);
    }
}

fn find_node_by<'a>(
    node: &'a UiAutomationNode,
    predicate: &impl Fn(&UiAutomationNode) -> bool,
) -> Option<&'a UiAutomationNode> {
    if predicate(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node_by(child, predicate))
}

fn selector_matches(node: &UiAutomationNode, selector: &UiAutomationSelector) -> bool {
    match selector {
        UiAutomationSelector::ByPoint { .. } => false,
        UiAutomationSelector::ByDebugId { debug_id } => {
            node.debug_id.as_deref() == Some(debug_id.as_str())
        }
        UiAutomationSelector::ByRoleAndText { role, text } => {
            node.role.eq_ignore_ascii_case(role)
                && node
                    .label
                    .as_deref()
                    .is_some_and(|label| text_matches(label, text))
        }
        UiAutomationSelector::ByText { text } => {
            node.label
                .as_deref()
                .is_some_and(|label| text_matches(label, text))
                || node
                    .value
                    .as_deref()
                    .is_some_and(|value| text_matches(value, text))
        }
    }
}

fn text_matches(value: &str, expected: &str) -> bool {
    value == expected
        || value
            .to_ascii_lowercase()
            .contains(&expected.to_ascii_lowercase())
}

fn is_interactive_role(role: &str) -> bool {
    let role = role.to_ascii_lowercase();
    matches!(
        role.as_str(),
        "button"
            | "checkbox"
            | "radio"
            | "text_input"
            | "text_area"
            | "link"
            | "menu"
            | "menu_item"
            | "tab"
            | "slider"
            | "switch"
            | "tree_item"
    )
}

fn format_node_state(summary: &UiAutomationNodeSummary) -> String {
    let mut state = String::new();
    if !summary.enabled {
        state.push_str("disabled");
    }
    if summary.focused {
        push_state(&mut state, "focused");
    }
    if summary.hovered {
        push_state(&mut state, "hovered");
    }
    if summary.interactive {
        push_state(&mut state, "interactive");
    }
    if state.is_empty() {
        state.push_str("static");
    }
    state
}

fn push_state(state: &mut String, value: &str) {
    if !state.is_empty() {
        state.push('|');
    }
    state.push_str(value);
}

fn compact_text(value: &str) -> String {
    let mut compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    const MAX_LEN: usize = 80;
    if compact.chars().count() > MAX_LEN {
        compact = compact.chars().take(MAX_LEN - 3).collect();
        compact.push_str("...");
    }
    format!("\"{compact}\"")
}

const fn default_true() -> bool {
    true
}

const fn default_drag_steps() -> usize {
    8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: u64, role: &str, label: Option<&str>) -> UiAutomationNode {
        UiAutomationNode {
            id,
            debug_id: None,
            role: role.to_string(),
            label: label.map(str::to_string),
            value: None,
            bounds: UiAutomationRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 40.0,
            },
            enabled: true,
            focused: false,
            hovered: false,
            children: Vec::new(),
        }
    }

    fn node_with_debug_id(
        id: u64,
        role: &str,
        debug_id: &str,
        label: Option<&str>,
        bounds: UiAutomationRect,
    ) -> UiAutomationNode {
        UiAutomationNode {
            id,
            debug_id: Some(debug_id.to_string()),
            role: role.to_string(),
            label: label.map(str::to_string),
            value: None,
            bounds,
            enabled: true,
            focused: false,
            hovered: false,
            children: Vec::new(),
        }
    }

    #[test]
    fn selector_finds_role_and_text_ui_automation() {
        let mut root = node(1, "window", None);
        root.children.push(node(2, "button", Some("Start Study")));

        let found = find_node(
            &root,
            &UiAutomationSelector::ByRoleAndText {
                role: "button".to_string(),
                text: "Study".to_string(),
            },
        )
        .expect("button node");

        assert_eq!(found.id, 2);
    }

    #[test]
    fn capture_request_defaults_to_png_and_tree_ui_automation() {
        let request = UiAutomationCaptureRequest::default();
        assert!(request.include_png);
        assert!(request.include_tree);
    }

    #[test]
    fn rect_helpers_return_position_data_ui_automation() {
        let rect = UiAutomationRect {
            x: 10.0,
            y: 20.0,
            width: 80.0,
            height: 40.0,
        };

        assert_eq!(rect.right(), 90.0);
        assert_eq!(rect.bottom(), 60.0);
        assert_eq!(rect.center(), UiAutomationPoint { x: 50.0, y: 40.0 });
        assert!(rect.contains_point(UiAutomationPoint { x: 20.0, y: 30.0 }));
        assert!(rect.contains_rect(UiAutomationRect {
            x: 20.0,
            y: 30.0,
            width: 10.0,
            height: 10.0,
        }));
    }

    #[test]
    fn inventory_exposes_bounds_and_selector_hints_ui_automation() {
        let mut root = node(1, "window", Some("Root"));
        root.children.push(node_with_debug_id(
            2,
            "button",
            "fixture.save",
            Some("Save"),
            UiAutomationRect {
                x: 12.0,
                y: 16.0,
                width: 88.0,
                height: 32.0,
            },
        ));

        let inventory = node_inventory(&root);
        assert_eq!(inventory.len(), 2);
        assert_eq!(inventory[1].depth, 1);
        assert_eq!(inventory[1].center, UiAutomationPoint { x: 56.0, y: 32.0 });
        assert!(inventory[1].interactive);
        assert_eq!(
            inventory[1].selector_hint,
            Some(UiAutomationSelector::debug_id("fixture.save"))
        );

        let report = format_tree_report(&root);
        assert!(report.contains("debug_id=fixture.save"));
        assert!(report.contains("(56.0,32.0)"));
    }

    #[test]
    fn point_lookup_returns_nodes_containing_position_ui_automation() {
        let mut root = node(1, "window", Some("Root"));
        root.children.push(node_with_debug_id(
            2,
            "button",
            "fixture.save",
            Some("Save"),
            UiAutomationRect {
                x: 12.0,
                y: 16.0,
                width: 88.0,
                height: 32.0,
            },
        ));

        let hits = nodes_at_point(&root, UiAutomationPoint { x: 56.0, y: 32.0 });
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[1].debug_id.as_deref(), Some("fixture.save"));
    }
}
