/// UI automation tests for docs sidebar tab switching.
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;
use tench_ui_test::{assert_capture_changed, CaptureAssertions};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

#[test]
fn sidebar_style_tab_present_when_style_panel_open() {
    let mut harness = make_harness();
    // Style panel is open by default
    let after = capture(&mut harness);
    assert_selector(&after, "docs.sidebar.style-tab");
    assert_selector(&after, "docs.sidebar.navigate-tab");
    assert_selector(&after, "docs.sidebar.ai-tab");
}

#[test]
fn sidebar_tab_click_switches_panel() {
    let mut harness = make_harness();
    // Style panel is open by default
    let before = capture(&mut harness);

    // Click AI tab
    let after = click(&mut harness, "docs.sidebar.ai-tab");
    assert_capture_changed(&before, &after);
    after.assert_png_valid();
}

#[test]
fn sidebar_tab_switch_preserves_document_content() {
    let mut harness = make_harness();
    // Type some text
    click(&mut harness, "docs.document");
    let _typed = type_text(&mut harness, "Test");

    // Style panel is open by default; switch tabs
    click(&mut harness, "docs.sidebar.ai-tab");
    click(&mut harness, "docs.sidebar.style-tab");

    // Document should still be there
    let after = capture(&mut harness);
    assert_selector(&after, "docs.document");
    after.assert_png_valid();
}

#[test]
fn sidebar_save_snapshot_button_present_on_style_tab() {
    let mut harness = make_harness();
    // Style panel is open by default (which shows Style tab)
    let after = capture(&mut harness);
    assert_selector(&after, "docs.sidebar.save_snapshot");
}

#[test]
fn sidebar_save_snapshot_not_present_on_ai_tab() {
    let mut harness = make_harness();
    // Style panel is open by default, switch to AI tab
    let after_click = click(&mut harness, "docs.sidebar.ai-tab");
    assert_absent(&after_click, "docs.sidebar.save_snapshot");
}

#[test]
fn sidebar_navigate_tab_hides_save_snapshot() {
    let mut harness = make_harness();
    // Style tab is default — save_snapshot present
    assert_selector(&capture(&mut harness), "docs.sidebar.save_snapshot");

    // Switch to Navigate tab
    let after = click(&mut harness, "docs.sidebar.navigate-tab");
    assert_absent(&after, "docs.sidebar.save_snapshot");
}

#[test]
fn sidebar_roundtrip_style_ai_navigate_restores_save_snapshot() {
    let mut harness = make_harness();
    // Start on Style tab
    assert_selector(&capture(&mut harness), "docs.sidebar.save_snapshot");

    // Switch to AI
    click(&mut harness, "docs.sidebar.ai-tab");
    // Switch to Navigate
    click(&mut harness, "docs.sidebar.navigate-tab");
    // Switch back to Style
    let back = click(&mut harness, "docs.sidebar.style-tab");

    // save_snapshot should be present again
    assert_selector(&back, "docs.sidebar.save_snapshot");
}

// ── Sidebar tab selected state tests ──

#[test]
fn sidebar_style_tab_selected_by_default() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_eq!(
        get_node_value(&cap, "docs.sidebar.style-tab"),
        Some("selected".to_string())
    );
    assert_eq!(
        get_node_value(&cap, "docs.sidebar.navigate-tab"),
        Some("unselected".to_string())
    );
    assert_eq!(
        get_node_value(&cap, "docs.sidebar.ai-tab"),
        Some("unselected".to_string())
    );
}

#[test]
fn sidebar_ai_tab_selected_after_click() {
    let mut harness = make_harness();
    let after = click(&mut harness, "docs.sidebar.ai-tab");
    assert_eq!(
        get_node_value(&after, "docs.sidebar.style-tab"),
        Some("unselected".to_string())
    );
    assert_eq!(
        get_node_value(&after, "docs.sidebar.ai-tab"),
        Some("selected".to_string())
    );
    after.assert_png_valid();
}

#[test]
fn sidebar_navigate_tab_selected_after_click() {
    let mut harness = make_harness();
    let after = click(&mut harness, "docs.sidebar.navigate-tab");
    assert_eq!(
        get_node_value(&after, "docs.sidebar.navigate-tab"),
        Some("selected".to_string())
    );
    assert_eq!(
        get_node_value(&after, "docs.sidebar.style-tab"),
        Some("unselected".to_string())
    );
    after.assert_png_valid();
}

#[test]
fn sidebar_tab_selection_roundtrip() {
    let mut harness = make_harness();
    // Start on Style
    assert_eq!(
        get_node_value(&capture(&mut harness), "docs.sidebar.style-tab"),
        Some("selected".to_string())
    );

    // Switch to AI
    click(&mut harness, "docs.sidebar.ai-tab");
    assert_eq!(
        get_node_value(&capture(&mut harness), "docs.sidebar.ai-tab"),
        Some("selected".to_string())
    );

    // Switch to Navigate
    click(&mut harness, "docs.sidebar.navigate-tab");
    assert_eq!(
        get_node_value(&capture(&mut harness), "docs.sidebar.navigate-tab"),
        Some("selected".to_string())
    );

    // Back to Style
    let back = click(&mut harness, "docs.sidebar.style-tab");
    assert_eq!(
        get_node_value(&back, "docs.sidebar.style-tab"),
        Some("selected".to_string())
    );
}

// ── Save snapshot button click tests ──

#[test]
fn sidebar_save_snapshot_button_is_clickable() {
    let mut harness = make_harness();
    // Style tab is default
    let after = click(&mut harness, "docs.sidebar.save_snapshot");
    // Should not crash; modal should still be visible
    assert_selector(&after, "docs.sidebar.save_snapshot");
    after.assert_png_valid();
}

// ── Outline heading tests ──

#[test]
fn sidebar_navigate_tab_shows_no_headings_for_empty_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.sidebar.navigate-tab");
    let cap = capture(&mut harness);
    // No heading nodes should exist for empty document
    assert_absent(&cap, "docs.outline.heading.0");
}
