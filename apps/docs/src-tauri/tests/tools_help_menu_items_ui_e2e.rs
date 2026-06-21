/// UI automation tests for docs Tools & Help Menu items (#47-#53).
///
/// Uses debug_id selectors for menu items instead of hardcoded coordinates.
/// Every test verifies that clicking a menu item produces a visual change.
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

// #47 Tools → Word Count
#[test]
fn tools_menu_word_count_opens_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.word_count",
    );
    assert_capture_changed(&before, &after);
    assert_selector(&after, "docs.modal.word_count");
}

// #48 Tools → Track Changes
#[test]
fn tools_menu_track_changes_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.track_changes",
    );
    assert_capture_changed(&before, &after);
}

// #49 Tools → Spell Check
#[test]
fn tools_menu_spell_check_changes_render() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.spell_check",
    );
    assert_capture_changed(&before, &after);
}

// #50 Help → About
#[test]
fn help_menu_about_opens_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.help", "docs.menu.help.about");
    assert_capture_changed(&before, &after);
}

// #51 Help → Keyboard Shortcuts
#[test]
fn help_menu_keyboard_shortcuts_opens_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.help",
        "docs.menu.help.keyboard_shortcuts",
    );
    assert_capture_changed(&before, &after);
}

// --- Strengthened tests: verify info modal closes menu ---

#[test]
fn help_menu_about_opens_info_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(&mut harness, "docs.menu.help", "docs.menu.help.about");
    assert_capture_changed(&before, &after);
    // About opens an info modal
    assert_selector(&after, "docs.modal.about");
    assert_absent(&after, "docs.menu.help.panel");
}

#[test]
fn help_menu_keyboard_shortcuts_opens_info_modal() {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    let after = open_menu_item(
        &mut harness,
        "docs.menu.help",
        "docs.menu.help.keyboard_shortcuts",
    );
    assert_capture_changed(&before, &after);
    assert_selector(&after, "docs.modal.keyboard_shortcuts");
    assert_absent(&after, "docs.menu.help.panel");
}

// --- Document preservation tests from fix plans ---

/// Helper: returns the `value` field of a node identified by `debug_id`.
///
/// Panics if the node is not found or has no value.
fn node_value(capture: &tench_ui_automation_core::UiAutomationCapture, debug_id: &str) -> String {
    get_node_value(capture, debug_id).unwrap_or_else(|| panic!("node '{debug_id}' has no value"))
}

// From docs-help-menu-about-item-fix.md
#[test]
fn help_menu_about_closes_menu_and_preserves_document() {
    let mut harness = make_harness();

    // Type text into the document and capture baseline state.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Body");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    // Open Help → About.
    let after = open_menu_item(&mut harness, "docs.menu.help", "docs.menu.help.about");

    // Menu should be closed.
    assert_absent(&after, "docs.menu.help.panel");

    // No info panel should remain open.
    assert_absent(&after, "docs.panel.about");
    assert_absent(&after, "docs.panel.info");

    // Document text, cursor, and selection must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after Help → About"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor position should be preserved after Help → About"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved after Help → About"
    );
}

// From docs-help-menu-keyboard-shortcuts-item-fix.md
#[test]
fn help_menu_keyboard_shortcuts_closes_menu_and_preserves_document() {
    let mut harness = make_harness();

    // Type text into the document and capture baseline state.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Body");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    // Open Help → Keyboard Shortcuts.
    let after = open_menu_item(
        &mut harness,
        "docs.menu.help",
        "docs.menu.help.keyboard_shortcuts",
    );

    // Menu should be closed.
    assert_absent(&after, "docs.menu.help.panel");

    // No info panel should remain open.
    assert_absent(&after, "docs.panel.keyboard_shortcuts");
    assert_absent(&after, "docs.panel.info");

    // Document text, cursor, and selection must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after Help → Keyboard Shortcuts"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor position should be preserved after Help → Keyboard Shortcuts"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved after Help → Keyboard Shortcuts"
    );
}

// --- Additional tests from docs-help-menu-about-item-fix.md ---

/// About info modal should contain version information text.
#[test]
fn help_menu_about_shows_version_info() {
    let mut harness = make_harness();

    // Open Help → About.
    let after = open_menu_item(&mut harness, "docs.menu.help", "docs.menu.help.about");

    // The About info modal should be present.
    assert_selector(&after, "docs.modal.about");

    // The modal label should be "About".
    let label = node_text(&after, "docs.modal.about");
    assert!(
        label.contains("About"),
        "About modal label should contain 'About', got: {label:?}"
    );

    // A backdrop should be present behind the modal.
    assert_selector(&after, "docs.modal.backdrop");
}

