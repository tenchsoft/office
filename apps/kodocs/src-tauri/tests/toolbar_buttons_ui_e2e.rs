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
fn toolbar_undo_no_crash() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.undo");
    click(&mut h, "kodocs.toolbar.undo").assert_png_valid();
}
#[test]
fn toolbar_redo_no_crash() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.redo");
    click(&mut h, "kodocs.toolbar.redo").assert_png_valid();
}
#[test]
fn toolbar_bullet_list() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.bullet_list");
    let a = click(&mut h, "kodocs.toolbar.bullet_list");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_numbered_list() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.numbered_list");
    let a = click(&mut h, "kodocs.toolbar.numbered_list");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_checklist() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.checklist");
    let a = click(&mut h, "kodocs.toolbar.checklist");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_align_left() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.align_left");
    let a = click(&mut h, "kodocs.toolbar.align_left");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_align_right() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.align_right");
    let a = click(&mut h, "kodocs.toolbar.align_right");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_insert_link() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.insert_link");
    let a = click(&mut h, "kodocs.toolbar.insert_link");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_insert_image_no_crash() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.insert_image");
    click(&mut h, "kodocs.toolbar.insert_image").assert_png_valid();
}
#[test]
fn toolbar_insert_table() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.insert_table");
    let a = click(&mut h, "kodocs.toolbar.insert_table");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_block_quote() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.block_quote");
    let a = click(&mut h, "kodocs.toolbar.block_quote");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_align_center() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.align_center");
    let a = click(&mut h, "kodocs.toolbar.align_center");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_align_justify() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.align_justify");
    let a = click(&mut h, "kodocs.toolbar.align_justify");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_outdent_no_crash() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.outdent");
    click(&mut h, "kodocs.toolbar.outdent").assert_png_valid();
}
#[test]
fn toolbar_indent_no_crash() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.indent");
    click(&mut h, "kodocs.toolbar.indent").assert_png_valid();
}
#[test]
fn toolbar_horizontal_rule() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.horizontal_rule");
    let a = click(&mut h, "kodocs.toolbar.horizontal_rule");
    assert_capture_changed(&b, &a);
}
#[test]
fn toolbar_more_overflow_no_crash() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.more_overflow");
    click(&mut h, "kodocs.toolbar.more_overflow").assert_png_valid();
}
#[test]
fn toolbar_text_color_no_crash() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.text_color");
    click(&mut h, "kodocs.toolbar.text_color").assert_png_valid();
}
#[test]
fn toolbar_highlight_color_no_crash() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.highlight_color");
    click(&mut h, "kodocs.toolbar.highlight_color").assert_png_valid();
}
