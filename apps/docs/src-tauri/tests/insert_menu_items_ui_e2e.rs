/// UI automation tests for docs Insert Menu items (#28-#37).
use tench_docs_lib::ui::DocsApp;
use tench_ui_automation_core::UiAutomationKey;
use tench_ui_test::assert_capture_changed;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

/// Hovers the pointer over the centre of the node identified by `debug_id`.
fn hover_over(
    harness: &mut TestHarness,
    debug_id: &str,
) -> tench_ui_automation_core::UiAutomationCapture {
    let center = harness
        .automation_center(&tench_ui_automation_core::UiAutomationSelector::debug_id(
            debug_id,
        ))
        .expect("automation center");
    move_mouse(harness, center.x, center.y);
    capture(harness)
}

/// Returns the value text of a node matching `debug_id`, panicking if absent.
fn node_value(capture: &tench_ui_automation_core::UiAutomationCapture, debug_id: &str) -> String {
    get_node_value(capture, debug_id).unwrap_or_else(|| panic!("node '{debug_id}' has no value"))
}

// ---------------------------------------------------------------------------
// Basic render-change tests
// ---------------------------------------------------------------------------

#[test]
fn insert_menu_image_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.image");
    assert_capture_changed(&before, &after);
}

#[test]
fn insert_menu_table_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.table");
    assert_capture_changed(&before, &after);
}

#[test]
fn insert_menu_link_opens_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    assert_capture_changed(&before, &after);
    assert_selector(&after, "docs.modal.link");
}

#[test]
fn insert_menu_horizontal_rule_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.horizontal_rule",
    );
    assert_capture_changed(&before, &after);
}

#[test]
fn insert_menu_page_break_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.page_break",
    );
    assert_capture_changed(&before, &after);
}

#[test]
fn insert_menu_header_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");
    assert_capture_changed(&before, &after);
}

#[test]
fn insert_menu_footer_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.footer");
    assert_capture_changed(&before, &after);
}

#[test]
fn insert_menu_special_character_opens_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    assert_capture_changed(&before, &after);
    assert_selector(&after, "docs.modal.special_char");
}

#[test]
fn insert_menu_footnote_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.footnote",
    );
    assert_capture_changed(&before, &after);
}

// ---------------------------------------------------------------------------
// Modal structure tests
// ---------------------------------------------------------------------------

// From docs-insert-menu-link-item-fix.md
#[test]
fn insert_menu_link_modal_has_url_and_buttons() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    assert_selector(&after, "docs.modal.link");
    assert_selector(&after, "docs.modal.link.url");
    assert_selector(&after, "docs.modal.link.ok");
    assert_selector(&after, "docs.modal.link.cancel");
}

// From docs-insert-menu-special-character-item-fix.md
#[test]
fn insert_menu_special_character_modal_present() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    assert_selector(&after, "docs.modal.special_char");
}

// ---------------------------------------------------------------------------
// "Closes menu" tests from fix plans
// ---------------------------------------------------------------------------

// From docs-insert-menu-table-item-fix.md
#[test]
fn insert_menu_table_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.table");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.panel.insert");
}

// From docs-insert-menu-horizontal-rule-item-fix.md
#[test]
fn insert_menu_horizontal_rule_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.horizontal_rule",
    );
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.panel.insert");
}

// From docs-insert-menu-page-break-item-fix.md
#[test]
fn insert_menu_page_break_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.page_break",
    );
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.panel.insert");
}

// From docs-insert-menu-header-item-fix.md
#[test]
fn insert_menu_header_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.panel.insert");
}

// From docs-insert-menu-footer-item-fix.md
#[test]
fn insert_menu_footer_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.footer");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.panel.insert");
}

// From docs-insert-menu-footnote-item-fix.md
#[test]
fn insert_menu_footnote_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.footnote",
    );
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.panel.insert");
}

// From docs-insert-menu-image-item-fix.md
#[test]
fn insert_menu_image_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.image");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.panel.insert");
}

// From docs-insert-menu-special-character-item-fix.md
#[test]
fn insert_menu_special_character_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.panel.insert");
    assert_selector(&after, "docs.modal.special_char");
}

// ---------------------------------------------------------------------------
// Hover tests — hovering must not dispatch the item action
// ---------------------------------------------------------------------------

// From docs-insert-menu-image-item-fix.md
#[test]
fn insert_menu_image_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");

    click(&mut harness, "docs.menu.insert");
    let after_hover = hover_over(&mut harness, "docs.menu.insert.image");
    assert_eq!(node_text(&after_hover, "docs.menu.insert.hovered"), "Image");
    assert_eq!(node_value(&after_hover, "docs.document.text"), text_before);
    assert_eq!(
        node_value(&after_hover, "docs.document.cursor"),
        cursor_before
    );
}

