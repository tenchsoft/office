/// UI automation tests for docs Menu Buttons (#11, #21, #28, #38, #47, #53).
///
/// Uses debug_id selectors instead of hardcoded coordinates.
/// Every test verifies the menu button selector exists and that clicking it
/// produces a visual change (dropdown appears).
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::assert_capture_changed;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

#[test]
fn edit_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.menu.edit");

    let after = click(&mut harness, "docs.menu.edit");
    assert_capture_changed(&before, &after);
}

#[test]
fn view_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.menu.view");

    let after = click(&mut harness, "docs.menu.view");
    assert_capture_changed(&before, &after);
}

#[test]
fn insert_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.menu.insert");

    let after = click(&mut harness, "docs.menu.insert");
    assert_capture_changed(&before, &after);
}

#[test]
fn format_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.menu.format");

    let after = click(&mut harness, "docs.menu.format");
    assert_capture_changed(&before, &after);
}

#[test]
fn tools_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.menu.tools");

    let after = click(&mut harness, "docs.menu.tools");
    assert_capture_changed(&before, &after);
}

#[test]
fn help_menu_button_opens_dropdown() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.menu.help");

    let after = click(&mut harness, "docs.menu.help");
    assert_capture_changed(&before, &after);
}

// --- Strengthened tests: verify panel opens and active label ---

#[test]
fn edit_menu_button_opens_and_shows_panel() {
    let mut harness = make_harness();
    let after = click(&mut harness, "docs.menu.edit");
    assert_eq!(node_text(&after, "docs.menu.active"), "Edit");
    assert_selector(&after, "docs.menu.edit.panel");
}

#[test]
fn file_menu_button_opens_and_shows_panel() {
    let mut harness = make_harness();
    let after = click(&mut harness, "docs.menu.file");
    assert_eq!(node_text(&after, "docs.menu.active"), "File");
    assert_selector(&after, "docs.menu.file.panel");
}

#[test]
fn format_menu_button_opens_and_shows_panel() {
    let mut harness = make_harness();
    let after = click(&mut harness, "docs.menu.format");
    assert_eq!(node_text(&after, "docs.menu.active"), "Format");
    assert_selector(&after, "docs.menu.format.panel");
}

#[test]
fn insert_menu_button_opens_and_shows_panel() {
    let mut harness = make_harness();
    let after = click(&mut harness, "docs.menu.insert");
    assert_eq!(node_text(&after, "docs.menu.active"), "Insert");
    assert_selector(&after, "docs.menu.insert.panel");
}

#[test]
fn tools_menu_button_opens_and_shows_panel() {
    let mut harness = make_harness();
    let after = click(&mut harness, "docs.menu.tools");
    assert_eq!(node_text(&after, "docs.menu.active"), "Tools");
    assert_selector(&after, "docs.menu.tools.panel");
}

#[test]
fn help_menu_button_opens_and_shows_panel() {
    let mut harness = make_harness();
    let after = click(&mut harness, "docs.menu.help");
    assert_eq!(node_text(&after, "docs.menu.active"), "Help");
    assert_selector(&after, "docs.menu.help.panel");
}

// --- Helpers for hover / document-state assertions ---

/// Returns the `value` field of a node identified by `debug_id`.
///
/// Panics if the node is not found or has no value.
fn node_value(capture: &tench_ui_automation_core::UiAutomationCapture, debug_id: &str) -> String {
    get_node_value(capture, debug_id).unwrap_or_else(|| panic!("node '{debug_id}' has no value"))
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

// --- Format menu fix-plan tests (#47) ---

#[test]
fn format_menu_hover_changes_highlight_only() {
    let mut harness = make_harness();

    // Open Format menu
    let open = click(&mut harness, "docs.menu.format");
    assert_eq!(node_text(&open, "docs.menu.active"), "Format");

    // Snapshot document state before any hover
    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");

    // Hover bold
    let hover_bold = hover_over(&mut harness, "docs.menu.format.bold");
    assert_eq!(node_text(&hover_bold, "docs.menu.format.hovered"), "Bold");
    assert_eq!(node_value(&hover_bold, "docs.document.text"), text_before);
    assert_eq!(
        node_value(&hover_bold, "docs.document.cursor"),
        cursor_before
    );

    // Hover block_quote
    let hover_bq = hover_over(&mut harness, "docs.menu.format.block_quote");
    assert_eq!(
        node_text(&hover_bq, "docs.menu.format.hovered"),
        "Block Quote"
    );
    assert_eq!(node_value(&hover_bq, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover_bq, "docs.document.cursor"), cursor_before);
}

#[test]
fn format_menu_outside_click_closes_without_mutation() {
    let mut harness = make_harness();

    // Open Format menu
    let open = click(&mut harness, "docs.menu.format");
    assert_selector(&open, "docs.menu.format.panel");

    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");
    let selection_before = node_value(&open, "docs.document.selection");

    // Click outside the menu (on the document canvas)
    let after = click(&mut harness, "docs.document");

    // Menu should be closed
    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.format.panel");

    // Document must be unchanged
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before
    );
}

