use tench_docs_lib::ui::DocsApp;
use tench_ui_automation_core::{UiAutomationCapture, UiAutomationSelector};
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::{
    assert_capture_changed, assert_capture_nonblank, assert_capture_png_valid, TestHarness,
};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

fn node_text(capture: &UiAutomationCapture, debug_id: &str) -> String {
    capture
        .find_node(&UiAutomationSelector::debug_id(debug_id))
        .and_then(|node| node.value.as_deref().or(node.label.as_deref()))
        .expect("node text")
        .to_string()
}

fn hover_over(harness: &mut TestHarness, debug_id: &str) -> UiAutomationCapture {
    let center = harness
        .automation_center(&UiAutomationSelector::debug_id(debug_id))
        .expect("automation center");
    move_mouse(harness, center.x, center.y);
    capture(harness)
}

#[test]
fn edit_menu_opens_with_expected_items() {
    let mut harness = make_harness();
    let before = capture(&mut harness);

    let after = click(&mut harness, "docs.menu.edit");

    assert_capture_changed(&before, &after);
    assert_capture_png_valid(&after);
    assert_capture_nonblank(&after);
    assert_eq!(node_text(&after, "docs.menu.active"), "Edit");
    assert_selector(&after, "docs.menu.edit.panel");
    for item in [
        "undo",
        "redo",
        "cut",
        "copy",
        "paste",
        "select_all",
        "find",
        "replace",
        "go_to",
    ] {
        assert_selector(&after, &format!("docs.menu.edit.{item}"));
    }
}

#[test]
fn edit_menu_find_click_closes_menu_before_dispatching_modal() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.edit");
    let after = click(&mut harness, "docs.menu.edit.find");

    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_selector(&after, "docs.modal.find_replace");
    assert_absent(&after, "docs.menu.edit.panel");
    assert_absent(&after, "docs.menu.edit.find");
}

#[test]
fn hovering_edit_items_does_not_dispatch_or_mutate_document() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "Body");
    let original_text = node_text(&typed, "docs.document.text");
    let original_cursor = node_text(&typed, "docs.document.cursor");

    click(&mut harness, "docs.menu.edit");
    let hovered = hover_over(&mut harness, "docs.menu.edit.find");

    assert_eq!(node_text(&hovered, "docs.menu.active"), "Edit");
    assert_eq!(node_text(&hovered, "docs.menu.edit.hovered"), "Find");
    assert_absent(&hovered, "docs.modal.find_replace");
    assert_eq!(node_text(&hovered, "docs.document.text"), original_text);
    assert_eq!(node_text(&hovered, "docs.document.cursor"), original_cursor);
}

#[test]
fn outside_click_closes_edit_menu_without_changing_document_state() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "Body");
    let original_text = node_text(&typed, "docs.document.text");
    let original_cursor = node_text(&typed, "docs.document.cursor");
    let original_selection = node_text(&typed, "docs.document.selection");

    let open = click(&mut harness, "docs.menu.edit");
    assert_selector(&open, "docs.menu.edit.panel");

    let after = click(&mut harness, "docs.document");

    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.edit.panel");
    assert_eq!(node_text(&after, "docs.document.text"), original_text);
    assert_eq!(node_text(&after, "docs.document.cursor"), original_cursor);
    assert_eq!(
        node_text(&after, "docs.document.selection"),
        original_selection
    );
}

#[test]
fn blank_menu_bar_click_closes_edit_menu_without_dispatch() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.edit");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.menu.edit.panel");

    harness
        .automation_action(tench_ui_automation_core::UiAutomationAction::Click {
            selector: UiAutomationSelector::point(500.0, 12.0),
            modifiers: tench_ui_automation_core::UiAutomationModifiers::default(),
        })
        .expect("blank menu bar click");
    let after = capture(&mut harness);

    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.edit.panel");
    assert_absent(&after, "docs.modal.find_replace");
}

#[test]
fn switching_from_file_to_edit_clears_previous_hover() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.file");
    let file_hovered = hover_over(&mut harness, "docs.menu.file.version_history");
    assert_eq!(
        node_text(&file_hovered, "docs.menu.file.hovered"),
        "Version History"
    );

    let edit_open = click(&mut harness, "docs.menu.edit");

    assert_eq!(node_text(&edit_open, "docs.menu.active"), "Edit");
    assert_absent(&edit_open, "docs.menu.file.panel");
    assert_eq!(node_text(&edit_open, "docs.menu.edit.hovered"), "none");
}

#[test]
fn moving_below_edit_menu_clears_hover_state() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.edit");
    let hovered = hover_over(&mut harness, "docs.menu.edit.find");
    assert_eq!(node_text(&hovered, "docs.menu.edit.hovered"), "Find");

    move_mouse(&mut harness, 40.0, 500.0);
    let after = capture(&mut harness);

    assert_eq!(node_text(&after, "docs.menu.active"), "Edit");
    assert_eq!(node_text(&after, "docs.menu.edit.hovered"), "none");
}

#[test]
fn repeated_open_and_outside_close_leaves_no_stale_menu_nodes() {
    let mut harness = make_harness();

    for _ in 0..3 {
        let open = click(&mut harness, "docs.menu.edit");
        assert_eq!(node_text(&open, "docs.menu.active"), "Edit");
        assert_selector(&open, "docs.menu.edit.panel");

        let closed = click(&mut harness, "docs.document");
        assert_eq!(node_text(&closed, "docs.menu.active"), "none");
        assert_absent(&closed, "docs.menu.edit.panel");
        assert_absent(&closed, "docs.menu.edit.undo");
    }
}
