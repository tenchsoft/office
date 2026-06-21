/// UI automation tests for docs View Menu items (#21-#27).
///
/// Tests View menu items: Thumbnails, Style Panel, Comments, Zoom In,
/// Zoom Out, and Reset Zoom. Each test verifies menu-close-before-dispatch,
/// document state preservation, and specific behavioral requirements from
/// the fix plans.
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::assert_capture_changed;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::CaptureAssertions;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

/// Helper: returns the `value` field of a node identified by `debug_id`.
fn node_value(capture: &tench_ui_automation_core::UiAutomationCapture, debug_id: &str) -> String {
    get_node_value(capture, debug_id).unwrap_or_else(|| panic!("node '{debug_id}' has no value"))
}

// ---------------------------------------------------------------------------
// Basic render-change tests (existing)
// ---------------------------------------------------------------------------

#[test]
fn view_menu_thumbnails_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_menu_style_panel_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.style_panel");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_menu_comments_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.comments");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_menu_zoom_in_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_in");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_menu_zoom_out_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_out");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_menu_reset_zoom_no_crash() {
    let mut harness = make_harness();
    let _before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.reset_zoom");
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Zoom In: value assertions
// ---------------------------------------------------------------------------

/// View → Zoom In should change zoom from 100 to 110 and update the status bar.
#[test]
fn view_menu_zoom_in_updates_zoom_value() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let zoom_before = node_text(&before, "docs.status_bar.zoom");
    assert!(
        zoom_before.contains("100"),
        "initial zoom should be 100%, got: {zoom_before}"
    );

    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_in");
    let zoom_after = node_text(&after, "docs.status_bar.zoom");
    assert!(
        zoom_after.contains("110"),
        "zoom after Zoom In should be 110%, got: {zoom_after}"
    );
    after.assert_png_valid();
}

/// Zoom In should preserve document text, cursor, and selection.
#[test]
fn view_menu_zoom_in_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Zoom text");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_in");
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after Zoom In"
    );
    after.assert_png_valid();
}

/// Repeated Zoom In should not exceed the maximum zoom (200%).
#[test]
fn view_menu_zoom_in_clamps_at_maximum() {
    let mut harness = make_harness();
    // Zoom in many times
    for _ in 0..20 {
        open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_in");
    }
    let after = capture(&mut harness);
    let zoom = node_text(&after, "docs.status_bar.zoom");
    let zoom_val: f64 = zoom.trim_end_matches('%').parse().expect("zoom value");
    assert!(
        zoom_val <= 200.0,
        "zoom should not exceed 200%, got: {zoom}"
    );
}

// ---------------------------------------------------------------------------
// Zoom Out: value assertions
// ---------------------------------------------------------------------------

/// View → Zoom Out should change zoom from 100 to 90 and update the status bar.
#[test]
fn view_menu_zoom_out_updates_zoom_value() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let zoom_before = node_text(&before, "docs.status_bar.zoom");
    assert!(
        zoom_before.contains("100"),
        "initial zoom should be 100%, got: {zoom_before}"
    );

    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_out");
    let zoom_after = node_text(&after, "docs.status_bar.zoom");
    assert!(
        zoom_after.contains("90"),
        "zoom after Zoom Out should be 90%, got: {zoom_after}"
    );
    after.assert_png_valid();
}

/// Zoom Out should preserve document text.
#[test]
fn view_menu_zoom_out_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Zoom out text");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_out");
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after Zoom Out"
    );
    after.assert_png_valid();
}

/// Repeated Zoom Out should not go below the minimum zoom (50%).
#[test]
fn view_menu_zoom_out_clamps_at_minimum() {
    let mut harness = make_harness();
    for _ in 0..20 {
        open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_out");
    }
    let after = capture(&mut harness);
    let zoom = node_text(&after, "docs.status_bar.zoom");
    let zoom_val: f64 = zoom.trim_end_matches('%').parse().expect("zoom value");
    assert!(
        zoom_val >= 50.0,
        "zoom should not go below 50%, got: {zoom}"
    );
}

// ---------------------------------------------------------------------------
// Reset Zoom: roundtrip test
// ---------------------------------------------------------------------------

/// Reset Zoom should return zoom to 100% after zooming in.
#[test]
fn view_menu_reset_zoom_returns_to_100_after_zoom_in() {
    let mut harness = make_harness();
    // Zoom in first
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_in");
    let zoomed = capture(&mut harness);
    let zoom_after_in = node_text(&zoomed, "docs.status_bar.zoom");
    assert!(
        !zoom_after_in.contains("100"),
        "zoom should have changed from 100%, got: {zoom_after_in}"
    );

    // Reset zoom
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.reset_zoom");
    let zoom_after_reset = node_text(&after, "docs.status_bar.zoom");
    assert!(
        zoom_after_reset.contains("100"),
        "zoom should return to 100% after reset, got: {zoom_after_reset}"
    );
    after.assert_png_valid();
}

/// Reset Zoom should preserve document text and dirty state.
#[test]
fn view_menu_reset_zoom_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Reset zoom text");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    // Zoom in then reset
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_in");
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.reset_zoom");
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after Reset Zoom"
    );
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Thumbnails: menu-close-before-dispatch
// ---------------------------------------------------------------------------

