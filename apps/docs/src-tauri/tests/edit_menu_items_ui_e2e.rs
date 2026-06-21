/// UI automation tests for docs Edit Menu items.
///
/// Uses semantic debug_id selectors to verify document state changes
/// after Edit menu actions (Undo, Redo, Cut, Select All, Find, Replace, Go To).
use tench_docs_lib::ui::DocsApp;
use tench_ui_automation_core::{UiAutomationKey, UiAutomationModifiers};
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::{assert_capture_changed, TestHarness};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

/// Helper: returns the `value` field of a node identified by `debug_id`.
///
/// Panics if the node is not found or has no value.
fn node_value(capture: &tench_ui_automation_core::UiAutomationCapture, debug_id: &str) -> String {
    get_node_value(capture, debug_id).unwrap_or_else(|| panic!("node '{debug_id}' has no value"))
}

/// Helper: returns the `label` field of a node identified by `debug_id`.
///
/// Panics if the node is not found or has no label.
fn node_label(capture: &tench_ui_automation_core::UiAutomationCapture, debug_id: &str) -> String {
    get_node_label(capture, debug_id).unwrap_or_else(|| panic!("node '{debug_id}' has no label"))
}

// ---------------------------------------------------------------------------
// Undo
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_undo_reverses_last_edit() {
    let mut harness = make_harness();

    // Type "Alpha" into the document.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");

    // Undo via menu — each character is a separate undo step, so undo 5 times.
    for _ in 0..5 {
        open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.undo");
    }
    let after = capture(&mut harness);

    // Document text should be empty after undo.
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.is_empty(),
        "expected empty text after undo, got: \"{text}\""
    );

    // Document should be dirty (undo changed content from saved state).
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(dirty, "true", "document should be dirty after undo");

    // Redo should now be available.
    let redo = node_value(&after, "docs.document.redo_available");
    assert_eq!(redo, "true", "redo should be available after undo");

    // No modal should be active (menu closes after item click).
    let active = node_label(&after, "docs.menu.active");
    assert_eq!(active, "none", "no modal should be active after undo");
}

#[test]
fn edit_menu_undo_with_empty_stack_is_noop() {
    let mut harness = make_harness();

    // Capture initial state.
    let initial = capture(&mut harness);
    let initial_text = node_value(&initial, "docs.document.text");
    let initial_cursor = node_value(&initial, "docs.document.cursor");
    let initial_dirty = node_value(&initial, "docs.document.dirty");

    // Undo on empty undo stack.
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.undo");

    // All values should remain unchanged.
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, initial_text,
        "text should not change on empty undo stack"
    );

    let cursor = node_value(&after, "docs.document.cursor");
    assert_eq!(
        cursor, initial_cursor,
        "cursor should not move on empty undo stack"
    );

    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(
        dirty, initial_dirty,
        "dirty flag should not change on empty undo stack"
    );
}

// ---------------------------------------------------------------------------
// Redo
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_redo_after_undo_restores_text() {
    let mut harness = make_harness();

    // Type "Alpha", then undo all 5 characters.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");
    for _ in 0..5 {
        open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.undo");
    }

    // Redo all 5 characters via menu — one redo per character.
    for _ in 0..5 {
        open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.redo");
    }
    let after = capture(&mut harness);

    // Text should be restored to "Alpha".
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, "Alpha",
        "expected text \"Alpha\" after redo, got: \"{text}\""
    );
}

// ---------------------------------------------------------------------------
// Cut
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_cut_removes_selected_text() {
    let mut harness = make_harness();

    // Type "Hello", select all, then cut.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");

    // Select all via Ctrl+A.
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    // Cut via menu.
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.cut");

    // Document text should be empty after cut.
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.is_empty(),
        "expected empty text after cut, got: \"{text}\""
    );
}

// ---------------------------------------------------------------------------
// Select All
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_select_all_selects_text() {
    let mut harness = make_harness();

    // Type "Hello World".
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello World");

    // Select All via menu.
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.select_all");

    // Selection should not be "none".
    let selection = node_value(&after, "docs.document.selection");
    assert_ne!(
        selection, "none",
        "expected a selection after Select All, got: \"{selection}\""
    );
}

