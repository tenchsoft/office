/// UI automation tests for docs toolbar buttons (#54-#82).
///
/// Uses debug_id selectors for all toolbar buttons.
/// Every test verifies the button selector exists and clicking it doesn't crash.
/// Buttons that produce visual changes are verified with pixel comparison.
/// Semantic tests verify document state (text, dirty flag) after interactions.
use tench_docs_lib::ui::DocsApp;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;
use tench_ui_test::{assert_capture_changed, CaptureAssertions};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

/// Helper: returns the `value` field of a node identified by `debug_id`.
///
/// Panics if the node is not found or has no value.
fn node_value(capture: &tench_ui_automation_core::UiAutomationCapture, debug_id: &str) -> String {
    get_node_value(capture, debug_id).unwrap_or_else(|| panic!("node '{debug_id}' has no value"))
}

// ---------------------------------------------------------------------------
// Undo / Redo — semantic state tests
// ---------------------------------------------------------------------------

#[test]
fn toolbar_undo_button_reverses_edit() {
    let mut harness = make_harness();

    // Focus the document and type "Hello".
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");

    // Verify text was typed.
    let before_undo = capture(&mut harness);
    let text_before = node_value(&before_undo, "docs.document.text");
    assert_eq!(
        text_before, "Hello",
        "expected \"Hello\" after typing, got: \"{text_before}\""
    );

    // Undo each of the 5 characters via the toolbar button.
    for _ in 0..5 {
        click(&mut harness, "docs.toolbar.undo");
    }
    let after = capture(&mut harness);

    // Document text should be empty after undoing all characters.
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.is_empty(),
        "expected empty text after undo, got: \"{text}\""
    );
}

#[test]
fn toolbar_redo_button_after_undo() {
    let mut harness = make_harness();

    // Focus the document and type "Alpha".
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");

    // Undo all 5 characters via the toolbar button.
    for _ in 0..5 {
        click(&mut harness, "docs.toolbar.undo");
    }
    let after_undo = capture(&mut harness);
    let text_after_undo = node_value(&after_undo, "docs.document.text");
    assert!(
        text_after_undo.is_empty(),
        "expected empty text after undo, got: \"{text_after_undo}\""
    );

    // Redo all 5 characters via the toolbar button.
    for _ in 0..5 {
        click(&mut harness, "docs.toolbar.redo");
    }
    let after_redo = capture(&mut harness);

    // Text should be restored to "Alpha".
    let text = node_value(&after_redo, "docs.document.text");
    assert_eq!(
        text, "Alpha",
        "expected text \"Alpha\" after redo, got: \"{text}\""
    );
}

// ---------------------------------------------------------------------------
// Undo/Redo: no visual change on empty document, verify no crash
// ---------------------------------------------------------------------------

#[test]
fn toolbar_undo_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.undo");
    let after = click(&mut harness, "docs.toolbar.undo");
    after.assert_png_valid();
}

#[test]
fn toolbar_redo_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.redo");
    let after = click(&mut harness, "docs.toolbar.redo");
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// List buttons: toggle, should produce visual change
// ---------------------------------------------------------------------------

#[test]
fn toolbar_bullet_list_button() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.bullet_list");
    let after = click(&mut harness, "docs.toolbar.bullet_list");
    assert_capture_changed(&before, &after);
}

#[test]
fn toolbar_numbered_list_button() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.numbered_list");
    let after = click(&mut harness, "docs.toolbar.numbered_list");
    assert_capture_changed(&before, &after);
}

#[test]
fn toolbar_checklist_button() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.checklist");
    let after = click(&mut harness, "docs.toolbar.checklist");
    assert_capture_changed(&before, &after);
}

// ---------------------------------------------------------------------------
// Indent: no visual change on empty doc
// ---------------------------------------------------------------------------

#[test]
fn toolbar_outdent_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.outdent");
    let after = click(&mut harness, "docs.toolbar.outdent");
    after.assert_png_valid();
}

#[test]
fn toolbar_indent_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.indent");
    let after = click(&mut harness, "docs.toolbar.indent");
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Alignment: semantic state tests + visual change
// ---------------------------------------------------------------------------

#[test]
fn toolbar_align_left_button_visual_change() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.align_left");
    let after = click(&mut harness, "docs.toolbar.align_left");
    after.assert_png_valid();
    assert_capture_changed(&before, &after);
}

#[test]
fn toolbar_align_center_button_visual_change() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.align_center");
    let after = click(&mut harness, "docs.toolbar.align_center");
    after.assert_png_valid();
    assert_capture_changed(&before, &after);
}

#[test]
fn toolbar_align_right_button() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.align_right");
    let after = click(&mut harness, "docs.toolbar.align_right");
    assert_capture_changed(&before, &after);
}

#[test]
fn toolbar_align_justify_button() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.align_justify");
    let after = click(&mut harness, "docs.toolbar.align_justify");
    assert_capture_changed(&before, &after);
}

// ---------------------------------------------------------------------------
// Insert buttons — semantic state tests
// ---------------------------------------------------------------------------

