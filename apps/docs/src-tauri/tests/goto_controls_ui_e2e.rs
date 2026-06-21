/// UI automation tests for docs Go To modal controls.
use tench_docs_lib::ui::DocsApp;
use tench_ui_automation_core::UiAutomationKey;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::CaptureAssertions;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

#[test]
fn goto_modal_has_page_and_line_buttons() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");
    assert_selector(&after, "docs.goto.page_mode");
    assert_selector(&after, "docs.goto.line_mode");
    assert_selector(&after, "docs.goto.input");
}

#[test]
fn goto_page_mode_button_toggles_label() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");

    // Click page mode button — verify label stays "Page"
    let after = click(&mut harness, "docs.goto.page_mode");
    assert_node_label(&after, "docs.goto.page_mode", "Page");
    after.assert_png_valid();
}

#[test]
fn goto_line_mode_button_changes_mode() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");

    // Click line mode — label should be "Line"
    let after = click(&mut harness, "docs.goto.line_mode");
    assert_node_label(&after, "docs.goto.line_mode", "Line");
    after.assert_png_valid();
}

#[test]
fn goto_mode_roundtrip_page_to_line_and_back() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");

    // Start on page mode
    let page = capture(&mut harness);
    assert_node_label(&page, "docs.goto.page_mode", "Page");

    // Switch to line mode
    let line = click(&mut harness, "docs.goto.line_mode");
    assert_node_label(&line, "docs.goto.line_mode", "Line");

    // Switch back to page mode
    let back = click(&mut harness, "docs.goto.page_mode");
    assert_node_label(&back, "docs.goto.page_mode", "Page");
}

// ── Goto input field tests ──

#[test]
fn goto_input_field_starts_empty() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");
    let cap = capture(&mut harness);
    assert_eq!(
        get_node_value(&cap, "docs.goto.input"),
        Some("".to_string())
    );
}

#[test]
fn goto_input_field_reflects_typed_text() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");
    let after = type_text(&mut harness, "5");
    assert_eq!(
        get_node_value(&after, "docs.goto.input"),
        Some("5".to_string())
    );
    after.assert_png_valid();
}

#[test]
fn goto_input_field_typing_does_not_edit_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Doc");
    let before = capture(&mut harness);
    let doc_before = get_node_label(&before, "docs.document").unwrap_or_default();

    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");
    type_text(&mut harness, "42");

    let after = capture(&mut harness);
    let doc_after = get_node_label(&after, "docs.document").unwrap_or_default();
    assert_eq!(
        doc_before, doc_after,
        "typing in goto modal must not edit document"
    );
}

// ── Goto mode selected state tests ──

#[test]
fn goto_page_mode_selected_by_default() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");
    let cap = capture(&mut harness);
    assert_eq!(
        get_node_value(&cap, "docs.goto.page_mode"),
        Some("selected".to_string())
    );
    assert_eq!(
        get_node_value(&cap, "docs.goto.line_mode"),
        Some("unselected".to_string())
    );
    assert_eq!(
        get_node_value(&cap, "docs.goto.mode"),
        Some("page".to_string())
    );
}

#[test]
fn goto_line_mode_selected_after_click() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");
    let after = click(&mut harness, "docs.goto.line_mode");
    assert_eq!(
        get_node_value(&after, "docs.goto.page_mode"),
        Some("unselected".to_string())
    );
    assert_eq!(
        get_node_value(&after, "docs.goto.line_mode"),
        Some("selected".to_string())
    );
    assert_eq!(
        get_node_value(&after, "docs.goto.mode"),
        Some("line".to_string())
    );
}

#[test]
fn goto_escape_closes_modal() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");
    assert_selector(&capture(&mut harness), "docs.modal.goto");

    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());
    assert_absent(&after, "docs.modal.goto");
}
