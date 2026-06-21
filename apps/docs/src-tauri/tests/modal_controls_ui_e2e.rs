/// UI automation tests for docs modal/dialog controls.
///
/// Uses debug_id selectors for modal open and internal controls.
/// Verifies modal appears in automation tree and contains expected sub-controls.
use tench_docs_lib::ui::DocsApp;
use tench_ui_automation_core::UiAutomationKey;
use tench_ui_test::assert_capture_changed;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::CaptureAssertions;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

#[test]
fn find_modal_opens_with_controls() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.find");
    assert_capture_changed(&before, &after);
    assert_selector(&after, "docs.modal.find_replace");
    assert_selector(&after, "docs.find.query");
    assert_selector(&after, "docs.find.close");
}

#[test]
fn link_modal_opens_with_controls() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    assert_capture_changed(&before, &after);
    assert_selector(&after, "docs.modal.link");
    assert_selector(&after, "docs.modal.link.url");
    assert_selector(&after, "docs.modal.link.ok");
    assert_selector(&after, "docs.modal.link.cancel");
}

#[test]
fn page_setup_modal_opens_with_controls() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    assert_capture_changed(&before, &after);
    assert_selector(&after, "docs.modal.page_setup");
    assert_selector(&after, "docs.modal.page_setup.ok");
    assert_selector(&after, "docs.modal.page_setup.cancel");
}

#[test]
fn word_count_modal_opens_with_controls() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.word_count",
    );
    assert_capture_changed(&before, &after);
    assert_selector(&after, "docs.modal.word_count");
    assert_selector(&after, "docs.modal.word_count.close");
}

#[test]
fn goto_modal_opens_with_controls() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");
    assert_capture_changed(&before, &after);
    assert_selector(&after, "docs.modal.goto");
    assert_selector(&after, "docs.goto.input");
}

#[test]
fn special_char_modal_opens_with_controls() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    assert_capture_changed(&before, &after);
    assert_selector(&after, "docs.modal.special_char");
    assert_selector(&after, "docs.modal.special_char.close");
}

// ── Link modal tests ──

#[test]
fn link_modal_url_field_starts_empty() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    let cap = capture(&mut harness);
    assert_eq!(
        get_node_value(&cap, "docs.modal.link.url"),
        Some("".to_string())
    );
}

#[test]
fn link_modal_url_field_reflects_typed_text() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    let after = type_text(&mut harness, "https://example.com");
    assert_eq!(
        get_node_value(&after, "docs.modal.link.url"),
        Some("https://example.com".to_string())
    );
    after.assert_png_valid();
}

#[test]
fn link_modal_cancel_dismisses() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    assert_selector(&capture(&mut harness), "docs.modal.link");

    let after = click(&mut harness, "docs.modal.link.cancel");
    assert_absent(&after, "docs.modal.link");
}

#[test]
fn link_modal_ok_button_is_present() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.modal.link.ok");
}

#[test]
fn link_modal_escape_closes() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    assert_selector(&capture(&mut harness), "docs.modal.link");

    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());
    assert_absent(&after, "docs.modal.link");
}

#[test]
fn link_modal_typing_does_not_edit_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Doc");
    let before = capture(&mut harness);
    let doc_before = get_node_label(&before, "docs.document").unwrap_or_default();

    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    type_text(&mut harness, "https://example.com");

    let after = capture(&mut harness);
    let doc_after = get_node_label(&after, "docs.document").unwrap_or_default();
    assert_eq!(
        doc_before, doc_after,
        "typing in link modal must not edit document"
    );
}

// ── Page setup modal tests ──

#[test]
fn page_setup_modal_has_orientation_buttons() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.modal.page_setup.portrait");
    assert_selector(&cap, "docs.modal.page_setup.landscape");
}

#[test]
fn page_setup_portrait_selected_by_default() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    let cap = capture(&mut harness);
    assert_eq!(
        get_node_value(&cap, "docs.modal.page_setup.portrait"),
        Some("selected".to_string())
    );
    assert_eq!(
        get_node_value(&cap, "docs.modal.page_setup.landscape"),
        Some("unselected".to_string())
    );
}

#[test]
fn page_setup_landscape_button_switches_orientation() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");

    let after = click(&mut harness, "docs.modal.page_setup.landscape");
    assert_eq!(
        get_node_value(&after, "docs.modal.page_setup.portrait"),
        Some("unselected".to_string())
    );
    assert_eq!(
        get_node_value(&after, "docs.modal.page_setup.landscape"),
        Some("selected".to_string())
    );
    after.assert_png_valid();
}