// ---------------------------------------------------------------------------
// Find
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_find_opens_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.find");
    assert_capture_changed(&before, &after);

    // Find modal should be present.
    assert_selector(&after, "docs.modal.find_replace");

    // Find query input should exist.
    assert_selector(&after, "docs.find.query");

    // Find/Replace uses its own state, not active_modal, so docs.menu.active
    // remains "none" (it tracks traditional modals like Go To, Hyperlink, etc.).
    let active = node_label(&after, "docs.menu.active");
    assert_eq!(
        active, "none",
        "docs.menu.active should be \"none\" — Find uses find_replace state"
    );
}

// ---------------------------------------------------------------------------
// Replace
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_replace_opens_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.replace");
    assert_capture_changed(&before, &after);

    // Find/Replace modal should be present.
    assert_selector(&after, "docs.modal.find_replace");

    // Replace mode should show the replace button.
    assert_selector(&after, "docs.find.replace");
}

// ---------------------------------------------------------------------------
// Go To
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_goto_opens_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");
    assert_capture_changed(&before, &after);

    // Go To modal should be present.
    assert_selector(&after, "docs.modal.goto");

    // Go To input field should exist.
    assert_selector(&after, "docs.goto.input");
}

// ---------------------------------------------------------------------------
// Copy
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_copy_copies_selection_and_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );
    let before_text = node_value(&capture(&mut harness), "docs.document.text");
    let before_selection = node_value(&capture(&mut harness), "docs.document.selection");
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.copy");
    assert_eq!(node_value(&after, "docs.clipboard.text"), "Alpha");
    assert_eq!(node_value(&after, "docs.document.text"), before_text);
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        before_selection
    );
}

// ---------------------------------------------------------------------------
// Paste
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_paste_inserts_clipboard_at_cursor() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.copy");
    key(
        &mut harness,
        UiAutomationKey::ArrowRight,
        UiAutomationModifiers::default(),
    );
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.paste");
    assert_eq!(node_value(&after, "docs.clipboard.text"), "Alpha");
    assert_eq!(node_value(&after, "docs.document.text"), "AlphaAlpha");
    assert_eq!(node_value(&after, "docs.document.selection"), "none");
}

#[test]
fn edit_menu_paste_with_no_clipboard_is_noop() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "Body");
    let before_text = node_value(&typed, "docs.document.text");
    let before_cursor = node_value(&typed, "docs.document.cursor");
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.paste");
    assert_eq!(node_value(&after, "docs.document.text"), before_text);
    assert_eq!(node_value(&after, "docs.document.cursor"), before_cursor);
}

// ---------------------------------------------------------------------------
// Undo — additional coverage from fix plans
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_undo_reverses_last_edit_and_closes_menu() {
    let mut harness = make_harness();

    // Type "Alpha" into the document.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");

    // Undo via menu — each character is a separate undo step, so undo 5 times.
    for _ in 0..5 {
        open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.undo");
    }
    let after = capture(&mut harness);

    // Menu should be closed (active = none).
    let active = node_label(&after, "docs.menu.active");
    assert_eq!(active, "none", "menu should be closed after undo");

    // Menu panel should be absent.
    assert_absent(&after, "docs.menu.edit.panel");

    // Document text should be empty.
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.is_empty(),
        "expected empty text after undo, got: \"{text}\""
    );

    // Cursor should be at 0:0.
    let cursor = node_value(&after, "docs.document.cursor");
    assert_eq!(
        cursor, "0:0",
        "cursor should be at 0:0 after undo, got: \"{cursor}\""
    );

    // Selection should be none.
    let selection = node_value(&after, "docs.document.selection");
    assert_eq!(selection, "none", "selection should be none after undo");

    // Document should be dirty.
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(dirty, "true", "document should be dirty after undo");

    // Redo should be available.
    let redo = node_value(&after, "docs.document.redo_available");
    assert_eq!(redo, "true", "redo should be available after undo");
}

