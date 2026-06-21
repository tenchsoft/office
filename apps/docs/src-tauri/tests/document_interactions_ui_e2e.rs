/// UI automation tests for docs document interactions (#138-#169).
///
/// Uses debug_id selectors for document canvas, tabs, sidebar, ruler.
/// Semantic tests verify document state (text, cursor, selection, dirty)
/// through the automation nodes `docs.document.text`, `docs.document.cursor`,
/// `docs.document.selection`, and `docs.document.dirty`.
use tench_docs_lib::ui::DocsApp;
use tench_ui::core::events::{Modifiers, PointerEvent, PointerScrollEvent, WindowEvent};
use tench_ui_automation_core::{
    UiAutomationAction, UiAutomationKey, UiAutomationModifiers, UiAutomationSelector,
};
use tench_ui_test::assert_capture_changed;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

/// Helper: returns the `value` field of a node identified by `debug_id`.
///
/// Panics if the node is not found or has no value.
fn node_value(capture: &tench_ui_automation_core::UiAutomationCapture, debug_id: &str) -> String {
    get_node_value(capture, debug_id).unwrap_or_else(|| panic!("node '{debug_id}' has no value"))
}

/// Parses a cursor value like "0:5" into (block_idx, offset).
fn parse_cursor(val: &str) -> (usize, usize) {
    let mut parts = val.splitn(2, ':');
    let block: usize = parts.next().unwrap().parse().unwrap();
    let offset: usize = parts.next().unwrap().parse().unwrap();
    (block, offset)
}

// ---------------------------------------------------------------------------
// Structural presence tests
// ---------------------------------------------------------------------------

#[test]
fn document_canvas_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.document");
    assert_bounds_inside(&cap, "docs.document");
}

#[test]
fn ruler_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.ruler");
    assert_bounds_inside(&cap, "docs.ruler");
}

#[test]
fn title_row_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.title_row");
}

#[test]
fn status_bar_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.status_bar");
    assert_bounds_inside(&cap, "docs.status_bar");
}

#[test]
fn sidebar_thumbnails_toggle() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.thumbnails");
    assert_capture_changed(&before, &after);
}

// ---------------------------------------------------------------------------
// Visual regression test (original)
// ---------------------------------------------------------------------------

#[test]
fn document_text_input_changes_render() {
    let mut harness = make_harness();
    // Click on document first to focus
    click(&mut harness, "docs.document");
    let before = capture(&mut harness);

    let after = type_text(&mut harness, "Hello");
    assert_capture_changed(&before, &after);
}

// ---------------------------------------------------------------------------
// Semantic interaction tests
// ---------------------------------------------------------------------------

#[test]
fn document_click_places_cursor() {
    let mut harness = make_harness();

    // Click on the document to focus it.
    click(&mut harness, "docs.document");
    let cap = capture(&mut harness);

    // The cursor node must exist.
    let cursor_val = node_value(&cap, "docs.document.cursor");
    // Cursor value is "block_idx:offset" — just verify it parses.
    let (_block, _offset) = parse_cursor(&cursor_val);
}

#[test]
fn document_text_input_updates_text_node() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let after = type_text(&mut harness, "Hello");

    // Text node should contain "Hello".
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.contains("Hello"),
        "expected docs.document.text to contain \"Hello\", got: \"{text}\""
    );

    // Dirty flag should be "true".
    let dirty = node_value(&after, "docs.document.dirty");
    assert_eq!(dirty, "true", "document should be dirty after typing");
}

#[test]
fn document_backspace_deletes_character() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "AB");

    // Press Backspace to delete "B".
    key(
        &mut harness,
        UiAutomationKey::Backspace,
        UiAutomationModifiers::default(),
    );

    let cap = capture(&mut harness);
    let text = node_value(&cap, "docs.document.text");
    assert!(
        text.contains("A") && !text.contains("AB"),
        "expected text to be \"A\" after backspace, got: \"{text}\""
    );
}

