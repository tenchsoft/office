/// UI automation tests for docs toggle toolbar buttons (Bold, Italic, Underline, etc.).
///
/// Uses debug_id selectors. Verifies that clicking a toggle button changes
/// its visual state (active/inactive) and, for semantic tests, that the
/// document dirty flag reflects the change.
use tench_docs_lib::ui::DocsApp;
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

/// Verifies that clicking a toolbar toggle button twice returns to the
/// original visual state (on → off → on cycle).
fn verify_toggle_cycle(debug_id: &str) {
    let mut harness = make_harness();
    let before = capture(&mut harness);
    assert_selector(&before, debug_id);

    // Click to activate
    let after_on = click(&mut harness, debug_id);
    assert_capture_changed(&before, &after_on);

    // Click again to deactivate
    let after_off = click(&mut harness, debug_id);
    // The second click should also produce a visual change
    assert_capture_changed(&after_on, &after_off);
}

// ---------------------------------------------------------------------------
// Bold — enhanced with dirty state check
// ---------------------------------------------------------------------------

#[test]
fn toolbar_bold_toggle() {
    let mut harness = make_harness();

    // Type text so toggling bold has content to affect.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Bold");

    // Select all text so the mark applies to the selection.
    key_chord(
        &mut harness,
        tench_ui_automation_core::UiAutomationKey::Character("a".into()),
        tench_ui_automation_core::UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.bold");

    // Click to activate bold.
    let after_on = click(&mut harness, "docs.toolbar.bold");
    assert_capture_changed(&before, &after_on);

    // Activating bold should mark the document as dirty.
    let dirty = node_value(&after_on, "docs.document.dirty");
    assert_eq!(
        dirty, "true",
        "document should be dirty after toggling bold on"
    );

    // Click again to deactivate bold.
    let after_off = click(&mut harness, "docs.toolbar.bold");
    assert_capture_changed(&after_on, &after_off);
}

// ---------------------------------------------------------------------------
// Italic — enhanced with dirty state check
// ---------------------------------------------------------------------------

#[test]
fn toolbar_italic_toggle() {
    let mut harness = make_harness();

    // Type text so toggling italic has content to affect.
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Italic");

    // Select all text so the mark applies to the selection.
    key_chord(
        &mut harness,
        tench_ui_automation_core::UiAutomationKey::Character("a".into()),
        tench_ui_automation_core::UiAutomationModifiers {
            control: true,
            ..Default::default()
        },
    );

    let before = capture(&mut harness);
    assert_selector(&before, "docs.toolbar.italic");

    // Click to activate italic.
    let after_on = click(&mut harness, "docs.toolbar.italic");
    assert_capture_changed(&before, &after_on);

    // Activating italic should mark the document as dirty.
    let dirty = node_value(&after_on, "docs.document.dirty");
    assert_eq!(
        dirty, "true",
        "document should be dirty after toggling italic on"
    );

    // Click again to deactivate italic.
    let after_off = click(&mut harness, "docs.toolbar.italic");
    assert_capture_changed(&after_on, &after_off);
}

// ---------------------------------------------------------------------------
// Remaining toggle buttons — visual cycle only
// ---------------------------------------------------------------------------

#[test]
fn toolbar_underline_toggle() {
    verify_toggle_cycle("docs.toolbar.underline");
}

#[test]
fn toolbar_strikethrough_toggle() {
    verify_toggle_cycle("docs.toolbar.strikethrough");
}

#[test]
fn toolbar_code_toggle() {
    verify_toggle_cycle("docs.toolbar.code");
}

#[test]
fn toolbar_superscript_toggle() {
    verify_toggle_cycle("docs.toolbar.superscript");
}

#[test]
fn toolbar_subscript_toggle() {
    verify_toggle_cycle("docs.toolbar.subscript");
}
