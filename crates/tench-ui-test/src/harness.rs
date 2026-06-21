//! Test harness for mounting and interacting with a widget tree in isolation.
//!
//! `TestHarness` provides a deterministic, headless environment for testing
//! tench-ui widgets without a real window or GPU surface. It manages:
//! - Widget tree lifecycle (mount, layout, paint).
//! - A fixed viewport size and theme for reproducible results.
//! - Event dispatch to the correct widget via hit-testing.
//! - Access to widget state for assertions.

use kurbo::{Point, Rect, Size, Vec2};
use tench_ui::core::events::{
    KeyboardEvent, LogicalKey, Modifiers, NamedKey, PointerEvent, TextEvent, WindowEvent,
};
use tench_ui::core::types::WidgetId;
use tench_ui::core::widget::{AccessRole, EventCtx, GlobalState, LayoutCtx, PaintCtx, WidgetPod};
use tench_ui::theme::Theme;
use tench_ui::vello::Scene;
use tench_ui_automation_core::{
    find_node, format_capture_report, is_ancestor_of, node_inventory,
    nodes_at_point as find_nodes_at_point, UiAutomationAction, UiAutomationCapture,
    UiAutomationCaptureRequest, UiAutomationError, UiAutomationKey, UiAutomationModifiers,
    UiAutomationNode, UiAutomationNodeSummary, UiAutomationPoint, UiAutomationRect,
    UiAutomationSelector,
};

use crate::component::Component;
use crate::{snapshot, EventSimulator};

/// Configuration for a `TestHarness`.
#[derive(Debug, Clone)]
pub struct HarnessConfig {
    /// Viewport size for layout.
    pub viewport: Size,
    /// Theme to use during paint and layout.
    pub theme: Theme,
    /// DPI scale factor (default: 1.0).
    pub scale_factor: f64,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            viewport: Size::new(800.0, 600.0),
            theme: Theme::default(),
            scale_factor: 1.0,
        }
    }
}

impl HarnessConfig {
    /// Creates a config with a specific viewport size.
    pub fn with_viewport(width: f64, height: f64) -> Self {
        Self {
            viewport: Size::new(width, height),
            ..Default::default()
        }
    }

    /// Creates a mobile-sized config (390x844 — iPhone 14).
    pub fn mobile() -> Self {
        Self::with_viewport(390.0, 844.0)
    }

    /// Sets the theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the scale factor.
    pub fn with_scale_factor(mut self, factor: f64) -> Self {
        self.scale_factor = factor;
        self
    }
}

/// Headless test harness for a single widget tree.
///
/// Wraps a root `WidgetPod` and provides methods to drive layout, paint,
/// and event dispatch in a deterministic manner.
pub struct TestHarness {
    root: WidgetPod,
    global: GlobalState,
    config: HarnessConfig,
    /// Last rendered scene (populated after `paint`).
    last_scene: Option<Scene>,
}

impl TestHarness {
    /// Creates a new harness with the given root widget and default config.
    pub fn new(root: impl tench_ui::Widget + 'static) -> Self {
        Self::with_config(root, HarnessConfig::default())
    }

    /// Creates a new harness with a custom config.
    pub fn with_config(root: impl tench_ui::Widget + 'static, config: HarnessConfig) -> Self {
        let mut harness = Self {
            root: WidgetPod::new(root),
            global: GlobalState::new(),
            config,
            last_scene: None,
        };
        // Notify the widget of the initial viewport size so it can initialize
        // any size-dependent state (e.g. last_window_size in KodocsApp).
        harness.dispatch_window(WindowEvent::Resize {
            width: harness.config.viewport.width as u32,
            height: harness.config.viewport.height as u32,
        });
        harness
    }

    /// Returns the root widget's ID.
    pub fn root_id(&self) -> WidgetId {
        self.root.id()
    }

    /// Returns a reference to the harness config.
    pub fn config(&self) -> &HarnessConfig {
        &self.config
    }

    /// Returns a reference to the global state.
    pub fn global(&self) -> &GlobalState {
        &self.global
    }

    /// Returns a mutable reference to the global state.
    pub fn global_mut(&mut self) -> &mut GlobalState {
        &mut self.global
    }

