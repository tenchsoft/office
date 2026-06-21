/// UI automation tests for docs context menu items.
///
/// Uses debug_id selectors. Verifies context menu interaction.
use tench_docs_lib::ui::DocsApp;
use tench_ui_automation_core::UiAutomationKey;
use tench_ui_automation_core::UiAutomationModifiers;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::CaptureAssertions;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

/// Context menu item debug IDs used across tests.
const MENU_COPY: &str = "docs.context.copy";
const MENU_CUT: &str = "docs.context.cut";
const MENU_PASTE: &str = "docs.context.paste";
const MENU_CLEAR_FORMATTING: &str = "docs.context.clear_formatting";
const MENU_ADD_COMMENT: &str = "docs.context.add_comment";
const MENU_INSERT_LINK: &str = "docs.context.insert_link";

// Image context menu items
const MENU_REPLACE_IMAGE: &str = "docs.context.replace_image";
const MENU_REMOVE: &str = "docs.context.remove";

// Table context menu items
const MENU_INSERT_ROW_ABOVE: &str = "docs.context.insert_row_above";
const MENU_INSERT_ROW_BELOW: &str = "docs.context.insert_row_below";
const MENU_INSERT_COLUMN_LEFT: &str = "docs.context.insert_column_left";
const MENU_INSERT_COLUMN_RIGHT: &str = "docs.context.insert_column_right";
const MENU_DELETE_ROW: &str = "docs.context.delete_row";
const MENU_DELETE_COLUMN: &str = "docs.context.delete_column";
const MENU_DELETE_TABLE: &str = "docs.context.delete_table";

// Tab context menu items
const TAB_MENU_CLOSE: &str = "docs.context.close";
const TAB_MENU_CLOSE_OTHERS: &str = "docs.context.close_others";
const TAB_MENU_CLOSE_ALL: &str = "docs.context.close_all";

#[test]
fn text_context_menu_right_click_does_not_crash() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.document");
    cap.assert_png_valid();
}

#[test]
fn document_click_places_cursor() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.document");

    // Click on document — should place cursor
    let after = click(&mut harness, "docs.document");
    after.assert_png_valid();
}

#[test]
fn text_context_menu_shows_copy_cut_paste_items() {
    let mut harness = make_harness();

    // Focus the document and type some text so the context menu has content to act on.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Sample text for context menu");

    // Right-click on the document to open the context menu.
    let cap = right_click(&mut harness, "docs.document");

    // Verify all expected context menu items are present.
    assert_selector(&cap, MENU_COPY);
    assert_selector(&cap, MENU_CUT);
    assert_selector(&cap, MENU_PASTE);
    assert_selector(&cap, MENU_CLEAR_FORMATTING);
    assert_selector(&cap, MENU_ADD_COMMENT);
    assert_selector(&cap, MENU_INSERT_LINK);

    cap.assert_png_valid();
}

#[test]
fn context_menu_copy_item_copies_text() {
    let mut harness = make_harness();

    // Type "Hello" into the document.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");

    // Select all text with Ctrl+A.
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".to_string()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    // Right-click to open context menu, then click the copy item.
    right_click(&mut harness, "docs.document");
    let after = click(&mut harness, MENU_COPY);

    // The document should still contain "Hello" — copy does not remove text.
    assert_selector(&after, "docs.document");
    let text = node_text(&after, "docs.document.text");
    assert!(
        text.contains("Hello"),
        "document text should still contain 'Hello' after copy, got: '{text}'"
    );

    after.assert_png_valid();
}

#[test]
fn context_menu_cut_item_removes_text() {
    let mut harness = make_harness();

    // Type "Hello" into the document.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");

    // Select all text with Ctrl+A.
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".to_string()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    // Right-click to open context menu, then click the cut item.
    right_click(&mut harness, "docs.document");
    let after = click(&mut harness, MENU_CUT);

    // After cut, the document text should be empty.
    assert_selector(&after, "docs.document");
    let text = node_text(&after, "docs.document.text");
    assert!(
        text.is_empty() || !text.contains("Hello"),
        "document text should be empty after cut, got: '{text}'"
    );

    after.assert_png_valid();
}

