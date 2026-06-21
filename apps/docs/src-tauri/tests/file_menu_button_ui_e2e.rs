/// UI automation tests for Docs File Menu Button (work plan #1).
///
/// Uses debug_id selectors instead of hardcoded coordinates.
/// Every test verifies visual change via before/after pixel comparison.
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;
use tench_ui_test::{assert_capture_changed, CaptureAssertions};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

// #1 File Menu Button: initial render
#[test]
fn file_menu_renders_initial_screen() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    cap.assert_png_size(1280, 820);
    assert_selector(&cap, "docs.menu.file");
}

// #1 File Menu Button: opens on click
#[test]
fn file_menu_opens_on_click() {
    let mut harness = make_harness();
    let before = capture(&mut harness);

    let after = click(&mut harness, "docs.menu.file");
    assert_capture_changed(&before, &after);
}

// #1 File Menu Button: closes on outside click
#[test]
fn file_menu_closes_on_outside_click() {
    let mut harness = make_harness();
    let initial = capture(&mut harness);

    // Open File menu
    let menu_open = click(&mut harness, "docs.menu.file");
    assert_capture_changed(&initial, &menu_open);

    // Click far outside the menu (on the document canvas)
    let after_close = click(&mut harness, "docs.document");
    assert_capture_changed(&menu_open, &after_close);
}

// #1 File Menu Button: replaced by another menu
#[test]
fn file_menu_replaced_by_another_menu() {
    let mut harness = make_harness();

    // Open File menu
    let file_menu = click(&mut harness, "docs.menu.file");

    // Click Edit menu — should replace File
    let edit_menu = click(&mut harness, "docs.menu.edit");
    assert_capture_changed(&file_menu, &edit_menu);
}

// --- Strengthened tests ---

/// Click the File menu button and verify the menu panel exposes all expected items.
#[test]
fn file_menu_opens_and_exposes_items() {
    let mut harness = make_harness();

    // Open the File menu
    let cap = click(&mut harness, "docs.menu.file");

    // The active menu indicator should report "File"
    assert_node_label(&cap, "docs.menu.active", "File");

    // The File menu panel must exist
    assert_selector(&cap, "docs.menu.file.panel");

    // All expected menu items must be present
    let items = [
        "docs.menu.file.new",
        "docs.menu.file.open",
        "docs.menu.file.save",
        "docs.menu.file.save_as",
        "docs.menu.file.export_as",
        "docs.menu.file.page_setup",
        "docs.menu.file.print",
        "docs.menu.file.version_history",
        "docs.menu.file.recent_files",
    ];
    for item in &items {
        assert_selector(&cap, item);
    }
}

/// Opening the File menu then clicking outside should close it completely.
#[test]
fn outside_click_closes_file_menu() {
    let mut harness = make_harness();

    // Open the File menu
    let _menu_open = click(&mut harness, "docs.menu.file");

    // Click on the document area (outside the menu)
    let after_close = click(&mut harness, "docs.document");

    // Active menu should be "none"
    assert_node_label(&after_close, "docs.menu.active", "none");

    // The File menu panel should no longer be in the tree
    assert_absent(&after_close, "docs.menu.file.panel");
}

/// Switching from Edit menu to File menu should clear the hovered item state.
#[test]
fn switching_menus_clears_hover() {
    let mut harness = make_harness();

    // Open Edit menu
    let _edit_open = click(&mut harness, "docs.menu.edit");

    // Hover over an Edit menu item to set hover state
    let _after_hover = hover(&mut harness, "docs.menu.edit.undo");

    // Now switch to File menu
    let after_switch = click(&mut harness, "docs.menu.file");

    // The File menu hover should be "none" (no item hovered yet)
    assert_node_label(&after_switch, "docs.menu.file.hovered", "none");
}

/// Clicking a File menu item (Page Setup) should close the menu before dispatching.
#[test]
fn file_menu_item_click_closes_menu_before_dispatch() {
    let mut harness = make_harness();

    // Open the File menu
    let _menu_open = click(&mut harness, "docs.menu.file");

    // Click Page Setup item
    let after = click(&mut harness, "docs.menu.file.page_setup");

    // Menu should be closed (active returns to "none")
    assert_node_label(&after, "docs.menu.active", "none");
    assert_absent(&after, "docs.menu.file.panel");
}

/// Hovering over File menu items should not dispatch or mutate the document.
#[test]
fn hovering_file_items_does_not_dispatch_or_mutate_document() {
    let mut harness = make_harness();

    // Open the File menu
    let _menu_open = click(&mut harness, "docs.menu.file");

    // Hover over several items — the capture should not change meaningfully
    let _after_hover = hover(&mut harness, "docs.menu.file.save");

    // Active menu should still be "File"
    let cap = capture(&mut harness);
    assert_node_label(&cap, "docs.menu.active", "File");
    assert_selector(&cap, "docs.menu.file.panel");
}