// --- Insert menu fix-plan tests (#28) ---

#[test]
fn insert_menu_hover_changes_highlight_only() {
    let mut harness = make_harness();

    // Open Insert menu
    let open = click(&mut harness, "docs.menu.insert");
    assert_eq!(node_text(&open, "docs.menu.active"), "Insert");

    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");

    // Hover image
    let hover_image = hover_over(&mut harness, "docs.menu.insert.image");
    assert_eq!(node_text(&hover_image, "docs.menu.insert.hovered"), "Image");
    assert_eq!(node_value(&hover_image, "docs.document.text"), text_before);
    assert_eq!(
        node_value(&hover_image, "docs.document.cursor"),
        cursor_before
    );

    // Hover footnote
    let hover_fn = hover_over(&mut harness, "docs.menu.insert.footnote");
    assert_eq!(node_text(&hover_fn, "docs.menu.insert.hovered"), "Footnote");
    assert_eq!(node_value(&hover_fn, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover_fn, "docs.document.cursor"), cursor_before);
}

#[test]
fn insert_menu_item_dispatch_closes_before_link_modal() {
    let mut harness = make_harness();

    // Open Insert menu and click link
    let after = open_menu_item(&mut harness, "docs.menu.insert", "docs.menu.insert.link");

    // Menu should be closed (panel absent); modal is now active
    assert_absent(&after, "docs.menu.insert.panel");

    // Link modal should be present
    assert_selector(&after, "docs.modal.link");
}

// --- Help menu fix-plan tests (#53) ---

#[test]
fn help_menu_hover_changes_highlight_only() {
    let mut harness = make_harness();

    // Open Help menu
    let open = click(&mut harness, "docs.menu.help");
    assert_eq!(node_text(&open, "docs.menu.active"), "Help");

    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");

    // Hover about
    let hover_about = hover_over(&mut harness, "docs.menu.help.about");
    assert_eq!(node_text(&hover_about, "docs.menu.help.hovered"), "About");
    assert_eq!(node_value(&hover_about, "docs.document.text"), text_before);
    assert_eq!(
        node_value(&hover_about, "docs.document.cursor"),
        cursor_before
    );

    // Hover keyboard_shortcuts
    let hover_ks = hover_over(&mut harness, "docs.menu.help.keyboard_shortcuts");
    assert_eq!(
        node_text(&hover_ks, "docs.menu.help.hovered"),
        "Keyboard Shortcuts"
    );
    assert_eq!(node_value(&hover_ks, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover_ks, "docs.document.cursor"), cursor_before);
}

#[test]
fn help_menu_item_dispatch_closes_before_about() {
    let mut harness = make_harness();

    // Open Help menu and click about
    let after = open_menu_item(&mut harness, "docs.menu.help", "docs.menu.help.about");

    // Menu should be closed (panel absent); modal is now active
    assert_absent(&after, "docs.menu.help.panel");
}

// --- Format menu expected-items test (#47) ---

#[test]
fn format_menu_button_opens_with_expected_items() {
    let mut harness = make_harness();

    let after = click(&mut harness, "docs.menu.format");
    assert_eq!(node_text(&after, "docs.menu.active"), "Format");
    assert_selector(&after, "docs.menu.format.panel");

    let items = [
        "docs.menu.format.bold",
        "docs.menu.format.italic",
        "docs.menu.format.underline",
        "docs.menu.format.strikethrough",
        "docs.menu.format.superscript",
        "docs.menu.format.subscript",
        "docs.menu.format.clear_formatting",
        "docs.menu.format.block_quote",
    ];
    for item in &items {
        assert_selector(&after, item);
    }
}

// --- Insert menu expected-items test (#28) ---