    /// Returns the root widget pod.
    pub fn root(&self) -> &WidgetPod {
        &self.root
    }

    /// Returns a mutable reference to the root widget pod.
    pub fn root_mut(&mut self) -> &mut WidgetPod {
        &mut self.root
    }

    /// Returns a mutable reference to the root widget, downcast to the
    /// concrete type `W`. Panics if the widget is not of type `W`.
    ///
    /// This is intended for test code that needs to call product-specific
    /// methods on the root widget (e.g. `DocsApp::load_plain_text`).
    pub fn root_widget_mut<W: tench_ui::Widget + 'static>(&mut self) -> &mut W {
        self.root
            .widget
            .downcast_mut::<W>()
            .expect("root widget downcast: type mismatch")
    }

    /// Creates a fluent [`Component`] handle for the given `debug_id`.
    pub fn component(&mut self, debug_id: impl Into<String>) -> Component<'_> {
        Component::new(self, debug_id.into())
    }

    // --- Layout ---

    /// Runs layout on the root widget with the configured viewport size.
    pub fn layout(&mut self) {
        self.layout_size(self.config.viewport);
    }

    /// Runs layout with a specific size.
    pub fn layout_size(&mut self, size: Size) {
        self.root.state.size = size;
        let mut ctx = LayoutCtx {
            state: &mut self.root.state,
            global: &mut self.global,
        };
        self.root.widget.layout(&mut ctx, size);
        self.root.state.needs_layout = false;
    }

    // --- Paint ---

    /// Runs paint on the root widget, producing a Vello `Scene`.
    pub fn paint(&mut self) -> &Scene {
        let mut scene = Scene::new();
        {
            let mut ctx = PaintCtx {
                state: &mut self.root.state,
                global: &mut self.global,
                theme: self.config.theme.clone(),
            };
            self.root.widget.paint(&mut ctx, &mut scene);
        }
        self.root.state.needs_paint = false;
        self.last_scene = Some(scene);
        self.last_scene.as_ref().unwrap()
    }

    /// Returns the last painted scene, if any.
    pub fn last_scene(&self) -> Option<&Scene> {
        self.last_scene.as_ref()
    }

    // --- Event dispatch ---

    /// Dispatches a pointer event to the widget under the pointer position.
    ///
    /// Uses hit-testing to find the target widget, then delivers the event
    /// by walking from the root down to the target. Every widget on the
    /// ancestor chain receives the event so that parent containers (scroll
    /// views, hit-test areas, etc.) can react.
    ///
    /// If no widget is hit, the event is delivered to the root only.
    pub fn dispatch_pointer(&mut self, event: PointerEvent) {
        let pos = match &event {
            PointerEvent::Move(e) => e.pos,
            PointerEvent::Down(e) => e.pos,
            PointerEvent::Up(e) => e.pos,
            PointerEvent::Scroll(e) => e.pos,
            PointerEvent::Enter | PointerEvent::Leave => Point::ZERO,
        };

        // Hit-test to find the deepest target widget.
        let target = self.root.hit_test_recursive(pos);

        match target {
            // No widget hit — deliver to root only.
            None => {
                let mut ctx = EventCtx {
                    state: &mut self.root.state,
                    global: &mut self.global,
                    anim_requested: false,
                };
                self.root.widget.on_pointer_event(&mut ctx, &event);
            }
            // Target is root — deliver directly.
            Some(target_id) if target_id == self.root.id() => {
                let mut ctx = EventCtx {
                    state: &mut self.root.state,
                    global: &mut self.global,
                    anim_requested: false,
                };
                self.root.widget.on_pointer_event(&mut ctx, &event);
            }
            // Target is a descendant — walk from root to target, delivering
            // the event to every widget on the ancestor chain.
            Some(target_id) => {
                dispatch_pointer_to_descendant(
                    &mut self.root.widget,
                    &mut self.root.state,
                    &mut self.global,
                    target_id,
                    &event,
                );
            }
        }
    }

    /// Dispatches a text event to the currently focused widget (or root if none).
    pub fn dispatch_text(&mut self, event: TextEvent) {
        let mut ctx = EventCtx {
            state: &mut self.root.state,
            global: &mut self.global,
            anim_requested: false,
        };
        self.root.widget.on_text_event(&mut ctx, &event);
    }

    /// Dispatches a window event to the root widget.
    pub fn dispatch_window(&mut self, event: WindowEvent) {
        let mut ctx = EventCtx {
            state: &mut self.root.state,
            global: &mut self.global,
            anim_requested: false,
        };
        self.root.widget.on_window_event(&mut ctx, &event);
    }

    /// Advances the animation clock by dispatching an `AnimFrame` event.
    pub fn advance_time(&mut self, duration: std::time::Duration) {
        let ms = duration.as_millis() as u64;
        self.dispatch_window(WindowEvent::AnimFrame(ms));
    }

    // --- Hit testing ---

    /// Hit-tests at the given position, returning the deepest widget ID.
    pub fn hit_test(&mut self, pos: Point) -> Option<WidgetId> {
        self.root.hit_test_recursive(pos)
    }

    // --- State queries ---

    /// Returns the root widget's bounding box.
    pub fn root_bounds(&self) -> Rect {
        self.root.state.bounding_box()
    }

    /// Returns the root widget's size.
    pub fn root_size(&self) -> Size {
        self.root.state.size
    }

    /// Returns whether the root widget is focused.
    pub fn is_root_focused(&self) -> bool {
        self.root.state.is_focused
    }

    /// Returns whether the root widget is hovered.
    pub fn is_root_hovered(&self) -> bool {
        self.root.state.is_hovered
    }

    /// Returns the currently focused widget ID from global state.
    pub fn focused_widget(&self) -> Option<WidgetId> {
        self.global.focused_widget
    }

    /// Drains pending actions from global state.
    pub fn drain_actions(&mut self) -> Vec<(tench_ui::core::events::Action, WidgetId)> {
        std::mem::take(&mut self.global.pending_actions)
    }

    // --- UI automation ---

    /// Captures the semantic UI automation tree without a platform window.
    pub fn automation_tree(&mut self) -> UiAutomationNode {
        self.layout();
        automation_node_from_pod(&mut self.root, Point::ZERO, &self.global)
    }

    /// Returns one automation node by selector, including bounds and center
    /// coordinates for point-based follow-up actions.
    pub fn automation_node(
        &mut self,
        selector: &UiAutomationSelector,
    ) -> Result<UiAutomationNode, UiAutomationError> {
        let tree = self.automation_tree();
        find_node(&tree, selector)
            .cloned()
            .ok_or(UiAutomationError::SelectorNotFound)
    }

    /// Returns the screen-space bounds for a selector.
    pub fn automation_bounds(
        &mut self,
        selector: &UiAutomationSelector,
    ) -> Result<UiAutomationRect, UiAutomationError> {
        Ok(self.automation_node(selector)?.bounds)
    }

    /// Returns the screen-space center point for a selector.
    pub fn automation_center(
        &mut self,
        selector: &UiAutomationSelector,
    ) -> Result<UiAutomationPoint, UiAutomationError> {
        Ok(self.automation_node(selector)?.center())
    }

    /// Returns all automation nodes whose bounds contain the given point.
    pub fn automation_nodes_at_point(&mut self, point: UiAutomationPoint) -> Vec<UiAutomationNode> {
        let tree = self.automation_tree();
        find_nodes_at_point(&tree, point)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Returns a flat UI inventory with selector hints, bounds, centers, role,
    /// text, enabled/focus/hover state, and depth.
    pub fn automation_inventory(&mut self) -> Vec<UiAutomationNodeSummary> {
        let tree = self.automation_tree();
        node_inventory(&tree)
    }

    /// Returns a human-readable UI report for test design and agent debugging.
    pub fn automation_report(&mut self) -> String {
        let capture = self.automation_capture(UiAutomationCaptureRequest::tree_only());
        format_capture_report(&capture)
    }

    /// Clicks a stable debug ID selector.
    pub fn automation_click_debug_id(
        &mut self,
        debug_id: impl Into<String>,
    ) -> Result<UiAutomationCapture, UiAutomationError> {
        self.automation_action(UiAutomationAction::Click {
            selector: UiAutomationSelector::debug_id(debug_id),
            modifiers: UiAutomationModifiers::default(),
        })
    }

    /// Captures headless automation state.
    pub fn automation_capture(
        &mut self,
        request: UiAutomationCaptureRequest,
    ) -> UiAutomationCapture {
        self.try_automation_capture(request)
            .expect("headless automation capture")
    }

    /// Captures headless automation state and returns rendering errors.
    pub fn try_automation_capture(
        &mut self,
        request: UiAutomationCaptureRequest,
    ) -> Result<UiAutomationCapture, UiAutomationError> {
        self.layout();
        let width = self.root.state.size.width.max(0.0).round() as u32;
        let height = self.root.state.size.height.max(0.0).round() as u32;
        let png_bytes = if request.include_png {
            self.paint();
            let scene = self
                .last_scene
                .as_ref()
                .ok_or(UiAutomationError::CaptureUnavailable)?;
            snapshot::render_scene_to_png(scene, width, height)?
        } else {
            Vec::new()
        };
        let ui_tree = if request.include_tree {
            Some(self.automation_tree())
        } else {
            None
        };

        Ok(UiAutomationCapture {
            width,
            height,
            png_bytes,
            ui_tree,
            focused_node: self.global.focused_widget.map(WidgetId::to_raw),
            hovered_node: self.global.hovered_widget.map(WidgetId::to_raw),
        })
    }

    /// Applies a selector-based automation action to the headless widget tree.
    pub fn automation_action(
        &mut self,
        action: UiAutomationAction,
    ) -> Result<UiAutomationCapture, UiAutomationError> {
        match action {
            UiAutomationAction::Capture { request } => {
                return self.try_automation_capture(request);
            }
            UiAutomationAction::Click {
                selector,
                modifiers,
            } => {
                // Ensure layout and paint are warm so that widget-internal state
                // derived during paint (e.g. last_window_size) is up-to-date.
                self.layout();
                self.paint();
                let point = self.automation_point(&selector)?;

                // Overlay detection: check if target is the topmost interactive
                // node at the click point. If another interactive node sits on
                // top (not an ancestor of the target), the click is blocked.
                //
                // NOTE: This check requires accurate automation-tree bounds that
                // match the paint/hit-test geometry (Section 3 of the hardening
                // plan). Until all products adopt the shared layout module, the
                // check is opt-in via the TENCH_UI_CLICK_BLOCKED environment
                // variable to avoid false positives from coordinate drift.
                if std::env::var("TENCH_UI_CLICK_BLOCKED").unwrap_or_default() == "1" {
                    let tree = self.automation_tree();
                    if let Some(target) = find_node(&tree, &selector) {
                        let target_id = target.id;
                        let auto_point = UiAutomationPoint {
                            x: point.x,
                            y: point.y,
                        };
                        let hits = find_nodes_at_point(&tree, auto_point);
                        // `hits` is in tree-order (pre-order DFS). Later entries
                        // are deeper/closer to the front. Iterate in reverse to
                        // check front-most nodes first.
                        for hit in hits.iter().rev() {
                            if hit.id != target_id
                                && hit.is_interactive()
                                && !is_ancestor_of(&tree, hit.id, target_id)
                            {
                                return Err(UiAutomationError::ClickBlocked {
                                    target_selector: selector.description(),
                                    blocked_by: hit.selector_hint().map(|s| s.description()),
                                    point: auto_point,
                                });
                            }
                        }
                    }
                }

                for event in modifier_down_events(modifiers) {
                    self.dispatch_text(event);
                }
                for event in EventSimulator::click(point) {
                    self.dispatch_pointer(event);
                }
                for event in modifier_up_events(modifiers) {
                    self.dispatch_text(event);
                }
            }
            UiAutomationAction::RightClick {
                selector,
                modifiers,
            } => {
                self.layout();
                self.paint();
                let point = self.automation_point(&selector)?;
                for event in modifier_down_events(modifiers) {
                    self.dispatch_text(event);
                }
                for event in EventSimulator::right_click(point) {
                    self.dispatch_pointer(event);
                }
                for event in modifier_up_events(modifiers) {
                    self.dispatch_text(event);
                }
            }
            UiAutomationAction::DoubleClick {
                selector,
                modifiers,
            } => {
                self.layout();
                self.paint();
                let point = self.automation_point(&selector)?;
                for event in modifier_down_events(modifiers) {
                    self.dispatch_text(event);
                }
                for event in EventSimulator::double_click(point) {
                    self.dispatch_pointer(event);
                }
                for event in modifier_up_events(modifiers) {
                    self.dispatch_text(event);
                }
            }
            UiAutomationAction::TypeText { selector, text } => {
                self.layout();
                self.paint();
                let point = self.automation_point(&selector)?;
                for event in EventSimulator::click(point) {
                    self.dispatch_pointer(event);
                }
                for event in EventSimulator::type_text(&text) {
                    self.dispatch_text(event);
                }
            }
            UiAutomationAction::KeyPress { key, modifiers } => {
                for event in automation_key_press_events(key, modifiers) {
                    self.dispatch_text(event);
                }
            }
            UiAutomationAction::Scroll {
                selector,
                delta_x,
                delta_y,
            } => {
                self.layout();
                self.paint();
                let point = self.automation_point(&selector)?;
                self.dispatch_pointer(EventSimulator::scroll(point, Vec2::new(delta_x, delta_y)));
            }
            UiAutomationAction::Drag { start, end, steps } => {
                let start = Point::new(start.x, start.y);
                let end = Point::new(end.x, end.y);
                for event in EventSimulator::drag(start, end, steps) {
                    self.dispatch_pointer(event);
                }
            }
            UiAutomationAction::DragFromTo {
                from_selector,
                to_selector,
                steps,
            } => {
                self.layout();
                self.paint();
                let start = self.automation_point(&from_selector)?;
                let end = self.automation_point(&to_selector)?;
                for event in EventSimulator::drag(start, end, steps) {
                    self.dispatch_pointer(event);
                }
            }
            UiAutomationAction::Hover { selector } => {
                self.layout();
                self.paint();
                let point = self.automation_point(&selector)?;
                self.dispatch_pointer(EventSimulator::pointer_move(point, Vec2::ZERO));
            }
            UiAutomationAction::WaitFor {
                selector,
                timeout_ms,
            } => {
                self.automation_wait_for(&selector, timeout_ms)?;
            }
            UiAutomationAction::AnimFrame { timestamp_ms } => {
                self.dispatch_window(WindowEvent::AnimFrame(timestamp_ms));
            }
        }

        self.try_automation_capture(UiAutomationCaptureRequest::default())
    }

    /// Waits until a selector is present in the headless automation tree.
    pub fn automation_wait_for(
        &mut self,
        selector: &UiAutomationSelector,
        timeout_ms: u64,
    ) -> Result<UiAutomationNode, UiAutomationError> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        loop {
            let tree = self.automation_tree();
            if let Some(node) = find_node(&tree, selector) {
                return Ok(node.clone());
            }
            if std::time::Instant::now() >= deadline {
                return Err(UiAutomationError::Timeout);
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }

    fn automation_point(
        &mut self,
        selector: &UiAutomationSelector,
    ) -> Result<Point, UiAutomationError> {
        match selector {
            UiAutomationSelector::ByPoint { point } => Ok(Point::new(point.x, point.y)),
            _ => {
                let tree = self.automation_tree();
                let node = find_node(&tree, selector).ok_or(UiAutomationError::SelectorNotFound)?;
                let center = node.center();
                Ok(Point::new(center.x, center.y))
            }
        }
    }
}

fn automation_node_from_pod(
    pod: &mut WidgetPod,
    parent_origin: Point,
    global: &GlobalState,
) -> UiAutomationNode {
    let origin = parent_origin + pod.state.position.to_vec2();
    let semantics = pod.widget.accessibility_tree(&pod.state);
    let mut children = pod
        .widget
        .automation_children(&pod.state)
        .into_iter()
        .map(|node| offset_automation_node(node, origin))
        .collect::<Vec<_>>();

    let child_ids = pod.widget.children();
    for child_id in child_ids {
        if let Some(child) = pod.widget.child_mut(child_id) {
            children.push(automation_node_from_pod(child, origin, global));
        }
    }

    UiAutomationNode {
        id: pod.state.id.to_raw(),
        debug_id: pod.widget.debug_id().map(str::to_string),
        role: access_role_name(semantics.role).to_string(),
        label: semantics.label,
        value: semantics.value,
        bounds: UiAutomationRect {
            x: origin.x,
            y: origin.y,
            width: pod.state.size.width,
            height: pod.state.size.height,
        },
        enabled: !pod.state.is_disabled,
        focused: pod.state.is_focused || global.focused_widget == Some(pod.state.id),
        hovered: pod.state.is_hovered || global.hovered_widget == Some(pod.state.id),
        children,
    }
}

fn offset_automation_node(mut node: UiAutomationNode, origin: Point) -> UiAutomationNode {
    node.bounds.x += origin.x;
    node.bounds.y += origin.y;
    node.children = node
        .children
        .into_iter()
        .map(|child| offset_automation_node(child, origin))
        .collect();
    node
}

fn access_role_name(role: AccessRole) -> &'static str {
    match role {
        AccessRole::Unknown => "unknown",
        AccessRole::Button => "button",
        AccessRole::CheckBox => "checkbox",
        AccessRole::RadioButton => "radio",
        AccessRole::TextInput => "text_input",
        AccessRole::MultilineTextInput => "text_area",
        AccessRole::Heading => "heading",
        AccessRole::List => "list",
        AccessRole::ListItem => "list_item",
        AccessRole::Table => "table",
        AccessRole::TableRow => "row",
        AccessRole::TableCell => "cell",
        AccessRole::Paragraph => "paragraph",
        AccessRole::Label => "label",
        AccessRole::Image => "image",
        AccessRole::Link => "link",
        AccessRole::Menu => "menu",
        AccessRole::MenuItem => "menu_item",
        AccessRole::TabList => "tab_list",
        AccessRole::Tab => "tab",
        AccessRole::ScrollArea => "scroll_area",
        AccessRole::ProgressBar => "progress",
        AccessRole::Slider => "slider",
        AccessRole::Dialog => "dialog",
        AccessRole::GenericContainer => "container",
        AccessRole::Switch => "switch",
        AccessRole::Tree => "tree",
        AccessRole::TreeItem => "tree_item",
        AccessRole::Window => "window",
    }
}