#[test]
fn hovering_undo_then_clicking_find_does_not_undo() {
    let mut harness = make_harness();

    // Type "Alpha" into the document.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");

    // Open Edit menu, hover over undo.
    click(&mut harness, "docs.menu.edit");
    let hovered = hover(&mut harness, "docs.menu.edit.undo");

    // Verify hovered state shows undo.
    let hovered_item = node_label(&hovered, "docs.menu.edit.hovered");
    assert_eq!(hovered_item, "Undo", "hovered item should be undo");

    // Click find instead of undo.
    let after = click(&mut harness, "docs.menu.edit.find");

    // Find modal should exist.
    assert_selector(&after, "docs.modal.find_replace");

    // Text should still be "Alpha" — undo was not dispatched.
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, "Alpha",
        "text should still be Alpha after hovering undo then clicking find"
    );

    // Undo should still be available.
    let undo = node_value(&after, "docs.document.undo_available");
    assert_eq!(undo, "true", "undo should still be available");
}

// ---------------------------------------------------------------------------
// Redo — additional coverage from fix plans
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_redo_with_empty_stack_is_noop() {
    let mut harness = make_harness();

    // Capture initial state on a fresh document (no edits, so redo stack is empty).
    let initial = capture(&mut harness);
    let initial_text = node_value(&initial, "docs.document.text");
    let initial_cursor = node_value(&initial, "docs.document.cursor");
    let initial_selection = node_value(&initial, "docs.document.selection");
    let initial_dirty = node_value(&initial, "docs.document.dirty");

    // Redo on empty redo stack.
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.redo");

    // All values should remain unchanged.
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, initial_text,
        "text should not change on empty redo stack"
    );

    let cursor = node_value(&after, "docs.document.cursor");
    assert_eq!(
        cursor, initial_cursor,
        "cursor should not move on empty redo stack"
    );

    let selection = node_value(&after, "docs.document.selection");
    assert_eq!(
        selection, initial_selection,
        "selection should not change on empty redo stack"
    );

    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(
        dirty, initial_dirty,
        "dirty flag should not change on empty redo stack"
    );
}

#[test]
fn hovering_redo_then_clicking_find_does_not_redo() {
    let mut harness = make_harness();

    // Type "Alpha", then undo all 5 characters.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");
    for _ in 0..5 {
        open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.undo");
    }

    // Open Edit menu, hover over redo.
    click(&mut harness, "docs.menu.edit");
    let hovered = hover(&mut harness, "docs.menu.edit.redo");

    // Verify hovered state shows redo.
    let hovered_item = node_label(&hovered, "docs.menu.edit.hovered");
    assert_eq!(hovered_item, "Redo", "hovered item should be redo");

    // Click find instead of redo.
    let after = click(&mut harness, "docs.menu.edit.find");

    // Find modal should exist.
    assert_selector(&after, "docs.modal.find_replace");

    // Text should still be empty — redo was not dispatched.
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.is_empty(),
        "text should still be empty after hovering redo then clicking find, got: \"{text}\""
    );

    // Redo should still be available.
    let redo = node_value(&after, "docs.document.redo_available");
    assert_eq!(redo, "true", "redo should still be available");
}

// ---------------------------------------------------------------------------
// Cut — additional coverage from fix plans
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_cut_with_no_selection_is_noop() {
    let mut harness = make_harness();

    // Type "Alpha" but leave cursor at end with no selection.
    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "Alpha");
    let before_text = node_value(&typed, "docs.document.text");
    let before_cursor = node_value(&typed, "docs.document.cursor");
    let before_dirty = node_value(&typed, "docs.document.dirty");

    // Cut via menu with no selection.
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.cut");

    // Clipboard should be empty (nothing to cut).
    let clipboard = node_value(&after, "docs.clipboard.text");
    assert!(
        clipboard.is_empty() || clipboard == "none",
        "clipboard should be empty after cut with no selection, got: \"{clipboard}\""
    );

    // Text should be unchanged.
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, before_text,
        "text should not change on cut with no selection"
    );

    // Cursor should be unchanged.
    let cursor = node_value(&after, "docs.document.cursor");
    assert_eq!(
        cursor, before_cursor,
        "cursor should not move on cut with no selection"
    );

    // Dirty should be unchanged.
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(
        dirty, before_dirty,
        "dirty flag should not change on cut with no selection"
    );
}