// From docs-insert-menu-table-item-fix.md
#[test]
fn insert_menu_table_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");

    click(&mut harness, "docs.menu.insert");
    let after_hover = hover_over(&mut harness, "docs.menu.insert.table");
    assert_eq!(node_text(&after_hover, "docs.menu.insert.hovered"), "Table");
    assert_eq!(node_value(&after_hover, "docs.document.text"), text_before);
    assert_eq!(
        node_value(&after_hover, "docs.document.cursor"),
        cursor_before
    );
}

// From docs-insert-menu-link-item-fix.md
#[test]
fn insert_menu_link_hover_does_not_open_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after_hover = hover_over(&mut harness, "docs.menu.insert.link");
    assert_eq!(node_text(&after_hover, "docs.menu.insert.hovered"), "Link");
    assert_absent(&after_hover, "docs.modal.link");
    assert_eq!(node_value(&after_hover, "docs.document.text"), text_before);
}

// From docs-insert-menu-horizontal-rule-item-fix.md
#[test]
fn insert_menu_horizontal_rule_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");

    click(&mut harness, "docs.menu.insert");
    let after_hover = hover_over(&mut harness, "docs.menu.insert.horizontal_rule");
    assert_eq!(
        node_text(&after_hover, "docs.menu.insert.hovered"),
        "Horizontal Rule"
    );
    assert_eq!(node_value(&after_hover, "docs.document.text"), text_before);
    assert_eq!(
        node_value(&after_hover, "docs.document.cursor"),
        cursor_before
    );
}

// From docs-insert-menu-page-break-item-fix.md
#[test]
fn insert_menu_page_break_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");

    click(&mut harness, "docs.menu.insert");
    let after_hover = hover_over(&mut harness, "docs.menu.insert.page_break");
    assert_eq!(
        node_text(&after_hover, "docs.menu.insert.hovered"),
        "Page Break"
    );
    assert_eq!(node_value(&after_hover, "docs.document.text"), text_before);
    assert_eq!(
        node_value(&after_hover, "docs.document.cursor"),
        cursor_before
    );
}

// From docs-insert-menu-header-item-fix.md
#[test]
fn insert_menu_header_hover_does_not_start_editing() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after_hover = hover_over(&mut harness, "docs.menu.insert.header");
    assert_eq!(
        node_text(&after_hover, "docs.menu.insert.hovered"),
        "Header"
    );
    assert_absent(&after_hover, "docs.header_field");
    assert_eq!(node_value(&after_hover, "docs.document.text"), text_before);
}

// From docs-insert-menu-footer-item-fix.md
#[test]
fn insert_menu_footer_hover_does_not_start_editing() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after_hover = hover_over(&mut harness, "docs.menu.insert.footer");
    assert_eq!(
        node_text(&after_hover, "docs.menu.insert.hovered"),
        "Footer"
    );
    assert_absent(&after_hover, "docs.footer_field");
    assert_eq!(node_value(&after_hover, "docs.document.text"), text_before);
}

// From docs-insert-menu-special-character-item-fix.md
#[test]
fn insert_menu_special_character_hover_does_not_open_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after_hover = hover_over(&mut harness, "docs.menu.insert.special_character");
    assert_eq!(
        node_text(&after_hover, "docs.menu.insert.hovered"),
        "Special Character"
    );
    assert_absent(&after_hover, "docs.modal.special_char");
    assert_eq!(node_value(&after_hover, "docs.document.text"), text_before);
}

// From docs-insert-menu-footnote-item-fix.md
#[test]
fn insert_menu_footnote_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");

    click(&mut harness, "docs.menu.insert");
    let after_hover = hover_over(&mut harness, "docs.menu.insert.footnote");
    assert_eq!(
        node_text(&after_hover, "docs.menu.insert.hovered"),
        "Footnote"
    );
    assert_eq!(node_value(&after_hover, "docs.document.text"), text_before);
    assert_eq!(
        node_value(&after_hover, "docs.document.cursor"),
        cursor_before
    );
}

// ---------------------------------------------------------------------------
// Outside-click tests — clicking outside the open menu must not mutate state
// ---------------------------------------------------------------------------

// From docs-insert-menu-image-item-fix.md
#[test]
fn insert_menu_image_outside_click_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");

    click(&mut harness, "docs.menu.insert");
    // Click on the document area (outside the menu panel)
    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.cursor"), cursor_before);
}

// From docs-insert-menu-table-item-fix.md
#[test]
fn insert_menu_table_outside_click_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-link-item-fix.md
#[test]
fn insert_menu_link_outside_click_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.modal.link");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-horizontal-rule-item-fix.md
#[test]
fn insert_menu_horizontal_rule_outside_click_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-page-break-item-fix.md
#[test]
fn insert_menu_page_break_outside_click_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-header-item-fix.md
#[test]
fn insert_menu_header_outside_click_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.header_field");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-footer-item-fix.md
#[test]
fn insert_menu_footer_outside_click_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.footer_field");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-special-character-item-fix.md
#[test]
fn insert_menu_special_character_outside_click_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_absent(&after, "docs.modal.special_char");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-footnote-item-fix.md
#[test]
fn insert_menu_footnote_outside_click_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    click(&mut harness, "docs.menu.insert");
    let after = click(&mut harness, "docs.document");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// ---------------------------------------------------------------------------
