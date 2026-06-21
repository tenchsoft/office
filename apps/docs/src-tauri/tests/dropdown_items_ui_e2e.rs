/// UI automation tests for docs dropdown items (#83-#137).
///
/// Uses debug_id selectors for dropdown controls.
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::assert_capture_changed;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::CaptureAssertions;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

// Font size dropdown
#[test]
fn font_size_dropdown_opens() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.font_size");
    let after = click(&mut harness, "docs.toolbar.font_size");
    assert_capture_changed(&before, &after);
}

// Font family dropdown
#[test]
fn font_family_dropdown_opens() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.font_family");
    let after = click(&mut harness, "docs.toolbar.font_family");
    assert_capture_changed(&before, &after);
}

// Paragraph style dropdown
#[test]
fn paragraph_style_dropdown_opens() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.paragraph_style");
    let after = click(&mut harness, "docs.toolbar.paragraph_style");
    assert_capture_changed(&before, &after);
}

// Verify all three dropdowns exist in initial render
#[test]
fn all_dropdowns_present_in_initial_render() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.font_size");
    assert_selector(&cap, "docs.toolbar.font_family");
    assert_selector(&cap, "docs.toolbar.paragraph_style");
}

// ---------------------------------------------------------------------------
// Font size dropdown: verify it exposes a dropdown node when open
// ---------------------------------------------------------------------------

#[test]
fn font_size_dropdown_shows_dropdown_node() {
    let mut harness = make_harness();
    click(&mut harness, "docs.toolbar.font_size");
    let cap = capture(&mut harness);
    // The dropdown node should be present
    assert_selector(&cap, "docs.dropdown.font_size");
    cap.assert_png_valid();
}

#[test]
fn font_size_dropdown_closes_on_outside_click() {
    let mut harness = make_harness();
    click(&mut harness, "docs.toolbar.font_size");
    let open_cap = capture(&mut harness);
    assert_selector(&open_cap, "docs.dropdown.font_size");

    // Click outside the dropdown (on the document)
    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.dropdown.font_size");
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Font family dropdown: verify it exposes a dropdown node when open
// ---------------------------------------------------------------------------

#[test]
fn font_family_dropdown_shows_dropdown_node() {
    let mut harness = make_harness();
    click(&mut harness, "docs.toolbar.font_family");
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.dropdown.font_family");
    cap.assert_png_valid();
}

#[test]
fn font_family_dropdown_closes_on_outside_click() {
    let mut harness = make_harness();
    click(&mut harness, "docs.toolbar.font_family");
    let open_cap = capture(&mut harness);
    assert_selector(&open_cap, "docs.dropdown.font_family");

    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.dropdown.font_family");
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Paragraph style dropdown: verify it exposes a dropdown node when open
// ---------------------------------------------------------------------------

#[test]
fn paragraph_style_dropdown_shows_dropdown_node() {
    let mut harness = make_harness();
    click(&mut harness, "docs.toolbar.paragraph_style");
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.dropdown.paragraph_style");
    cap.assert_png_valid();
}

#[test]
fn paragraph_style_dropdown_closes_on_outside_click() {
    let mut harness = make_harness();
    click(&mut harness, "docs.toolbar.paragraph_style");
    let open_cap = capture(&mut harness);
    assert_selector(&open_cap, "docs.dropdown.paragraph_style");

    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.dropdown.paragraph_style");
    after.assert_png_valid();
}
