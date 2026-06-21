use tench_kodocs_lib::ui::KodocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(
        KodocsApp::new(),
        HarnessConfig::with_viewport(1440.0, 900.0),
    )
}

/// Tab bar nodes (kodocs.tab.{i}, kodocs.tab.close.{i}) only appear when
/// there are multiple open tabs. In the default single-tab state the tab bar
/// height is zero, so those selectors are absent. This test verifies the
/// default single-tab state does not expose tab-bar selectors.
#[test]
fn tab_bar_single_tab_no_bar_selectors() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.tab.0");
    assert_absent(&c, "kodocs.tab.close.0");
}

/// Verify the tab context menu item selector exists when the tab bar context
/// menu is open. With a single tab the tab bar is not visible, so we test
/// that the context menu item is absent by default.
#[test]
fn tab_context_menu_close_tab_absent_by_default() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.탭_닫기");
}