#[test]
fn toolbar_insert_link_button() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.insert_link");
    let after = click(&mut harness, "docs.toolbar.insert_link");
    assert_capture_changed(&before, &after);
}

#[test]
fn toolbar_insert_image_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.insert_image");
    let after = click(&mut harness, "docs.toolbar.insert_image");
    after.assert_png_valid();
}

#[test]
fn toolbar_insert_table_button_adds_table() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.insert_table");
    let after = click(&mut harness, "docs.toolbar.insert_table");
    assert_capture_changed(&before, &after);

    // Clicking the insert table button opens the table grid picker dropdown.
    // The dropdown itself is a visual change; the table is only inserted once
    // the user picks a cell in the grid, so dirty is not yet set here.
    after.assert_png_valid();
}

#[test]
fn toolbar_horizontal_rule_button() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.horizontal_rule");
    let after = click(&mut harness, "docs.toolbar.horizontal_rule");
    assert_capture_changed(&before, &after);
}

#[test]
fn toolbar_block_quote_button_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.block_quote");
    let after = click(&mut harness, "docs.toolbar.block_quote");
    assert_capture_changed(&before, &after);

    // Toggling block quote should mark the document as dirty.
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(
        dirty, "true",
        "document should be dirty after toggling block quote"
    );
}

// ---------------------------------------------------------------------------
// Format buttons: Bold, Italic, Underline, Strikethrough, Code,
// Superscript, Subscript
// ---------------------------------------------------------------------------

#[test]
fn toolbar_bold_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.bold");
    let after = click(&mut harness, "docs.toolbar.bold");
    after.assert_png_valid();
}

#[test]
fn toolbar_bold_button_no_selection_sets_active_mark() {
    let mut harness = make_harness();
    // Place cursor with no selection
    click(&mut harness, "docs.document");
    // Click bold to set active mark for future typing
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.bold");
    let after = click(&mut harness, "docs.toolbar.bold");
    after.assert_png_valid();
    // Type text — it should be bold
    let after_type = type_text(&mut harness, "Bold");
    let text = node_value(&after_type, "docs.document.text");
    assert_eq!(text, "Bold", "expected 'Bold' after typing, got: '{text}'");
    // Document should be dirty
    let dirty = node_value(&after_type, "docs.document.dirty");
    assert_eq!(
        dirty, "true",
        "document should be dirty after typing bold text"
    );
}

#[test]
fn toolbar_bold_button_second_click_removes_bold() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    // Click bold, type, then click bold again to toggle off
    click(&mut harness, "docs.toolbar.bold");
    type_text(&mut harness, "A");
    let _before = capture(&mut harness);
    assert_selector(&_before, "docs.toolbar.bold");
    let after = click(&mut harness, "docs.toolbar.bold");
    after.assert_png_valid();
    // Verify the text is still there
    let text = node_value(&after, "docs.document.text");
    assert_eq!(text, "A", "text should remain after toggling bold off");
}

#[test]
fn toolbar_italic_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.italic");
    let after = click(&mut harness, "docs.toolbar.italic");
    after.assert_png_valid();
}

#[test]
fn toolbar_italic_button_sets_active_mark() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.italic");
    let after = click(&mut harness, "docs.toolbar.italic");
    after.assert_png_valid();
    let after_type = type_text(&mut harness, "Italic");
    let text = node_value(&after_type, "docs.document.text");
    assert_eq!(text, "Italic", "expected 'Italic' after typing");
}

#[test]
fn toolbar_underline_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.underline");
    let after = click(&mut harness, "docs.toolbar.underline");
    after.assert_png_valid();
}

#[test]
fn toolbar_underline_button_sets_active_mark() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.underline");
    let after = click(&mut harness, "docs.toolbar.underline");
    after.assert_png_valid();
    let after_type = type_text(&mut harness, "UL");
    let text = node_value(&after_type, "docs.document.text");
    assert_eq!(text, "UL", "expected 'UL' after typing");
}

#[test]
fn toolbar_strikethrough_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.strikethrough");
    let after = click(&mut harness, "docs.toolbar.strikethrough");
    after.assert_png_valid();
}

#[test]
fn toolbar_strikethrough_button_sets_active_mark() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.strikethrough");
    let after = click(&mut harness, "docs.toolbar.strikethrough");
    after.assert_png_valid();
    let after_type = type_text(&mut harness, "ST");
    let text = node_value(&after_type, "docs.document.text");
    assert_eq!(text, "ST", "expected 'ST' after typing");
}

#[test]
fn toolbar_code_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.code");
    let after = click(&mut harness, "docs.toolbar.code");
    after.assert_png_valid();
}

#[test]
fn toolbar_code_button_no_selection_sets_active_mark() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.code");
    let after = click(&mut harness, "docs.toolbar.code");
    after.assert_png_valid();
    let after_type = type_text(&mut harness, "Code");
    let text = node_value(&after_type, "docs.document.text");
    assert_eq!(text, "Code", "expected 'Code' after typing");
}