// ---------------------------------------------------------------------------
// Copy — additional coverage from fix plans
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_copy_with_no_selection_is_noop() {
    let mut harness = make_harness();

    // Type "Alpha" but leave cursor at end with no selection.
    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "Alpha");
    let before_text = node_value(&typed, "docs.document.text");
    let before_dirty = node_value(&typed, "docs.document.dirty");

    // Copy via menu with no selection.
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.copy");

    // Clipboard should be empty (nothing to copy).
    let clipboard = node_value(&after, "docs.clipboard.text");
    assert!(
        clipboard.is_empty() || clipboard == "none",
        "clipboard should be empty after copy with no selection, got: \"{clipboard}\""
    );

    // Text should be unchanged.
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, before_text,
        "text should not change on copy with no selection"
    );

    // Dirty should be unchanged.
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(
        dirty, before_dirty,
        "dirty flag should not change on copy with no selection"
    );
}

// ---------------------------------------------------------------------------
// Select All — additional coverage from fix plans
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_select_all_on_empty_document_is_noop() {
    let mut harness = make_harness();

    // Fresh document — capture initial state.
    let initial = capture(&mut harness);
    let initial_text = node_value(&initial, "docs.document.text");
    let initial_dirty = node_value(&initial, "docs.document.dirty");

    // Select All via menu on empty document.
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.select_all");

    // Selection should be none (nothing to select).
    // Select All on an empty document selects the zero-width range.
    let selection = node_value(&after, "docs.document.selection");
    assert!(
        selection == "none" || selection == "0:0-0:0",
        "selection should be none or zero-width on empty document, got: {selection}"
    );

    // Text should be unchanged.
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, initial_text,
        "text should not change on select all of empty document"
    );

    // Dirty should be unchanged.
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(
        dirty, initial_dirty,
        "dirty flag should not change on select all of empty document"
    );
}

// ---------------------------------------------------------------------------
// Copy — hover-then-navigate coverage from fix plans
// ---------------------------------------------------------------------------

#[test]
fn hovering_copy_then_clicking_find_does_not_copy() {
    let mut harness = make_harness();

    // Type "Alpha" and select all.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    // Open Edit menu, hover over copy.
    click(&mut harness, "docs.menu.edit");
    let hovered = hover(&mut harness, "docs.menu.edit.copy");

    // Verify hovered state shows copy.
    let hovered_item = node_label(&hovered, "docs.menu.edit.hovered");
    assert_eq!(hovered_item, "Copy", "hovered item should be copy");

    // Click find instead of copy.
    let after = click(&mut harness, "docs.menu.edit.find");

    // Find modal should exist.
    assert_selector(&after, "docs.modal.find_replace");

    // Clipboard should be "none" — copy was not dispatched.
    let clipboard = node_value(&after, "docs.clipboard.text");
    assert!(
        clipboard.is_empty() || clipboard == "none",
        "clipboard should be empty/none after hovering copy then clicking find, got: \"{clipboard}\""
    );

    // Text should still be "Alpha".
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, "Alpha",
        "text should still be Alpha after hovering copy then clicking find"
    );
}

// ---------------------------------------------------------------------------
// Paste — hover-then-navigate coverage from fix plans
// ---------------------------------------------------------------------------

#[test]
fn hovering_paste_then_clicking_find_does_not_paste() {
    let mut harness = make_harness();

    // Type "Alpha", select all, copy, then move cursor to end.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.copy");
    key(
        &mut harness,
        UiAutomationKey::ArrowRight,
        UiAutomationModifiers::default(),
    );

    // Capture state before hovering paste.
    let text_before = node_value(&capture(&mut harness), "docs.document.text");

    // Open Edit menu, hover over paste.
    click(&mut harness, "docs.menu.edit");
    let hovered = hover(&mut harness, "docs.menu.edit.paste");

    // Verify hovered state shows paste.
    let hovered_item = node_label(&hovered, "docs.menu.edit.hovered");
    assert_eq!(hovered_item, "Paste", "hovered item should be paste");

    // Click find instead of paste.
    let after = click(&mut harness, "docs.menu.edit.find");

    // Find modal should exist.
    assert_selector(&after, "docs.modal.find_replace");

    // Text should be unchanged — paste was not dispatched.
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, text_before,
        "text should be unchanged after hovering paste then clicking find"
    );

    // Clipboard should still contain "Alpha".
    let clipboard = node_value(&after, "docs.clipboard.text");
    assert_eq!(clipboard, "Alpha", "clipboard should still be Alpha");
}

