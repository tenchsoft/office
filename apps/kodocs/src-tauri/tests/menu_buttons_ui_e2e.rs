/// UI automation tests for Kodocs Menu Buttons (#2-#7).
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
fn edit_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "kodocs.menu.edit");
    let after = click(&mut harness, "kodocs.menu.edit");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "kodocs.menu.view");
    let after = click(&mut harness, "kodocs.menu.view");
    assert_capture_changed(&before, &after);
}

#[test]
fn insert_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "kodocs.menu.insert");
    let after = click(&mut harness, "kodocs.menu.insert");
    assert_capture_changed(&before, &after);
}

#[test]
fn format_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "kodocs.menu.format");
    let after = click(&mut harness, "kodocs.menu.format");
    assert_capture_changed(&before, &after);
}

#[test]
fn tools_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "kodocs.menu.tools");
    let after = click(&mut harness, "kodocs.menu.tools");
    assert_capture_changed(&before, &after);
}

#[test]
fn help_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "kodocs.menu.help");
    let after = click(&mut harness, "kodocs.menu.help");
    assert_capture_changed(&before, &after);
}
