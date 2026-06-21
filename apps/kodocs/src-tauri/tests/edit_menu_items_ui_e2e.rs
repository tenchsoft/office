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
fn edit_undo_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.실행_취소").assert_png_valid();
}
#[test]
fn edit_redo_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.다시_실행").assert_png_valid();
}
#[test]
fn edit_cut_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.잘라내기").assert_png_valid();
}
#[test]
fn edit_copy_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.복사").assert_png_valid();
}
#[test]
fn edit_paste_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.붙여넣기").assert_png_valid();
}
#[test]
fn edit_select_all_no_crash() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.모두_선택").assert_png_valid();
}

#[test]
fn edit_find_opens_modal() {
    let mut h = make_harness();
    let before = capture(&mut h);
    let after = open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.찾기");
    assert_capture_changed(&before, &after);
    assert_selector(&after, "kodocs.modal.find_replace");
}

#[test]
fn edit_replace_opens_modal() {
    let mut h = make_harness();
    let before = capture(&mut h);
    let after = open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.바꾸기");
    assert_capture_changed(&before, &after);
    assert_selector(&after, "kodocs.modal.find_replace");
}
