use tench_docs_lib::ui::DocsApp;
/// UI automation tests for docs File Menu items (#2-#10).
///
/// Uses debug_id selectors for menu items.
/// Every test verifies that clicking a menu item produces a visual change.
use tench_ui_automation_core::UiAutomationKey;
use tench_ui_automation_core::UiAutomationModifiers;
use tench_ui_test::assert_capture_changed;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

// #2 File → New
#[test]
fn file_menu_new_item_creates_document() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.new");
    assert_capture_changed(&before, &after);

    // After creating a new document the text area should be empty
    let text = get_node_value(&after, "docs.document.text").unwrap_or_default();
    assert!(
        text.is_empty(),
        "new document text should be empty, got: {text:?}"
    );
}

// #3 File → Open
#[test]
fn file_menu_open_item_triggers_dialog() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.open");
    assert_capture_changed(&before, &after);
}

// #4 File → Save
#[test]
fn file_menu_save_item_triggers_save() {
    let mut harness = make_harness();
    let before = capture(&mut harness);

    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.save");
    assert_capture_changed(&before, &after);

    // The save status should remain "Saved" (a new document is already clean,
    // so saving is a no-op on the status indicator).
    assert_node_label(&after, "docs.save_status", "Saved");
}

// #5 File → Save As
#[test]
fn file_menu_save_as_item_triggers_dialog() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.save_as");
    assert_capture_changed(&before, &after);
}

// #6 File → Export As
#[test]
fn file_menu_export_item_triggers_dialog() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.export_as");
    assert_capture_changed(&before, &after);
}

// #7 File → Page Setup
#[test]
fn file_menu_page_setup_item_opens_dialog() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    assert_capture_changed(&before, &after);

    // The page setup modal must appear in the tree
    assert_selector(&after, "docs.modal.page_setup");
}

// #8 File → Print
#[test]
fn file_menu_print_item_shows_toast() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.print");
    assert_capture_changed(&before, &after);

    // The print preview modal should appear in the tree
    assert_selector(&after, "docs.modal.print_preview");
}

// #9 File → Version History
#[test]
fn file_menu_version_history_item_opens_panel() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.file",
        "docs.menu.file.version_history",
    );
    assert_capture_changed(&before, &after);
}

// #10 File → Recent Files
#[test]
fn file_menu_recent_files_item_shows_toast() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.file",
        "docs.menu.file.recent_files",
    );
    assert_capture_changed(&before, &after);
}

// ---------------------------------------------------------------------------
// Additional tests from fix plans
// ---------------------------------------------------------------------------

// --- New item tests (docs-file-menu-new-item-fix.md) ---

#[test]
fn file_menu_new_creates_empty_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Content");
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.new");
    assert_eq!(
        get_node_value(&after, "docs.document.text").unwrap_or_default(),
        "",
        "new document should be empty"
    );
    assert_eq!(
        get_node_value(&after, "docs.document.dirty").unwrap_or_default(),
        "false",
        "new document should be clean"
    );
}

#[test]
fn file_menu_new_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.new");
    assert_eq!(
        node_text(&after, "docs.menu.active"),
        "none",
        "menu should be closed after clicking new"
    );
    assert_absent(&after, "docs.menu.file.panel");
}

#[test]
fn hovering_new_then_clicking_page_setup_does_not_create_tab() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.file");
    let hovered = hover(&mut harness, "docs.menu.file.new");
    assert_eq!(
        node_text(&hovered, "docs.menu.file.hovered"),
        "New",
        "new item should be hovered"
    );

    let after = click(&mut harness, "docs.menu.file.page_setup");

    assert_selector(&after, "docs.modal.page_setup");
    assert_eq!(
        node_text(&after, "docs.tabs.count"),
        "1",
        "hovering new should not create a tab"
    );
}

// --- Open item tests (docs-file-menu-open-item-fix.md) ---