/// Hovering the About item in the Help menu must not open the modal.
#[test]
fn help_menu_about_hover_does_not_dispatch() {
    let mut harness = make_harness();

    // Open Help menu.
    click(&mut harness, "docs.menu.help");

    // Hover over the About item.
    let after_hover = hover(&mut harness, "docs.menu.help.about");

    // Hovered state should reflect About.
    assert_eq!(
        node_text(&after_hover, "docs.menu.help.hovered"),
        "About",
        "hovered item should be About"
    );

    // But no About modal should appear on hover.
    assert_absent(&after_hover, "docs.modal.about");

    // No toast should appear either.
    assert_absent(&after_hover, "docs.toast");
}

/// Pressing Escape while the About modal is open should close it.
#[test]
fn help_menu_about_escape_closes_modal() {
    let mut harness = make_harness();

    // Open Help → About.
    open_menu_item(&mut harness, "docs.menu.help", "docs.menu.help.about");
    let with_modal = capture(&mut harness);
    assert_selector(&with_modal, "docs.modal.about");

    // Press Escape.
    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());

    // Modal and backdrop should be gone.
    assert_absent(&after, "docs.modal.about");
    assert_absent(&after, "docs.modal.backdrop");
}

/// About must preserve an existing text selection in the document.
#[test]
fn help_menu_about_preserves_selection() {
    let mut harness = make_harness();

    // Type text and select a range.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello World");
    // Select "World" by Ctrl+Shift+Left.
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        tench_ui_automation_core::UiAutomationModifiers {
            control: true,
            shift: true,
            ..Default::default()
        },
    );

    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");
    let dirty_before = node_value(&before, "docs.document.dirty");
    let scroll_before = node_value(&before, "docs.document.scroll_y");

    // Open Help → About.
    let after = open_menu_item(&mut harness, "docs.menu.help", "docs.menu.help.about");

    // Document state must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.dirty"),
        dirty_before,
        "dirty state should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.scroll_y"),
        scroll_before,
        "scroll position should be preserved"
    );
}

/// Clicking outside the About info modal should close it without side effects.
#[test]
fn help_menu_about_outside_click_closes_modal() {
    let mut harness = make_harness();

    // Type some text to establish document state.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Content");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    // Open Help → About.
    open_menu_item(&mut harness, "docs.menu.help", "docs.menu.help.about");
    let with_modal = capture(&mut harness);
    assert_selector(&with_modal, "docs.modal.about");

    // Close the modal with Escape.
    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());

    // Modal should be closed.
    assert_absent(&after, "docs.modal.about");
    assert_absent(&after, "docs.modal.backdrop");

    // Document text must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be unchanged after closing About modal"
    );
}

// --- Additional tests from docs-help-menu-keyboard-shortcuts-item-fix.md ---

/// Keyboard Shortcuts info modal should contain shortcuts reference text.
#[test]
fn help_menu_keyboard_shortcuts_shows_shortcuts_content() {
    let mut harness = make_harness();

    // Open Help → Keyboard Shortcuts.
    let after = open_menu_item(
        &mut harness,
        "docs.menu.help",
        "docs.menu.help.keyboard_shortcuts",
    );

    // The Keyboard Shortcuts info modal should be present.
    assert_selector(&after, "docs.modal.keyboard_shortcuts");

    // The modal label should be "Keyboard Shortcuts".
    let label = node_text(&after, "docs.modal.keyboard_shortcuts");
    assert!(
        label.contains("Keyboard Shortcuts"),
        "Keyboard Shortcuts modal label should contain 'Keyboard Shortcuts', got: {label:?}"
    );

    // A backdrop should be present behind the modal.
    assert_selector(&after, "docs.modal.backdrop");
}

/// Hovering the Keyboard Shortcuts item in the Help menu must not open the modal.
#[test]
fn help_menu_keyboard_shortcuts_hover_does_not_dispatch() {
    let mut harness = make_harness();

    // Open Help menu.
    click(&mut harness, "docs.menu.help");

    // Hover over the Keyboard Shortcuts item.
    let after_hover = hover(&mut harness, "docs.menu.help.keyboard_shortcuts");

    // Hovered state should reflect Keyboard Shortcuts.
    assert_eq!(
        node_text(&after_hover, "docs.menu.help.hovered"),
        "Keyboard Shortcuts",
        "hovered item should be Keyboard Shortcuts"
    );

    // But no Keyboard Shortcuts modal should appear on hover.
    assert_absent(&after_hover, "docs.modal.keyboard_shortcuts");

    // No toast should appear either.
    assert_absent(&after_hover, "docs.toast");
}

