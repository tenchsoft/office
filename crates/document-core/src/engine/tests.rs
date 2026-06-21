use super::*;

fn empty_doc() -> TenchDocument {
    TenchDocument::new("Test")
}

fn doc_with_text(text: &str) -> TenchDocument {
    TenchDocument::plain_text(text)
}

#[test]
fn new_engine_has_clean_state() {
    let engine = DocumentEngine::new(empty_doc());
    assert!(!engine.is_dirty());
    assert_eq!(engine.get_cursor().block_idx, 0);
    assert_eq!(engine.get_cursor().offset, 0);
    assert!(engine.get_selection().is_none());
}

#[test]
fn insert_text_creates_block_and_advances_cursor() {
    let mut engine = DocumentEngine::new(empty_doc());
    let result = engine.insert_text("Hello");

    assert!(result.dirty);
    assert_eq!(result.cursor.offset, 5);
    assert_eq!(result.document.to_plain_text(), "Hello");
}

#[test]
fn backspace_removes_character() {
    let mut engine = DocumentEngine::new(doc_with_text("ABC"));
    engine.cursor.offset = 3;

    let result = engine.backspace();
    assert_eq!(result.document.to_plain_text(), "AB");
    assert_eq!(result.cursor.offset, 2);
}

#[test]
fn backspace_at_block_start_merges_blocks() {
    let mut engine = DocumentEngine::new(doc_with_text("Hello\nWorld"));
    engine.cursor.block_idx = 1;
    engine.cursor.offset = 0;

    let result = engine.backspace();
    assert_eq!(result.document.to_plain_text(), "HelloWorld");
    assert_eq!(result.cursor.block_idx, 0);
    assert_eq!(result.cursor.offset, 5);
}

#[test]
fn delete_forward_removes_next_character() {
    let mut engine = DocumentEngine::new(doc_with_text("ABC"));
    engine.cursor.offset = 1;

    let result = engine.delete_forward();
    assert_eq!(result.document.to_plain_text(), "AC");
    assert_eq!(result.cursor.offset, 1);
}

#[test]
fn delete_forward_at_block_end_merges_next_block() {
    let mut engine = DocumentEngine::new(doc_with_text("Hello\nWorld"));
    engine.cursor.block_idx = 0;
    engine.cursor.offset = 5;

    let result = engine.delete_forward();
    assert_eq!(result.document.to_plain_text(), "HelloWorld");
    assert_eq!(result.cursor.block_idx, 0);
    assert_eq!(result.cursor.offset, 5);
}

#[test]
fn undo_restores_previous_state() {
    let mut engine = DocumentEngine::new(empty_doc());
    engine.insert_text("First");
    engine.insert_text(" Second");

    let result = engine.undo();
    assert_eq!(result.document.to_plain_text(), "First");
}

#[test]
fn redo_restores_undone_state() {
    let mut engine = DocumentEngine::new(empty_doc());
    engine.insert_text("First");
    engine.insert_text(" Second");
    engine.undo();

    let result = engine.redo();
    assert_eq!(result.document.to_plain_text(), "First Second");
}

#[test]
fn toggle_mark_updates_active_marks() {
    let mut engine = DocumentEngine::new(empty_doc());
    engine.toggle_mark(MarkType::Bold);
    assert!(engine.active_marks.bold);

    engine.toggle_mark(MarkType::Bold);
    assert!(!engine.active_marks.bold);
}

