/// UI automation tests for docs header/footer editing fields.
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::{assert_capture_changed, CaptureAssertions, TestHarness};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

#[test]
fn header_editing_field_appears() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");
    assert_selector(&after, "docs.header_field");
}

#[test]
fn footer_editing_field_appears() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.footer");
    assert_selector(&after, "docs.footer_field");
}

#[test]
fn header_editing_accepts_text_and_reflects_value() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");
    let before = capture(&mut harness);
    let after = type_text(&mut harness, "My Header");
    assert_capture_changed(&before, &after);
    // Verify the typed text appears in the header field value
    let value = get_node_value(&after, "docs.header_field");
    assert_eq!(
        value,
        Some("My Header".to_string()),
        "header field value should reflect typed text"
    );
}

#[test]
fn footer_editing_accepts_text_and_reflects_value() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.footer");
    let before = capture(&mut harness);
    let after = type_text(&mut harness, "Page 1");
    assert_capture_changed(&before, &after);
    // Verify the typed text appears in the footer field value
    let value = get_node_value(&after, "docs.footer_field");
    assert_eq!(
        value,
        Some("Page 1".to_string()),
        "footer field value should reflect typed text"
    );
}

// ---------------------------------------------------------------------------
// Header editing: Enter commits and exits
// ---------------------------------------------------------------------------

#[test]
fn header_editing_enter_commits_and_exits() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");
    type_text(&mut harness, "Committed Header");

    // Press Enter to commit
    let after = key(
        &mut harness,
        tench_ui_automation_core::UiAutomationKey::Enter,
        tench_ui_automation_core::UiAutomationModifiers::default(),
    );
    after.assert_png_valid();

    // Header field should no longer be in editing mode
    assert_absent(&after, "docs.header_field");
}

#[test]
fn header_editing_escape_commits_and_exits() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");
    type_text(&mut harness, "Escape Header");

    // Press Escape to commit
    let after = key(
        &mut harness,
        tench_ui_automation_core::UiAutomationKey::Escape,
        tench_ui_automation_core::UiAutomationModifiers::default(),
    );
    after.assert_png_valid();

    // Header field should no longer be in editing mode
    assert_absent(&after, "docs.header_field");
}

#[test]
fn header_editing_preserves_document_text() {
    let mut harness = make_harness();
    // Type document text first
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Body text");
    let doc_text_before = get_node_value(&capture(&mut harness), "docs.document.text");

    // Open header editing
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");
    type_text(&mut harness, "Header");

    // Commit with Enter
    key(
        &mut harness,
        tench_ui_automation_core::UiAutomationKey::Enter,
        tench_ui_automation_core::UiAutomationModifiers::default(),
    );

    // Document text should be preserved
    let after = capture(&mut harness);
    let doc_text_after = get_node_value(&after, "docs.document.text");
    assert_eq!(
        doc_text_before, doc_text_after,
        "document text should be preserved after header editing"
    );
}

#[test]
fn footer_editing_enter_commits_and_exits() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.footer");
    type_text(&mut harness, "Committed Footer");

    let after = key(
        &mut harness,
        tench_ui_automation_core::UiAutomationKey::Enter,
        tench_ui_automation_core::UiAutomationModifiers::default(),
    );
    after.assert_png_valid();
    assert_absent(&after, "docs.footer_field");
}

#[test]
fn footer_editing_preserves_document_text() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Body text");
    let doc_text_before = get_node_value(&capture(&mut harness), "docs.document.text");

    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.footer");
    type_text(&mut harness, "Footer");

    key(
        &mut harness,
        tench_ui_automation_core::UiAutomationKey::Enter,
        tench_ui_automation_core::UiAutomationModifiers::default(),
    );

    let after = capture(&mut harness);
    let doc_text_after = get_node_value(&after, "docs.document.text");
    assert_eq!(
        doc_text_before, doc_text_after,
        "document text should be preserved after footer editing"
    );
}

#[test]
fn header_editing_backspace_works() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");
    type_text(&mut harness, "Hello");

    // Press backspace to remove last character
    let after = key(
        &mut harness,
        tench_ui_automation_core::UiAutomationKey::Backspace,
        tench_ui_automation_core::UiAutomationModifiers::default(),
    );
    after.assert_png_valid();

    // Header text should be "Hell"
    let value = get_node_value(&after, "docs.header_field");
    assert_eq!(
        value,
        Some("Hell".to_string()),
        "header field should show 'Hell' after backspace"
    );
}