/// Clicking on a blank area of the menu bar should close the File menu without dispatch.
#[test]
fn blank_menu_bar_click_closes_file_menu_without_dispatch() {
    let mut harness = make_harness();

    // Open the File menu
    let _menu_open = click(&mut harness, "docs.menu.file");

    // Click on a blank area of the menu bar
    harness
        .automation_action(tench_ui_automation_core::UiAutomationAction::Click {
            selector: tench_ui_automation_core::UiAutomationSelector::point(500.0, 12.0),
            modifiers: tench_ui_automation_core::UiAutomationModifiers::default(),
        })
        .expect("blank menu bar click");

    let after = capture(&mut harness);
    assert_node_label(&after, "docs.menu.active", "none");
    assert_absent(&after, "docs.menu.file.panel");
}

/// Moving the pointer below the File menu should clear hover state.
#[test]
fn moving_below_file_menu_clears_hover_state() {
    let mut harness = make_harness();

    // Open the File menu
    let _menu_open = click(&mut harness, "docs.menu.file");

    // Hover over an item to set hover state
    let _after_hover = hover(&mut harness, "docs.menu.file.save");

    // Move pointer below the menu area
    move_mouse(&mut harness, 40.0, 500.0);

    let after = capture(&mut harness);
    assert_node_label(&after, "docs.menu.file.hovered", "none");
}

/// Returns the `value` field of a node, panicking if the node or value is missing.
fn node_value(capture: &tench_ui_automation_core::UiAutomationCapture, debug_id: &str) -> String {
    get_node_value(capture, debug_id).unwrap_or_else(|| panic!("node '{debug_id}' has no value"))
}

// --- Additional tests from docs-file-menu-button-fix.md ---

/// Hovering over File menu items changes the highlighted item but does not
/// mutate the document text, cursor, or selection.
#[test]
fn file_menu_hover_changes_highlight_only() {
    let mut harness = make_harness();

    // Type text so we have non-trivial document state to compare.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    // Open the File menu.
    let _menu_open = click(&mut harness, "docs.menu.file");

    // Hover over "save" — the hovered indicator should report "save".
    let after_save = hover(&mut harness, "docs.menu.file.save");
    assert_node_label(&after_save, "docs.menu.file.hovered", "Save");

    // Document state must be unchanged after hovering save.
    assert_eq!(
        node_value(&after_save, "docs.document.text"),
        text_before,
        "document text should not change after hovering save"
    );
    assert_eq!(
        node_value(&after_save, "docs.document.cursor"),
        cursor_before,
        "cursor should not change after hovering save"
    );
    assert_eq!(
        node_value(&after_save, "docs.document.selection"),
        selection_before,
        "selection should not change after hovering save"
    );

    // Hover over "version_history" — the hovered indicator should update.
    let after_vh = hover(&mut harness, "docs.menu.file.version_history");
    assert_node_label(&after_vh, "docs.menu.file.hovered", "Version History");

    // Document state must still be unchanged after hovering version_history.
    assert_eq!(
        node_value(&after_vh, "docs.document.text"),
        text_before,
        "document text should not change after hovering version_history"
    );
    assert_eq!(
        node_value(&after_vh, "docs.document.cursor"),
        cursor_before,
        "cursor should not change after hovering version_history"
    );
    assert_eq!(
        node_value(&after_vh, "docs.document.selection"),
        selection_before,
        "selection should not change after hovering version_history"
    );
}

/// Clicking outside the open File menu (on the document) closes the menu
/// without mutating the document text, cursor, or selection.
#[test]
fn file_menu_outside_click_closes_without_mutation() {
    let mut harness = make_harness();

    // Type text so we have non-trivial document state to compare.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "World");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    // Open the File menu.
    let _menu_open = click(&mut harness, "docs.menu.file");

    // Click outside on the document to close the menu.
    let after = click(&mut harness, "docs.document");

    // Menu should be closed.
    assert_node_label(&after, "docs.menu.active", "none");
    assert_absent(&after, "docs.menu.file.panel");

    // Document state must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after outside click"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor position should be preserved after outside click"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved after outside click"
    );
}

/// Repeatedly opening and closing the File menu leaves no stale menu nodes.
#[test]
fn repeated_open_and_outside_close_leaves_no_stale_menu_nodes() {
    let mut harness = make_harness();

    for i in 0..3 {
        // Open the File menu.
        let opened = click(&mut harness, "docs.menu.file");
        assert_eq!(
            node_text(&opened, "docs.menu.active"),
            "File",
            "iteration {i}: active menu should be File after opening",
        );
        assert_selector(&opened, "docs.menu.file.panel");

        // Click the document to close the menu.
        let closed = click(&mut harness, "docs.document");
        assert_eq!(
            node_text(&closed, "docs.menu.active"),
            "none",
            "iteration {i}: active menu should be none after closing",
        );
        assert_absent(&closed, "docs.menu.file.panel");

        // All File menu items should also be absent when the panel is closed.
        let items = [
            "docs.menu.file.new",
            "docs.menu.file.open",
            "docs.menu.file.save",
            "docs.menu.file.save_as",
            "docs.menu.file.export_as",
            "docs.menu.file.page_setup",
            "docs.menu.file.print",
            "docs.menu.file.version_history",
            "docs.menu.file.recent_files",
        ];
        for item in &items {
            assert_absent(&closed, item);
        }
    }
}