// ---------------------------------------------------------------------------
// Cut — hover-then-navigate coverage from fix plans
// ---------------------------------------------------------------------------

#[test]
fn hovering_cut_then_clicking_find_does_not_cut() {
    let mut harness = make_harness();

    // Type "Alpha" and select all.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    // Open Edit menu, hover over cut.
    click(&mut harness, "docs.menu.edit");
    let hovered = hover(&mut harness, "docs.menu.edit.cut");

    // Verify hovered state shows cut.
    let hovered_item = node_label(&hovered, "docs.menu.edit.hovered");
    assert_eq!(hovered_item, "Cut", "hovered item should be cut");

    // Click find instead of cut.
    let after = click(&mut harness, "docs.menu.edit.find");

    // Find modal should exist.
    assert_selector(&after, "docs.modal.find_replace");

    // Clipboard should be "none" — cut was not dispatched.
    let clipboard = node_value(&after, "docs.clipboard.text");
    assert!(
        clipboard.is_empty() || clipboard == "none",
        "clipboard should be empty/none after hovering cut then clicking find, got: \"{clipboard}\""
    );

    // Text should still be "Alpha".
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, "Alpha",
        "text should still be Alpha after hovering cut then clicking find"
    );
}

// ---------------------------------------------------------------------------
// Select All — hover-then-navigate coverage from fix plans
// ---------------------------------------------------------------------------

#[test]
fn hovering_select_all_then_clicking_find_does_not_select() {
    let mut harness = make_harness();

    // Type "Alpha".
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");

    // Open Edit menu, hover over select_all.
    click(&mut harness, "docs.menu.edit");
    let hovered = hover(&mut harness, "docs.menu.edit.select_all");

    // Verify hovered state shows select_all.
    let hovered_item = node_label(&hovered, "docs.menu.edit.hovered");
    assert_eq!(
        hovered_item, "Select All",
        "hovered item should be select_all"
    );

    // Click find instead of select_all.
    let after = click(&mut harness, "docs.menu.edit.find");

    // Find modal should exist.
    assert_selector(&after, "docs.modal.find_replace");

    // Selection should be "none" — select all was not dispatched.
    let selection = node_value(&after, "docs.document.selection");
    assert_eq!(
        selection, "none",
        "selection should be none after hovering select_all then clicking find"
    );

    // Text should still be "Alpha".
    let text = node_value(&after, "docs.document.text");
    assert_eq!(
        text, "Alpha",
        "text should still be Alpha after hovering select_all then clicking find"
    );
}

// ---------------------------------------------------------------------------
// Find — menu-close-before-dispatch from fix plans
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_find_closes_menu_before_dispatching() {
    let mut harness = make_harness();

    // Open Edit menu, click find.
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.find");

    // Menu should be closed (active = none).
    let active = node_label(&after, "docs.menu.active");
    assert_eq!(
        active, "none",
        "menu should be closed before find dispatches"
    );

    // Edit panel should be absent.
    assert_absent(&after, "docs.menu.edit.panel");

    // Find/Replace modal should be present.
    assert_selector(&after, "docs.modal.find_replace");
}

// ---------------------------------------------------------------------------
// Replace — menu-close-before-dispatch from fix plans
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_replace_closes_menu_before_dispatching() {
    let mut harness = make_harness();

    // Open Edit menu, click replace.
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.replace");

    // Menu should be closed (active = none).
    let active = node_label(&after, "docs.menu.active");
    assert_eq!(
        active, "none",
        "menu should be closed before replace dispatches"
    );

    // Edit panel should be absent.
    assert_absent(&after, "docs.menu.edit.panel");

    // Find/Replace modal should be present.
    assert_selector(&after, "docs.modal.find_replace");
}

