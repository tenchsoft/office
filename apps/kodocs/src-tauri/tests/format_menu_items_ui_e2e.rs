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

#[test]
fn format_bold_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.format", "kodocs.menu.서식.굵게");
    assert_capture_changed(&b, &a);
}
#[test]
fn format_italic_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.format", "kodocs.menu.서식.기울임");
    assert_capture_changed(&b, &a);
}
#[test]
fn format_underline_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.format", "kodocs.menu.서식.밑줄");
    assert_capture_changed(&b, &a);
}
#[test]
fn format_strikethrough_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.format", "kodocs.menu.서식.취소선");
    assert_capture_changed(&b, &a);
}
#[test]
fn format_superscript_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.format", "kodocs.menu.서식.위_첨자");
    assert_capture_changed(&b, &a);
}
#[test]
fn format_subscript_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.format", "kodocs.menu.서식.아래_첨자");
    assert_capture_changed(&b, &a);
}
#[test]
fn format_clear_formatting_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.format", "kodocs.menu.서식.서식_지우기").assert_png_valid();
}
#[test]
fn format_block_quote_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.format", "kodocs.menu.서식.인용");
    assert_capture_changed(&b, &a);
}
#[test]
fn format_hanja_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.format", "kodocs.menu.서식.한자_변환");
    assert_capture_changed(&b, &a);
}
