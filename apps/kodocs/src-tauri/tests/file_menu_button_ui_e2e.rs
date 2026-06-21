/// UI automation tests for Kodocs File Menu Button (work plan #1).
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
fn file_menu_renders_initial_screen() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    cap.assert_png_size(1440, 900);
    assert_selector(&cap, "kodocs.menu.file");
}

#[test]
fn file_menu_opens_on_click() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = click(&mut harness, "kodocs.menu.file");
    assert_capture_changed(&before, &after);
}

#[test]
fn file_menu_closes_on_outside_click() {
    let mut harness = make_harness();
    let initial = capture(&mut harness);
    let menu_open = click(&mut harness, "kodocs.menu.file");
    assert_capture_changed(&initial, &menu_open);
    let after_close = click(&mut harness, "kodocs.document");
    assert_capture_changed(&menu_open, &after_close);
}

#[test]
fn file_menu_replaced_by_another_menu() {
    let mut harness = make_harness();
    let file_menu = click(&mut harness, "kodocs.menu.file");
    let edit_menu = click(&mut harness, "kodocs.menu.edit");
    assert_capture_changed(&file_menu, &edit_menu);
}
