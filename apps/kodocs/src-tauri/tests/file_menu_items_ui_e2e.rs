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
fn file_menu_new_creates_document() {
    let mut h = make_harness();
    let before = capture(&mut h);
    let after = open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.새_문서");
    assert_capture_changed(&before, &after);
}

#[test]
fn file_menu_open_no_crash() {
    let mut h = make_harness();
    let after = open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.열기");
    after.assert_png_valid();
}

#[test]
fn file_menu_save_no_crash() {
    let mut h = make_harness();
    let after = open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.저장");
    after.assert_png_valid();
}

#[test]
fn file_menu_save_as_no_crash() {
    let mut h = make_harness();
    let after = open_menu_item(
        &mut h,
        "kodocs.menu.file",
        "kodocs.menu.파일.다른_이름으로_저장",
    );
    after.assert_png_valid();
}

#[test]
fn file_menu_export_no_crash() {
    let mut h = make_harness();
    let after = open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.내보내기");
    after.assert_png_valid();
}

#[test]
fn file_menu_page_setup_opens_modal() {
    let mut h = make_harness();
    let before = capture(&mut h);
    let after = open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    assert_capture_changed(&before, &after);
    assert_selector(&after, "kodocs.modal.page_setup");
}

#[test]
fn file_menu_print_no_crash() {
    let mut h = make_harness();
    let after = open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.인쇄");
    after.assert_png_valid();
}

#[test]
fn file_menu_version_history_no_crash() {
    let mut h = make_harness();
    let after = open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.버전_기록");
    after.assert_png_valid();
}

#[test]
fn file_menu_hwp_import_no_crash() {
    let mut h = make_harness();
    let after = open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.hwp_가져오기");
    after.assert_png_valid();
}

#[test]
fn file_menu_hwpx_export_no_crash() {
    let mut h = make_harness();
    let after = open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.hwpx_내보내기");
    after.assert_png_valid();
}