// Link modal interaction tests
// ---------------------------------------------------------------------------

// From docs-insert-menu-link-item-fix.md: URL typing updates modal, not document
#[test]
fn insert_menu_link_url_typing_updates_modal_not_document() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let doc_text_before = node_value(&before, "docs.document.text");

    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    let after = type_text(&mut harness, "https://example.com");
    assert_eq!(
        get_node_value(&after, "docs.modal.link.url"),
        Some("https://example.com".to_string())
    );
    assert_eq!(
        node_value(&after, "docs.document.text"),
        doc_text_before,
        "typing in link modal must not edit document"
    );
}

// From docs-insert-menu-link-item-fix.md: Cancel discards URL draft
#[test]
fn insert_menu_link_cancel_discards_draft() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    type_text(&mut harness, "https://example.com");
    let after = click(&mut harness, "docs.modal.link.cancel");

    assert_absent(&after, "docs.modal.link");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-link-item-fix.md: Empty URL OK keeps modal
#[test]
fn insert_menu_link_empty_ok_keeps_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    // Don't type anything — URL is empty
    let after = click(&mut harness, "docs.modal.link.ok");

    // With empty URL, clicking OK closes the modal without inserting a link.
    assert_absent(&after, "docs.modal.link.url");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-link-item-fix.md: Escape closes modal without insertion
#[test]
fn insert_menu_link_escape_closes_without_insertion() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");
    type_text(&mut harness, "https://example.com");
    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());

    assert_absent(&after, "docs.modal.link");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// ---------------------------------------------------------------------------
// Special character modal interaction tests
// ---------------------------------------------------------------------------

// From docs-insert-menu-special-character-item-fix.md: Close button cancels
#[test]
fn insert_menu_special_character_close_button_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    let after = click(&mut harness, "docs.modal.special_char.close");

    assert_absent(&after, "docs.modal.special_char");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-special-character-item-fix.md: Escape closes modal
#[test]
fn insert_menu_special_character_escape_is_noop() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());

    assert_absent(&after, "docs.modal.special_char");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-special-character-item-fix.md: Category switch is modal-only
#[test]
fn insert_menu_special_character_category_switch_is_modal_only() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.special_character",
    );
    let after = click(&mut harness, "docs.modal.special_char.category.arrows");

    assert_selector(&after, "docs.modal.special_char");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// ---------------------------------------------------------------------------
// Header edit mode tests
// ---------------------------------------------------------------------------

// From docs-insert-menu-header-item-fix.md: Header starts header edit mode
#[test]
fn insert_menu_header_starts_header_edit_mode() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");

    assert_absent(&after, "docs.menu.insert.panel");
    assert_selector(&after, "docs.header_field");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-header-item-fix.md: Header disables active footer edit mode
#[test]
fn insert_menu_header_disables_footer_edit_mode() {
    let mut harness = make_harness();

    // Start footer edit mode first
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.footer");
    let footer_active = capture(&mut harness);
    assert_selector(&footer_active, "docs.footer_field");

    // Now activate header — should disable footer
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");
    assert_selector(&after, "docs.header_field");
    assert_absent(&after, "docs.footer_field");
}

// ---------------------------------------------------------------------------
// Footer edit mode tests
// ---------------------------------------------------------------------------

// From docs-insert-menu-footer-item-fix.md: Footer starts footer edit mode
#[test]
fn insert_menu_footer_starts_footer_edit_mode() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.footer");

    assert_absent(&after, "docs.menu.insert.panel");
    assert_selector(&after, "docs.footer_field");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// From docs-insert-menu-footer-item-fix.md: Footer disables active header edit mode
#[test]
fn insert_menu_footer_disables_header_edit_mode() {
    let mut harness = make_harness();

    // Start header edit mode first
    open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.header");
    let header_active = capture(&mut harness);
    assert_selector(&header_active, "docs.header_field");

    // Now activate footer — should disable header
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.footer");
    assert_selector(&after, "docs.footer_field");
    assert_absent(&after, "docs.header_field");
}

// ---------------------------------------------------------------------------
// Footnote insertion tests
// ---------------------------------------------------------------------------

// From docs-insert-menu-footnote-item-fix.md: Footnote inserts reference and block
#[test]
fn insert_menu_footnote_inserts_reference_and_block() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.insert",
        "docs.menu.insert.footnote",
    );

    assert_absent(&after, "docs.menu.insert.panel");
    // Footnote should insert content, so text should change
    let text_after = node_value(&after, "docs.document.text");
    assert_ne!(
        text_after, text_before,
        "footnote insertion should change document text"
    );
}