/// Generates modifier key-down events for the given modifiers.
fn modifier_down_events(modifiers: UiAutomationModifiers) -> Vec<TextEvent> {
    let mut events = Vec::new();
    if modifiers.shift {
        events.push(modifier_key_event(NamedKey::Shift, true));
    }
    if modifiers.control {
        events.push(modifier_key_event(NamedKey::Control, true));
    }
    if modifiers.alt {
        events.push(modifier_key_event(NamedKey::Alt, true));
    }
    if modifiers.meta {
        events.push(modifier_key_event(NamedKey::Super, true));
    }
    events
}

/// Generates modifier key-up events for the given modifiers (in reverse order).
fn modifier_up_events(modifiers: UiAutomationModifiers) -> Vec<TextEvent> {
    let mut events = Vec::new();
    if modifiers.meta {
        events.push(modifier_key_event(NamedKey::Super, false));
    }
    if modifiers.alt {
        events.push(modifier_key_event(NamedKey::Alt, false));
    }
    if modifiers.control {
        events.push(modifier_key_event(NamedKey::Control, false));
    }
    if modifiers.shift {
        events.push(modifier_key_event(NamedKey::Shift, false));
    }
    events
}

fn modifier_key_event(named: NamedKey, is_pressed: bool) -> TextEvent {
    TextEvent::Keyboard(KeyboardEvent {
        physical_key: 0,
        logical_key: LogicalKey::Named(named),
        is_pressed,
        is_repeat: false,
        modifiers: Modifiers::default(),
    })
}

