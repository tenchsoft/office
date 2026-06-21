use super::*;

#[test]
fn new_state_uses_tdm_as_document_source() {
    let state = DocsState::new();

    assert_eq!(state.title(), "Untitled Document");
    assert_eq!(state.document_text(), "");
    assert!(!state.is_dirty());
}

#[test]
fn apply_edit_result_updates_document_and_counts() {
    let mut state = DocsState::new();

    let mut engine = tench_document_core::DocumentEngine::new(state.current_document().clone());
    let result = engine.insert_text("Hello Tench");

    state.apply_edit_result(result);

    assert_eq!(state.document_text(), "Hello Tench");
    assert_eq!(state.word_count, 2);
    assert!(state.is_dirty());
}

#[test]
fn apply_edit_result_undo_redo_keep_document_in_sync() {
    let mut state = DocsState::new();

    let mut engine = tench_document_core::DocumentEngine::new(state.current_document().clone());
    let result = engine.insert_text("Alpha");
    state.apply_edit_result(result);
    let result = engine.insert_text("\n");
    state.apply_edit_result(result);
    let result = engine.insert_text("Beta");
    state.apply_edit_result(result);
    let result = engine.backspace();
    state.apply_edit_result(result);

    assert_eq!(state.document_text(), "Alpha\nBet");

    let result = engine.undo();
    state.apply_edit_result(result);
    assert_eq!(state.document_text(), "Alpha\nBeta");

    let result = engine.redo();
    state.apply_edit_result(result);
    assert_eq!(state.document_text(), "Alpha\nBet");
}

#[test]
fn applying_saved_artifact_clears_dirty_baseline() {
    let mut state = DocsState::new();

    let mut engine = tench_document_core::DocumentEngine::new(state.current_document().clone());
    let result = engine.insert_text("Saved text");
    state.apply_edit_result(result);

    let mut artifact = state.current_artifact().clone();
    artifact.dirty = false;
    artifact.path = Some("C:/tmp/saved.docx".to_string());

    state.apply_saved_artifact(artifact);

    assert!(!state.is_dirty());
    assert_eq!(state.status_line(), "Saved C:/tmp/saved.docx");

    let result = engine.insert_text(" again");
    state.apply_edit_result(result);
    assert!(state.is_dirty());
}
