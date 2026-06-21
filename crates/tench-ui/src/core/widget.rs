//! Widget trait — the core abstraction for all UI elements.
//!
//! Also defines the accessibility tree types ([`AccessRole`], [`AccessibilityNode`])
//! that every widget can populate via the [`Widget::accessibility_tree`] method.

use std::any::Any;

use accesskit::{Node, Role};
use kurbo::{Axis, Point, Rect, Size};
use smallvec::SmallVec;
use tench_ui_automation_core::UiAutomationNode;
use vello::Scene;

use super::events::{Action, PointerEvent, TextEvent, WindowEvent};
use super::types::{CursorIcon, WidgetId};

/// A collection of child widget IDs.
pub type ChildrenIds = SmallVec<[WidgetId; 16]>;

// ---------------------------------------------------------------------------
// Accessibility tree types
// ---------------------------------------------------------------------------

/// Semantic role for an accessibility node, mirroring common ARIA roles.
///
/// Each variant maps to an [`accesskit::Role`] when the tree is handed to the
/// platform accessibility bridge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccessRole {
    /// Unknown / unspecified role.
    #[default]
    Unknown,
    /// A button that can be pressed.
    Button,
    /// A checkbox toggle.
    CheckBox,
    /// A radio button within a radio group.
    RadioButton,
    /// A single-line text input field.
    TextInput,
    /// A multi-line text area.
    MultilineTextInput,
    /// A heading (e.g. section title).
    Heading,
    /// A list container.
    List,
    /// A single item within a list.
    ListItem,
    /// A table.
    Table,
    /// A table row.
    TableRow,
    /// A table cell.
    TableCell,
    /// A paragraph of text.
    Paragraph,
    /// A static text label.
    Label,
    /// An image.
    Image,
    /// A hyperlink.
    Link,
    /// A menu.
    Menu,
    /// A menu item.
    MenuItem,
    /// A tab strip.
    TabList,
    /// A single tab.
    Tab,
    /// A scrollable region.
    ScrollArea,
    /// A progress indicator.
    ProgressBar,
    /// A slider control.
    Slider,
    /// A dialog / modal window.
    Dialog,
    /// A generic container with no semantic meaning.
    GenericContainer,
    /// A toggle switch.
    Switch,
    /// A tree view.
    Tree,
    /// A tree item.
    TreeItem,
    /// A top-level window.
    Window,
}

impl From<AccessRole> for Role {
    fn from(role: AccessRole) -> Role {
        match role {
            AccessRole::Unknown => Role::Unknown,
            AccessRole::Button => Role::Button,
            AccessRole::CheckBox => Role::CheckBox,
            AccessRole::RadioButton => Role::RadioButton,
            AccessRole::TextInput => Role::TextInput,
            AccessRole::MultilineTextInput => Role::MultilineTextInput,
            AccessRole::Heading => Role::Heading,
            AccessRole::List => Role::List,
            AccessRole::ListItem => Role::ListItem,
            AccessRole::Table => Role::Table,
            AccessRole::TableRow => Role::Row,
            AccessRole::TableCell => Role::Cell,
            AccessRole::Paragraph => Role::Paragraph,
            AccessRole::Label => Role::Label,
            AccessRole::Image => Role::Image,
            AccessRole::Link => Role::Link,
            AccessRole::Menu => Role::Menu,
            AccessRole::MenuItem => Role::MenuItem,
            AccessRole::TabList => Role::TabList,
            AccessRole::Tab => Role::Tab,
            AccessRole::ScrollArea => Role::ScrollView,
            AccessRole::ProgressBar => Role::ProgressIndicator,
            AccessRole::Slider => Role::Slider,
            AccessRole::Dialog => Role::Dialog,
            AccessRole::GenericContainer => Role::GenericContainer,
            AccessRole::Switch => Role::Switch,
            AccessRole::Tree => Role::Tree,
            AccessRole::TreeItem => Role::TreeItem,
            AccessRole::Window => Role::Window,
        }
    }
}