fn automation_key_press_events(
    key: UiAutomationKey,
    modifiers: UiAutomationModifiers,
) -> [TextEvent; 2] {
    let logical = automation_logical_key(key);
    [
        automation_key_event(logical.clone(), true, modifiers),
        automation_key_event(logical, false, modifiers),
    ]
}

fn automation_key_event(
    logical_key: LogicalKey,
    is_pressed: bool,
    modifiers: UiAutomationModifiers,
) -> TextEvent {
    TextEvent::Keyboard(KeyboardEvent {
        physical_key: 0,
        logical_key,
        is_pressed,
        is_repeat: false,
        modifiers: Modifiers {
            shift: modifiers.shift,
            control: modifiers.control,
            alt: modifiers.alt,
            super_key: modifiers.meta,
        },
    })
}

fn automation_logical_key(key: UiAutomationKey) -> LogicalKey {
    match key {
        UiAutomationKey::Character(value) => LogicalKey::Character(value),
        UiAutomationKey::Enter => LogicalKey::Named(NamedKey::Enter),
        UiAutomationKey::Tab => LogicalKey::Named(NamedKey::Tab),
        UiAutomationKey::Escape => LogicalKey::Named(NamedKey::Escape),
        UiAutomationKey::Backspace => LogicalKey::Named(NamedKey::Backspace),
        UiAutomationKey::Delete => LogicalKey::Named(NamedKey::Delete),
        UiAutomationKey::ArrowLeft => LogicalKey::Named(NamedKey::ArrowLeft),
        UiAutomationKey::ArrowRight => LogicalKey::Named(NamedKey::ArrowRight),
        UiAutomationKey::ArrowUp => LogicalKey::Named(NamedKey::ArrowUp),
        UiAutomationKey::ArrowDown => LogicalKey::Named(NamedKey::ArrowDown),
        UiAutomationKey::Home => LogicalKey::Named(NamedKey::Home),
        UiAutomationKey::End => LogicalKey::Named(NamedKey::End),
        UiAutomationKey::PageUp => LogicalKey::Named(NamedKey::PageUp),
        UiAutomationKey::PageDown => LogicalKey::Named(NamedKey::PageDown),
        UiAutomationKey::F2 => LogicalKey::Named(NamedKey::F(2)),
        UiAutomationKey::F12 => LogicalKey::Named(NamedKey::F(12)),
    }
}