#[test]
fn document_enter_creates_new_block() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "Line1");

    // Press Enter to create a new block.
    key(
        &mut harness,
        UiAutomationKey::Enter,
        UiAutomationModifiers::default(),
    );

    type_text(&mut harness, "Line2");

    let cap = capture(&mut harness);
    let text = node_value(&cap, "docs.document.text");
    assert!(
        text.contains("Line1"),
        "expected text to contain \"Line1\", got: \"{text}\""
    );
    assert!(
        text.contains("Line2"),
        "expected text to contain \"Line2\", got: \"{text}\""
    );
}

#[test]
fn document_arrow_keys_move_cursor() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "ABCD");

    // Record cursor position after typing.
    let before = capture(&mut harness);
    let cursor_before = node_value(&before, "docs.document.cursor");
    let (_, offset_before) = parse_cursor(&cursor_before);

    // Press Left arrow — cursor offset should decrease.
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );

    let after = capture(&mut harness);
    let cursor_after = node_value(&after, "docs.document.cursor");
    let (_, offset_after) = parse_cursor(&cursor_after);

    assert!(
        offset_after < offset_before,
        "expected cursor offset to decrease after Left arrow: before={offset_before}, after={offset_after}"
    );
}

#[test]
fn document_shift_arrow_selects_text() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");

    // Shift+Left to select one character.
    key_chord(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );

    let cap = capture(&mut harness);
    let selection = node_value(&cap, "docs.document.selection");
    assert_ne!(
        selection, "none",
        "expected a selection after Shift+Left, got: \"{selection}\""
    );
}

#[test]
fn document_delete_key_removes_char() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "AB");

    // Move cursor left so it sits between "A" and "B".
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );

    // Press Delete — should remove "B" (the character to the right of cursor).
    key(
        &mut harness,
        UiAutomationKey::Delete,
        UiAutomationModifiers::default(),
    );

    let cap = capture(&mut harness);
    let text = node_value(&cap, "docs.document.text");
    assert!(
        text.contains("A") && !text.contains("AB"),
        "expected text to be \"A\" after Delete, got: \"{text}\""
    );
}

// ---------------------------------------------------------------------------
// Arrow cursor movement — comprehensive tests
// ---------------------------------------------------------------------------

#[test]
fn arrow_left_right_move_cursor_by_character() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "abc");
    assert_eq!(node_value(&typed, "docs.document.cursor"), "0:3");

    let left = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&left, "docs.document.cursor"), "0:2");
    assert_eq!(node_value(&left, "docs.document.selection"), "none");

    let left2 = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&left2, "docs.document.cursor"), "0:1");

    let right = key(
        &mut harness,
        UiAutomationKey::ArrowRight,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&right, "docs.document.cursor"), "0:2");
}

#[test]
fn arrow_movement_clears_existing_selection() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "abc");
    let selected = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_ne!(node_value(&selected, "docs.document.selection"), "none");

    let moved = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&moved, "docs.document.selection"), "none");
}

#[test]
fn arrow_left_at_document_start_is_clamped() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let before = capture(&mut harness);
    assert_eq!(node_value(&before, "docs.document.cursor"), "0:0");
    let text_before = node_value(&before, "docs.document.text");

    for _ in 0..5 {
        key(
            &mut harness,
            UiAutomationKey::ArrowLeft,
            UiAutomationModifiers::default(),
        );
    }
    let after = capture(&mut harness);
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:0");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

#[test]
fn arrow_right_at_document_end_is_clamped() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "abc");
    assert_eq!(node_value(&typed, "docs.document.cursor"), "0:3");

    for _ in 0..5 {
        key(
            &mut harness,
            UiAutomationKey::ArrowRight,
            UiAutomationModifiers::default(),
        );
    }
    let after = capture(&mut harness);
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:3");
    assert_eq!(node_value(&after, "docs.document.text"), "abc");
}

#[test]
fn arrow_movement_handles_unicode_character_boundaries() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "한글");
    assert_eq!(node_value(&typed, "docs.document.text"), "한글");

    // "한" is 3 bytes, "글" is 3 bytes. After typing "한글", cursor is at offset 6.
    let left = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );
    // Cursor should move back one Unicode character (3 bytes)
    assert_eq!(node_value(&left, "docs.document.cursor"), "0:3");

    let left2 = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&left2, "docs.document.cursor"), "0:0");
}

