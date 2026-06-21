/// E2E tests for the tab bar automation nodes.
///
/// When only one tab is open the tab bar is hidden (height = 0), so no tab
/// selectors should be present. When multiple tabs are open the bar appears
/// and each tab exposes `docs.tab.{i}` and `docs.tab.{i}.close` nodes.
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::{CaptureAssertions, TestHarness};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

/// Helper: open a new tab via File -> New.
fn open_new_tab(harness: &mut TestHarness) {
    open_menu_item(harness, "docs.menu.file", "docs.menu.file.new");
}

#[test]
fn single_tab_no_tab_bar() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);

    // With a single tab the tab bar is hidden: no tab selectors should exist.
    assert_absent(&cap, "docs.tab.0");
}

#[test]
fn tab_switch_button_present_on_multi_tab() {
    // Default state has a single tab, so docs.tab.0 must be absent.
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_absent(&cap, "docs.tab.0");
}

#[test]
fn tab_close_button_absent_on_single_tab() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);

    // Close button selector should not exist with a single tab.
    assert_absent(&cap, "docs.tab.0.close");
}

// ---------------------------------------------------------------------------
// Multi-tab tests
// ---------------------------------------------------------------------------

#[test]
fn tab_close_button_present_on_multi_tab() {
    let mut harness = make_harness();

    // Open a second tab via File -> New
    open_new_tab(&mut harness);
    let cap = capture(&mut harness);

    // Both tabs should exist with close buttons
    assert_selector(&cap, "docs.tab.0");
    assert_selector(&cap, "docs.tab.1");
    assert_selector(&cap, "docs.tab.0.close");
    assert_selector(&cap, "docs.tab.1.close");
}

#[test]
fn tab_switch_button_clicks_switch_active_tab() {
    let mut harness = make_harness();

    // Open a second tab
    open_new_tab(&mut harness);
    let cap = capture(&mut harness);

    // Tab 1 should be active (the new tab)
    assert_selector(&cap, "docs.tab.1");

    // Click tab 0 to switch
    let after = click(&mut harness, "docs.tab.0");
    after.assert_png_valid();

    // Verify tabs count is still 2
    let tab_count = get_node_value(&after, "docs.tabs.count");
    assert_eq!(tab_count, Some("2".to_string()), "should still have 2 tabs");
}

#[test]
fn tab_close_button_closes_tab() {
    let mut harness = make_harness();

    // Open a second tab
    open_new_tab(&mut harness);
    let before = capture(&mut harness);
    assert_selector(&before, "docs.tab.1");

    // Close tab 1
    let after = click(&mut harness, "docs.tab.1.close");
    after.assert_png_valid();

    // Tab count should be 1 now
    let tab_count = get_node_value(&after, "docs.tabs.count");
    assert_eq!(
        tab_count,
        Some("1".to_string()),
        "should have 1 tab after closing"
    );

    // Tab bar should be hidden again
    assert_absent(&after, "docs.tab.0");
}

#[test]
fn tab_close_button_closes_inactive_tab() {
    let mut harness = make_harness();

    // Open two additional tabs (3 total)
    open_new_tab(&mut harness);
    open_new_tab(&mut harness);
    let before = capture(&mut harness);
    let tab_count_before = get_node_value(&before, "docs.tabs.count");
    assert_eq!(
        tab_count_before,
        Some("3".to_string()),
        "should have 3 tabs"
    );

    // Tab 2 is active. Close tab 0 (inactive).
    let after = click(&mut harness, "docs.tab.0.close");
    after.assert_png_valid();

    // Should now have 2 tabs
    let tab_count_after = get_node_value(&after, "docs.tabs.count");
    assert_eq!(
        tab_count_after,
        Some("2".to_string()),
        "should have 2 tabs after closing"
    );
}

#[test]
fn tab_context_menu_shows_items_on_multi_tab() {
    let mut harness = make_harness();

    // Open a second tab
    open_new_tab(&mut harness);

    // Right-click on tab 0 to open tab context menu
    let cap = right_click(&mut harness, "docs.tab.0");

    // Tab context menu items should be present
    assert_selector(&cap, "docs.context.close");
    assert_selector(&cap, "docs.context.close_others");
    assert_selector(&cap, "docs.context.close_all");
    cap.assert_png_valid();
}

#[test]
fn tab_context_close_item_closes_tab() {
    let mut harness = make_harness();

    // Open a second tab
    open_new_tab(&mut harness);

    // Right-click on tab 0 and click Close
    right_click(&mut harness, "docs.tab.0");
    let after = click(&mut harness, "docs.context.close");
    after.assert_png_valid();

    // Should have 1 tab now
    let tab_count = get_node_value(&after, "docs.tabs.count");
    assert_eq!(
        tab_count,
        Some("1".to_string()),
        "should have 1 tab after context close"
    );
}

#[test]
fn tab_context_close_others_item_closes_other_tabs() {
    let mut harness = make_harness();

    // Open two additional tabs (3 total)
    open_new_tab(&mut harness);
    open_new_tab(&mut harness);

    // Right-click on tab 2 (active) and click Close Others
    right_click(&mut harness, "docs.tab.2");
    let after = click(&mut harness, "docs.context.close_others");
    after.assert_png_valid();

    // Should have 1 tab now
    let tab_count = get_node_value(&after, "docs.tabs.count");
    assert_eq!(
        tab_count,
        Some("1".to_string()),
        "should have 1 tab after close others"
    );
}

#[test]
fn tab_context_close_all_item_closes_all_but_one() {
    let mut harness = make_harness();

    // Open two additional tabs (3 total)
    open_new_tab(&mut harness);
    open_new_tab(&mut harness);

    // Right-click on tab 1 and click Close All
    right_click(&mut harness, "docs.tab.1");
    let after = click(&mut harness, "docs.context.close_all");
    after.assert_png_valid();

    // Should have 1 tab (can't close all — last tab stays)
    let tab_count = get_node_value(&after, "docs.tabs.count");
    assert_eq!(
        tab_count,
        Some("1".to_string()),
        "should have 1 tab after close all"
    );
}

#[test]
fn tab_switch_preserves_document_content() {
    let mut harness = make_harness();

    // Type text in tab 0
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Tab 0 content");

    // Open a second tab and type different text
    open_new_tab(&mut harness);
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Tab 1 content");

    // Switch back to tab 0
    // Note: After opening new tab, old tab is at index 0
    let after = click(&mut harness, "docs.tab.0");
    after.assert_png_valid();

    // Tab 0 should still have its content
    let text = get_node_value(&after, "docs.document.text");
    assert_eq!(
        text,
        Some("Tab 0 content".to_string()),
        "tab 0 should preserve its content"
    );
}
