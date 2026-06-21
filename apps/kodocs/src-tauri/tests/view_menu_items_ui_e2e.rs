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
fn view_thumbnails_changes_render() {
    let mut h = make_harness();
    let before = capture(&mut h);
    let after = open_menu_item(&mut h, "kodocs.menu.view", "kodocs.menu.보기.미리보기");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_style_panel_changes_render() {
    let mut h = make_harness();
    let before = capture(&mut h);
    let after = open_menu_item(&mut h, "kodocs.menu.view", "kodocs.menu.보기.스타일_패널");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_comments_changes_render() {
    let mut h = make_harness();
    let before = capture(&mut h);
    let after = open_menu_item(&mut h, "kodocs.menu.view", "kodocs.menu.보기.메모");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_zoom_in_changes_render() {
    let mut h = make_harness();
    let before = capture(&mut h);
    let after = open_menu_item(&mut h, "kodocs.menu.view", "kodocs.menu.보기.확대");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_zoom_out_changes_render() {
    let mut h = make_harness();
    let before = capture(&mut h);
    let after = open_menu_item(&mut h, "kodocs.menu.view", "kodocs.menu.보기.축소");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_reset_zoom_no_crash() {
    let mut h = make_harness();
    open_menu_item(
        &mut h,
        "kodocs.menu.view",
        "kodocs.menu.보기.확대/축소_초기화",
    )
    .assert_png_valid();
}

#[test]
fn view_vertical_writing_changes_render() {
    let mut h = make_harness();
    let before = capture(&mut h);
    let after = open_menu_item(&mut h, "kodocs.menu.view", "kodocs.menu.보기.세로쓰기");
    assert_capture_changed(&before, &after);
}
