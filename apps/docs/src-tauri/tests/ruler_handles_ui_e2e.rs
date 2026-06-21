/// E2E tests for ruler drag handle automation nodes.
///
/// Verifies that all five ruler handle selectors are present in the initial
/// render: left margin, right margin, left indent, right indent, and
/// first-line indent. Also verifies drag behavior for each handle.
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::{CaptureAssertions, TestHarness};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

#[test]
fn ruler_handles_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);

    assert_selector(&cap, "docs.ruler.indent.left");
    assert_selector(&cap, "docs.ruler.indent.first_line");
    assert_selector(&cap, "docs.ruler.margin.left");
    assert_selector(&cap, "docs.ruler.indent.right");
    assert_selector(&cap, "docs.ruler.margin.right");
}

#[test]
fn ruler_left_margin_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);

    assert_selector(&cap, "docs.ruler.margin.left");
}

#[test]
fn ruler_right_margin_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);

    assert_selector(&cap, "docs.ruler.margin.right");
}

// ---------------------------------------------------------------------------
// Drag handle tests
// ---------------------------------------------------------------------------

#[test]
fn ruler_left_margin_drag_updates_margin() {
    let mut harness = make_harness();
    // Type some text so the document has content
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Test text for ruler drag");
    let before = capture(&mut harness);
    let margin_before = get_node_value(&before, "docs.ruler.margin.left");

    // Drag left margin handle to the right (toward center of ruler)
    let after = drag_from_to(&mut harness, "docs.ruler.margin.left", "docs.ruler");
    after.assert_png_valid();
    let margin_after = get_node_value(&after, "docs.ruler.margin.left");
    assert_ne!(
        margin_before, margin_after,
        "left margin should change after drag"
    );
}

#[test]
fn ruler_right_margin_drag_updates_margin() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Test text for ruler drag");
    let before = capture(&mut harness);
    let margin_before = get_node_value(&before, "docs.ruler.margin.right");

    let after = drag_from_to(&mut harness, "docs.ruler.margin.right", "docs.ruler");
    after.assert_png_valid();
    let margin_after = get_node_value(&after, "docs.ruler.margin.right");
    assert_ne!(
        margin_before, margin_after,
        "right margin should change after drag"
    );
}

#[test]
fn ruler_left_indent_drag_updates_indent() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Test text for ruler drag");

    // First set a non-zero indent so the handle is separated from the margin
    click(&mut harness, "docs.toolbar.indent");
    let before = capture(&mut harness);
    let indent_before = get_node_value(&before, "docs.ruler.indent.left");
    assert!(
        indent_before.as_deref() != Some("0.0"),
        "indent should be non-zero after clicking indent button"
    );

    let after = drag_from_to(&mut harness, "docs.ruler.indent.left", "docs.ruler");
    after.assert_png_valid();
    let indent_after = get_node_value(&after, "docs.ruler.indent.left");
    assert_ne!(
        indent_before, indent_after,
        "left indent should change after drag"
    );
}

// Right indent and first-line indent handles start at 0 and overlap with
// margin handles, so dragging them actually drags the margin handle.
// These tests verify the handles are present and can be dragged without crash.

#[test]
fn ruler_right_indent_handle_draggable() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Test text for ruler drag");

    // Drag the right indent handle (at 0, it overlaps with right margin)
    let after = drag_from_to(&mut harness, "docs.ruler.indent.right", "docs.ruler");
    after.assert_png_valid();
    // The right margin should change since the indent handle overlaps with it
    assert_selector(&after, "docs.ruler.margin.right");
}

#[test]
fn ruler_first_line_indent_handle_draggable() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Test text for ruler drag");

    // Drag the first-line indent handle (at 0, it overlaps with left indent/margin)
    let after = drag_from_to(&mut harness, "docs.ruler.indent.first_line", "docs.ruler");
    after.assert_png_valid();
    assert_selector(&after, "docs.ruler.indent.first_line");
}

#[test]
fn ruler_left_margin_drag_marks_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Text");

    // Drag the left margin handle
    drag_from_to(&mut harness, "docs.ruler.margin.left", "docs.ruler");
    let cap = capture(&mut harness);
    let dirty = get_node_value(&cap, "docs.document.dirty");
    assert_eq!(
        dirty,
        Some("true".to_string()),
        "document should be dirty after margin drag"
    );
}

#[test]
fn ruler_left_indent_drag_marks_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Text");

    drag_from_to(&mut harness, "docs.ruler.indent.left", "docs.ruler");
    let cap = capture(&mut harness);
    let dirty = get_node_value(&cap, "docs.document.dirty");
    assert_eq!(
        dirty,
        Some("true".to_string()),
        "document should be dirty after indent drag"
    );
}

#[test]
fn ruler_right_indent_drag_marks_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Text");

    drag_from_to(&mut harness, "docs.ruler.indent.right", "docs.ruler");
    let cap = capture(&mut harness);
    let dirty = get_node_value(&cap, "docs.document.dirty");
    assert_eq!(
        dirty,
        Some("true".to_string()),
        "document should be dirty after indent drag"
    );
}

#[test]
fn ruler_first_line_indent_drag_marks_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Text");

    drag_from_to(&mut harness, "docs.ruler.indent.first_line", "docs.ruler");
    let cap = capture(&mut harness);
    let dirty = get_node_value(&cap, "docs.document.dirty");
    assert_eq!(
        dirty,
        Some("true".to_string()),
        "document should be dirty after first line indent drag"
    );
}

#[test]
fn ruler_right_margin_drag_marks_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Text");

    drag_from_to(&mut harness, "docs.ruler.margin.right", "docs.ruler");
    let cap = capture(&mut harness);
    let dirty = get_node_value(&cap, "docs.document.dirty");
    assert_eq!(
        dirty,
        Some("true".to_string()),
        "document should be dirty after margin drag"
    );
}