// ---------------------------------------------------------------------------
// Backspace key — comprehensive tests
// ---------------------------------------------------------------------------

#[test]
fn backspace_deletes_previous_character_and_updates_cursor() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "abc");
    assert_eq!(node_value(&typed, "docs.document.text"), "abc");
    assert_eq!(node_value(&typed, "docs.document.cursor"), "0:3");

    let after = key(
        &mut harness,
        UiAutomationKey::Backspace,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&after, "docs.document.text"), "ab");
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:2");
    assert_eq!(node_value(&after, "docs.document.selection"), "none");
    assert_eq!(node_value(&after, "docs.document.dirty"), "true");
}

#[test]
fn backspace_deletes_selected_range() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "abcd");
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    let selected = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_ne!(node_value(&selected, "docs.document.selection"), "none");

    let after = key(
        &mut harness,
        UiAutomationKey::Backspace,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&after, "docs.document.text"), "ab");
    assert_eq!(node_value(&after, "docs.document.selection"), "none");
}

#[test]
fn backspace_at_document_start_is_stable() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let before = capture(&mut harness);
    assert_eq!(node_value(&before, "docs.document.cursor"), "0:0");
    assert_eq!(node_value(&before, "docs.document.text"), "");

    for _ in 0..5 {
        key(
            &mut harness,
            UiAutomationKey::Backspace,
            UiAutomationModifiers::default(),
        );
    }
    let after = capture(&mut harness);
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:0");
    assert_eq!(node_value(&after, "docs.document.text"), "");
    assert_eq!(node_value(&after, "docs.document.selection"), "none");
}

#[test]
fn backspace_deletes_one_unicode_character() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "한글");
    assert_eq!(node_value(&typed, "docs.document.text"), "한글");

    let after = key(
        &mut harness,
        UiAutomationKey::Backspace,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&after, "docs.document.text"), "한");
}

// ---------------------------------------------------------------------------
// Delete key — comprehensive tests
// ---------------------------------------------------------------------------

#[test]
fn delete_removes_character_after_cursor_and_keeps_cursor_position() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "abc");
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );
    let before = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&before, "docs.document.cursor"), "0:1");

    let after = key(
        &mut harness,
        UiAutomationKey::Delete,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&after, "docs.document.text"), "ac");
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:1");
    assert_eq!(node_value(&after, "docs.document.selection"), "none");
}

#[test]
fn delete_removes_selected_range() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "abcd");
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    let selected = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_ne!(node_value(&selected, "docs.document.selection"), "none");

    let after = key(
        &mut harness,
        UiAutomationKey::Delete,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&after, "docs.document.text"), "ab");
    assert_eq!(node_value(&after, "docs.document.selection"), "none");
}

#[test]
fn delete_at_document_end_is_stable() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "abc");
    assert_eq!(node_value(&typed, "docs.document.cursor"), "0:3");

    for _ in 0..5 {
        key(
            &mut harness,
            UiAutomationKey::Delete,
            UiAutomationModifiers::default(),
        );
    }
    let after = capture(&mut harness);
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:3");
    assert_eq!(node_value(&after, "docs.document.text"), "abc");
    assert_eq!(node_value(&after, "docs.document.selection"), "none");
}

#[test]
fn delete_removes_one_unicode_character() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "한글");
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );
    let before = capture(&mut harness);
    assert_eq!(node_value(&before, "docs.document.cursor"), "0:3");

    let after = key(
        &mut harness,
        UiAutomationKey::Delete,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&after, "docs.document.text"), "한");
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:3");
}

// ---------------------------------------------------------------------------
// Enter key — comprehensive tests
// ---------------------------------------------------------------------------