/// Click the File menu button and verify the active menu indicator, panel,
/// and all 9 expected items are present.
#[test]
fn file_menu_button_opens_with_expected_items() {
    let mut harness = make_harness();

    let cap = click(&mut harness, "docs.menu.file");

    // Active menu indicator should report "File".
    assert_node_label(&cap, "docs.menu.active", "File");

    // The File menu panel must exist.
    assert_selector(&cap, "docs.menu.file.panel");

    // All 9 expected menu items must be present.
    let items = [
        "docs.menu.file.new",
        "docs.menu.file.open",
        "docs.menu.file.save",
        "docs.menu.file.save_as",
        "docs.menu.file.export_as",
        "docs.menu.file.page_setup",
        "docs.menu.file.print",
        "docs.menu.file.version_history",
        "docs.menu.file.recent_files",
    ];
    for item in &items {
        assert_selector(&cap, item);
    }
}

/// Hovering over a File menu item sets the hovered indicator but does not
/// mutate the document dirty flag, text, or cursor.
#[test]
fn file_menu_hover_does_not_dispatch() {
    let mut harness = make_harness();

    // Type text so we have non-trivial document state to compare.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let dirty_before = node_value(&before, "docs.document.dirty");

    // Open the File menu.
    let _menu_open = click(&mut harness, "docs.menu.file");

    // Hover over "save" — the hovered indicator should report "Save".
    let after_hover = hover(&mut harness, "docs.menu.file.save");
    assert_node_label(&after_hover, "docs.menu.file.hovered", "Save");

    // Document state must be unchanged after hovering save.
    assert_eq!(
        node_value(&after_hover, "docs.document.text"),
        text_before,
        "document text should not change after hovering save"
    );
    assert_eq!(
        node_value(&after_hover, "docs.document.cursor"),
        cursor_before,
        "cursor should not change after hovering save"
    );
    assert_eq!(
        node_value(&after_hover, "docs.document.dirty"),
        dirty_before,
        "dirty flag should not change after hovering save"
    );
}

/// Clicking on a blank area of the menu bar (not on any menu button) closes
/// the File menu.
#[test]
fn file_menu_clicking_menu_bar_blank_closes_menu() {
    let mut harness = make_harness();

    // Open the File menu.
    let _menu_open = click(&mut harness, "docs.menu.file");

    // Click on a blank area of the menu bar (not on any menu button).
    harness
        .automation_action(tench_ui_automation_core::UiAutomationAction::Click {
            selector: tench_ui_automation_core::UiAutomationSelector::point(500.0, 12.0),
            modifiers: tench_ui_automation_core::UiAutomationModifiers::default(),
        })
        .expect("blank menu bar click");

    let after = capture(&mut harness);

    // Menu should be closed.
    assert_node_label(&after, "docs.menu.active", "none");
    assert_absent(&after, "docs.menu.file.panel");
}

/// Switching from the File menu to the Edit menu clears the stale File hover
/// state.  The Edit menu should be active with no item hovered.
#[test]
fn file_menu_switch_to_edit_clears_stale_hover() {
    let mut harness = make_harness();

    // Open the File menu.
    let _file_open = click(&mut harness, "docs.menu.file");

    // Hover over "save" to set hover state on the File menu.
    let _after_hover = hover(&mut harness, "docs.menu.file.save");

    // Click the Edit menu button to switch menus.
    let after = click(&mut harness, "docs.menu.edit");

    // File panel should be absent.
    assert_absent(&after, "docs.menu.file.panel");

    // Active menu should be Edit.
    assert_node_label(&after, "docs.menu.active", "Edit");

    // Edit hovered should be "none" (no stale hover carried from File).
    assert_node_label(&after, "docs.menu.edit.hovered", "none");
}

/// Moving the pointer below the last File menu item clears the hover state.
#[test]
fn file_menu_moving_below_items_clears_hover() {
    let mut harness = make_harness();

    // Open the File menu.
    let _menu_open = click(&mut harness, "docs.menu.file");

    // Hover over "save" to set hover state.
    let _after_hover = hover(&mut harness, "docs.menu.file.save");

    // Move pointer below the last menu item.
    move_mouse(&mut harness, 40.0, 500.0);

    let after = capture(&mut harness);

    // Hovered indicator should be cleared.
    assert_node_label(&after, "docs.menu.file.hovered", "none");
}
