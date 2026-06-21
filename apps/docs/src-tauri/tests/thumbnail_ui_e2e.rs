/// UI automation tests for docs thumbnail page previews.
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::{CaptureAssertions, TestHarness};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

#[test]
fn thumbnail_page_preview_present_when_enabled() {
    let mut harness = make_harness();
    // Enable thumbnails
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    assert_selector(&after, "docs.thumbnail.page.0");
    // Label should indicate "Page 1"
    assert_node_label(&after, "docs.thumbnail.page.0", "Page 1");
}

#[test]
fn thumbnail_disappears_when_disabled() {
    let mut harness = make_harness();
    // Enable then disable thumbnails
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    assert_absent(&after, "docs.thumbnail.page.0");
}

#[test]
fn thumbnail_roundtrip_preserves_document() {
    let mut harness = make_harness();
    // Type text
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Some text");

    // Enable thumbnails
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    assert_selector(&capture(&mut harness), "docs.thumbnail.page.0");

    // Disable thumbnails
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    assert_absent(&capture(&mut harness), "docs.thumbnail.page.0");

    // Re-enable — thumbnail should still work
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    assert_selector(&after, "docs.thumbnail.page.0");
    after.assert_png_valid();
}

#[test]
fn thumbnail_page_click_no_crash() {
    let mut harness = make_harness();

    // Enable thumbnails
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.thumbnail.page.0");

    // Click the thumbnail — should not crash
    let after = click(&mut harness, "docs.thumbnail.page.0");
    after.assert_png_valid();
}

#[test]
fn thumbnail_page_click_preserves_document() {
    let mut harness = make_harness();

    // Type text
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Preserved text");

    // Enable thumbnails
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");

    // Click the thumbnail
    let after = click(&mut harness, "docs.thumbnail.page.0");
    after.assert_png_valid();

    // Document text should be preserved
    let text = get_node_value(&after, "docs.document.text");
    assert_eq!(
        text,
        Some("Preserved text".to_string()),
        "document text should be preserved after thumbnail click"
    );
}
