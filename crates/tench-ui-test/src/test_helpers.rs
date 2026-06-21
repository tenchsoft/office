//! Convenience helpers for product E2E tests.
//!
//! These thin wrappers reduce boilerplate in product test files by combining
//! common `TestHarness` operations (capture, click, type, assert) into single
//! function calls. Product tests should `use tench_ui_test::test_helpers::*`
//! instead of copying these functions into every test file.

use tench_ui_automation_core::{
    find_node, UiAutomationAction, UiAutomationCapture, UiAutomationCaptureRequest,
    UiAutomationKey, UiAutomationModifiers, UiAutomationNode, UiAutomationSelector,
};

use crate::harness::TestHarness;

// ---------------------------------------------------------------------------
// Selector construction
// ---------------------------------------------------------------------------

/// Shorthand for `UiAutomationSelector::ByDebugId`.
pub fn selector(debug_id: &str) -> UiAutomationSelector {
    UiAutomationSelector::ByDebugId {
        debug_id: debug_id.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Capture helpers
// ---------------------------------------------------------------------------

/// Captures the current widget tree and PNG with both tree and image included.
pub fn capture(harness: &mut TestHarness) -> UiAutomationCapture {
    harness.automation_capture(UiAutomationCaptureRequest::default())
}

/// Decodes the PNG bytes from an automation capture into an `image::RgbaImage`.
pub fn decode_png(capture: &UiAutomationCapture) -> image::RgbaImage {
    image::load_from_memory(&capture.png_bytes)
        .expect("PNG decode")
        .into_rgba8()
}

/// Returns the root automation node from a capture.
pub fn tree(capture: &UiAutomationCapture) -> &UiAutomationNode {
    capture
        .ui_tree
        .as_ref()
        .expect("capture did not include UI tree")
}

// ---------------------------------------------------------------------------
// Interaction helpers
// ---------------------------------------------------------------------------

/// Clicks a `debug_id` selector and returns the post-click capture.
pub fn click(harness: &mut TestHarness, debug_id: &str) -> UiAutomationCapture {
    harness
        .automation_click_debug_id(debug_id)
        .unwrap_or_else(|e| panic!("click on '{debug_id}' failed: {e:?}"))
}

/// Types text into the currently focused widget (no selector targeting).
pub fn type_text(harness: &mut TestHarness, text: &str) -> UiAutomationCapture {
    for ev in crate::EventSimulator::type_text(text) {
        harness.dispatch_text(ev);
    }
    capture(harness)
}

/// Presses a key with optional modifiers and returns the post-event capture.
pub fn key(
    harness: &mut TestHarness,
    k: UiAutomationKey,
    mods: UiAutomationModifiers,
) -> UiAutomationCapture {
    harness
        .automation_action(tench_ui_automation_core::UiAutomationAction::KeyPress {
            key: k,
            modifiers: mods,
        })
        .expect("key press")
}

/// Presses a key combination (e.g., Ctrl+Shift+K) in a single call.
///
/// This is a semantic alias for [`key`] that makes modifier-chord calls
/// self-documenting in test code.
pub fn key_chord(
    harness: &mut TestHarness,
    k: UiAutomationKey,
    mods: UiAutomationModifiers,
) -> UiAutomationCapture {
    key(harness, k, mods)
}

// ---------------------------------------------------------------------------
// Assertion helpers (thin wrappers around automation module)
// ---------------------------------------------------------------------------

/// Asserts that a node with the given `debug_id` exists in the capture's UI tree.
pub fn assert_selector(capture: &UiAutomationCapture, debug_id: &str) {
    crate::assert_selector_present(capture, &selector(debug_id));
}

/// Asserts that no node with the given `debug_id` exists in the capture's UI tree.
pub fn assert_absent(capture: &UiAutomationCapture, debug_id: &str) {
    crate::assert_selector_absent(capture, &selector(debug_id));
}

/// Asserts that a node's bounds are fully inside the capture viewport.
pub fn assert_bounds_inside(capture: &UiAutomationCapture, debug_id: &str) {
    let node = crate::assert_selector_present(capture, &selector(debug_id));
    crate::assert_node_bounds_within_capture(capture, node);
}

/// Asserts that a rect is inside the given width/height.
pub fn assert_rect_inside(
    rect: &tench_ui_automation_core::UiAutomationRect,
    width: f64,
    height: f64,
    label: &str,
) {
    assert!(
        rect.x >= 0.0
            && rect.y >= 0.0
            && rect.x + rect.width <= width
            && rect.y + rect.height <= height,
        "{label} bounds {rect:?} exceed viewport ({width},{height})"
    );
}

// ---------------------------------------------------------------------------
// Menu helpers
// ---------------------------------------------------------------------------

/// Opens a menu by `debug_id`, then clicks the menu item by `debug_id`.
/// Returns the capture after the menu item click.
pub fn open_menu_item(
    harness: &mut TestHarness,
    menu_id: &str,
    item_id: &str,
) -> UiAutomationCapture {
    click(harness, menu_id);
    click(harness, item_id)
}

// ---------------------------------------------------------------------------
// Pointer / interaction helpers
// ---------------------------------------------------------------------------

/// Dispatches a pointer move event to the given position.
pub fn move_mouse(harness: &mut TestHarness, x: f64, y: f64) {
    let event = crate::EventSimulator::pointer_move(kurbo::Point::new(x, y), kurbo::Vec2::ZERO);
    harness.dispatch_pointer(event);
}

/// Hovers over a `debug_id` selector and returns the post-hover capture.
pub fn hover(harness: &mut TestHarness, debug_id: &str) -> UiAutomationCapture {
    harness
        .automation_action(UiAutomationAction::Hover {
            selector: selector(debug_id),
        })
        .unwrap_or_else(|e| panic!("hover on '{debug_id}' failed: {e:?}"))
}

/// Drags from one `debug_id` selector to another and returns the post-drag capture.
pub fn drag_from_to(harness: &mut TestHarness, from_id: &str, to_id: &str) -> UiAutomationCapture {
    harness
        .automation_action(UiAutomationAction::DragFromTo {
            from_selector: selector(from_id),
            to_selector: selector(to_id),
            steps: 8,
        })
        .unwrap_or_else(|e| panic!("drag_from_to '{from_id}' -> '{to_id}' failed: {e:?}"))
}

/// Right-clicks on a `debug_id` selector and returns the post-click capture.
pub fn right_click(harness: &mut TestHarness, debug_id: &str) -> UiAutomationCapture {
    harness
        .automation_action(UiAutomationAction::RightClick {
            selector: selector(debug_id),
            modifiers: UiAutomationModifiers::default(),
        })
        .unwrap_or_else(|e| panic!("right-click on '{debug_id}' failed: {e:?}"))
}

/// Dispatches a scroll event at the given position with the specified delta.
pub fn scroll_at(
    harness: &mut TestHarness,
    x: f64,
    y: f64,
    delta_x: f64,
    delta_y: f64,
) -> UiAutomationCapture {
    let event =
        crate::EventSimulator::scroll(kurbo::Point::new(x, y), kurbo::Vec2::new(delta_x, delta_y));
    harness.dispatch_pointer(event);
    capture(harness)
}

/// Clicks a `debug_id` selector with the Ctrl modifier held.
pub fn ctrl_click(harness: &mut TestHarness, debug_id: &str) -> UiAutomationCapture {
    click_with_modifiers(
        harness,
        debug_id,
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    )
}

/// Clicks a `debug_id` selector with arbitrary modifiers held.
pub fn click_with_modifiers(
    harness: &mut TestHarness,
    debug_id: &str,
    modifiers: UiAutomationModifiers,
) -> UiAutomationCapture {
    harness
        .automation_action(UiAutomationAction::Click {
            selector: selector(debug_id),
            modifiers,
        })
        .unwrap_or_else(|e| panic!("click_with_modifiers on '{debug_id}' failed: {e:?}"))
}

// ---------------------------------------------------------------------------
// Node label helpers
// ---------------------------------------------------------------------------

/// Returns the combined text representation of a node matching `debug_id`.
///
/// Checks the node's `value` first, then falls back to `label`. Returns an
/// empty string if the node exists but has neither field set, or panics if
/// the node is not found at all.
pub fn node_text(capture: &UiAutomationCapture, debug_id: &str) -> String {
    let tree = capture
        .ui_tree
        .as_ref()
        .unwrap_or_else(|| panic!("capture did not include UI tree"));
    let node = find_node(tree, &selector(debug_id))
        .unwrap_or_else(|| panic!("node '{debug_id}' not found in capture"));
    node.value
        .clone()
        .or_else(|| node.label.clone())
        .unwrap_or_default()
}

/// Returns the value text of a node matching `debug_id`, or `None` if not found.
pub fn get_node_value(capture: &UiAutomationCapture, debug_id: &str) -> Option<String> {
    let tree = capture.ui_tree.as_ref()?;
    let node = find_node(tree, &selector(debug_id))?;
    node.value.clone()
}

/// Returns the label text of a node matching `debug_id`, or `None` if not found.
pub fn get_node_label(capture: &UiAutomationCapture, debug_id: &str) -> Option<String> {
    let tree = capture.ui_tree.as_ref()?;
    let node = find_node(tree, &selector(debug_id))?;
    node.label.clone()
}

/// Asserts that a node's label exactly equals `expected`.
///
/// On failure, includes a truncated automation report for debugging.
pub fn assert_node_label(capture: &UiAutomationCapture, debug_id: &str, expected: &str) {
    let tree = capture
        .ui_tree
        .as_ref()
        .unwrap_or_else(|| panic!("capture did not include UI tree"));
    let node = find_node(tree, &selector(debug_id)).unwrap_or_else(|| {
        let report = tench_ui_automation_core::format_capture_report(capture);
        panic!(
            "node '{}' not found in capture\n{}",
            debug_id,
            crate::automation::truncate_report(&report, 4096)
        )
    });
    let label = node.label.as_deref().unwrap_or("");
    if label != expected {
        let report = tench_ui_automation_core::format_capture_report(capture);
        panic!(
            "label mismatch for '{debug_id}': expected {expected:?}, got {label:?}\n{}",
            crate::automation::truncate_report(&report, 4096)
        );
    }
}

/// Asserts that a node's label contains the `expected` substring.
///
/// On failure, includes a truncated automation report for debugging.
pub fn assert_node_label_contains(capture: &UiAutomationCapture, debug_id: &str, expected: &str) {
    let tree = capture
        .ui_tree
        .as_ref()
        .unwrap_or_else(|| panic!("capture did not include UI tree"));
    let node = find_node(tree, &selector(debug_id)).unwrap_or_else(|| {
        let report = tench_ui_automation_core::format_capture_report(capture);
        panic!(
            "node '{}' not found in capture\n{}",
            debug_id,
            crate::automation::truncate_report(&report, 4096)
        )
    });
    let label = node.label.as_deref().unwrap_or("");
    if !label.contains(expected) {
        let report = tench_ui_automation_core::format_capture_report(capture);
        panic!(
            "label {label:?} for '{debug_id}' does not contain {expected:?}\n{}",
            crate::automation::truncate_report(&report, 4096)
        );
    }
}

/// Asserts that a node's label changed between two captures.
pub fn assert_label_changed(
    before: &UiAutomationCapture,
    after: &UiAutomationCapture,
    debug_id: &str,
) {
    let before_label = get_node_label(before, debug_id);
    let after_label = get_node_label(after, debug_id);
    assert_ne!(
        before_label, after_label,
        "label for '{}' should have changed but remained {:?}",
        debug_id, before_label
    );
}