#[test]
fn page_setup_orientation_roundtrip() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");

    // Switch to landscape
    click(&mut harness, "docs.modal.page_setup.landscape");
    // Switch back to portrait
    let back = click(&mut harness, "docs.modal.page_setup.portrait");
    assert_eq!(
        get_node_value(&back, "docs.modal.page_setup.portrait"),
        Some("selected".to_string())
    );
    assert_eq!(
        get_node_value(&back, "docs.modal.page_setup.landscape"),
        Some("unselected".to_string())
    );
}

#[test]
fn page_setup_has_paper_size_options() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.modal.page_setup.paper.a4");
    assert_selector(&cap, "docs.modal.page_setup.paper.letter");
}

#[test]
fn page_setup_paper_a4_selected_by_default() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    let cap = capture(&mut harness);
    assert_eq!(
        get_node_value(&cap, "docs.modal.page_setup.paper.a4"),
        Some("selected".to_string())
    );
}

#[test]
fn page_setup_has_margin_fields() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.modal.page_setup.margin.top");
    assert_selector(&cap, "docs.modal.page_setup.margin.bottom");
    assert_selector(&cap, "docs.modal.page_setup.margin.left");
    assert_selector(&cap, "docs.modal.page_setup.margin.right");
}

#[test]
fn page_setup_margin_fields_have_default_values() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    let cap = capture(&mut harness);
    // Margin fields should have numeric values
    let top = get_node_value(&cap, "docs.modal.page_setup.margin.top").unwrap_or_default();
    assert!(!top.is_empty(), "margin top should have a value");
    // Verify it parses as a float
    top.parse::<f64>().expect("margin top should be a number");
}

#[test]
fn page_setup_cancel_dismisses() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    assert_selector(&capture(&mut harness), "docs.modal.page_setup");

    let after = click(&mut harness, "docs.modal.page_setup.cancel");
    assert_absent(&after, "docs.modal.page_setup");
}

#[test]
fn page_setup_ok_dismisses() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    assert_selector(&capture(&mut harness), "docs.modal.page_setup");

    let after = click(&mut harness, "docs.modal.page_setup.ok");
    assert_absent(&after, "docs.modal.page_setup");
}

#[test]
fn page_setup_escape_closes() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    assert_selector(&capture(&mut harness), "docs.modal.page_setup");

    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());
    assert_absent(&after, "docs.modal.page_setup");
}

// ── Special character modal tests ──

#[test]
fn special_char_modal_has_category_tabs() {
    let mut harness = make_harness();
    open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.modal.special_char.category.common_symbols");
    assert_selector(&cap, "docs.modal.special_char.category.arrows");
    assert_selector(&cap, "docs.modal.special_char.category.math");
}

#[test]
fn special_char_first_category_selected_by_default() {
    let mut harness = make_harness();
    open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    let cap = capture(&mut harness);
    assert_eq!(
        get_node_value(&cap, "docs.modal.special_char.category.common_symbols"),
        Some("selected".to_string())
    );
    assert_eq!(
        get_node_value(&cap, "docs.modal.special_char.category.arrows"),
        Some("unselected".to_string())
    );
}

#[test]
fn special_char_has_grid_cells() {
    let mut harness = make_harness();
    open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    let cap = capture(&mut harness);
    // First category (Common Symbols) should have grid cells
    assert_selector(&cap, "docs.modal.special_char.cell.0");
    assert_selector(&cap, "docs.modal.special_char.cell.1");
}

#[test]
fn special_char_grid_cells_have_character_values() {
    let mut harness = make_harness();
    open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    let cap = capture(&mut harness);
    let value = get_node_value(&cap, "docs.modal.special_char.cell.0").unwrap_or_default();
    assert!(!value.is_empty(), "cell.0 should have a character value");
}

#[test]
fn special_char_category_click_switches_selection() {
    let mut harness = make_harness();
    open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );

    let after = click(&mut harness, "docs.modal.special_char.category.arrows");
    assert_eq!(
        get_node_value(&after, "docs.modal.special_char.category.common_symbols"),
        Some("unselected".to_string())
    );
    assert_eq!(
        get_node_value(&after, "docs.modal.special_char.category.arrows"),
        Some("selected".to_string())
    );
    after.assert_png_valid();
}

#[test]
fn special_char_close_dismisses() {
    let mut harness = make_harness();
    open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    assert_selector(&capture(&mut harness), "docs.modal.special_char");

    let after = click(&mut harness, "docs.modal.special_char.close");
    assert_absent(&after, "docs.modal.special_char");
}

#[test]
fn special_char_escape_closes() {
    let mut harness = make_harness();
    open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    assert_selector(&capture(&mut harness), "docs.modal.special_char");

    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());
    assert_absent(&after, "docs.modal.special_char");
}
