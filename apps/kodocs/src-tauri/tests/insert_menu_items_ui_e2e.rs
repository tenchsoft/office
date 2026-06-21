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
fn insert_image_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.그림").assert_png_valid();
}
#[test]
fn insert_table_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.표");
    assert_capture_changed(&b, &a);
}
#[test]
fn insert_link_opens_modal() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.링크");
    assert_capture_changed(&b, &a);
    assert_selector(&a, "kodocs.modal.link");
}
#[test]
fn insert_horizontal_rule_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.가로줄");
    assert_capture_changed(&b, &a);
}
#[test]
fn insert_page_break_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(
        &mut h,
        "kodocs.menu.insert",
        "kodocs.menu.삽입.페이지_나누기",
    );
    assert_capture_changed(&b, &a);
}
#[test]
fn insert_header_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.머리글");
    assert_capture_changed(&b, &a);
}
#[test]
fn insert_footer_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.바닥글").assert_png_valid();
}
#[test]
fn insert_equation_opens_editor() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.수식");
    assert_capture_changed(&b, &a);
    assert_selector(&a, "kodocs.equation_editor");
}