/// A node in the widget accessibility tree.
///
/// Each widget can produce one `AccessibilityNode` describing its semantic
/// role, label, value, and children. The framework walks the widget tree
/// and assembles the full accessibility tree for the platform bridge.
#[derive(Clone, Debug)]
pub struct AccessibilityNode {
    /// Semantic role.
    pub role: AccessRole,
    /// Human-readable label (e.g. button text).
    pub label: Option<String>,
    /// Current value (e.g. text input content, checkbox state).
    pub value: Option<String>,
    /// Whether this node currently has keyboard focus.
    pub focused: bool,
    /// Whether this node is disabled.
    pub disabled: bool,
    /// Accessible child nodes.
    pub children: Vec<AccessibilityNode>,
}

impl Default for AccessibilityNode {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibilityNode {
    /// Creates an empty node with [`AccessRole::Unknown`].
    pub fn new() -> Self {
        Self {
            role: AccessRole::Unknown,
            label: None,
            value: None,
            focused: false,
            disabled: false,
            children: Vec::new(),
        }
    }

    /// Creates a node with the given role.
    pub fn with_role(role: AccessRole) -> Self {
        Self {
            role,
            ..Self::new()
        }
    }

    /// Sets the label and returns `self` for chaining.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the value and returns `self` for chaining.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the focused flag and returns `self` for chaining.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the disabled flag and returns `self` for chaining.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Adds a child node and returns `self` for chaining.
    pub fn child(mut self, child: AccessibilityNode) -> Self {
        self.children.push(child);
        self
    }

    /// Converts this node into an [`accesskit::Node`] for the platform bridge.
    pub fn to_accesskit_node(&self, node: &mut Node) {
        node.set_role(Role::from(self.role));
        if let Some(label) = &self.label {
            node.set_label(label.clone());
        }
        if let Some(value) = &self.value {
            node.set_value(value.clone());
        }
        if self.focused {
            node.add_action(accesskit::Action::Focus);
        }
        if self.disabled {
            node.set_disabled();
        }
    }
}

/// Per-widget state managed by the framework.
#[derive(Debug)]
pub struct WidgetState {
    pub id: WidgetId,
    pub is_hovered: bool,
    pub is_active: bool,
    pub is_disabled: bool,
    pub is_focused: bool,
    pub is_stashed: bool,
    pub needs_layout: bool,
    pub needs_paint: bool,
    pub cursor_icon: CursorIcon,
    /// Position relative to parent.
    pub position: Point,
    /// Border-box size.
    pub size: Size,
}

impl WidgetState {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            is_hovered: false,
            is_active: false,
            is_disabled: false,
            is_focused: false,
            is_stashed: false,
            needs_layout: true,
            needs_paint: true,
            cursor_icon: CursorIcon::Default,
            position: Point::ZERO,
            size: Size::ZERO,
        }
    }

    /// The widget's bounding box in its own coordinate space.
    pub fn bounding_box(&self) -> Rect {
        Rect::from_origin_size(self.position, self.size)
    }
}

/// The trait implemented by all widgets.
pub trait Widget: Any + Send {
    /// Handles a pointer event.
    fn on_pointer_event(&mut self, ctx: &mut EventCtx<'_>, event: &PointerEvent) {
        let _ = (ctx, event);
    }

    /// Handles a text event.
    fn on_text_event(&mut self, ctx: &mut EventCtx<'_>, event: &TextEvent) {
        let _ = (ctx, event);
    }

    /// Handles a window event.
    fn on_window_event(&mut self, ctx: &mut EventCtx<'_>, event: &WindowEvent) {
        let _ = (ctx, event);
    }

    /// Computes the preferred length along the given axis.
    fn measure(&mut self, ctx: &mut MeasureCtx<'_>, axis: Axis, available: f64) -> f64;

    /// Lays out the widget within the given size.
    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, size: Size);