#[test]
fn insert_menu_button_opens_with_expected_items() {
    let mut harness = make_harness();

    let after = click(&mut harness, "docs.menu.insert");
    assert_eq!(node_text(&after, "docs.menu.active"), "Insert");
    assert_selector(&after, "docs.menu.insert.panel");

    let items = [
        "docs.menu.insert.image",
        "docs.menu.insert.table",
        "docs.menu.insert.link",
        "docs.menu.insert.horizontal_rule",
        "docs.menu.insert.page_break",
        "docs.menu.insert.header",
        "docs.menu.insert.footer",
        "docs.menu.insert.special_character",
        "docs.menu.insert.footnote",
    ];
    for item in &items {
        assert_selector(&after, item);
    }
}

// --- Help menu expected-items test (#53) ---

#[test]
fn help_menu_button_opens_with_expected_items() {
    let mut harness = make_harness();

    let after = click(&mut harness, "docs.menu.help");
    assert_eq!(node_text(&after, "docs.menu.active"), "Help");
    assert_selector(&after, "docs.menu.help.panel");

    let items = ["docs.menu.help.about", "docs.menu.help.keyboard_shortcuts"];
    for item in &items {
        assert_selector(&after, item);
    }
}

// --- Help menu outside-click preserves document state (#53) ---

#[test]
fn help_menu_outside_click_closes_without_mutation() {
    let mut harness = make_harness();

    // Type some text so there is meaningful document state to preserve
    let typed = type_text(&mut harness, "Hello");
    let text_before = node_value(&typed, "docs.document.text");
    let cursor_before = node_value(&typed, "docs.document.cursor");

    // Open Help menu
    let open = click(&mut harness, "docs.menu.help");
    assert_selector(&open, "docs.menu.help.panel");

    // Click outside the menu (on the document canvas)
    let after = click(&mut harness, "docs.document");

    // Menu should be closed
    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.help.panel");

    // Document must be unchanged
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.cursor"), cursor_before);
}

// --- Cross-menu stale-hover tests (#47, #28, #53) ---

#[test]
fn format_menu_switch_to_file_clears_stale_hover() {
    let mut harness = make_harness();

    // Open Format menu
    let open = click(&mut harness, "docs.menu.format");
    assert_eq!(node_text(&open, "docs.menu.active"), "Format");
    assert_selector(&open, "docs.menu.format.panel");

    // Hover bold to establish a hovered item
    let hover_bold = hover_over(&mut harness, "docs.menu.format.bold");
    assert_eq!(node_text(&hover_bold, "docs.menu.format.hovered"), "Bold");

    // Click File menu — format panel must close, file becomes active
    let after = click(&mut harness, "docs.menu.file");
    assert_absent(&after, "docs.menu.format.panel");
    assert_eq!(node_text(&after, "docs.menu.active"), "File");
    assert_eq!(node_text(&after, "docs.menu.file.hovered"), "none");
}

#[test]
fn insert_menu_switch_to_format_clears_stale_hover() {
    let mut harness = make_harness();

    // Open Insert menu
    let open = click(&mut harness, "docs.menu.insert");
    assert_eq!(node_text(&open, "docs.menu.active"), "Insert");
    assert_selector(&open, "docs.menu.insert.panel");

    // Hover image to establish a hovered item
    let hover_image = hover_over(&mut harness, "docs.menu.insert.image");
    assert_eq!(node_text(&hover_image, "docs.menu.insert.hovered"), "Image");

    // Click Format menu — insert panel must close, format becomes active
    let after = click(&mut harness, "docs.menu.format");
    assert_absent(&after, "docs.menu.insert.panel");
    assert_eq!(node_text(&after, "docs.menu.active"), "Format");
}

#[test]
fn help_menu_switch_to_file_clears_stale_hover() {
    let mut harness = make_harness();

    // Open Help menu
    let open = click(&mut harness, "docs.menu.help");
    assert_eq!(node_text(&open, "docs.menu.active"), "Help");
    assert_selector(&open, "docs.menu.help.panel");

    // Hover about to establish a hovered item
    let hover_about = hover_over(&mut harness, "docs.menu.help.about");
    assert_eq!(node_text(&hover_about, "docs.menu.help.hovered"), "About");

    // Click File menu — help panel must close, file becomes active
    let after = click(&mut harness, "docs.menu.file");
    assert_absent(&after, "docs.menu.help.panel");
    assert_eq!(node_text(&after, "docs.menu.active"), "File");
}