#[test]
fn enter_inserts_newline_and_moves_cursor() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "Alpha");
    let after_enter = key(
        &mut harness,
        UiAutomationKey::Enter,
        UiAutomationModifiers::default(),
    );
    let after_text = type_text(&mut harness, "Beta");

    assert_capture_changed(&capture(&mut harness), &after_enter);
    assert_eq!(node_value(&after_text, "docs.document.text"), "Alpha\nBeta");
    // After "Alpha\nBeta", cursor should be at offset 10 in block 0
    assert_eq!(node_value(&after_text, "docs.document.cursor"), "0:10");
    assert_eq!(node_value(&after_text, "docs.document.selection"), "none");
    assert_eq!(node_value(&after_text, "docs.document.dirty"), "true");
}

#[test]
fn enter_replaces_selected_range_with_newline() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "abcd");
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    let selected = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_ne!(node_value(&selected, "docs.document.selection"), "none");

    let after = key(
        &mut harness,
        UiAutomationKey::Enter,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&after, "docs.document.text"), "ab\n");
    assert_eq!(node_value(&after, "docs.document.selection"), "none");
}

#[test]
fn enter_repeatedly_inserts_bounded_newlines() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "A");
    for _ in 0..3 {
        key(
            &mut harness,
            UiAutomationKey::Enter,
            UiAutomationModifiers::default(),
        );
    }
    let after = capture(&mut harness);
    assert_eq!(node_value(&after, "docs.document.text"), "A\n\n\n");
}

#[test]
fn enter_around_unicode_keeps_valid_text_and_cursor() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "한글");
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );
    let after_enter = key(
        &mut harness,
        UiAutomationKey::Enter,
        UiAutomationModifiers::default(),
    );
    // "한\n글" — 한 is 3 bytes, \n is 1 byte, 글 is 3 bytes
    assert_eq!(node_value(&after_enter, "docs.document.text"), "한\n글");
}

// ---------------------------------------------------------------------------
// Ctrl+Arrow movement — word and document boundaries
// ---------------------------------------------------------------------------

fn ctrl() -> UiAutomationModifiers {
    UiAutomationModifiers {
        control: true,
        ..Default::default()
    }
}

#[test]
fn ctrl_left_and_right_move_by_word() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "alpha beta gamma");
    assert_eq!(node_value(&typed, "docs.document.cursor"), "0:16");

    let left = key(&mut harness, UiAutomationKey::ArrowLeft, ctrl());
    assert_eq!(node_value(&left, "docs.document.cursor"), "0:11");
    assert_eq!(node_value(&left, "docs.document.selection"), "none");

    let left_again = key(&mut harness, UiAutomationKey::ArrowLeft, ctrl());
    assert_eq!(node_value(&left_again, "docs.document.cursor"), "0:6");

    let right = key(&mut harness, UiAutomationKey::ArrowRight, ctrl());
    assert_eq!(node_value(&right, "docs.document.cursor"), "0:10");
}

#[test]
fn ctrl_up_down_home_end_move_to_document_boundaries() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "first");
    key(
        &mut harness,
        UiAutomationKey::Enter,
        UiAutomationModifiers::default(),
    );
    let typed = type_text(&mut harness, "second");
    assert_eq!(node_value(&typed, "docs.document.cursor"), "0:12");

    let start = key(&mut harness, UiAutomationKey::ArrowUp, ctrl());
    assert_eq!(node_value(&start, "docs.document.cursor"), "0:0");

    let end = key(&mut harness, UiAutomationKey::ArrowDown, ctrl());
    assert_eq!(node_value(&end, "docs.document.cursor"), "0:12");

    let start_home = key(&mut harness, UiAutomationKey::Home, ctrl());
    assert_eq!(node_value(&start_home, "docs.document.cursor"), "0:0");

    let end_key = key(&mut harness, UiAutomationKey::End, ctrl());
    assert_eq!(node_value(&end_key, "docs.document.cursor"), "0:12");
}

