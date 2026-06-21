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
fn font_size_dropdown_opens() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.font_size");
    let a = click(&mut h, "kodocs.toolbar.font_size");
    assert_capture_changed(&b, &a);
}

#[test]
fn font_family_dropdown_opens() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.font_family");
    let a = click(&mut h, "kodocs.toolbar.font_family");
    assert_capture_changed(&b, &a);
}

#[test]
fn paragraph_style_dropdown_opens() {
    let mut h = make_harness();
    let b = capture(&mut h);
    assert_selector(&b, "kodocs.toolbar.paragraph_style");
    let a = click(&mut h, "kodocs.toolbar.paragraph_style");
    assert_capture_changed(&b, &a);
}

#[test]
fn all_dropdowns_present() {
    let mut h = make_harness();
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size");
    assert_selector(&c, "kodocs.toolbar.font_family");
    assert_selector(&c, "kodocs.toolbar.paragraph_style");
}

// --- Font family dropdown items ---

#[test]
fn font_family_nanum_gothic_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.나눔고딕");
}

#[test]
fn font_family_nanum_myeongjo_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.나눔명조");
}

#[test]
fn font_family_malgun_gothic_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.맑은_고딕");
}

#[test]
fn font_family_gulim_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.굴림");
}

#[test]
fn font_family_batang_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.바탕");
}

#[test]
fn font_family_gungsuh_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.궁서");
}

#[test]
fn font_family_hambatang_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.함초롬바탕");
}

#[test]
fn font_family_arial_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.arial");
}

#[test]
fn font_family_helvetica_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.helvetica");
}

#[test]
fn font_family_times_new_roman_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.times_new_roman");
}

#[test]
fn font_family_georgia_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.georgia");
}

#[test]
fn font_family_courier_new_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.courier_new");
}

#[test]
fn font_family_monospace_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_family");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_family.item.monospace");
}

// --- Font size dropdown items ---

#[test]
fn font_size_8px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.8px");
}

#[test]
fn font_size_9px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.9px");
}

#[test]
fn font_size_10px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.10px");
}

#[test]
fn font_size_11px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.11px");
}

#[test]
fn font_size_12px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.12px");
}

#[test]
fn font_size_14px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.14px");
}

#[test]
fn font_size_16px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.16px");
}

#[test]
fn font_size_18px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.18px");
}

#[test]
fn font_size_20px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.20px");
}

#[test]
fn font_size_24px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.24px");
}

#[test]
fn font_size_28px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.28px");
}

#[test]
fn font_size_32px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.32px");
}

#[test]
fn font_size_36px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.36px");
}

#[test]
fn font_size_48px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.48px");
}

#[test]
fn font_size_72px_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.font_size");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.font_size.item.72px");
}

// --- Paragraph style dropdown items ---

#[test]
fn paragraph_style_paragraph_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.paragraph_style");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.paragraph_style.item.paragraph");
}

#[test]
fn paragraph_style_heading_1_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.paragraph_style");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.paragraph_style.item.heading_1");
}

#[test]
fn paragraph_style_heading_2_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.paragraph_style");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.paragraph_style.item.heading_2");
}

#[test]
fn paragraph_style_heading_3_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.paragraph_style");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.paragraph_style.item.heading_3");
}

#[test]
fn paragraph_style_heading_4_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.paragraph_style");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.paragraph_style.item.heading_4");
}

#[test]
fn paragraph_style_heading_5_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.paragraph_style");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.paragraph_style.item.heading_5");
}

#[test]
fn paragraph_style_heading_6_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.paragraph_style");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.paragraph_style.item.heading_6");
}

#[test]
fn paragraph_style_block_quote_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.paragraph_style");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.paragraph_style.item.block_quote");
}

#[test]
fn paragraph_style_code_block_item_present() {
    let mut h = make_harness();
    click(&mut h, "kodocs.toolbar.paragraph_style");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.toolbar.paragraph_style.item.code_block");
}
