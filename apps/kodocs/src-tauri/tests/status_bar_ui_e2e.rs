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
fn status_bar_page_indicator_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.status_bar.page_indicator");
}

#[test]
fn status_bar_zoom_out_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.status_bar.zoom_out");
}

#[test]
fn status_bar_zoom_in_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.status_bar.zoom_in");
}

#[test]
fn status_bar_zoom_out_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.status_bar.zoom_out");
    let a = click(&mut h, "kodocs.status_bar.zoom_out");
    a.assert_png_valid();
    assert_capture_changed(&b, &a);
}

#[test]
fn status_bar_zoom_in_changes_render() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.status_bar.zoom_in");
    let a = click(&mut h, "kodocs.status_bar.zoom_in");
    a.assert_png_valid();
    assert_capture_changed(&b, &a);
}
