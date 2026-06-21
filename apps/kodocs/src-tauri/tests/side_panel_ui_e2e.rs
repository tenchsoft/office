use tench_kodocs_lib::ui::KodocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;
use tench_ui_test::{assert_capture_changed, CaptureAssertions};

fn make_harness() -> TestHarness {
    TestHarness::with_config(
        KodocsApp::new(),
        HarnessConfig::with_viewport(1440.0, 900.0),
    )
}

fn open_style_panel(h: &mut TestHarness) {
    click(h, "kodocs.menu.view");
    click(h, "kodocs.menu.보기.스타일_패널");
}

#[test]
fn style_panel_style_tab_present() {
    let mut h = make_harness();
    open_style_panel(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.style_panel.tab.style");
}

#[test]
fn style_panel_navigate_tab_present() {
    let mut h = make_harness();
    open_style_panel(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.style_panel.tab.navigate");
}

#[test]
fn style_panel_ai_tab_present() {
    let mut h = make_harness();
    open_style_panel(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.style_panel.tab.ai");
}

#[test]
fn style_panel_save_recovery_copy_present() {
    let mut h = make_harness();
    open_style_panel(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.style_panel.action.save_recovery_copy");
}

#[test]
fn style_panel_style_tab_click_no_crash() {
    let mut h = make_harness();
    open_style_panel(&mut h);
    click(&mut h, "kodocs.style_panel.tab.style").assert_png_valid();
}

#[test]
fn style_panel_navigate_tab_click_changes_render() {
    let mut h = make_harness();
    open_style_panel(&mut h);
    let b = capture(&mut h);
    let a = click(&mut h, "kodocs.style_panel.tab.navigate");
    a.assert_png_valid();
    assert_capture_changed(&b, &a);
}

#[test]
fn style_panel_ai_tab_click_changes_render() {
    let mut h = make_harness();
    open_style_panel(&mut h);
    let b = capture(&mut h);
    let a = click(&mut h, "kodocs.style_panel.tab.ai");
    a.assert_png_valid();
    assert_capture_changed(&b, &a);
}