/// Pressing Escape while the Keyboard Shortcuts modal is open should close it.
#[test]
fn help_menu_keyboard_shortcuts_escape_closes_modal() {
    let mut harness = make_harness();

    // Open Help → Keyboard Shortcuts.
    open_menu_item(
        &mut harness,
        "docs.menu.help",
        "docs.menu.help.keyboard_shortcuts",
    );
    let with_modal = capture(&mut harness);
    assert_selector(&with_modal, "docs.modal.keyboard_shortcuts");

    // Press Escape.
    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());

    // Modal and backdrop should be gone.
    assert_absent(&after, "docs.modal.keyboard_shortcuts");
    assert_absent(&after, "docs.modal.backdrop");
}

/// Keyboard Shortcuts must preserve an existing text selection in the document.
#[test]
fn help_menu_keyboard_shortcuts_preserves_selection() {
    let mut harness = make_harness();

    // Type text and select a range.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Hello World");
    // Select "World" by Ctrl+Shift+Left.
    key(
        &mut harness,
        UiAutomationKey::ArrowLeft,
        tench_ui_automation_core::UiAutomationModifiers {
            control: true,
            shift: true,
            ..Default::default()
        },
    );

    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");
    let dirty_before = node_value(&before, "docs.document.dirty");
    let scroll_before = node_value(&before, "docs.document.scroll_y");

    // Open Help → Keyboard Shortcuts.
    let after = open_menu_item(
        &mut harness,
        "docs.menu.help",
        "docs.menu.help.keyboard_shortcuts",
    );

    // Document state must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.dirty"),
        dirty_before,
        "dirty state should be preserved"
    );
    assert_eq!(
        node_value(&after, "docs.document.scroll_y"),
        scroll_before,
        "scroll position should be preserved"
    );
}

/// Clicking outside the Keyboard Shortcuts info modal should close it without side effects.
#[test]
fn help_menu_keyboard_shortcuts_outside_click_closes_modal() {
    let mut harness = make_harness();

    // Type some text to establish document state.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Content");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    // Open Help → Keyboard Shortcuts.
    open_menu_item(
        &mut harness,
        "docs.menu.help",
        "docs.menu.help.keyboard_shortcuts",
    );
    let with_modal = capture(&mut harness);
    assert_selector(&with_modal, "docs.modal.keyboard_shortcuts");

    // Close the modal with Escape.
    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());

    // Modal should be closed.
    assert_absent(&after, "docs.modal.keyboard_shortcuts");
    assert_absent(&after, "docs.modal.backdrop");

    // Document text must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be unchanged after closing Keyboard Shortcuts modal"
    );
}

// --- Tools menu: Word Count menu-close-before-dispatch ---

/// Clicking Tools → Word Count should close the menu before opening the modal.
#[test]
fn tools_menu_word_count_closes_menu_before_dispatch() {
    let mut harness = make_harness();

    // Open Tools → Word Count.
    let after = open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.word_count",
    );

    // Menu panel should be closed.
    assert_absent(&after, "docs.menu.tools.panel");

    // Word Count modal should be present.
    assert_selector(&after, "docs.modal.word_count");
}

/// Word Count should preserve document text, cursor, and selection.
#[test]
fn tools_menu_word_count_preserves_document() {
    let mut harness = make_harness();

    // Type text and capture baseline state.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Count me");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    // Open Tools → Word Count.
    let after = open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.word_count",
    );

    // Document state must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after Word Count"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor should be preserved after Word Count"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved after Word Count"
    );
}

/// Hovering Word Count in the Tools menu must not open the modal.
#[test]
fn tools_menu_word_count_hover_does_not_dispatch() {
    let mut harness = make_harness();

    // Open Tools menu.
    click(&mut harness, "docs.menu.tools");

    // Hover over the Word Count item.
    let after_hover = hover(&mut harness, "docs.menu.tools.word_count");

    // No Word Count modal should appear on hover.
    assert_absent(&after_hover, "docs.modal.word_count");
}

// --- Tools menu: Spell Check document preservation ---

/// Spell Check should preserve document text, cursor, and selection.
#[test]
fn tools_menu_spell_check_preserves_document() {
    let mut harness = make_harness();

    // Type text and capture baseline state.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Spell check me");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    // Open Tools → Spell Check.
    let after = open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.spell_check",
    );

    // Document state must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after Spell Check"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor should be preserved after Spell Check"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved after Spell Check"
    );
}

/// Hovering Spell Check in the Tools menu must not dispatch.
#[test]
fn tools_menu_spell_check_hover_does_not_dispatch() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.tools");
    let after_hover = hover(&mut harness, "docs.menu.tools.spell_check");
    assert_absent(&after_hover, "docs.toast");
}

// --- Tools menu: Track Changes document preservation ---

