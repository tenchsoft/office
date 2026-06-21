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

#[test]
fn document_canvas_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.document");
    assert_bounds_inside(&c, "kodocs.document");
}

#[test]
fn document_text_input_changes_render() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let b = capture(&mut h);
    let a = type_text(&mut h, "Hello");
    assert_capture_changed(&b, &a);
}

#[test]
fn ruler_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.ruler");
    assert_bounds_inside(&c, "kodocs.ruler");
}

#[test]
fn title_row_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.title_row");
}

#[test]
fn status_bar_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.status_bar");
    assert_bounds_inside(&c, "kodocs.status_bar");
}

#[test]
fn sidebar_thumbnails_toggle() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.view", "kodocs.menu.보기.미리보기");
    assert_capture_changed(&b, &a);
}
