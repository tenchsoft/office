use tench_kodocs_lib::ui::KodocsApp;
use tench_ui_test::assert_capture_changed;
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

fn open_equation_editor(h: &mut TestHarness) {
    click(h, "kodocs.menu.insert");
    click(h, "kodocs.menu.삽입.수식");
}

#[test]
fn equation_editor_input_field_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.input");
}

#[test]
fn equation_editor_insert_button_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.insert");
}

#[test]
fn equation_editor_cancel_button_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.cancel");
}

#[test]
fn equation_editor_close_button_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.close");
}

#[test]
fn equation_editor_cancel_closes_dialog() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let before = capture(&mut h);
    assert_selector(&before, "kodocs.equation_editor");
    let after = click(&mut h, "kodocs.equation_editor.cancel");
    assert_absent(&after, "kodocs.equation_editor");
    assert_capture_changed(&before, &after);
}

#[test]
fn equation_editor_insert_with_empty_input_no_crash() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    click(&mut h, "kodocs.equation_editor.insert").assert_png_valid();
}

#[test]
fn equation_editor_plus_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.plus");
}

#[test]
fn equation_editor_minus_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.minus");
}

#[test]
fn equation_editor_multiply_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.multiply");
}

#[test]
fn equation_editor_divide_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.divide");
}

#[test]
fn equation_editor_equals_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.equals");
}

#[test]
fn equation_editor_not_equal_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.not_equal");
}

#[test]
fn equation_editor_less_or_equal_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.less_or_equal");
}

#[test]
fn equation_editor_greater_or_equal_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.greater_or_equal");
}

#[test]
fn equation_editor_left_parenthesis_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.left_parenthesis");
}

#[test]
fn equation_editor_right_parenthesis_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.right_parenthesis");
}

#[test]
fn equation_editor_left_bracket_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.left_bracket");
}

#[test]
fn equation_editor_right_bracket_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.right_bracket");
}

#[test]
fn equation_editor_left_brace_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.left_brace");
}

#[test]
fn equation_editor_right_brace_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.right_brace");
}

#[test]
fn equation_editor_pi_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.pi");
}

#[test]
fn equation_editor_square_root_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.square_root");
}

#[test]
fn equation_editor_sum_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.sum");
}

#[test]
fn equation_editor_integral_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.integral");
}

#[test]
fn equation_editor_superscript_two_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.superscript_two");
}

#[test]
fn equation_editor_superscript_three_symbol_present() {
    let mut h = make_harness();
    open_equation_editor(&mut h);
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.equation_editor.superscript_three");
}