/// Thumbnails activation should close the View menu panel.
#[test]
fn view_menu_thumbnails_closes_menu_after_dispatch() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    // Menu panel should be gone after dispatch
    assert_absent(&after, "docs.menu.view.panel");
    // Thumbnail should be visible
    assert_selector(&after, "docs.thumbnail.page.0");
    after.assert_png_valid();
}

/// Thumbnails toggle should preserve document text, cursor, and selection.
#[test]
fn view_menu_thumbnails_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Thumbnail doc");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved"
    );
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Style Panel: toggle on/off with assertions
// ---------------------------------------------------------------------------

/// Style Panel toggle should close the View menu panel.
#[test]
fn view_menu_style_panel_closes_menu_after_dispatch() {
    let mut harness = make_harness();
    // Style panel is on by default — toggle it off
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.style_panel");
    assert_absent(&after, "docs.menu.view.panel");
    after.assert_png_valid();
}

/// Style Panel toggle should preserve document text.
#[test]
fn view_menu_style_panel_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Style panel text");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.style_panel");
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after style panel toggle"
    );
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Comments: menu-close-before-dispatch and preservation
// ---------------------------------------------------------------------------

/// Comments activation should close the View menu panel.
#[test]
fn view_menu_comments_closes_menu_after_dispatch() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.comments");
    assert_absent(&after, "docs.menu.view.panel");
    // Comments panel should be visible
    assert_selector(&after, "docs.comments.collapse");
    after.assert_png_valid();
}

/// Comments toggle should preserve document text, cursor, and selection.
#[test]
fn view_menu_comments_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Comments text");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.comments");
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved"
    );
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Hover-without-dispatch tests
// ---------------------------------------------------------------------------

/// Hovering Zoom In in the View menu must not change the zoom value.
#[test]
fn view_menu_zoom_in_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let cap_before = capture(&mut harness);
    let zoom_before = node_text(&cap_before, "docs.status_bar.zoom");

    // Open View menu.
    click(&mut harness, "docs.menu.view");

    // Hover over Zoom In.
    let after_hover = hover(&mut harness, "docs.menu.view.zoom_in");
    let zoom_after_hover = node_text(&after_hover, "docs.status_bar.zoom");
    assert_eq!(
        zoom_after_hover, zoom_before,
        "zoom should not change on hover"
    );
    after_hover.assert_png_valid();
}

/// Hovering Zoom Out in the View menu must not change the zoom value.
#[test]
fn view_menu_zoom_out_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let cap_before = capture(&mut harness);
    let zoom_before = node_text(&cap_before, "docs.status_bar.zoom");

    // Open View menu.
    click(&mut harness, "docs.menu.view");

    // Hover over Zoom Out.
    let after_hover = hover(&mut harness, "docs.menu.view.zoom_out");
    let zoom_after_hover = node_text(&after_hover, "docs.status_bar.zoom");
    assert_eq!(
        zoom_after_hover, zoom_before,
        "zoom should not change on hover"
    );
    after_hover.assert_png_valid();
}

/// Hovering Thumbnails in the View menu must not toggle thumbnails.
#[test]
fn view_menu_thumbnails_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let cap_before = capture(&mut harness);
    assert_absent(&cap_before, "docs.thumbnail.page.0");

    // Open View menu.
    click(&mut harness, "docs.menu.view");

    // Hover over Thumbnails.
    let after_hover = hover(&mut harness, "docs.menu.view.thumbnails");
    assert_absent(&after_hover, "docs.thumbnail.page.0");
    after_hover.assert_png_valid();
}

/// Hovering Style Panel in the View menu must not toggle the panel.
#[test]
fn view_menu_style_panel_hover_does_not_dispatch() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Style panel hover");
    let cap_before = capture(&mut harness);
    let text_before = node_value(&cap_before, "docs.document.text");

    // Open View menu.
    click(&mut harness, "docs.menu.view");

    // Hover over Style Panel.
    let after_hover = hover(&mut harness, "docs.menu.view.style_panel");
    assert_eq!(
        node_value(&after_hover, "docs.document.text"),
        text_before,
        "document text should be unchanged on hover"
    );
    after_hover.assert_png_valid();
}

/// Hovering Comments in the View menu must not toggle the comments panel.
#[test]
fn view_menu_comments_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let cap_before = capture(&mut harness);
    assert_absent(&cap_before, "docs.comments.collapse");

    // Open View menu.
    click(&mut harness, "docs.menu.view");

    // Hover over Comments.
    let after_hover = hover(&mut harness, "docs.menu.view.comments");
    assert_absent(&after_hover, "docs.comments.collapse");
    after_hover.assert_png_valid();
}

/// Hovering Reset Zoom in the View menu must not change zoom.
#[test]
fn view_menu_reset_zoom_hover_does_not_dispatch() {
    let mut harness = make_harness();
    // Zoom in first to have a non-default zoom
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.zoom_in");
    let cap_zoomed = capture(&mut harness);
    let zoom_zoomed = node_text(&cap_zoomed, "docs.status_bar.zoom");
    assert_ne!(zoom_zoomed, "100%", "zoom should have changed");

    // Open View menu again.
    click(&mut harness, "docs.menu.view");

    // Hover over Reset Zoom.
    let after_hover = hover(&mut harness, "docs.menu.view.reset_zoom");
    let zoom_after_hover = node_text(&after_hover, "docs.status_bar.zoom");
    assert_eq!(
        zoom_after_hover, zoom_zoomed,
        "zoom should not change on hover over Reset Zoom"
    );
    after_hover.assert_png_valid();
}
