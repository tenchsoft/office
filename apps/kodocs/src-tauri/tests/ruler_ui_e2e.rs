use tench_kodocs_lib::ui::KodocsApp;
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
fn ruler_first_line_indent_marker_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.ruler.indent.first_line");
}

#[test]
fn ruler_left_indent_marker_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.ruler.indent.left");
}

#[test]
fn ruler_left_margin_marker_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.ruler.margin.left");
}

#[test]
fn ruler_right_indent_marker_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.ruler.indent.right");
}

#[test]
fn ruler_right_margin_marker_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.ruler.margin.right");
}