    /// Paints the widget's visual content into the Vello Scene.
    fn paint(&mut self, ctx: &mut PaintCtx<'_>, scene: &mut Scene);

    /// Returns the IDs of this widget's children.
    fn children(&self) -> Vec<WidgetId> {
        Vec::new()
    }

    /// Returns a mutable reference to a child widget by ID.
    fn child_mut(&mut self, _id: WidgetId) -> Option<&mut WidgetPod> {
        None
    }

    /// Returns the accessibility role.
    fn accessibility_role(&self) -> Role {
        Role::Unknown
    }

    /// Populates the accessibility node.
    fn accessibility(&mut self, _ctx: &mut AccessCtx<'_>, _node: &mut Node) {}

    /// Returns the accessibility tree rooted at this widget.
    ///
    /// The default implementation returns a single node whose role matches
    /// [`Self::accessibility_role`] and whose focused/disabled state is taken
    /// from [`WidgetState`]. Widgets should override this to provide richer
    /// semantics (labels, values, children).
    fn accessibility_tree(&self, state: &WidgetState) -> AccessibilityNode {
        let role = self.accessibility_role();
        let tench_role = match role {
            Role::Button | Role::DefaultButton => AccessRole::Button,
            Role::CheckBox => AccessRole::CheckBox,
            Role::RadioButton => AccessRole::RadioButton,
            Role::TextInput => AccessRole::TextInput,
            Role::MultilineTextInput => AccessRole::MultilineTextInput,
            Role::Heading => AccessRole::Heading,
            Role::List => AccessRole::List,
            Role::ListItem => AccessRole::ListItem,
            Role::Table => AccessRole::Table,
            Role::Row | Role::LayoutTableRow => AccessRole::TableRow,
            Role::Cell | Role::LayoutTableCell => AccessRole::TableCell,
            Role::Paragraph => AccessRole::Paragraph,
            Role::Label => AccessRole::Label,
            Role::Image => AccessRole::Image,
            Role::Link => AccessRole::Link,
            Role::Menu => AccessRole::Menu,
            Role::MenuItem => AccessRole::MenuItem,
            Role::TabList => AccessRole::TabList,
            Role::Tab => AccessRole::Tab,
            Role::ScrollView => AccessRole::ScrollArea,
            Role::ProgressIndicator => AccessRole::ProgressBar,
            Role::Slider => AccessRole::Slider,
            Role::Dialog => AccessRole::Dialog,
            Role::GenericContainer => AccessRole::GenericContainer,
            Role::Switch => AccessRole::Switch,
            Role::Tree => AccessRole::Tree,
            Role::TreeItem => AccessRole::TreeItem,
            Role::Window => AccessRole::Window,
            _ => AccessRole::Unknown,
        };
        AccessibilityNode {
            role: tench_role,
            label: None,
            value: None,
            focused: state.is_focused,
            disabled: state.is_disabled,
            children: Vec::new(),
        }
    }

    /// Stable automation identifier used by UI E2E and agent debugging.
    ///
    /// Most widgets can rely on role/label selectors. Product-specific widgets
    /// should override this when visible text is dynamic or localized.
    fn debug_id(&self) -> Option<&str> {
        None
    }

    /// Product or widget supplied semantic automation nodes.
    ///
    /// This is primarily for custom-drawn monolithic widgets that do not expose
    /// child `WidgetPod`s but still need selector-based UI automation. Returned
    /// bounds are interpreted in the widget's local coordinate space.
    fn automation_children(&self, _state: &WidgetState) -> Vec<UiAutomationNode> {
        Vec::new()
    }

    /// Whether this widget accepts pointer events. Default: true.
    fn accepts_pointer(&self) -> bool {
        true
    }

    /// Whether this widget can receive keyboard focus. Default: false.
    fn accepts_focus(&self) -> bool {
        false
    }

    /// Whether this widget accepts text input (IME). Default: false.
    fn accepts_text_input(&self) -> bool {
        false
    }

    /// Returns the cursor icon for this widget at the given position.
    fn cursor(&self, _pos: Point) -> CursorIcon {
        CursorIcon::Default
    }

    /// Downcast to `Any` for reflective access (e.g. reading internal state).
    fn as_any(&self) -> &dyn Any {
        unimplemented!("Widget::as_any not overridden")
    }

    /// Downcast to `Any` for reflective mutable access.
    fn as_any_mut(&mut self) -> &mut dyn Any {
        unimplemented!("Widget::as_any_mut not overridden")
    }
}

/// Downcast support for Widget trait objects.
impl dyn Widget {
    /// Downcast to a concrete widget type.
    pub fn downcast_ref<T: Widget>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }

    /// Downcast to a concrete widget type (mutable).
    pub fn downcast_mut<T: Widget>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut()
    }
}

// --- Context types ---

