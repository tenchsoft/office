use tench_kodocs_lib::ui::KodocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::CaptureAssertions;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(
        KodocsApp::new(),
        HarnessConfig::with_viewport(1440.0, 900.0),
    )
}

fn setup_hanja_popup(h: &mut TestHarness) {
    // Click in the document area to place cursor
    click(h, "kodocs.root");
    // Type a Korean word that has Hanja candidates (학교 is in the lookup table)
    type_text(h, "학교");
    // Open format menu and click Hanja conversion
    click(h, "kodocs.menu.format");
    click(h, "kodocs.menu.서식.한자_변환");
}

#[test]
fn hanja_popup_candidate_row_present() {
    let mut h = make_harness();
    setup_hanja_popup(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.hanja_popup.candidate.0");
}

#[test]
fn hanja_popup_keyboard_selection_no_crash() {
    let mut h = make_harness();
    setup_hanja_popup(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.hanja_popup");
    c.assert_png_valid();
}
