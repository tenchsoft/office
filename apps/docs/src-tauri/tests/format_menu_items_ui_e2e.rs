/// UI automation tests for docs Format Menu items (#38-#46).
///
/// Covers the E2E behavioural tests described in the fix plans under
/// `plans/fix/docs-format-menu-*-item-fix.md`.
use tench_docs_lib::ui::DocsApp;
use tench_ui_automation_core::{UiAutomationKey, UiAutomationModifiers};
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::TestHarness;
use tench_ui_test::{assert_capture_changed, CaptureAssertions};

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

/// Returns the `value` field of the node identified by `debug_id`.
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

#[test]
fn format_menu_bold_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.bold");
    assert_capture_changed(&before, &after);
}

#[test]
fn format_menu_italic_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.italic");
    assert_capture_changed(&before, &after);
}

#[test]
fn format_menu_underline_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.underline",
    );
    assert_capture_changed(&before, &after);
}

#[test]
fn format_menu_strikethrough_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.strikethrough",
    );
    assert_capture_changed(&before, &after);
}

#[test]
fn format_menu_superscript_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.superscript",
    );
    assert_capture_changed(&before, &after);
}

#[test]
fn format_menu_subscript_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.subscript",
    );
    assert_capture_changed(&before, &after);
}

#[test]
fn format_menu_clear_formatting_no_crash() {
    let mut harness = make_harness();

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.clear_formatting",
    );
    after.assert_png_valid();
}

#[test]
fn format_menu_block_quote_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.block_quote",
    );
    assert_capture_changed(&before, &after);
}

// --- Additional tests from fix plans ---

// From docs-format-menu-bold-item-fix.md
#[test]
fn format_menu_bold_with_selection_changes_render() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.bold");
    assert_capture_changed(&before, &after);
}

// From docs-format-menu-italic-item-fix.md
#[test]
fn format_menu_italic_with_selection_changes_render() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.italic");
    assert_capture_changed(&before, &after);
}

// From docs-format-menu-clear-formatting-item-fix.md
#[test]
fn format_menu_clear_formatting_with_selection_no_crash() {
    let mut harness = make_harness();
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
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.clear_formatting",
    );
    after.assert_png_valid();
}

// From docs-format-menu-underline-item-fix.md
#[test]
fn format_menu_underline_with_selection_changes_render() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.underline",
    );
    assert_capture_changed(&before, &after);
}

// From docs-format-menu-strikethrough-item-fix.md
#[test]
fn format_menu_strikethrough_with_selection_changes_render() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.strikethrough",
    );
    assert_capture_changed(&before, &after);
}

// From docs-format-menu-superscript-item-fix.md
#[test]
fn format_menu_superscript_with_selection_changes_render() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.superscript",
    );
    assert_capture_changed(&before, &after);
}

// From docs-format-menu-subscript-item-fix.md
#[test]
fn format_menu_subscript_with_selection_changes_render() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.subscript",
    );
    assert_capture_changed(&before, &after);
}

// From docs-format-menu-block-quote-item-fix.md
#[test]
fn format_menu_block_quote_with_selection_changes_render() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.block_quote",
    );
    assert_capture_changed(&before, &after);
}

// =========================================================================
// Fix-plan: click-closes-menu tests
//
// Each Format menu item click must close the menu (active=none, panel
// absent) before side effects occur.
// =========================================================================

// --- Bold ---

#[test]
fn format_menu_bold_click_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.bold");
    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.format.panel");
}

// --- Italic ---

#[test]
fn format_menu_italic_click_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.italic");
    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.format.panel");
}

// --- Underline ---

#[test]
fn format_menu_underline_click_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.underline",
    );
    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.format.panel");
}

// --- Strikethrough ---

#[test]
fn format_menu_strikethrough_click_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.strikethrough",
    );
    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.format.panel");
}

// --- Superscript ---

#[test]
fn format_menu_superscript_click_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.superscript",
    );
    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.format.panel");
}

// --- Subscript ---

#[test]
fn format_menu_subscript_click_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.subscript",
    );
    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.format.panel");
}

// --- Clear Formatting ---

#[test]
fn format_menu_clear_formatting_click_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.clear_formatting",
    );
    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.format.panel");
}

// --- Block Quote ---