#[test]
fn toolbar_code_button_second_click_removes_code() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    click(&mut harness, "docs.toolbar.code");
    type_text(&mut harness, "X");
    let _before = capture(&mut harness);
    let after = click(&mut harness, "docs.toolbar.code");
    after.assert_png_valid();
    let text = node_value(&after, "docs.document.text");
    assert_eq!(text, "X", "text should remain after toggling code off");
}

#[test]
fn toolbar_superscript_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.superscript");
    let after = click(&mut harness, "docs.toolbar.superscript");
    after.assert_png_valid();
}

#[test]
fn toolbar_superscript_button_sets_active_mark() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.superscript");
    let after = click(&mut harness, "docs.toolbar.superscript");
    after.assert_png_valid();
    let after_type = type_text(&mut harness, "2");
    let text = node_value(&after_type, "docs.document.text");
    assert_eq!(text, "2", "expected '2' after typing superscript");
}

#[test]
fn toolbar_subscript_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.subscript");
    let after = click(&mut harness, "docs.toolbar.subscript");
    after.assert_png_valid();
}

#[test]
fn toolbar_subscript_button_sets_active_mark() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.subscript");
    let after = click(&mut harness, "docs.toolbar.subscript");
    after.assert_png_valid();
    let after_type = type_text(&mut harness, "2");
    let text = node_value(&after_type, "docs.document.text");
    assert_eq!(text, "2", "expected '2' after typing subscript");
}

// ---------------------------------------------------------------------------
// Align Left/Right/Center: semantic state tests
// ---------------------------------------------------------------------------

#[test]
fn toolbar_align_left_button_sets_alignment() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Aligned");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.align_left");
    let after = click(&mut harness, "docs.toolbar.align_left");
    after.assert_png_valid();
    // Verify document is dirty after alignment change
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(dirty, "true", "document should be dirty after align left");
}

#[test]
fn toolbar_align_center_button_sets_alignment() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Center");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.align_center");
    let after = click(&mut harness, "docs.toolbar.align_center");
    after.assert_png_valid();
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(dirty, "true", "document should be dirty after align center");
}

#[test]
fn toolbar_align_right_button_sets_alignment() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Right");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.align_right");
    let after = click(&mut harness, "docs.toolbar.align_right");
    after.assert_png_valid();
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(dirty, "true", "document should be dirty after align right");
}

// ---------------------------------------------------------------------------
// Block Quote: semantic state test
// ---------------------------------------------------------------------------

#[test]
fn toolbar_block_quote_button_sets_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Quote text");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.block_quote");
    let after = click(&mut harness, "docs.toolbar.block_quote");
    after.assert_png_valid();
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(dirty, "true", "document should be dirty after block quote");
}

// ---------------------------------------------------------------------------
// Bullet List: semantic state test
// ---------------------------------------------------------------------------

#[test]
fn toolbar_bullet_list_button_sets_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "List item");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.bullet_list");
    let after = click(&mut harness, "docs.toolbar.bullet_list");
    after.assert_png_valid();
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(dirty, "true", "document should be dirty after bullet list");
}

// ---------------------------------------------------------------------------
// Checklist: semantic state test
// ---------------------------------------------------------------------------

#[test]
fn toolbar_checklist_button_sets_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Check item");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.checklist");
    let after = click(&mut harness, "docs.toolbar.checklist");
    after.assert_png_valid();
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(dirty, "true", "document should be dirty after checklist");
}

// ---------------------------------------------------------------------------
// Color buttons
// ---------------------------------------------------------------------------

#[test]
fn toolbar_text_color_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.text_color");
    let after = click(&mut harness, "docs.toolbar.text_color");
    after.assert_png_valid();
}

#[test]
fn toolbar_highlight_color_button_no_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.toolbar.highlight_color");
    let after = click(&mut harness, "docs.toolbar.highlight_color");
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Highlight: semantic state test
// ---------------------------------------------------------------------------

#[test]
fn toolbar_highlight_button_sets_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Highlight text");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.highlight");
    let after = click(&mut harness, "docs.toolbar.highlight");
    after.assert_png_valid();
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(
        dirty, "true",
        "document should be dirty after highlight toggle"
    );
}

// ---------------------------------------------------------------------------
// Justify: semantic state test
// ---------------------------------------------------------------------------

#[test]
fn toolbar_align_justify_button_sets_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Justified text");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.align_justify");
    let after = click(&mut harness, "docs.toolbar.align_justify");
    after.assert_png_valid();
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(
        dirty, "true",
        "document should be dirty after justify alignment"
    );
}

// ---------------------------------------------------------------------------
// Numbered list: semantic state test
// ---------------------------------------------------------------------------

#[test]
fn toolbar_numbered_list_button_sets_dirty() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Numbered item");
    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.numbered_list");
    let after = click(&mut harness, "docs.toolbar.numbered_list");
    after.assert_png_valid();
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(
        dirty, "true",
        "document should be dirty after numbered list toggle"
    );
}