// ---------------------------------------------------------------------------
// Go To — menu-close-before-dispatch from fix plans
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_goto_closes_menu_before_dispatching() {
    let mut harness = make_harness();

    // Open Edit menu, click go_to.
    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.go_to");

    // Menu should be closed (active = none).
    let active = node_label(&after, "docs.menu.active");
    assert_eq!(
        active, "none",
        "menu should be closed before goto dispatches"
    );

    // Edit panel should be absent.
    assert_absent(&after, "docs.menu.edit.panel");

    // Go To modal should be present.
    assert_selector(&after, "docs.modal.goto");
}

// ---------------------------------------------------------------------------
// Copy — repeated-copy edge case from fix plans
// ---------------------------------------------------------------------------

#[test]
fn repeated_copy_overwrites_clipboard_only_for_non_empty_selection() {
    let mut harness = make_harness();

    // Type "First", select all, copy.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "First");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.copy");
    let first = capture(&mut harness);
    assert_eq!(node_value(&first, "docs.clipboard.text"), "First");

    // Deselect (arrow right), copy again with no selection — clipboard should stay "First".
    key(
        &mut harness,
        UiAutomationKey::ArrowRight,
        UiAutomationModifiers::default(),
    );
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.copy");
    let no_selection = capture(&mut harness);
    assert_eq!(
        node_value(&no_selection, "docs.clipboard.text"),
        "First",
        "clipboard should still be First after copy with no selection"
    );

    // Type " Second", select all, copy — clipboard should now be "First Second".
    type_text(&mut harness, " Second");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.copy");
    let second = capture(&mut harness);
    assert_eq!(
        node_value(&second, "docs.clipboard.text"),
        "First Second",
        "clipboard should be updated to First Second after copying new selection"
    );
}

// ---------------------------------------------------------------------------
// Cut — clipboard verification from fix plans
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_cut_copies_to_clipboard() {
    let mut harness = make_harness();

    // Type "Hello", select all, cut.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.cut");

    // Clipboard should contain the cut text.
    let clipboard = node_value(&after, "docs.clipboard.text");
    assert_eq!(clipboard, "Hello", "clipboard should contain cut text");

    // Document text should be empty.
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.is_empty(),
        "expected empty text after cut, got: \"{text}\""
    );

    // Selection should be cleared.
    let selection = node_value(&after, "docs.document.selection");
    assert_eq!(selection, "none", "selection should be cleared after cut");

    // Document should be dirty.
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(dirty, "true", "document should be dirty after cut");
}

// ---------------------------------------------------------------------------
// Find — hover-then-navigate from fix plans
// ---------------------------------------------------------------------------

#[test]
fn hovering_find_then_clicking_undo_does_not_open_modal() {
    let mut harness = make_harness();

    // Type "A" so undo is available.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "A");

    // Open Edit menu, hover over find.
    click(&mut harness, "docs.menu.edit");
    let hovered = hover(&mut harness, "docs.menu.edit.find");

    // Verify hovered state shows find.
    let hovered_item = node_label(&hovered, "docs.menu.edit.hovered");
    assert_eq!(hovered_item, "Find", "hovered item should be Find");

    // Click undo instead of find.
    let after = click(&mut harness, "docs.menu.edit.undo");

    // Find modal should NOT be present.
    assert_absent(&after, "docs.modal.find_replace");

    // Undo should have been dispatched — text should be empty.
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.is_empty(),
        "text should be empty after undo, got: \"{text}\""
    );
}

// ---------------------------------------------------------------------------
// Go To — hover-then-navigate from fix plans
// ---------------------------------------------------------------------------

#[test]
fn hovering_goto_then_clicking_undo_does_not_open_modal() {
    let mut harness = make_harness();

    // Type "A" so undo is available.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "A");

    // Open Edit menu, hover over go_to.
    click(&mut harness, "docs.menu.edit");
    let hovered = hover(&mut harness, "docs.menu.edit.go_to");

    // Verify hovered state shows go_to.
    let hovered_item = node_label(&hovered, "docs.menu.edit.hovered");
    assert_eq!(hovered_item, "Go To", "hovered item should be Go To");

    // Click undo instead of go_to.
    let after = click(&mut harness, "docs.menu.edit.undo");

    // Go To modal should NOT be present.
    assert_absent(&after, "docs.modal.goto");

    // Undo should have been dispatched — text should be empty.
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.is_empty(),
        "text should be empty after undo, got: \"{text}\""
    );
}