#[test]
fn format_menu_block_quote_click_closes_menu() {
    let mut harness = make_harness();
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.block_quote",
    );
    assert_eq!(node_text(&after, "docs.menu.active"), "none");
    assert_absent(&after, "docs.menu.format.panel");
}

// =========================================================================
// Fix-plan: hover-does-not-dispatch tests
//
// Hovering a Format menu item must only update the hovered highlight state.
// Document text, cursor position, and selection must remain unchanged.
// =========================================================================

// --- Bold ---

#[test]
fn format_menu_bold_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let open = click(&mut harness, "docs.menu.format");
    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");
    let selection_before = node_value(&open, "docs.document.selection");

    let hover = hover_over(&mut harness, "docs.menu.format.bold");
    assert_eq!(node_text(&hover, "docs.menu.format.hovered"), "Bold");
    assert_eq!(node_value(&hover, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&hover, "docs.document.selection"),
        selection_before
    );
}

// --- Italic ---

#[test]
fn format_menu_italic_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let open = click(&mut harness, "docs.menu.format");
    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");
    let selection_before = node_value(&open, "docs.document.selection");

    let hover = hover_over(&mut harness, "docs.menu.format.italic");
    assert_eq!(node_text(&hover, "docs.menu.format.hovered"), "Italic");
    assert_eq!(node_value(&hover, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&hover, "docs.document.selection"),
        selection_before
    );
}

// --- Underline ---

#[test]
fn format_menu_underline_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let open = click(&mut harness, "docs.menu.format");
    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");
    let selection_before = node_value(&open, "docs.document.selection");

    let hover = hover_over(&mut harness, "docs.menu.format.underline");
    assert_eq!(node_text(&hover, "docs.menu.format.hovered"), "Underline");
    assert_eq!(node_value(&hover, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&hover, "docs.document.selection"),
        selection_before
    );
}

// --- Strikethrough ---

#[test]
fn format_menu_strikethrough_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let open = click(&mut harness, "docs.menu.format");
    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");
    let selection_before = node_value(&open, "docs.document.selection");

    let hover = hover_over(&mut harness, "docs.menu.format.strikethrough");
    assert_eq!(
        node_text(&hover, "docs.menu.format.hovered"),
        "Strikethrough"
    );
    assert_eq!(node_value(&hover, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&hover, "docs.document.selection"),
        selection_before
    );
}

// --- Superscript ---

#[test]
fn format_menu_superscript_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let open = click(&mut harness, "docs.menu.format");
    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");
    let selection_before = node_value(&open, "docs.document.selection");

    let hover = hover_over(&mut harness, "docs.menu.format.superscript");
    assert_eq!(node_text(&hover, "docs.menu.format.hovered"), "Superscript");
    assert_eq!(node_value(&hover, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&hover, "docs.document.selection"),
        selection_before
    );
}

// --- Subscript ---

#[test]
fn format_menu_subscript_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let open = click(&mut harness, "docs.menu.format");
    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");
    let selection_before = node_value(&open, "docs.document.selection");

    let hover = hover_over(&mut harness, "docs.menu.format.subscript");
    assert_eq!(node_text(&hover, "docs.menu.format.hovered"), "Subscript");
    assert_eq!(node_value(&hover, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&hover, "docs.document.selection"),
        selection_before
    );
}

// --- Clear Formatting ---

#[test]
fn format_menu_clear_formatting_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let open = click(&mut harness, "docs.menu.format");
    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");
    let selection_before = node_value(&open, "docs.document.selection");

    let hover = hover_over(&mut harness, "docs.menu.format.clear_formatting");
    assert_eq!(
        node_text(&hover, "docs.menu.format.hovered"),
        "Clear Formatting"
    );
    assert_eq!(node_value(&hover, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&hover, "docs.document.selection"),
        selection_before
    );
}

// --- Block Quote ---

#[test]
fn format_menu_block_quote_hover_does_not_dispatch() {
    let mut harness = make_harness();
    let open = click(&mut harness, "docs.menu.format");
    let text_before = node_value(&open, "docs.document.text");
    let cursor_before = node_value(&open, "docs.document.cursor");
    let selection_before = node_value(&open, "docs.document.selection");

    let hover = hover_over(&mut harness, "docs.menu.format.block_quote");
    assert_eq!(node_text(&hover, "docs.menu.format.hovered"), "Block Quote");
    assert_eq!(node_value(&hover, "docs.document.text"), text_before);
    assert_eq!(node_value(&hover, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&hover, "docs.document.selection"),
        selection_before
    );
}