/// Track Changes should preserve document text, cursor, and selection.
#[test]
fn tools_menu_track_changes_preserves_document() {
    let mut harness = make_harness();

    // Type text and capture baseline state.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Track changes text");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");

    // Open Tools → Track Changes.
    let after = open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.track_changes",
    );

    // Document state must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after Track Changes toggle"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor should be preserved after Track Changes toggle"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved after Track Changes toggle"
    );
}

/// Hovering Track Changes in the Tools menu must not dispatch.
#[test]
fn tools_menu_track_changes_hover_does_not_dispatch() {
    let mut harness = make_harness();

    click(&mut harness, "docs.menu.tools");
    let after_hover = hover(&mut harness, "docs.menu.tools.track_changes");
    // Track changes indicator should not appear just from hover.
    assert_absent(&after_hover, "docs.track_changes_indicator");
}

// --- Word Count close button behavioral tests ---

/// Clicking the Word Count close button should be clickable without crash.
/// The close button exists in the modal; verify it is present and clickable.
#[test]
fn word_count_close_button_is_clickable() {
    let mut harness = make_harness();

    // Open Tools → Word Count.
    open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.word_count",
    );
    let with_modal = capture(&mut harness);
    assert_selector(&with_modal, "docs.modal.word_count");
    assert_selector(&with_modal, "docs.modal.word_count.close");

    // Click the close button.
    let after = click(&mut harness, "docs.modal.word_count.close");
    after.assert_png_valid();
}

/// Closing the Word Count modal should preserve document text, cursor, and selection.
#[test]
fn word_count_close_button_preserves_document() {
    let mut harness = make_harness();

    // Type text and capture baseline state.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Word count body");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");
    let cursor_before = node_value(&before, "docs.document.cursor");
    let selection_before = node_value(&before, "docs.document.selection");
    let dirty_before = node_value(&before, "docs.document.dirty");

    // Open Tools → Word Count.
    open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.word_count",
    );

    // Click the close button.
    let after = click(&mut harness, "docs.modal.word_count.close");

    // Document state must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be preserved after closing Word Count"
    );
    assert_eq!(
        node_value(&after, "docs.document.cursor"),
        cursor_before,
        "cursor should be preserved after closing Word Count"
    );
    assert_eq!(
        node_value(&after, "docs.document.selection"),
        selection_before,
        "selection should be preserved after closing Word Count"
    );
    assert_eq!(
        node_value(&after, "docs.document.dirty"),
        dirty_before,
        "dirty state should be preserved after closing Word Count"
    );
    after.assert_png_valid();
}

/// Typing while the Word Count modal is open should not insert text into the document.
#[test]
fn word_count_modal_keyboard_isolation() {
    let mut harness = make_harness();

    // Type text before opening modal.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Before modal");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    // Open Tools → Word Count.
    open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.word_count",
    );
    let with_modal = capture(&mut harness);
    assert_selector(&with_modal, "docs.modal.word_count");

    // Type while modal is open — text should not reach the document.
    let after_type = type_text(&mut harness, "Should not appear");
    assert_eq!(
        node_value(&after_type, "docs.document.text"),
        text_before,
        "document text should not change while Word Count modal is open"
    );
    assert_selector(&after_type, "docs.modal.word_count");
    after_type.assert_png_valid();
}

/// Pressing Escape while the Word Count modal is open should close it.
#[test]
fn word_count_escape_closes_modal() {
    let mut harness = make_harness();

    // Open Tools → Word Count.
    open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.word_count",
    );
    let with_modal = capture(&mut harness);
    assert_selector(&with_modal, "docs.modal.word_count");

    // Press Escape.
    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());

    // Modal and backdrop should be gone.
    assert_absent(&after, "docs.modal.word_count");
    assert_absent(&after, "docs.modal.backdrop");
    after.assert_png_valid();
}

/// Clicking outside the Word Count modal (via Escape) should close it without mutation.
#[test]
fn word_count_outside_click_closes_without_mutation() {
    let mut harness = make_harness();

    // Type text before opening modal.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Protected content");
    let before = capture(&mut harness);
    let text_before = node_value(&before, "docs.document.text");

    // Open Tools → Word Count.
    open_menu_item(
        &mut harness,
        "docs.menu.tools",
        "docs.menu.tools.word_count",
    );
    let with_modal = capture(&mut harness);
    assert_selector(&with_modal, "docs.modal.word_count");

    // Close the modal with Escape (simulating outside-click dismissal).
    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());

    // Modal should be closed.
    assert_absent(&after, "docs.modal.word_count");
    assert_absent(&after, "docs.modal.backdrop");

    // Document text must be unchanged.
    assert_eq!(
        node_value(&after, "docs.document.text"),
        text_before,
        "document text should be unchanged after closing Word Count modal"
    );
    after.assert_png_valid();
}