#[test]
fn repeated_ctrl_movement_clamps_at_document_boundaries() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "alpha beta");
    let text_before = node_value(&typed, "docs.document.text");

    for _ in 0..5 {
        key(&mut harness, UiAutomationKey::ArrowLeft, ctrl());
    }
    let at_start = capture(&mut harness);
    assert_eq!(node_value(&at_start, "docs.document.cursor"), "0:0");
    assert_eq!(node_value(&at_start, "docs.document.text"), text_before);

    for _ in 0..5 {
        key(&mut harness, UiAutomationKey::ArrowRight, ctrl());
    }
    let at_end = capture(&mut harness);
    assert_eq!(node_value(&at_end, "docs.document.cursor"), "0:10");
    assert_eq!(node_value(&at_end, "docs.document.text"), text_before);
}

#[test]
fn ctrl_arrow_clears_existing_selection() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "alpha beta");
    let selected = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_ne!(node_value(&selected, "docs.document.selection"), "none");

    let moved = key(&mut harness, UiAutomationKey::ArrowLeft, ctrl());
    assert_eq!(node_value(&moved, "docs.document.selection"), "none");
}

// ---------------------------------------------------------------------------
// Shift+Arrow selection — comprehensive tests
// ---------------------------------------------------------------------------

#[test]
fn shift_left_repeated_extends_selection() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "abcd");

    let one = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_ne!(node_value(&one, "docs.document.selection"), "none");

    let two = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    // Selection should be larger after two shift+left presses
    assert_ne!(node_value(&two, "docs.document.selection"), "none");
    // The selected text should be "cd" (two chars selected)
    let sel_text = node_value(&two, "docs.document.selected_text");
    assert_eq!(sel_text, "cd");
}

#[test]
fn shift_arrow_at_boundary_does_not_create_invalid_selection() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let start = capture(&mut harness);
    assert_eq!(node_value(&start, "docs.document.cursor"), "0:0");

    for _ in 0..3 {
        key(
            &mut harness,
            UiAutomationKey::ArrowLeft,
            UiAutomationModifiers {
                shift: true,
                ..Default::default()
            },
        );
    }
    let after = capture(&mut harness);
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:0");
}

#[test]
fn plain_arrow_after_shift_selection_clears_selection() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "abcd");
    let selected = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_ne!(node_value(&selected, "docs.document.selection"), "none");

    let moved = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers::default(),
    );
    assert_eq!(node_value(&moved, "docs.document.selection"), "none");
}

#[test]
fn shift_arrow_selection_handles_unicode_text() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "한글");
    let selected = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    // Should select "글" (one Unicode character)
    let sel_text = node_value(&selected, "docs.document.selected_text");
    assert_eq!(sel_text, "글");
}

// ---------------------------------------------------------------------------
// Text input — comprehensive tests
// ---------------------------------------------------------------------------

#[test]
fn typed_characters_insert_at_cursor_and_mark_dirty() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let before = capture(&mut harness);
    let after = type_text(&mut harness, "Hello");

    assert_capture_changed(&before, &after);
    assert_eq!(node_value(&after, "docs.document.text"), "Hello");
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:5");
    assert_eq!(node_value(&after, "docs.document.selection"), "none");
    assert_eq!(node_value(&after, "docs.document.dirty"), "true");
}

#[test]
fn text_input_replaces_selected_text() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "abcd");
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    let selected = key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_eq!(node_value(&selected, "docs.document.selected_text"), "cd");

    let replaced = type_text(&mut harness, "XY");
    assert_eq!(node_value(&replaced, "docs.document.text"), "abXY");
    assert_eq!(node_value(&replaced, "docs.document.selection"), "none");
}

#[test]
fn ime_commit_inserts_unicode_text() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    harness.dispatch_text(tench_ui::core::events::TextEvent::Ime(
        tench_ui::core::events::ImeEvent::Commit("한글".to_string()),
    ));
    let after = capture(&mut harness);

    assert_eq!(node_value(&after, "docs.document.text"), "한글");
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:6");
    assert_eq!(node_value(&after, "docs.document.dirty"), "true");
}

#[test]
fn clipboard_paste_inserts_multiline_text() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    harness.dispatch_text(tench_ui::core::events::TextEvent::ClipboardPaste(
        "Alpha\nBeta".to_string(),
    ));
    let after = capture(&mut harness);

    assert_eq!(node_value(&after, "docs.document.text"), "Alpha\nBeta");
    assert_eq!(node_value(&after, "docs.document.cursor"), "0:10");
}

