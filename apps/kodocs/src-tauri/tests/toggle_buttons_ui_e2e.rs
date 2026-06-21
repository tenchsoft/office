use tench_kodocs_lib::ui::KodocsApp;
use tench_ui_test::assert_capture_changed;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(
        KodocsApp::new(),
        HarnessConfig::with_viewport(1440.0, 900.0),
    )
}

fn verify_toggle_cycle(debug_id: &str) {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, debug_id);
    let after_on = click(&mut harness, debug_id);
    assert_capture_changed(&before, &after_on);
    let after_off = click(&mut harness, debug_id);
    assert_capture_changed(&after_on, &after_off);
}

#[test]
fn toolbar_bold_toggle() {
    verify_toggle_cycle("kodocs.toolbar.bold");
}
#[test]
fn toolbar_italic_toggle() {
    verify_toggle_cycle("kodocs.toolbar.italic");
}
#[test]
fn toolbar_underline_toggle() {
    verify_toggle_cycle("kodocs.toolbar.underline");
}
#[test]
fn toolbar_strikethrough_toggle() {
    verify_toggle_cycle("kodocs.toolbar.strikethrough");
}
#[test]
fn toolbar_code_toggle() {
    verify_toggle_cycle("kodocs.toolbar.code");
}
#[test]
fn toolbar_superscript_toggle() {
    verify_toggle_cycle("kodocs.toolbar.superscript");
}
#[test]
fn toolbar_subscript_toggle() {
    verify_toggle_cycle("kodocs.toolbar.subscript");
}