/// Recursively walks the widget tree to deliver a pointer event to the
/// descendant identified by `target_id`. Each intermediate widget on the
/// path from root to target receives the event.
///
/// The root widget (identified by `root_state`) receives the event first,
/// then the walk proceeds into its children.
///
/// Returns `true` if the target was found and the event was delivered.
fn dispatch_pointer_to_descendant(
    widget: &mut Box<dyn tench_ui::Widget>,
    root_state: &mut tench_ui::core::widget::WidgetState,
    global: &mut tench_ui::core::widget::GlobalState,
    target_id: WidgetId,
    event: &PointerEvent,
) -> bool {
    // Deliver to the root widget first so it can react to events in its subtree.
    {
        let mut ctx = EventCtx {
            state: root_state,
            global,
            anim_requested: false,
        };
        widget.on_pointer_event(&mut ctx, event);
    }

    dispatch_to_descendant_inner(widget, global, target_id, event)
}

/// Inner recursion: delivers the event to each child on the path toward
/// `target_id`.
fn dispatch_to_descendant_inner(
    widget: &mut Box<dyn tench_ui::Widget>,
    global: &mut tench_ui::core::widget::GlobalState,
    target_id: WidgetId,
    event: &PointerEvent,
) -> bool {
    let child_ids = widget.children();
    for child_id in child_ids {
        if let Some(child) = widget.child_mut(child_id) {
            // Deliver to this child.
            {
                let mut ctx = EventCtx {
                    state: &mut child.state,
                    global,
                    anim_requested: false,
                };
                child.widget.on_pointer_event(&mut ctx, event);
            }

            if child_id == target_id {
                return true;
            }

            // Recurse into this child's subtree.
            if dispatch_to_descendant_inner(&mut child.widget, global, target_id, event) {
                return true;
            }
        }
    }
    false
}
