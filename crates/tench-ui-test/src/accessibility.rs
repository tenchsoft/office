//! Accessibility tree snapshot utilities.
//!
//! Provides tools to capture and assert on the accessibility tree
//! structure produced by a widget tree. Useful for verifying:
//! - Correct ARIA roles on widgets.
//! - Proper label text.
//! - Focus management.
//! - Keyboard navigation structure.

use accesskit::{Node, Role};
use tench_ui::core::types::WidgetId;

/// A snapshot of a single node in the accessibility tree.
#[derive(Debug, Clone)]
pub struct AccessibilityNode {
    /// The widget ID that produced this node.
    pub widget_id: WidgetId,
    /// The accessibility role.
    pub role: Role,
    /// The node's label, if any.
    pub label: Option<String>,
    /// Whether this node is focused.
    pub is_focused: bool,
    /// Whether this node is disabled.
    pub is_disabled: bool,
    /// Child node indices in the snapshot's flat list.
    pub children: Vec<usize>,
}

/// A captured accessibility tree snapshot.
#[derive(Debug, Clone)]
pub struct AccessibilitySnapshot {
    /// Flat list of all nodes, depth-first order.
    nodes: Vec<AccessibilityNode>,
}

impl AccessibilitySnapshot {
    /// Creates an empty snapshot.
    pub fn empty() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Creates a snapshot from a flat list of nodes.
    pub fn from_nodes(nodes: Vec<AccessibilityNode>) -> Self {
        Self { nodes }
    }

    /// Returns the total number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if the snapshot is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns an iterator over all nodes.
    pub fn iter(&self) -> impl Iterator<Item = &AccessibilityNode> {
        self.nodes.iter()
    }

    /// Returns the root node (first in the list).
    pub fn root(&self) -> Option<&AccessibilityNode> {
        self.nodes.first()
    }

    /// Finds a node by widget ID.
    pub fn find_by_id(&self, id: WidgetId) -> Option<&AccessibilityNode> {
        self.nodes.iter().find(|n| n.widget_id == id)
    }

    /// Finds all nodes with a given role.
    pub fn find_by_role(&self, role: Role) -> Vec<&AccessibilityNode> {
        self.nodes.iter().filter(|n| n.role == role).collect()
    }

    /// Returns all focused nodes.
    pub fn focused_nodes(&self) -> Vec<&AccessibilityNode> {
        self.nodes.iter().filter(|n| n.is_focused).collect()
    }

    /// Returns all disabled nodes.
    pub fn disabled_nodes(&self) -> Vec<&AccessibilityNode> {
        self.nodes.iter().filter(|n| n.is_disabled).collect()
    }

    /// Returns all nodes with labels containing the given text.
    pub fn find_by_label(&self, text: &str) -> Vec<&AccessibilityNode> {
        self.nodes
            .iter()
            .filter(|n| n.label.as_ref().is_some_and(|l| l.contains(text)))
            .collect()
    }

    // --- Assertions ---

    /// Asserts that at least one node has the given role.
    pub fn assert_has_role(&self, role: Role) {
        assert!(
            !self.find_by_role(role).is_empty(),
            "Expected at least one node with role {:?}, but found none",
            role
        );
    }

    /// Asserts that no node has the given role.
    pub fn assert_no_role(&self, role: Role) {
        let found = self.find_by_role(role);
        assert!(
            found.is_empty(),
            "Expected no nodes with role {:?}, but found {}",
            role,
            found.len()
        );
    }

    /// Asserts that exactly one node is focused.
    pub fn assert_single_focus(&self) -> &AccessibilityNode {
        let focused = self.focused_nodes();
        assert_eq!(
            focused.len(),
            1,
            "Expected exactly one focused node, found {}",
            focused.len()
        );
        focused[0]
    }

    /// Asserts that no node is focused.
    pub fn assert_no_focus(&self) {
        let focused = self.focused_nodes();
        assert!(
            focused.is_empty(),
            "Expected no focused nodes, found {}",
            focused.len()
        );
    }

    /// Asserts that a specific widget is focused.
    pub fn assert_focus_on(&self, id: WidgetId) {
        let focused = self.focused_nodes();
        assert!(
            focused.iter().any(|n| n.widget_id == id),
            "Expected widget {} to be focused, but focused widgets are: {:?}",
            id,
            focused.iter().map(|n| n.widget_id).collect::<Vec<_>>()
        );
    }

    /// Asserts that a node with the given ID exists.
    pub fn assert_node_exists(&self, id: WidgetId) {
        assert!(
            self.find_by_id(id).is_some(),
            "Expected node with ID {} to exist in accessibility tree",
            id
        );
    }

    /// Asserts that a node with the given ID has a specific role.
    pub fn assert_role(&self, id: WidgetId, expected_role: Role) {
        let node = self.find_by_id(id).unwrap_or_else(|| {
            panic!(
                "Expected node with ID {} to exist in accessibility tree",
                id
            )
        });
        assert_eq!(
            node.role, expected_role,
            "Expected node {} to have role {:?}, but found {:?}",
            id, expected_role, node.role
        );
    }

    /// Asserts that a node with the given ID has a label containing the given text.
    pub fn assert_label_contains(&self, id: WidgetId, text: &str) {
        let node = self.find_by_id(id).unwrap_or_else(|| {
            panic!(
                "Expected node with ID {} to exist in accessibility tree",
                id
            )
        });
        match &node.label {
            Some(label) => assert!(
                label.contains(text),
                "Expected node {} label to contain {:?}, but label is {:?}",
                id,
                text,
                label
            ),
            None => panic!(
                "Expected node {} to have a label containing {:?}, but it has no label",
                id, text
            ),
        }
    }

    /// Asserts that the tree is non-empty (at least one node).
    pub fn assert_nonblank(&self) {
        assert!(
            !self.is_empty(),
            "Expected accessibility tree to be non-empty"
        );
    }
}

/// Builder for constructing accessibility nodes during tree capture.
pub struct AccessibilityNodeBuilder {
    widget_id: WidgetId,
    role: Role,
    label: Option<String>,
    is_focused: bool,
    is_disabled: bool,
    children: Vec<usize>,
}

impl AccessibilityNodeBuilder {
    /// Creates a new builder for the given widget.
    pub fn new(widget_id: WidgetId, role: Role) -> Self {
        Self {
            widget_id,
            role,
            label: None,
            is_focused: false,
            is_disabled: false,
            children: Vec::new(),
        }
    }

    /// Sets the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets whether the node is focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    /// Sets whether the node is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }

    /// Adds a child node index.
    pub fn child(mut self, index: usize) -> Self {
        self.children.push(index);
        self
    }

    /// Builds the accessibility node.
    pub fn build(self) -> AccessibilityNode {
        AccessibilityNode {
            widget_id: self.widget_id,
            role: self.role,
            label: self.label,
            is_focused: self.is_focused,
            is_disabled: self.is_disabled,
            children: self.children,
        }
    }
}

/// Helper to extract accessibility info from an accesskit `Node`.
pub fn node_role(node: &Node) -> Role {
    node.role()
}

/// Helper to extract the label from an accesskit `Node`.
pub fn node_label(node: &Node) -> Option<String> {
    node.label().map(|s| s.to_string())
}