// ---------------------------------------------------------------------------
// Paste — selection replacement and repeated paste from fix plans
// ---------------------------------------------------------------------------

#[test]
fn edit_menu_paste_replaces_selection_and_undo_restores() {
    let mut harness = make_harness();

    // Type "Alpha", select all, copy.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.copy");

    // Type "Target" to replace document content, select all, then paste.
    type_text(&mut harness, "Target");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    let pasted = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.paste");
    assert_eq!(node_value(&pasted, "docs.document.text"), "Alpha");
    assert_eq!(node_value(&pasted, "docs.document.selection"), "none");

    // Undo should restore "Target".
    let undone = key(
        &mut harness,
        UiAutomationKey::Character("z".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );
    assert_eq!(
        node_value(&undone, "docs.document.text"),
        "Target",
        "undo should restore pre-paste text"
    );
}

#[test]
fn repeated_paste_reuses_clipboard_content() {
    let mut harness = make_harness();

    // Type "X", select all, copy, then move cursor to end.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "X");
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".into()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.copy");
    key(
        &mut harness,
        UiAutomationKey::ArrowRight,
        UiAutomationModifiers::default(),
    );

    // Paste twice.
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.paste");
    let second = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.paste");

    // Clipboard should still contain "X".
    assert_eq!(node_value(&second, "docs.clipboard.text"), "X");

    // Document should be "XXX" (original X + two pastes).
    assert_eq!(
        node_value(&second, "docs.document.text"),
        "XXX",
        "repeated paste should reuse clipboard content"
    );
}

// ---------------------------------------------------------------------------
// Replace — hover-then-navigate from fix plans
// ---------------------------------------------------------------------------

#[test]
fn hovering_replace_then_clicking_undo_does_not_open_modal() {
    let mut harness = make_harness();

    // Type "A" so undo is available.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "A");

    // Open Edit menu, hover over replace.
    click(&mut harness, "docs.menu.edit");
    let hovered = hover(&mut harness, "docs.menu.edit.replace");

    // Verify hovered state shows replace.
    let hovered_item = node_label(&hovered, "docs.menu.edit.hovered");
    assert_eq!(hovered_item, "Replace", "hovered item should be Replace");

    // Click undo instead of replace.
    let after = click(&mut harness, "docs.menu.edit.undo");

    // Find/Replace modal should NOT be present.
    assert_absent(&after, "docs.modal.find_replace");

    // Undo should have been dispatched — text should be empty.
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.is_empty(),
        "text should be empty after undo, got: \"{text}\""
    );
}

// ---------------------------------------------------------------------------
// Undo — stale selection and repeated undo from fix plans
// ---------------------------------------------------------------------------

#[test]
fn undo_clears_stale_selection() {
    let mut harness = make_harness();

    // Type "A", select all, then undo.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "A");
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.select_all");

    // Verify selection exists before undo.
    let selected = capture(&mut harness);
    let selection_before = node_value(&selected, "docs.document.selection");
    assert_ne!(
        selection_before, "none",
        "selection should exist before undo"
    );

    let after = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.undo");

    // Text should be empty (undone).
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.is_empty(),
        "text should be empty after undo, got: \"{text}\""
    );

    // Selection should be cleared (stale selection should not survive undo).
    let selection = node_value(&after, "docs.document.selection");
    assert_eq!(
        selection, "none",
        "stale selection should be cleared after undo"
    );
}

#[test]
fn repeated_undo_stops_at_initial_document() {
    let mut harness = make_harness();

    // Type "A" (one character = one undo step).
    click(&mut harness, "docs.document");
    type_text(&mut harness, "A");

    // Undo once — should clear text.
    open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.undo");

    // Undo again on empty stack — should remain at initial state.
    let second = open_menu_item(&mut harness, "docs.menu.edit", "docs.menu.edit.undo");

    let text = node_value(&second, "docs.document.text");
    assert!(
        text.is_empty(),
        "text should remain empty after repeated undo past initial state"
    );

    let cursor = node_value(&second, "docs.document.cursor");
    assert_eq!(
        cursor, "0:0",
        "cursor should remain at 0:0 after repeated undo past initial state"
    );
}