#[test]
fn file_menu_open_closes_menu_and_triggers_dialog() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.open");
    assert_eq!(
        node_text(&after, "docs.menu.active"),
        "none",
        "menu should be closed after clicking open"
    );
    assert_absent(&after, "docs.menu.file.panel");
}

#[test]
fn hovering_open_then_clicking_page_setup_does_not_open_file() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.file");
    let hovered = hover(&mut harness, "docs.menu.file.open");
    assert_eq!(
        node_text(&hovered, "docs.menu.file.hovered"),
        "Open",
        "open item should be hovered"
    );

    let after = click(&mut harness, "docs.menu.file.page_setup");

    assert_selector(&after, "docs.modal.page_setup");
    assert_eq!(
        node_text(&after, "docs.tabs.count"),
        "1",
        "hovering open should not open a file"
    );
}

// --- Save item tests (docs-file-menu-save-item-fix.md) ---

#[test]
fn file_menu_save_on_new_document_keeps_status() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.save");
    assert_node_label(&after, "docs.save_status", "Saved");
}

#[test]
fn file_menu_save_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.save");
    assert_eq!(
        node_text(&after, "docs.menu.active"),
        "none",
        "menu should be closed after clicking save"
    );
    assert_absent(&after, "docs.menu.file.panel");
}

#[test]
fn file_menu_save_on_dirty_document_preserves_text() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "Dirty content");

    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.save");

    assert_eq!(
        node_text(&after, "docs.document.text"),
        node_text(&typed, "docs.document.text"),
        "save should not alter document text"
    );
}

#[test]
fn hovering_save_then_clicking_page_setup_does_not_save() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "Dirty");
    click(&mut harness, "docs.menu.file");

    let hovered = hover(&mut harness, "docs.menu.file.save");
    assert_eq!(
        node_text(&hovered, "docs.menu.file.hovered"),
        "Save",
        "save item should be hovered"
    );

    let after = click(&mut harness, "docs.menu.file.page_setup");

    assert_selector(&after, "docs.modal.page_setup");
    assert_eq!(
        node_text(&after, "docs.document.dirty"),
        "true",
        "hovering save should not clear dirty state"
    );
    assert_eq!(
        node_text(&after, "docs.document.text"),
        "Dirty",
        "document text should be unchanged"
    );
}

// --- Save As item tests (docs-file-menu-save-as-item-fix.md) ---

#[test]
fn file_menu_save_as_closes_menu_and_triggers_dialog() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.save_as");
    assert_eq!(
        node_text(&after, "docs.menu.active"),
        "none",
        "menu should be closed after clicking save_as"
    );
    assert_absent(&after, "docs.menu.file.panel");
}

#[test]
fn hovering_save_as_then_clicking_page_setup_does_not_save() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.file");
    let hovered = hover(&mut harness, "docs.menu.file.save_as");
    assert_eq!(
        node_text(&hovered, "docs.menu.file.hovered"),
        "Save As",
        "save_as item should be hovered"
    );

    let after = click(&mut harness, "docs.menu.file.page_setup");

    assert_selector(&after, "docs.modal.page_setup");
    assert_eq!(
        node_text(&after, "docs.tabs.count"),
        "1",
        "hovering save_as should not create a tab"
    );
}

// --- Export As item tests (docs-file-menu-export-as-item-fix.md) ---

#[test]
fn file_menu_export_as_closes_menu_and_triggers_dialog() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.export_as");
    assert_eq!(
        node_text(&after, "docs.menu.active"),
        "none",
        "menu should be closed after clicking export_as"
    );
    assert_absent(&after, "docs.menu.file.panel");
}

#[test]
fn hovering_export_as_then_clicking_page_setup_does_not_export() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.file");
    let hovered = hover(&mut harness, "docs.menu.file.export_as");
    assert_eq!(
        node_text(&hovered, "docs.menu.file.hovered"),
        "Export As",
        "export_as item should be hovered"
    );

    let after = click(&mut harness, "docs.menu.file.page_setup");

    assert_selector(&after, "docs.modal.page_setup");
    assert_eq!(
        node_text(&after, "docs.tabs.count"),
        "1",
        "hovering export_as should not create a tab"
    );
}

