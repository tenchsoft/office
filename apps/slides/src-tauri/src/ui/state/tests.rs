use super::*;
use tench_office_io::slides::format as format_io;

#[test]
fn new_state_starts_empty() {
    let state = SlidesState::new();

    assert_eq!(state.current_artifact().title, "Untitled Presentation");
    assert!(matches!(state.current_content(), OfficeContent::Slides(_)));
    assert!(!state.is_dirty());
    // Phase 0.3: no mock data
    assert_eq!(state.slides.len(), 1);
    assert!(state.slides[0].elements.is_empty() || state.slides[0].elements.len() <= 1);
}

#[test]
fn adding_text_element_updates_content_and_dirty_status() {
    let mut state = SlidesState::new();

    assert!(state.insert_text_element("Native text box"));

    assert!(
        format_io::presentation_to_plain_text(state.current_content()).contains("Native text box")
    );
    assert!(state.is_dirty());
    assert_eq!(state.status_line(), "Unsaved changes");
}

#[test]
fn applying_saved_artifact_resets_dirty_baseline() {
    let mut state = SlidesState::new();
    state.add_slide();
    let mut artifact = state.current_artifact().clone();
    artifact.path = Some("C:/tmp/deck.pptx".into());
    artifact.dirty = false;

    state.apply_saved_artifact(artifact);

    assert!(!state.is_dirty());
    assert_eq!(state.status_line(), "Saved C:/tmp/deck.pptx");
}

#[test]
fn undo_redo_round_trips() {
    let mut state = SlidesState::new();
    assert!(!state.undo());
    state.insert_text_element("First");
    assert!(state.is_dirty());
    assert!(state.undo());
    assert!(!state.is_dirty());
    assert!(state.redo());
    assert!(state.is_dirty());
}

#[test]
fn z_order_operations() {
    let mut state = SlidesState::new();
    state.insert_text_element("A");
    state.insert_text_element("B");
    // Two elements: A at 0, B at 1. Selected is B (index 1).
    assert_eq!(state.selected_element, Some(1));
    state.bring_forward(); // B is already at end, no-op
    state.send_backward(); // B goes to 0, A to 1
    assert_eq!(state.selected_element, Some(0));
}

#[test]
fn slide_crud() {
    let mut state = SlidesState::new();
    state.add_slide();
    assert_eq!(state.slides.len(), 2);
    state.duplicate_slide(0);
    assert_eq!(state.slides.len(), 3);
    state.delete_slide(1);
    assert_eq!(state.slides.len(), 2);
}

#[test]
fn find_replace_works() {
    let mut state = SlidesState::new();
    state.insert_text_element("Hello World");
    state.find_text("World");
    assert_eq!(state.find_replace.matches.len(), 1);
    state.find_replace.replace_text = "Tench".into();
    state.replace_current();
    let slide = state.current_slide().unwrap();
    let elem = slide.elements.last().unwrap();
    assert_eq!(elem.text.as_deref(), Some("Hello Tench"));
}

#[test]
fn clipboard_operations() {
    let mut state = SlidesState::new();
    state.insert_text_element("Copy me");
    state.copy_selected();
    assert!(state.clipboard.is_some());
    state.paste();
    let slide = state.current_slide().unwrap();
    assert_eq!(slide.elements.len(), 2); // original + pasted
}

#[test]
fn content_to_slides_round_trip() {
    let mut state = SlidesState::new();
    state.insert_text_element("Test roundtrip");
    let content = state.current_content().clone();
    let slides = content_to_slides(&content);
    assert_eq!(slides.len(), state.slides.len());
    assert!(slides[0]
        .elements
        .iter()
        .any(|e| e.text.as_deref() == Some("Test roundtrip")));
}
