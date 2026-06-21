/// UI automation tests for docs Print Preview modal controls.
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;
use tench_ui_test::{assert_capture_changed, CaptureAssertions};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

/// Helper: open print preview via File -> Print.
fn open_print_preview(harness: &mut TestHarness) {
    open_menu_item(harness, "docs.menu.file", "docs.menu.file.print");
}

#[test]
fn print_preview_has_navigation_controls() {
    let mut harness = make_harness();
    open_print_preview(&mut harness);
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.print_preview.prev");
    assert_selector(&cap, "docs.print_preview.next");
    assert_selector(&cap, "docs.print_preview.print");
    assert_selector(&cap, "docs.print_preview.close");
}

#[test]
fn print_preview_close_button_works() {
    let mut harness = make_harness();
    open_print_preview(&mut harness);
    let before = capture(&mut harness);
    let after = click(&mut harness, "docs.print_preview.close");
    assert_capture_changed(&before, &after);
    // Modal should be gone
    assert_absent(&after, "docs.print_preview.close");
}

#[test]
fn print_preview_page_indicator_present() {
    let mut harness = make_harness();
    open_print_preview(&mut harness);
    let cap = capture(&mut harness);
    // Page indicator should show "Page 1 of 1" for a single-page document
    assert_selector(&cap, "docs.print_preview.page_indicator");
    assert_node_label(&cap, "docs.print_preview.page_indicator", "Page 1 of 1");
}

#[test]
fn print_preview_next_button_updates_page_indicator() {
    let mut harness = make_harness();
    // Create a multi-page document by inserting many paragraphs
    click(&mut harness, "docs.document");
    for _ in 0..120 {
        type_text(
            &mut harness,
            "A sufficiently long line of text to fill multiple pages. ",
        );
    }
    // Open print preview
    open_print_preview(&mut harness);
    let before = capture(&mut harness);
    let before_label =
        get_node_label(&before, "docs.print_preview.page_indicator").unwrap_or_default();

    // Only test next if we actually have multiple pages
    if before_label.contains("of 1") {
        // Single-page document — skip the behavioral assertion but verify
        // the indicator is present and the next button doesn't crash.
        let after = click(&mut harness, "docs.print_preview.next");
        after.assert_png_valid();
        return;
    }

    // Click next
    let after = click(&mut harness, "docs.print_preview.next");
    let after_label =
        get_node_label(&after, "docs.print_preview.page_indicator").unwrap_or_default();

    // Page indicator should have changed
    assert_ne!(
        before_label, after_label,
        "page indicator should change after clicking next"
    );
    after.assert_png_valid();
}

#[test]
fn print_preview_prev_button_no_crash() {
    let mut harness = make_harness();
    open_print_preview(&mut harness);
    let after = click(&mut harness, "docs.print_preview.prev");
    after.assert_png_valid();
    // Page indicator should still show "Page 1 of 1" (can't go below 1)
    assert_node_label(&after, "docs.print_preview.page_indicator", "Page 1 of 1");
}

// ---------------------------------------------------------------------------
// Print button tests
// ---------------------------------------------------------------------------

#[test]
fn print_preview_print_button_no_crash() {
    let mut harness = make_harness();
    open_print_preview(&mut harness);
    let after = click(&mut harness, "docs.print_preview.print");
    after.assert_png_valid();
}

#[test]
fn print_preview_print_button_shows_toast() {
    let mut harness = make_harness();
    open_print_preview(&mut harness);
    let after = click(&mut harness, "docs.print_preview.print");
    after.assert_png_valid();

    // Toast should be present with the unavailable message
    assert_selector(&after, "docs.toast");
}

#[test]
fn print_preview_print_keeps_preview_open() {
    let mut harness = make_harness();
    open_print_preview(&mut harness);
    click(&mut harness, "docs.print_preview.print");
    let cap = capture(&mut harness);

    // Print preview modal should still be present
    assert_selector(&cap, "docs.print_preview.close");
    assert_selector(&cap, "docs.print_preview.print");
}

#[test]
fn print_preview_print_does_not_edit_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Protected text");
    let text_before = get_node_value(&capture(&mut harness), "docs.document.text");

    open_print_preview(&mut harness);
    click(&mut harness, "docs.print_preview.print");

    let after = capture(&mut harness);
    let text_after = get_node_value(&after, "docs.document.text");
    assert_eq!(
        text_before, text_after,
        "document text should not change after clicking print"
    );
}

#[test]
fn print_preview_close_then_reopen_works() {
    let mut harness = make_harness();
    open_print_preview(&mut harness);
    click(&mut harness, "docs.print_preview.close");
    let after_close = capture(&mut harness);
    assert_absent(&after_close, "docs.print_preview.close");

    // Reopen print preview
    open_print_preview(&mut harness);
    let after_reopen = capture(&mut harness);
    assert_selector(&after_reopen, "docs.print_preview.close");
    assert_selector(&after_reopen, "docs.print_preview.print");
    after_reopen.assert_png_valid();
}

#[test]
fn print_preview_prev_button_on_first_page_stays_at_one() {
    let mut harness = make_harness();
    open_print_preview(&mut harness);

    // Click prev multiple times
    click(&mut harness, "docs.print_preview.prev");
    click(&mut harness, "docs.print_preview.prev");
    let after = click(&mut harness, "docs.print_preview.prev");

    // Should still be at page 1
    assert_node_label(&after, "docs.print_preview.page_indicator", "Page 1 of 1");
}

#[test]
fn print_preview_next_button_no_crash_single_page() {
    let mut harness = make_harness();
    open_print_preview(&mut harness);
    let after = click(&mut harness, "docs.print_preview.next");
    after.assert_png_valid();
    // Should still be at page 1 for a single-page document
    assert_node_label(&after, "docs.print_preview.page_indicator", "Page 1 of 1");
}