#[test]
fn toggle_mark_applies_to_selection() {
    let mut engine = DocumentEngine::new(doc_with_text("Hello"));
    engine.selection = Some(SelectionRange {
        start: CursorState {
            block_idx: 0,
            offset: 0,
        },
        end: CursorState {
            block_idx: 0,
            offset: 5,
        },
    });

    let result = engine.toggle_mark(MarkType::Bold);
    match &result.document.content[0] {
        BlockNode::Paragraph { content, .. } => match &content[0] {
            InlineNode::Text { marks, .. } => assert!(marks.bold),
            other => panic!("expected text node, got {other:?}"),
        },
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn superscript_and_subscript_active_marks_are_exclusive() {
    let mut engine = DocumentEngine::new(empty_doc());
    engine.toggle_mark(MarkType::Superscript);
    assert!(engine.active_marks.superscript);
    assert!(!engine.active_marks.subscript);

    engine.toggle_mark(MarkType::Subscript);
    assert!(!engine.active_marks.superscript);
    assert!(engine.active_marks.subscript);
}

#[test]
fn insert_link_without_selection_inserts_link_at_cursor() {
    let mut engine = DocumentEngine::new(doc_with_text("Open  now"));
    engine.cursor.offset = 5;

    let result = engine.insert_link("https://example.com");

    assert!(result.dirty);
    assert_eq!(result.cursor.offset, 24);
    assert_eq!(
        result.document.to_plain_text(),
        "Open https://example.com now"
    );
    match &result.document.content[0] {
        BlockNode::Paragraph { content, .. } => {
            assert!(matches!(
                &content[1],
                InlineNode::Link { href, text, .. }
                    if href == "https://example.com" && text == "https://example.com"
            ));
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn set_block_type_converts_paragraph_to_heading() {
    let mut engine = DocumentEngine::new(doc_with_text("Title"));
    engine.cursor.offset = 5;

    let result = engine.set_block_type(BlockType::Heading(2));
    match &result.document.content[0] {
        BlockNode::Heading { level, .. } => assert_eq!(*level, 2),
        other => panic!("expected heading, got {other:?}"),
    }
}

#[test]
fn move_cursor_left_and_right() {
    let mut engine = DocumentEngine::new(doc_with_text("Hello"));
    engine.cursor.offset = 5;

    engine.move_cursor(MoveDirection::Left);
    assert_eq!(engine.cursor.offset, 4);

    engine.move_cursor(MoveDirection::Right);
    assert_eq!(engine.cursor.offset, 5);
}

#[test]
fn move_cursor_up_and_down() {
    let mut engine = DocumentEngine::new(doc_with_text("Line1\nLine2\nLine3"));
    engine.cursor.block_idx = 1;
    engine.cursor.offset = 3;

    engine.move_cursor(MoveDirection::Up);
    assert_eq!(engine.cursor.block_idx, 0);

    engine.move_cursor(MoveDirection::Down);
    assert_eq!(engine.cursor.block_idx, 1);
}

#[test]
fn select_all_covers_entire_document() {
    let mut engine = DocumentEngine::new(doc_with_text("A\nB"));
    let result = engine.select_all();

    let sel = result.selection.unwrap();
    assert_eq!(sel.start.block_idx, 0);
    assert_eq!(sel.start.offset, 0);
    assert_eq!(sel.end.block_idx, 1);
    assert_eq!(sel.end.offset, 1);
}

#[test]
fn insert_image_adds_block() {
    let mut engine = DocumentEngine::new(doc_with_text("Before"));
    engine.cursor.offset = 6;

    let result = engine.insert_image(
        ImageSource::Referenced {
            path: "img.png".into(),
        },
        100.0,
        200.0,
    );
    assert_eq!(result.document.content.len(), 2); // "Before" + image
    match &result.document.content[1] {
        BlockNode::Image { source, .. } => match source {
            ImageSource::Referenced { path } => assert_eq!(path, "img.png"),
            _ => panic!("expected referenced image"),
        },
        other => panic!("expected image block, got {other:?}"),
    }
}

#[test]
fn insert_table_adds_correct_dimensions() {
    let mut engine = DocumentEngine::new(empty_doc());
    let result = engine.insert_table(3, 4);

    match &result.document.content[1] {
        BlockNode::Table { rows } => {
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[0].cells.len(), 4);
        }
        other => panic!("expected table, got {other:?}"),
    }
}

#[test]
fn insert_horizontal_rule_and_page_break() {
    let mut engine = DocumentEngine::new(doc_with_text("A"));
    engine.cursor.offset = 1;

    let result = engine.insert_horizontal_rule();
    assert!(matches!(
        &result.document.content[1],
        BlockNode::HorizontalRule
    ));

    let result = engine.insert_page_break();
    assert!(matches!(&result.document.content[2], BlockNode::PageBreak));
}

#[test]
fn delete_selection_removes_text() {
    let mut engine = DocumentEngine::new(doc_with_text("Hello World"));
    engine.cursor.block_idx = 0;
    engine.cursor.offset = 11;

    engine.selection = Some(SelectionRange {
        start: CursorState {
            block_idx: 0,
            offset: 5,
        },
        end: CursorState {
            block_idx: 0,
            offset: 11,
        },
    });

    let result = engine.delete_selection();
    assert_eq!(result.document.to_plain_text(), "Hello");
    assert_eq!(result.cursor.offset, 5);
}

#[test]
fn set_alignment_updates_paragraph() {
    let mut engine = DocumentEngine::new(doc_with_text("Center me"));
    let result = engine.set_alignment(Alignment::Center);
    match &result.document.content[0] {
        BlockNode::Paragraph { attrs, .. } => assert_eq!(attrs.alignment, Alignment::Center),
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn indent_and_outdent_adjust_indent_left() {
    let mut engine = DocumentEngine::new(doc_with_text("Para"));
    let result = engine.indent();
    match &result.document.content[0] {
        BlockNode::Paragraph { attrs, .. } => assert_eq!(attrs.indent_left, 36.0),
        other => panic!("expected paragraph, got {other:?}"),
    }

    let result = engine.outdent();
    match &result.document.content[0] {
        BlockNode::Paragraph { attrs, .. } => assert_eq!(attrs.indent_left, 0.0),
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn mark_saved_clears_dirty_flag() {
    let mut engine = DocumentEngine::new(empty_doc());
    engine.insert_text("Text");
    assert!(engine.is_dirty());

    engine.mark_saved();
    assert!(!engine.is_dirty());
}

#[test]
fn move_cursor_home_end_doc_start_doc_end() {
    let mut engine = DocumentEngine::new(doc_with_text("AB\nCD\nEF"));
    engine.cursor.block_idx = 1;
    engine.cursor.offset = 1;

    engine.move_cursor(MoveDirection::Home);
    assert_eq!(engine.cursor.offset, 0);

    engine.move_cursor(MoveDirection::End);
    assert_eq!(engine.cursor.offset, 2);

    engine.move_cursor(MoveDirection::DocStart);
    assert_eq!(engine.cursor.block_idx, 0);
    assert_eq!(engine.cursor.offset, 0);

    engine.move_cursor(MoveDirection::DocEnd);
    assert_eq!(engine.cursor.block_idx, 2);
    assert_eq!(engine.cursor.offset, 2);
}

#[test]
fn insert_text_with_selection_replaces_selection() {
    let mut engine = DocumentEngine::new(doc_with_text("Hello World"));
    engine.selection = Some(SelectionRange {
        start: CursorState {
            block_idx: 0,
            offset: 5,
        },
        end: CursorState {
            block_idx: 0,
            offset: 11,
        },
    });

    let result = engine.insert_text(" Tench");
    assert_eq!(result.document.to_plain_text(), "Hello Tench");
}

#[test]
fn set_page_setup_updates_paper_size() {
    let mut engine = DocumentEngine::new(empty_doc());
    let result = engine.set_paper_size(PaperSize::Letter);
    assert_eq!(result.document.page_setup.paper_size, PaperSize::Letter);
}

#[test]
fn set_orientation_swaps_page_dimensions() {
    let mut engine = DocumentEngine::new(empty_doc());
    let result = engine.set_orientation(Orientation::Landscape);
    assert_eq!(
        result.document.page_setup.orientation,
        Orientation::Landscape
    );
}

#[test]
fn set_margins_updates_all_sides() {
    let mut engine = DocumentEngine::new(empty_doc());
    let margins = Margins {
        top: 20.0,
        right: 15.0,
        bottom: 20.0,
        left: 15.0,
    };
    let result = engine.set_margins(margins);
    assert_eq!(result.document.page_setup.margins, margins);
}

#[test]
fn set_default_header_and_footer() {
    let mut engine = DocumentEngine::new(empty_doc());
    let result = engine.set_default_header("Page {{page}} of {{pages}}".into());
    assert_eq!(
        result.document.headers_footers.default_header,
        Some("Page {{page}} of {{pages}}".to_string())
    );
    let result = engine.set_default_footer("{{title}} - {{date}}".into());
    assert_eq!(
        result.document.headers_footers.default_footer,
        Some("{{title}} - {{date}}".to_string())
    );
}

#[test]
fn set_indent_left_and_right() {
    let mut engine = DocumentEngine::new(doc_with_text("Para"));
    let result = engine.set_indent_left(72.0);
    match &result.document.content[0] {
        BlockNode::Paragraph { attrs, .. } => assert_eq!(attrs.indent_left, 72.0),
        other => panic!("expected paragraph, got {other:?}"),
    }
    let result = engine.set_indent_right(48.0);
    match &result.document.content[0] {
        BlockNode::Paragraph { attrs, .. } => assert_eq!(attrs.indent_right, 48.0),
        other => panic!("expected paragraph, got {other:?}"),
    }
}

#[test]
fn page_setup_px_dimensions() {
    let setup = PageSetup::default(); // A4 Portrait
    let (w, h) = setup.page_size_px();
    // A4 = 210x297mm, at 96 DPI: 210*96/25.4 ≈ 793.7, 297*96/25.4 ≈ 1122.5
    assert!(w > 790.0 && w < 800.0);
    assert!(h > 1110.0 && h < 1130.0);
}

// -----------------------------------------------------------------------
// Release validation tests
// -----------------------------------------------------------------------

#[test]
fn document_engine_undo_redo_release() {
    let mut engine = DocumentEngine::new(empty_doc());

    // Insert multiple edits and verify undo/redo integrity.
    engine.insert_text("Hello");
    engine.insert_text(" World");
    assert_eq!(engine.get_document().to_plain_text(), "Hello World");

    // Undo twice → back to empty.
    let result = engine.undo();
    assert_eq!(result.document.to_plain_text(), "Hello");
    let result = engine.undo();
    assert!(result.document.to_plain_text().is_empty());

    // Redo twice → back to full text.
    let result = engine.redo();
    assert_eq!(result.document.to_plain_text(), "Hello");
    let result = engine.redo();
    assert_eq!(result.document.to_plain_text(), "Hello World");

    // Verify dirty flag is set after redo.
    assert!(result.dirty);
}