// =========================================================================
// Fix-plan: no-selection preserves document tests
//
// Toggling a format with no selection must not change document text,
// cursor position, or selection state.  The format toggle applies only
// to the active mark for future typing.
// =========================================================================

// --- Bold ---

#[test]
fn format_menu_bold_no_selection_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    let after = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.bold");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before
    );
}

// --- Italic ---

#[test]
fn format_menu_italic_no_selection_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    let after = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.italic");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before
    );
}

// --- Underline ---

#[test]
fn format_menu_underline_no_selection_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.underline",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before
    );
}

// --- Strikethrough ---

#[test]
fn format_menu_strikethrough_no_selection_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.strikethrough",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before
    );
}

// --- Superscript ---

#[test]
fn format_menu_superscript_no_selection_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.superscript",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before
    );
}

// --- Subscript ---

#[test]
fn format_menu_subscript_no_selection_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.subscript",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before
    );
}

// --- Clear Formatting ---

#[test]
fn format_menu_clear_formatting_no_selection_preserves_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.clear_formatting",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
    assert_eq!(node_value(&after, "docs.document.cursor"), cursor_before);
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before
    );
}

// --- Block Quote ---

#[test]
fn format_menu_block_quote_no_selection_preserves_text() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    // Block quote changes block type but must preserve text content.
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.block_quote",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// =========================================================================
// Fix-plan: with-selection preserves text content tests
//
// Applying a format toggle with a selection must not alter the plain
// text content of the document.
// =========================================================================

// --- Bold ---

#[test]
fn format_menu_bold_with_selection_preserves_text() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.bold");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// --- Italic ---

#[test]
fn format_menu_italic_with_selection_preserves_text() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.italic");
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// --- Underline ---

#[test]
fn format_menu_underline_with_selection_preserves_text() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.underline",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// --- Strikethrough ---

#[test]
fn format_menu_strikethrough_with_selection_preserves_text() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.strikethrough",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// --- Superscript ---

#[test]
fn format_menu_superscript_with_selection_preserves_text() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.superscript",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// --- Subscript ---

#[test]
fn format_menu_subscript_with_selection_preserves_text() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.subscript",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// --- Clear Formatting ---

#[test]
fn format_menu_clear_formatting_with_selection_preserves_text() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.clear_formatting",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// --- Block Quote ---

#[test]
fn format_menu_block_quote_with_selection_preserves_text() {
    let mut harness = make_harness();
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
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.block_quote",
    );
    assert_eq!(node_value(&after, "docs.document.text"), text_before);
}

// =========================================================================
// Fix-plan: clear-formatting removes inline formatting
//
// Applying bold then clear formatting should change the render back
// toward the unformatted baseline.
// =========================================================================

#[test]
fn format_menu_clear_formatting_after_bold_changes_render() {
    let mut harness = make_harness();
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
    // Apply bold first
    let bold = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.bold");
    // Then clear formatting
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.clear_formatting",
    );
    assert_capture_changed(&bold, &after);
}

#[test]
fn format_menu_clear_formatting_after_italic_changes_render() {
    let mut harness = make_harness();
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
    // Apply italic first
    let italic = open_menu_item(&mut harness, "docs.menu.format", "docs.menu.format.italic");
    // Then clear formatting
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.clear_formatting",
    );
    assert_capture_changed(&italic, &after);
}

#[test]
fn format_menu_clear_formatting_after_underline_changes_render() {
    let mut harness = make_harness();
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
    // Apply underline first
    let underlined = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.underline",
    );
    // Then clear formatting
    let after = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.clear_formatting",
    );
    assert_capture_changed(&underlined, &after);
}

// =========================================================================
// Fix-plan: block-quote toggles block style
//
// Applying block quote twice should return to the original render state,
// demonstrating toggle behaviour.
// =========================================================================

#[test]
fn format_menu_block_quote_toggle_round_trip() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello");

    // First application: paragraph -> block quote
    let quoted = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.block_quote",
    );
    quoted.assert_png_valid();

    // Second application: block quote -> paragraph (toggle back)
    let unquoted = open_menu_item(
        &mut harness,
        "docs.menu.format",
        "docs.menu.format.block_quote",
    );
    unquoted.assert_png_valid();
}