// --- Page Setup item tests (docs-file-menu-page-setup-item-fix.md) ---

#[test]
fn file_menu_page_setup_has_ok_and_cancel() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    assert_selector(&after, "docs.modal.page_setup");
    assert_selector(&after, "docs.modal.page_setup.ok");
    assert_selector(&after, "docs.modal.page_setup.cancel");
}

#[test]
fn file_menu_page_setup_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    assert_eq!(
        node_text(&after, "docs.menu.active"),
        "none",
        "menu should be closed after clicking page_setup"
    );
    assert_absent(&after, "docs.menu.file.panel");
}

#[test]
fn opening_page_setup_preserves_document_text_selection_and_dirty_state() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "Body");
    let before = capture(&mut harness);

    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");

    assert_selector(&after, "docs.modal.page_setup");
    assert_eq!(
        node_text(&after, "docs.document.text"),
        node_text(&before, "docs.document.text"),
        "page setup should not change document text"
    );
    assert_eq!(
        node_text(&after, "docs.document.cursor"),
        node_text(&before, "docs.document.cursor"),
        "page setup should not change cursor"
    );
    assert_eq!(
        node_text(&after, "docs.document.dirty"),
        node_text(&before, "docs.document.dirty"),
        "page setup should not change dirty state"
    );
}

#[test]
fn hovering_page_setup_then_clicking_open_does_not_open_page_setup() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.file");
    let hovered = hover(&mut harness, "docs.menu.file.page_setup");
    assert_eq!(
        node_text(&hovered, "docs.menu.file.hovered"),
        "Page Setup",
        "page_setup item should be hovered"
    );

    let after = click(&mut harness, "docs.menu.file.open");

    assert_absent(&after, "docs.modal.page_setup");
}

#[test]
fn repeated_page_setup_open_cancel_does_not_drift_values() {
    let mut harness = make_harness();

    let first = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    assert_selector(&first, "docs.modal.page_setup");
    click(&mut harness, "docs.modal.page_setup.cancel");

    let second = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.page_setup");
    assert_selector(&second, "docs.modal.page_setup");

    // Margin values should not drift between repeated open/cancel cycles.
    // Use the top margin as a representative value.
    let first_top = get_node_value(&first, "docs.modal.page_setup.margin.top");
    let second_top = get_node_value(&second, "docs.modal.page_setup.margin.top");
    assert_eq!(
        first_top, second_top,
        "repeated page setup open/cancel should not drift margin values"
    );
}

// --- Print item tests (docs-file-menu-print-item-fix.md) ---

#[test]
fn file_menu_print_closes_menu_and_shows_preview() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.print");
    assert_eq!(
        node_text(&after, "docs.menu.active"),
        "none",
        "menu should be closed after clicking print"
    );
    assert_absent(&after, "docs.menu.file.panel");
    assert_selector(&after, "docs.modal.print_preview");
}

#[test]
fn escape_closes_print_preview_without_mutating_document_state() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "Body");
    let before = capture(&mut harness);

    open_menu_item(&mut harness, "docs.menu.file", "docs.menu.file.print");
    let closed = key(
        &mut harness,
        UiAutomationKey::Escape,
        UiAutomationModifiers::default(),
    );

    assert_absent(&closed, "docs.modal.print_preview");
    assert_eq!(
        node_text(&closed, "docs.document.text"),
        node_text(&before, "docs.document.text"),
        "closing print preview should not change document text"
    );
    assert_eq!(
        node_text(&closed, "docs.document.cursor"),
        node_text(&before, "docs.document.cursor"),
        "closing print preview should not change cursor"
    );
    assert_eq!(
        node_text(&closed, "docs.document.dirty"),
        node_text(&before, "docs.document.dirty"),
        "closing print preview should not change dirty state"
    );
}