// ---------------------------------------------------------------------------
// Scroll wheel — comprehensive tests
// ---------------------------------------------------------------------------

fn scroll_document(
    harness: &mut TestHarness,
    delta_y: f64,
) -> tench_ui_automation_core::UiAutomationCapture {
    harness
        .automation_action(UiAutomationAction::Scroll {
            selector: UiAutomationSelector::ByDebugId {
                debug_id: "docs.document".to_string(),
            },
            delta_x: 0.0,
            delta_y,
        })
        .expect("scroll document")
}

fn fill_long_document(harness: &mut TestHarness) {
    click(harness, "docs.document");
    for idx in 0..80 {
        type_text(harness, &format!("Line {idx}"));
        key(
            harness,
            UiAutomationKey::Enter,
            UiAutomationModifiers::default(),
        );
    }
}

#[test]
fn plain_wheel_scrolls_viewport_without_mutating_document() {
    let mut harness = make_harness();
    fill_long_document(&mut harness);

    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let zoom_before = node_value(&before, "docs.document.zoom");

    let after = scroll_document(&mut harness, 240.0);

    assert_capture_changed(&before, &after);
    assert_ne!(node_value(&after, "docs.document.scroll_y"), "0");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.zoom"), zoom_before);
}

#[test]
fn plain_wheel_up_at_top_clamps_to_zero() {
    let mut harness = make_harness();
    fill_long_document(&mut harness);

    let before = capture(&mut harness);
    assert_eq!(node_value(&before, "docs.document.scroll_y"), "0");

    let after = scroll_document(&mut harness, -240.0);
    assert_eq!(node_value(&after, "docs.document.scroll_y"), "0");
}

#[test]
fn plain_wheel_with_existing_selection_preserves_selection() {
    let mut harness = make_harness();
    fill_long_document(&mut harness);

    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        UiAutomationModifiers {
            shift: true,
            ..Default::default()
        },
    );
    let selected = capture(&mut harness);
    assert_ne!(node_value(&selected, "docs.document.selection"), "none");
    let selection_before = node_value(&selected, "docs.document.selection");

    let after = scroll_document(&mut harness, 240.0);
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before
    );
}

// ---------------------------------------------------------------------------
// Ctrl+Wheel zoom — comprehensive tests
// ---------------------------------------------------------------------------

fn ctrl_scroll(
    harness: &mut TestHarness,
    delta_y: f64,
) -> tench_ui_automation_core::UiAutomationCapture {
    let point = harness
        .automation_bounds(&UiAutomationSelector::ByDebugId {
            debug_id: "docs.document".to_string(),
        })
        .expect("document bounds")
        .center();
    harness.dispatch_pointer(PointerEvent::Scroll(PointerScrollEvent {
        pos: tench_ui::kurbo::Point::new(point.x, point.y),
        delta: tench_ui::kurbo::Vec2::new(0.0, delta_y),
        modifiers: Modifiers {
            control: true,
            ..Default::default()
        },
    }));
    capture(harness)
}

#[test]
fn ctrl_wheel_zoom_in_updates_zoom() {
    let mut harness = make_harness();

    let before = capture(&mut harness);
    assert_eq!(node_value(&before, "docs.document.zoom"), "100");

    let after = ctrl_scroll(&mut harness, 120.0);
    assert_eq!(node_value(&after, "docs.document.zoom"), "110");
}

#[test]
fn ctrl_wheel_zoom_out_updates_zoom() {
    let mut harness = make_harness();

    let after = ctrl_scroll(&mut harness, -120.0);
    assert_eq!(node_value(&after, "docs.document.zoom"), "90");
}

#[test]
fn plain_wheel_does_not_zoom() {
    let mut harness = make_harness();

    let before = capture(&mut harness);
    let zoom_before = node_value(&before, "docs.document.zoom");

    let after = scroll_document(&mut harness, 120.0);
    assert_eq!(node_value(&after, "docs.document.zoom"), zoom_before);
}

