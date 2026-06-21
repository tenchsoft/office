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
fn tools_word_count_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.tools", "kodocs.menu.도구.단어_수").assert_png_valid();
}
#[test]
fn tools_track_changes_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(
        &mut h,
        "kodocs.menu.tools",
        "kodocs.menu.도구.변경_내용_추적",
    );
    assert_capture_changed(&b, &a);
}
#[test]
fn tools_spell_check_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.tools", "kodocs.menu.도구.맞춤법_검사");
    assert_capture_changed(&b, &a);
}
#[test]
fn help_about_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.help", "kodocs.menu.도움말.정보");
    assert_capture_changed(&b, &a);
}
#[test]
fn help_keyboard_shortcuts_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(
        &mut h,
        "kodocs.menu.help",
        "kodocs.menu.도움말.키보드_단축키",
    );
    assert_capture_changed(&b, &a);
}
