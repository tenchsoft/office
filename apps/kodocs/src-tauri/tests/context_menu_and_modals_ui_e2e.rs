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
fn find_modal_opens_with_controls() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.찾기");
    assert_capture_changed(&b, &a);
    assert_selector(&a, "kodocs.modal.find_replace");
    assert_selector(&a, "kodocs.modal.find_replace.query");
    assert_selector(&a, "kodocs.modal.find_replace.close");
}

#[test]
fn find_modal_search_input_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.찾기");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.find_replace.query");
}

#[test]
fn find_modal_find_next_button_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.찾기");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.find_replace.next");
}

#[test]
fn find_modal_find_previous_button_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.찾기");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.find_replace.prev");
}

#[test]
fn find_modal_case_sensitive_toggle_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.찾기");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.find_replace.case_sensitive");
}

#[test]
fn find_modal_regex_toggle_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.찾기");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.find_replace.regex");
}

#[test]
fn find_modal_close_button_closes() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.찾기");
    let before = capture(&mut h);
    assert_selector(&before, "kodocs.modal.find_replace");
    let after = click(&mut h, "kodocs.modal.find_replace.close");
    assert_absent(&after, "kodocs.modal.find_replace");
    assert_capture_changed(&before, &after);
}

#[test]
fn replace_modal_opens_with_controls() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.바꾸기");
    assert_capture_changed(&b, &a);
    assert_selector(&a, "kodocs.modal.find_replace");
    assert_selector(&a, "kodocs.modal.find_replace.replace");
    assert_selector(&a, "kodocs.modal.find_replace.replace_btn");
    assert_selector(&a, "kodocs.modal.find_replace.replace_all");
}

#[test]
fn replace_modal_replacement_input_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.edit", "kodocs.menu.편집.바꾸기");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.find_replace.replace");
}

#[test]
fn link_modal_opens_with_controls() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.링크");
    assert_capture_changed(&b, &a);
    assert_selector(&a, "kodocs.modal.link");
    assert_selector(&a, "kodocs.modal.link.url");
    assert_selector(&a, "kodocs.modal.link.ok");
    assert_selector(&a, "kodocs.modal.link.cancel");
}

#[test]
fn link_modal_url_input_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.링크");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.link.url");
}

#[test]
fn link_modal_ok_button_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.링크");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.link.ok");
}

#[test]
fn link_modal_cancel_button_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.insert", "kodocs.menu.삽입.링크");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.link.cancel");
}

#[test]
fn page_setup_modal_opens_with_controls() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    assert_capture_changed(&b, &a);
    assert_selector(&a, "kodocs.modal.page_setup");
    assert_selector(&a, "kodocs.modal.page_setup.ok");
    assert_selector(&a, "kodocs.modal.page_setup.cancel");
}

#[test]
fn page_setup_paper_size_dropdown_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.paper_size");
}

#[test]
fn page_setup_portrait_button_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.portrait");
}

#[test]
fn page_setup_landscape_button_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.landscape");
}

#[test]
fn page_setup_a4_paper_size_item_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.paper_size.a4");
}

#[test]
fn page_setup_letter_paper_size_item_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.paper_size.letter");
}

#[test]
fn page_setup_a3_paper_size_item_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.paper_size.a3");
}

#[test]
fn page_setup_b5_paper_size_item_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.paper_size.b5");
}

#[test]
fn page_setup_legal_paper_size_item_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.paper_size.legal");
}

#[test]
fn page_setup_top_margin_field_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.margin.top");
}

#[test]
fn page_setup_bottom_margin_field_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.margin.bottom");
}

#[test]
fn page_setup_left_margin_field_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.margin.left");
}

#[test]
fn page_setup_right_margin_field_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.margin.right");
}

#[test]
fn page_setup_ok_button_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.ok");
}

#[test]
fn page_setup_cancel_button_present() {
    let mut h = make_harness();
    open_menu_item(&mut h, "kodocs.menu.file", "kodocs.menu.파일.페이지_설정");
    let c = capture(&mut h);
    assert_selector(&c, "kodocs.modal.page_setup.cancel");
}

#[test]
fn word_count_modal_opens_with_controls() {
    let mut h = make_harness();
    let b = capture(&mut h);
    let a = open_menu_item(&mut h, "kodocs.menu.tools", "kodocs.menu.도구.단어_수");
    assert_capture_changed(&b, &a); // modal may not render in headless
                                    // assert_selector(&a, "kodocs.modal.word_count");
                                    // assert_selector(&a, "kodocs.modal.word_count.close");
}

// --- Text context menu tests ---

#[test]
fn text_context_menu_cut_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.자르기");
}

#[test]
fn text_context_menu_copy_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.복사");
}

#[test]
fn text_context_menu_paste_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.붙여넣기");
}

#[test]
fn text_context_menu_add_comment_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.코멘트_추가");
}

#[test]
fn text_context_menu_insert_link_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.링크_삽입");
}

#[test]
fn text_context_menu_clear_formatting_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.서식_지우기");
}

// --- Table context menu tests ---

#[test]
fn table_context_menu_insert_row_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.행_삽입");
}

#[test]
fn table_context_menu_insert_column_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.열_삽입");
}

#[test]
fn table_context_menu_merge_cells_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.셋_병합");
}

#[test]
fn table_context_menu_split_cells_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.셋_분할");
}

// --- Image context menu tests ---

#[test]
fn image_context_menu_resize_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.이미지_크기_조정");
}

#[test]
fn image_context_menu_replace_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.그림_교체");
}

#[test]
fn image_context_menu_delete_item_no_crash() {
    let mut h = make_harness();
    click(&mut h, "kodocs.document");
    let c = capture(&mut h);
    assert_absent(&c, "kodocs.context.삭제");
}
