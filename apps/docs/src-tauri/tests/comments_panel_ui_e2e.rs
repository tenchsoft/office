/// UI automation tests for docs comments panel.
use tench_docs_lib::ui::DocsApp;
use tench_ui_automation_core::UiAutomationKey;
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::CaptureAssertions;
use tench_ui_test::TestHarness;
fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

#[test]
fn comments_panel_empty_state_when_opened() {
    let mut harness = make_harness();
    let after = open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.comments");
    assert_selector(&after, "docs.comments.collapse");
    assert_selector(&after, "docs.comments.empty");
    assert_node_label_contains(&after, "docs.comments.empty", "No comments yet");
}

#[test]
fn comments_panel_collapse_toggle_label_changes() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.comments");

    // Before collapse: label is "Comments"
    let before = capture(&mut harness);
    assert_node_label(&before, "docs.comments.collapse", "Comments");

    // Click collapse header
    let collapsed = click(&mut harness, "docs.comments.collapse");
    // After collapse: label changes to "Comments (collapsed)"
    assert_node_label(&collapsed, "docs.comments.collapse", "Comments (collapsed)");
    // Empty state should not be visible
    assert_absent(&collapsed, "docs.comments.empty");
    collapsed.assert_png_valid();
}

#[test]
fn comments_panel_uncollapse_shows_empty_state() {
    let mut harness = make_harness();
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.comments");
    // Collapse
    click(&mut harness, "docs.comments.collapse");
    // Uncollapse
    let uncollapsed = click(&mut harness, "docs.comments.collapse");
    assert_selector(&uncollapsed, "docs.comments.empty");
    // Label should return to "Comments"
    assert_node_label(&uncollapsed, "docs.comments.collapse", "Comments");
}

#[test]
fn comments_panel_roundtrip_preserves_state() {
    let mut harness = make_harness();
    // Type text
    click(&mut harness, "docs.document");
    let _typed = type_text(&mut harness, "Body text");

    // Open comments panel
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.comments");
    // Collapse → uncollapse round-trip
    click(&mut harness, "docs.comments.collapse");
    click(&mut harness, "docs.comments.collapse");

    // Document still present
    let after = capture(&mut harness);
    assert_selector(&after, "docs.document");
    after.assert_png_valid();
}

// ── Comment modal tests ──

/// Helper: open the comment modal via right-click context menu.
fn open_comment_modal(harness: &mut TestHarness) {
    right_click(harness, "docs.document");
    click(harness, "docs.context.add_comment");
}

#[test]
fn comment_modal_opens_with_text_field_and_buttons() {
    let mut harness = make_harness();
    open_comment_modal(&mut harness);
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.comment_modal");
    assert_selector(&cap, "docs.comment_modal.text");
    assert_selector(&cap, "docs.comment_modal.submit");
    assert_selector(&cap, "docs.comment_modal.cancel");
}

#[test]
fn comment_modal_text_field_starts_empty() {
    let mut harness = make_harness();
    open_comment_modal(&mut harness);
    let cap = capture(&mut harness);
    assert_eq!(
        get_node_value(&cap, "docs.comment_modal.text"),
        Some("".to_string())
    );
}

#[test]
fn comment_modal_text_field_reflects_typed_text() {
    let mut harness = make_harness();
    open_comment_modal(&mut harness);
    let after = type_text(&mut harness, "Test comment");
    assert_eq!(
        get_node_value(&after, "docs.comment_modal.text"),
        Some("Test comment".to_string())
    );
    after.assert_png_valid();
}

#[test]
fn comment_modal_escape_closes_modal() {
    let mut harness = make_harness();
    open_comment_modal(&mut harness);
    assert_selector(&capture(&mut harness), "docs.comment_modal");

    let after = key(&mut harness, UiAutomationKey::Escape, Default::default());
    assert_absent(&after, "docs.comment_modal");
}

#[test]
fn comment_modal_submit_with_text_adds_comment() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Some text");
    open_comment_modal(&mut harness);
    type_text(&mut harness, "My comment");

    // Submit with Enter
    let after = key(&mut harness, UiAutomationKey::Enter, Default::default());
    // Modal should close
    assert_absent(&after, "docs.comment_modal");
    after.assert_png_valid();
}

#[test]
fn comment_modal_submit_empty_closes_without_adding() {
    let mut harness = make_harness();
    open_comment_modal(&mut harness);
    // Submit empty text
    let after = key(&mut harness, UiAutomationKey::Enter, Default::default());
    assert_absent(&after, "docs.comment_modal");
}

#[test]
fn comment_modal_typing_does_not_edit_document() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Doc");
    let before = capture(&mut harness);
    let doc_before = get_node_label(&before, "docs.document").unwrap_or_default();

    open_comment_modal(&mut harness);
    type_text(&mut harness, "Comment text");

    let after = capture(&mut harness);
    let doc_after = get_node_label(&after, "docs.document").unwrap_or_default();
    assert_eq!(
        doc_before, doc_after,
        "typing in comment modal must not edit document"
    );
}

#[test]
fn comment_row_appears_after_submit() {
    let mut harness = make_harness();
    click(&mut harness, "docs.document");
    type_text(&mut harness, "Body text");

    // Open comments panel first
    open_menu_item(&mut harness, "docs.menu.view", "docs.menu.view.comments");

    // Add a comment via context menu
    right_click(&mut harness, "docs.document");
    click(&mut harness, "docs.context.add_comment");
    type_text(&mut harness, "First comment");
    key(&mut harness, UiAutomationKey::Enter, Default::default());

    // Check that the comment row appears
    let after = capture(&mut harness);
    assert_selector(&after, "docs.comments.row.0");
    assert_absent(&after, "docs.comments.empty");
}