/// Context provided during event handling.
pub struct EventCtx<'a> {
    pub state: &'a mut WidgetState,
    pub global: &'a mut GlobalState,
    /// Whether an animation frame has been requested for the next vsync.
    pub anim_requested: bool,
}

impl<'a> EventCtx<'a> {
    /// The widget's ID.
    pub fn widget_id(&self) -> WidgetId {
        self.state.id
    }

    /// Whether the widget is hovered.
    pub fn is_hovered(&self) -> bool {
        self.state.is_hovered
    }

    /// Whether the widget is active (pressed).
    pub fn is_active(&self) -> bool {
        self.state.is_active
    }

    /// Request a repaint.
    pub fn request_paint(&mut self) {
        self.state.needs_paint = true;
    }

    /// Request a relayout.
    pub fn request_layout(&mut self) {
        self.state.needs_layout = true;
    }

    /// Request an animation frame. The backend will deliver an
    /// `WindowEvent::AnimFrame(timestamp)` on the next vsync.
    pub fn request_anim_frame(&mut self) {
        self.anim_requested = true;
        self.state.needs_paint = true;
    }
}

/// Context provided during measurement.
pub struct MeasureCtx<'a> {
    pub state: &'a mut WidgetState,
    pub global: &'a mut GlobalState,
}

/// Context provided during layout.
pub struct LayoutCtx<'a> {
    pub state: &'a mut WidgetState,
    pub global: &'a mut GlobalState,
}

/// Context provided during painting.
pub struct PaintCtx<'a> {
    pub state: &'a mut WidgetState,
    pub global: &'a mut GlobalState,
    pub theme: crate::theme::Theme,
}

impl<'a> PaintCtx<'a> {
    /// The widget's size.
    pub fn size(&self) -> Size {
        self.state.size
    }

    /// Whether the widget is hovered.
    pub fn is_hovered(&self) -> bool {
        self.state.is_hovered
    }

    /// Returns the current theme.
    pub fn theme(&self) -> &crate::theme::Theme {
        &self.theme
    }
}
/// Context provided during accessibility tree building.
pub struct AccessCtx<'a> {
    pub state: &'a WidgetState,
}

/// Global state shared across the widget tree.
pub struct GlobalState {
    pub focus_target: Option<WidgetId>,
    pub focused_widget: Option<WidgetId>,
    pub pointer_capture: Option<WidgetId>,
    pub hovered_widget: Option<WidgetId>,
    pub pending_actions: Vec<(Action, WidgetId)>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            focus_target: None,
            focused_widget: None,
            pointer_capture: None,
            hovered_widget: None,
            pending_actions: Vec::new(),
        }
    }
}

/// A widget wrapped with its state, ready for insertion into the tree.
pub struct WidgetPod {
    pub widget: Box<dyn Widget>,
    pub state: WidgetState,
}

impl WidgetPod {
    /// Creates a new WidgetPod from a widget.
    pub fn new(widget: impl Widget + 'static) -> Self {
        let id = WidgetId::next();
        Self {
            widget: Box::new(widget),
            state: WidgetState::new(id),
        }
    }

    /// The widget's ID.
    pub fn id(&self) -> WidgetId {
        self.state.id
    }

    /// Hit-test: returns the ID of the deepest descendant (or self) whose bounds
    /// contain `pos`. Children are checked first in reverse Z-order (last painted
    /// = topmost).
    ///
    /// `pos` is in the **parent's** coordinate space. The widget's own
    /// `state.position` offset is subtracted before checking children or the
    /// local bounding box.
    pub fn hit_test_recursive(&mut self, pos: Point) -> Option<WidgetId> {
        // Translate into this widget's local coordinate space.
        let local = pos - self.state.position.to_vec2();

        // Check if the point is within this widget's bounding box.
        let bounds = Rect::from_origin_size(Point::ZERO, self.state.size);
        if !bounds.contains(local) {
            return None;
        }

        // Delegate to children first (reverse Z-order so topmost wins).
        let child_ids = self.widget.children();
        for child_id in child_ids.into_iter().rev() {
            if let Some(child) = self.widget.child_mut(child_id) {
                if let Some(hit) = child.hit_test_recursive(local) {
                    return Some(hit);
                }
            }
        }

        Some(self.state.id)
    }
}