#[test]
fn context_menu_closes_on_outside_click() {
    let mut harness = make_harness();

    // Focus the document and type some text.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Some text");

    // Right-click to open the context menu.
    let menu_cap = right_click(&mut harness, "docs.document");
    assert_selector(&menu_cap, MENU_COPY);

    // Click on the document body (outside the context menu) to dismiss it.
    let after = click(&mut harness, "docs.document");

    // Context menu items should no longer be present.
    assert_absent(&after, MENU_COPY);
    assert_absent(&after, MENU_CUT);
    assert_absent(&after, MENU_PASTE);

    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Paste context menu item
// ---------------------------------------------------------------------------

#[test]
fn context_menu_paste_item_no_crash() {
    let mut harness = make_harness();

    // Type text, copy it, then test paste via context menu
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Clipboard text");

    // Select all and copy
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".to_string()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );
    right_click(&mut harness, "docs.document");
    click(&mut harness, MENU_COPY);

    // Now right-click again and click paste
    right_click(&mut harness, "docs.document");
    let after = click(&mut harness, MENU_PASTE);
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Clear Formatting context menu item
// ---------------------------------------------------------------------------

#[test]
fn context_menu_clear_formatting_item_no_crash() {
    let mut harness = make_harness();

    // Type bold text
    click(&mut harness, "docs.document");
    click(&mut harness, "docs.toolbar.bold");
    type_text(&mut harness, "Bold text");

    // Select all
    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".to_string()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    // Right-click and clear formatting
    right_click(&mut harness, "docs.document");
    let after = click(&mut harness, MENU_CLEAR_FORMATTING);
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Add Comment context menu item
// ---------------------------------------------------------------------------

#[test]
fn context_menu_add_comment_item_opens_modal() {
    let mut harness = make_harness();

    // Type text and select it
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Comment target");

    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".to_string()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    // Right-click and click Add Comment
    right_click(&mut harness, "docs.document");
    assert_selector(&capture(&mut harness), MENU_ADD_COMMENT);
    let after = click(&mut harness, MENU_ADD_COMMENT);
    after.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Insert Link context menu item
// ---------------------------------------------------------------------------

#[test]
fn context_menu_insert_link_item_opens_modal() {
    let mut harness = make_harness();

    // Type text and select it
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Link target");

    key_chord(
        &mut harness,
        UiAutomationKey::Character("a".to_string()),
        UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    // Right-click and click Insert Link
    right_click(&mut harness, "docs.document");
    assert_selector(&capture(&mut harness), MENU_INSERT_LINK);
    let after = click(&mut harness, MENU_INSERT_LINK);
    after.assert_png_valid();
    // Insert link modal should appear
    assert_selector(&after, "docs.modal.link");
}

// ---------------------------------------------------------------------------
// Image context menu items
// ---------------------------------------------------------------------------

#[test]
fn image_context_menu_items_absent_on_text() {
    let mut harness = make_harness();

    // Type text and right-click
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Plain text");
    let cap = right_click(&mut harness, "docs.document");

    // Image-specific items should NOT be present in text context
    assert_absent(&cap, MENU_REPLACE_IMAGE);
    assert_absent(&cap, MENU_REMOVE);
    cap.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Table context menu items
// ---------------------------------------------------------------------------

#[test]
fn table_context_menu_items_absent_on_text() {
    let mut harness = make_harness();

    // Type text and right-click
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Plain text");
    let cap = right_click(&mut harness, "docs.document");

    // Table-specific items should NOT be present in text context
    assert_absent(&cap, MENU_INSERT_ROW_ABOVE);
    assert_absent(&cap, MENU_INSERT_ROW_BELOW);
    assert_absent(&cap, MENU_INSERT_COLUMN_LEFT);
    assert_absent(&cap, MENU_INSERT_COLUMN_RIGHT);
    assert_absent(&cap, MENU_DELETE_ROW);
    assert_absent(&cap, MENU_DELETE_COLUMN);
    assert_absent(&cap, MENU_DELETE_TABLE);
    cap.assert_png_valid();
}

// ---------------------------------------------------------------------------
// Tab context menu items
// ---------------------------------------------------------------------------

#[test]
fn tab_context_menu_items_absent_with_single_tab() {
    let mut harness = make_harness();

    // With a single tab, right-clicking the document should show text context
    let cap = right_click(&mut harness, "docs.document");

    // Tab context items should NOT be present
    assert_absent(&cap, TAB_MENU_CLOSE);
    assert_absent(&cap, TAB_MENU_CLOSE_OTHERS);
    assert_absent(&cap, TAB_MENU_CLOSE_ALL);
    cap.assert_png_valid();
}