#[test]
fn ctrl_wheel_zoom_clamps_to_min_and_max() {
    let mut harness = make_harness();

    // Zoom in a lot
    for _ in 0..20 {
        ctrl_scroll(&mut harness, 120.0);
    }
    let maxed = capture(&mut harness);
    assert_eq!(node_value(&maxed, "docs.document.zoom"), "200");

    // Zoom out a lot
    for _ in 0..30 {
        ctrl_scroll(&mut harness, -120.0);
    }
    let mined = capture(&mut harness);
    assert_eq!(node_value(&mined, "docs.document.zoom"), "50");
}

#[test]
fn ctrl_wheel_zoom_preserves_document_text() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    let typed = type_text(&mut harness, "Body");
    let text_before = node_value(&typed, "docs.document.text");

    let after = ctrl_scroll(&mut harness, 120.0);
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// ---------------------------------------------------------------------------
// File drop open — basic tests
// ---------------------------------------------------------------------------

#[test]
fn dropping_supported_text_file_opens_new_active_tab() {
    let mut harness = make_harness();

    let dir = std::env::temp_dir().join(format!(
        "tench_docs_drop_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Dropped.txt");
    std::fs::write(&file, "Dropped content").unwrap();

    let before = capture(&mut harness);
    assert_eq!(node_value(&before, "docs.tabs.count"), "1");

    harness.dispatch_window(WindowEvent::FileDrop {
        paths: vec![file.to_string_lossy().to_string()],
    });
    let after = capture(&mut harness);

    assert_eq!(node_value(&after, "docs.tabs.count"), "2");
    assert_eq!(node_value(&after, "docs.tabs.active_index"), "1");
    assert!(
        node_value(&after, "docs.document.text").contains("Dropped content"),
        "expected dropped file text, got: {:?}",
        node_value(&after, "docs.document.text")
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn empty_file_drop_is_ignored() {
    let mut harness = make_harness();

    let before = capture(&mut harness);
    harness.dispatch_window(WindowEvent::FileDrop { paths: Vec::new() });
    let after = capture(&mut harness);

    assert_eq!(
        node_value(&after, "docs.tabs.count"),
        node_value(&before, "docs.tabs.count")
    );
    assert_eq!(
        node_value(&after, "docs.document.text"),
        node_value(&before, "docs.document.text")
    );
}

// ---------------------------------------------------------------------------
// Drag text selection — basic tests
// ---------------------------------------------------------------------------

#[test]
fn drag_from_to_creates_selection() {
    let mut harness = make_harness();

    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello World");

    // Drag from document area to create a selection
    let after = drag_from_to(&mut harness, "docs.document", "docs.document");
    // The drag should not crash and the document should still contain the text
    let text = node_value(&after, "docs.document.text");
    assert!(
        text.contains("Hello"),
        "expected text to still contain 'Hello' after drag, got: \"{text}\""
    );
}

// ---------------------------------------------------------------------------
// Automation node presence tests for new nodes
// ---------------------------------------------------------------------------

#[test]
fn document_cursor_visible_node_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.document.cursor_visible");
}

#[test]
fn document_zoom_node_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.document.zoom");
    assert_eq!(node_value(&cap, "docs.document.zoom"), "100");
}

#[test]
fn document_scroll_y_node_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.document.scroll_y");
    assert_eq!(node_value(&cap, "docs.document.scroll_y"), "0");
}

#[test]
fn document_paragraph_count_node_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.document.paragraph_count");
    assert_eq!(node_value(&cap, "docs.document.paragraph_count"), "1");
}

#[test]
fn document_selected_text_node_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.document.selected_text");
    assert_eq!(node_value(&cap, "docs.document.selected_text"), "");
}

#[test]
fn document_page_node_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.document.page");
}

#[test]
fn tabs_count_node_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.tabs.count");
    assert_eq!(node_value(&cap, "docs.tabs.count"), "1");
}

#[test]
fn tabs_active_index_node_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.tabs.active_index");
    assert_eq!(node_value(&cap, "docs.tabs.active_index"), "0");
}
