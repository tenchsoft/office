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

// --- Text color palette tests ---

#[test]
fn text_color_palette_opens() {
    let mut h = make_harness();
    let before = capture(&mut h);
    assert_selector(&before, "kodocs.toolbar.text_color");
    let after = click(&mut h, "kodocs.toolbar.text_color");
    assert_capture_changed(&before, &after);
}

#[test]
fn text_color_palette_number000000_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.number000000");
}

#[test]
fn text_color_palette_number434343_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.number434343");
}

#[test]
fn text_color_palette_number666666_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.number666666");
}

#[test]
fn text_color_palette_number999999_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.number999999");
}

#[test]
fn text_color_palette_numberb7b7b7_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberb7b7b7");
}

#[test]
fn text_color_palette_numbercccccc_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numbercccccc");
}

#[test]
fn text_color_palette_numberd9d9d9_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberd9d9d9");
}

#[test]
fn text_color_palette_numberefefef_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberefefef");
}

#[test]
fn text_color_palette_numberf3f3f3_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberf3f3f3");
}

#[test]
fn text_color_palette_numberffffff_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberffffff");
}

#[test]
fn text_color_palette_number980000_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.number980000");
}

#[test]
fn text_color_palette_numberff0000_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberff0000");
}

#[test]
fn text_color_palette_numberff9900_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberff9900");
}

#[test]
fn text_color_palette_numberffff00_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberffff00");
}

#[test]
fn text_color_palette_number00ff00_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.number00ff00");
}

#[test]
fn text_color_palette_number00ffff_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.number00ffff");
}

#[test]
fn text_color_palette_number4a86e8_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.number4a86e8");
}

#[test]
fn text_color_palette_number0000ff_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.number0000ff");
}

#[test]
fn text_color_palette_number9900ff_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.number9900ff");
}

#[test]
fn text_color_palette_numberff00ff_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberff00ff");
}

#[test]
fn text_color_palette_numbere6b8af_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numbere6b8af");
}

#[test]
fn text_color_palette_numberf4cccc_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberf4cccc");
}

#[test]
fn text_color_palette_numberfce5cd_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberfce5cd");
}

#[test]
fn text_color_palette_numberfff2cc_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberfff2cc");
}

#[test]
fn text_color_palette_numberd9ead3_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberd9ead3");
}

#[test]
fn text_color_palette_numberd0e0e3_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberd0e0e3");
}

#[test]
fn text_color_palette_numberc9daf8_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberc9daf8");
}

#[test]
fn text_color_palette_numbercfe2f3_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numbercfe2f3");
}

#[test]
fn text_color_palette_numberd9d2e9_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberd9d2e9");
}

#[test]
fn text_color_palette_numberead1dc_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.text_color");
    assert_selector(&c, "kodocs.toolbar.text_color_palette.numberead1dc");
}

// --- Background color palette tests ---

#[test]
fn background_color_palette_opens() {
    let mut h = make_harness();
    let before = capture(&mut h);
    assert_selector(&before, "kodocs.toolbar.highlight_color");
    let after = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_capture_changed(&before, &after);
}

#[test]
fn background_color_palette_number000000_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.number000000");
}

#[test]
fn background_color_palette_number434343_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.number434343");
}

#[test]
fn background_color_palette_number666666_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.number666666");
}

#[test]
fn background_color_palette_number999999_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.number999999");
}

#[test]
fn background_color_palette_numberb7b7b7_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberb7b7b7");
}

#[test]
fn background_color_palette_numbercccccc_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numbercccccc");
}

#[test]
fn background_color_palette_numberd9d9d9_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberd9d9d9");
}

#[test]
fn background_color_palette_numberefefef_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberefefef");
}

#[test]
fn background_color_palette_numberf3f3f3_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberf3f3f3");
}

#[test]
fn background_color_palette_numberffffff_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberffffff");
}

#[test]
fn background_color_palette_number980000_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.number980000");
}

#[test]
fn background_color_palette_numberff0000_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberff0000");
}

#[test]
fn background_color_palette_numberff9900_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberff9900");
}

#[test]
fn background_color_palette_numberffff00_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberffff00");
}

#[test]
fn background_color_palette_number00ff00_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.number00ff00");
}

#[test]
fn background_color_palette_number00ffff_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.number00ffff");
}

#[test]
fn background_color_palette_number4a86e8_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.number4a86e8");
}

#[test]
fn background_color_palette_number0000ff_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.number0000ff");
}

#[test]
fn background_color_palette_number9900ff_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.number9900ff");
}

#[test]
fn background_color_palette_numberff00ff_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberff00ff");
}

#[test]
fn background_color_palette_numbere6b8af_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numbere6b8af");
}

#[test]
fn background_color_palette_numberf4cccc_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberf4cccc");
}

#[test]
fn background_color_palette_numberfce5cd_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberfce5cd");
}

#[test]
fn background_color_palette_numberfff2cc_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberfff2cc");
}

#[test]
fn background_color_palette_numberd9ead3_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberd9ead3");
}

#[test]
fn background_color_palette_numberd0e0e3_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberd0e0e3");
}

#[test]
fn background_color_palette_numberc9daf8_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberc9daf8");
}

#[test]
fn background_color_palette_numbercfe2f3_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numbercfe2f3");
}

#[test]
fn background_color_palette_numberd9d2e9_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberd9d2e9");
}

#[test]
fn background_color_palette_numberead1dc_present() {
    let mut h = make_harness();
    let c = click(&mut h, "kodocs.toolbar.highlight_color");
    assert_selector(&c, "kodocs.toolbar.background_color_palette.numberead1dc");
}