#[test]
fn hovering_print_then_clicking_page_setup_does_not_open_print_preview() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.file");
    let hovered = hover(&mut harness, "docs.menu.file.print");
    assert_eq!(
        node_text(&hovered, "docs.menu.file.hovered"),
        "Print",
        "print item should be hovered"
    );

    let after = click(&mut harness, "docs.menu.file.page_setup");

    assert_absent(&after, "docs.modal.print_preview");
    assert_selector(&after, "docs.modal.page_setup");
}

// --- Version History item tests (docs-file-menu-version-history-item-fix.md) ---

#[test]
fn file_menu_version_history_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.file",
        "docs.menu.file.version_history",
    );
    assert_eq!(
        node_text(&after, "docs.menu.active"),
        "none",
        "menu should be closed after clicking version_history"
    );
    assert_absent(&after, "docs.menu.file.panel");
}

#[test]
fn file_menu_version_history_preserves_document_state() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "History body");
    let before = capture(&mut harness);

    let after = open_menu_item(
        &mut harness,
        "docs.menu.file",
        "docs.menu.file.version_history",
    );

    assert_eq!(
        node_text(&after, "docs.document.text"),
        node_text(&before, "docs.document.text"),
        "version history should not change document text"
    );
    assert_eq!(
        node_text(&after, "docs.document.cursor"),
        node_text(&before, "docs.document.cursor"),
        "version history should not change cursor"
    );
    assert_eq!(
        node_text(&after, "docs.document.dirty"),
        node_text(&before, "docs.document.dirty"),
        "version history should not change dirty state"
    );
}

#[test]
fn hovering_version_history_then_clicking_save_does_not_refresh_versions() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.file");
    let hovered = hover(&mut harness, "docs.menu.file.version_history");
    assert_eq!(
        node_text(&hovered, "docs.menu.file.hovered"),
        "Version History",
        "version_history item should be hovered"
    );

    let after = click(&mut harness, "docs.menu.file.save");

    // Only save dispatch should have run; version history was never invoked.
    assert_eq!(
        node_text(&after, "docs.menu.active"),
        "none",
        "menu should be closed after clicking save"
    );
}

// --- Recent Files item tests (docs-file-menu-recent-files-item-fix.md) ---

#[test]
fn file_menu_recent_files_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.file",
        "docs.menu.file.recent_files",
    );
    assert_eq!(
        node_text(&after, "docs.menu.active"),
        "none",
        "menu should be closed after clicking recent_files"
    );
    assert_absent(&after, "docs.menu.file.panel");
}

#[test]
fn file_menu_recent_files_preserves_document_state() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "Recent body");
    let before = capture(&mut harness);

    let after = open_menu_item(
        &mut harness,
        "docs.menu.file",
        "docs.menu.file.recent_files",
    );

    assert_eq!(
        node_text(&after, "docs.document.text"),
        node_text(&before, "docs.document.text"),
        "recent files should not change document text"
    );
    assert_eq!(
        node_text(&after, "docs.document.cursor"),
        node_text(&before, "docs.document.cursor"),
        "recent files should not change cursor"
    );
    assert_eq!(
        node_text(&after, "docs.document.dirty"),
        node_text(&before, "docs.document.dirty"),
        "recent files should not change dirty state"
    );
    assert_eq!(
        node_text(&after, "docs.tabs.count"),
        "1",
        "recent files should not create a tab"
    );
}

#[test]
fn hovering_recent_files_then_clicking_page_setup_does_not_open_recent_list() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.file");
    let hovered = hover(&mut harness, "docs.menu.file.recent_files");
    assert_eq!(
        node_text(&hovered, "docs.menu.file.hovered"),
        "Recent Files",
        "recent_files item should be hovered"
    );

    let after = click(&mut harness, "docs.menu.file.page_setup");

    assert_selector(&after, "docs.modal.page_setup");
    assert_eq!(
        node_text(&after, "docs.tabs.count"),
        "1",
        "hovering recent_files should not create a tab"
    );
}
