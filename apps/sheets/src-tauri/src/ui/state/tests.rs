use std::time::Instant;

use tench_document_core::OfficeContent;
use tench_office_io::sheets::format as format_io;

use super::*;

fn state_with_mock() -> SheetsState {
    SheetsState::new_with_grid(mock_grid())
}

#[test]
fn new_state_uses_workbook_content_as_grid_source() {
    let state = state_with_mock();

    assert_eq!(state.current_artifact().title, "Quarterly Plan");
    assert!(matches!(state.current_content(), OfficeContent::Sheets(_)));
    assert!(format_io::workbook_to_csv(state.current_content()).contains("Revenue"));
    assert_eq!(state.active_cell_ref(), "B2");
    assert!(!state.is_dirty());
}

#[test]
fn editing_active_cell_updates_workbook_content_and_status() {
    let mut state = state_with_mock();
    state.select_cell(1, 1);

    assert!(state.delete_from_active_cell());
    assert!(state.push_text_to_active_cell("9"));

    assert_eq!(state.active_cell().unwrap().value, "12509");
    assert!(format_io::workbook_to_csv(state.current_content()).contains("12509"));
    assert!(state.is_dirty());
    assert_eq!(state.status_line(), "Unsaved changes");
    assert!(state.status_count > 0);
}

#[test]
fn applying_saved_artifact_resets_dirty_baseline() {
    let mut state = SheetsState::new();
    state.select_cell(1, 1);
    state.push_text_to_active_cell("0");
    let mut artifact = state.current_artifact().clone();
    artifact.path = Some("C:/tmp/book.tenchsheet".into());
    artifact.dirty = false;

    state.apply_saved_artifact(artifact);

    assert!(!state.is_dirty());
    assert_eq!(state.status_line(), "Saved C:/tmp/book.tenchsheet");
}

// ----- Find/Replace tests -----

#[test]
fn find_locates_matching_cells() {
    let mut state = state_with_mock();
    state.find_replace.find_text = "Revenue".into();
    state.find();
    assert_eq!(state.find_replace.matches.len(), 1);
    assert_eq!(state.find_replace.matches[0].row, 1);
    assert_eq!(state.find_replace.matches[0].col, 0);
}

#[test]
fn find_is_case_insensitive_by_default() {
    let mut state = state_with_mock();
    state.find_replace.find_text = "revenue".into();
    state.find();
    assert_eq!(state.find_replace.matches.len(), 1);
}

#[test]
fn find_respects_case_sensitive_flag() {
    let mut state = state_with_mock();
    state.find_replace.find_text = "revenue".into();
    state.find_replace.case_sensitive = true;
    state.find();
    assert!(state.find_replace.matches.is_empty());
}

#[test]
fn find_next_cycles_through_matches() {
    let mut state = state_with_mock();
    state.find_replace.find_text = "Revenue".into();
    state.find();
    assert_eq!(state.find_replace.current_match, Some(0));
    // Only one match, so find_next wraps back to 0
    state.find_next();
    assert_eq!(state.find_replace.current_match, Some(0));
}

#[test]
fn find_prev_goes_to_last_match() {
    let mut state = state_with_mock();
    state.find_replace.find_text = "Revenue".into();
    state.find();
    state.find_prev();
    // Only one match, so find_prev stays at 0
    assert_eq!(state.find_replace.current_match, Some(0));
}

#[test]
fn replace_next_replaces_current_match() {
    let mut state = state_with_mock();
    state.find_replace.find_text = "Revenue".into();
    state.find_replace.replace_text = "Income".into();
    state.find();
    let replaced = state.replace_next();
    assert!(replaced);
    assert_eq!(state.grid[1][0].value, "Income");
    assert!(state.find_replace.matches.is_empty());
}

#[test]
fn replace_all_replaces_every_match() {
    let mut state = state_with_mock();
    // Search for "Profit" which appears once
    state.find_replace.find_text = "Profit".into();
    state.find_replace.replace_text = "Gain".into();
    let count = state.replace_all();
    assert_eq!(count, 1);
    assert_eq!(state.grid[3][0].value, "Gain");
}

#[test]
fn find_with_regex() {
    let mut state = state_with_mock();
    state.find_replace.find_text = r"Reven.*".into();
    state.find_replace.use_regex = true;
    state.find();
    assert_eq!(state.find_replace.matches.len(), 1);
    assert_eq!(state.find_replace.matches[0].row, 1);
    assert_eq!(state.find_replace.matches[0].col, 0);
}

#[test]
fn search_match_highlighting() {
    let mut state = state_with_mock();
    state.find_replace.find_text = "Revenue".into();
    state.find();
    assert!(state.is_search_match(1, 0));
    assert!(!state.is_search_match(0, 0));
    assert!(state.is_current_match(1, 0));
}

#[test]
fn find_skips_formulas_when_not_searching_formulas() {
    let mut state = state_with_mock();
    state.find_replace.find_text = "B2".into();
    state.find_replace.search_in_formulas = false;
    state.find();
    // B2 appears in formulas only, so should find nothing
    assert!(state.find_replace.matches.is_empty());
}

#[test]
fn find_in_formulas() {
    let mut state = state_with_mock();
    state.find_replace.find_text = "B2".into();
    state.find_replace.search_in_formulas = true;
    state.find();
    assert!(!state.find_replace.matches.is_empty());
}

#[test]
fn search_scope_variants_exist() {
    // Ensure all scope variants compile and are usable
    let scope = SearchScope::CurrentSheet;
    assert_eq!(scope, SearchScope::CurrentSheet);
    let scope = SearchScope::EntireWorkbook;
    assert_eq!(scope, SearchScope::EntireWorkbook);
    let scope = SearchScope::Selection;
    assert_eq!(scope, SearchScope::Selection);
}

// ----- Clipboard tests -----

#[test]
fn copy_and_paste() {
    let mut state = state_with_mock();
    state.select_cell(1, 0); // "Revenue"
    assert!(state.edit_copy());
    state.select_cell(0, 0);
    assert!(state.edit_paste());
    assert_eq!(state.grid[0][0].value, "Revenue");
}

#[test]
fn cut_and_paste() {
    let mut state = state_with_mock();
    state.select_cell(1, 0); // "Revenue"
    assert!(state.edit_cut());
    assert!(state.grid[1][0].value.is_empty());
    state.select_cell(0, 0);
    assert!(state.edit_paste());
    assert_eq!(state.grid[0][0].value, "Revenue");
}

#[test]
fn paste_special_values_only() {
    let mut state = state_with_mock();
    state.select_cell(3, 1); // =B2-B3 (formula)
    assert!(state.edit_copy());
    state.select_cell(0, 0);
    assert!(state.edit_paste_special(PasteSpecialMode::ValuesOnly));
    // Source is a formula, so values-only should skip it
    assert_eq!(state.grid[0][0].value, "Item"); // unchanged
}

#[test]
fn paste_special_formulas_only() {
    let mut state = state_with_mock();
    state.select_cell(3, 1); // =B2-B3 (formula)
    assert!(state.edit_copy());
    state.select_cell(0, 0);
    assert!(state.edit_paste_special(PasteSpecialMode::FormulasOnly));
    assert_eq!(state.grid[0][0].value, "=B2-B3");
    assert!(state.grid[0][0].is_formula);
}

#[test]
fn paste_special_all() {
    let mut state = state_with_mock();
    state.select_cell(1, 1); // "12500"
    assert!(state.edit_copy());
    state.select_cell(0, 0);
    assert!(state.edit_paste_special(PasteSpecialMode::All));
    assert_eq!(state.grid[0][0].value, "12500");
}

// ----- Undo/Redo tests -----

#[test]
fn undo_restores_previous_state() {
    let mut state = state_with_mock();
    state.push_text_to_active_cell_with_undo("X");
    assert_eq!(state.grid[1][1].value, "12500X");
    assert!(state.undo());
    assert_eq!(state.grid[1][1].value, "12500");
}

#[test]
fn redo_reapplies_undone_state() {
    let mut state = state_with_mock();
    state.push_text_to_active_cell_with_undo("X");
    state.undo();
    assert!(state.redo());
    assert_eq!(state.grid[1][1].value, "12500X");
}

#[test]
fn undo_and_redo_counts() {
    let mut state = state_with_mock();
    assert_eq!(state.undo_count(), 0);
    assert_eq!(state.redo_count(), 0);
    state.push_text_to_active_cell_with_undo("X");
    assert_eq!(state.undo_count(), 1);
    assert_eq!(state.redo_count(), 0);
    state.undo();
    assert_eq!(state.undo_count(), 0);
    assert_eq!(state.redo_count(), 1);
    state.redo();
    assert_eq!(state.undo_count(), 1);
    assert_eq!(state.redo_count(), 0);
}

#[test]
fn undo_returns_false_when_empty() {
    let mut state = SheetsState::new();
    assert!(!state.undo());
    assert!(!state.redo());
}

// ----- Named Ranges tests -----

#[test]
fn define_and_resolve_named_range() {
    let mut state = SheetsState::new();
    let range = CellRange::single(0, 0);
    state.define_name("Header".into(), None, range);
    let resolved = state.resolve_name("Header").unwrap();
    assert_eq!(resolved.start_row, 0);
    assert_eq!(resolved.start_col, 0);
}

#[test]
fn resolve_name_is_case_insensitive() {
    let mut state = SheetsState::new();
    let range = CellRange::single(0, 0);
    state.define_name("Header".into(), None, range);
    assert!(state.resolve_name("header").is_some());
    assert!(state.resolve_name("HEADER").is_some());
}

#[test]
fn define_name_updates_existing() {
    let mut state = SheetsState::new();
    let range1 = CellRange::single(0, 0);
    state.define_name("Data".into(), None, range1);
    let range2 = CellRange::single(1, 1);
    state.define_name("Data".into(), Some(0), range2.clone());
    assert_eq!(state.named_ranges.len(), 1);
    let resolved = state.resolve_name("Data").unwrap();
    assert_eq!(*resolved, range2);
}

#[test]
fn delete_named_range() {
    let mut state = SheetsState::new();
    let range = CellRange::single(0, 0);
    state.define_name("ToRemove".into(), None, range);
    assert_eq!(state.named_ranges.len(), 1);
    assert!(state.delete_named_range("ToRemove"));
    assert!(state.named_ranges.is_empty());
    assert!(!state.delete_named_range("NonExistent"));
}

#[test]
fn cell_range_single() {
    let range = CellRange::single(5, 3);
    assert_eq!(range.start_row, 5);
    assert_eq!(range.start_col, 3);
    assert_eq!(range.end_row, 5);
    assert_eq!(range.end_col, 3);
    assert_eq!(range.to_address(), "D6");
}

#[test]
fn cell_range_to_address_multi() {
    let range = CellRange {
        start_row: 0,
        start_col: 0,
        end_row: 2,
        end_col: 3,
    };
    assert_eq!(range.to_address(), "A1:D3");
}

// ----- Auto-save tests -----

#[test]
fn auto_save_should_not_save_when_clean() {
    let state = SheetsState::new();
    assert!(!state.auto_save.should_save(false));
}

#[test]
fn auto_save_should_save_when_dirty_and_interval_passed() {
    let mut autosave = AutoSaveState::new(0); // 0 sec interval = always due
    autosave.last_save_time = Instant::now() - std::time::Duration::from_secs(1);
    assert!(autosave.should_save(true));
}

// ----- 10.1 Page Setup tests -----

#[test]
fn default_page_setup_is_a4_portrait() {
    let state = SheetsState::new();
    let setup = state.get_page_setup();
    assert_eq!(setup.paper_size, PaperSize::A4);
    assert_eq!(setup.orientation, Orientation::Portrait);
    assert_eq!(setup.scaling, Scaling::Percentage(100.0));
    assert!(setup.print_area.is_none());
    assert!(!setup.gridlines_print);
    assert!(!setup.center_horizontally);
}

#[test]
fn set_and_get_page_setup() {
    let mut state = SheetsState::new();
    let new_setup = PageSetup {
        paper_size: PaperSize::Letter,
        orientation: Orientation::Landscape,
        gridlines_print: true,
        ..PageSetup::default()
    };
    state.set_page_setup(new_setup.clone());
    let setup = state.get_page_setup();
    assert_eq!(setup.paper_size, PaperSize::Letter);
    assert_eq!(setup.orientation, Orientation::Landscape);
    assert!(setup.gridlines_print);
}

#[test]
fn set_and_clear_print_area() {
    let mut state = SheetsState::new();
    let range = CellRange {
        start_row: 0,
        start_col: 0,
        end_row: 3,
        end_col: 3,
    };
    state.set_print_area(range);
    assert!(state.page_setup.print_area.is_some());
    let area = state.page_setup.print_area.as_ref().unwrap();
    assert_eq!(area.start_row, 0);
    assert_eq!(area.end_col, 3);

    state.clear_print_area();
    assert!(state.page_setup.print_area.is_none());
}

#[test]
fn paper_size_dimensions() {
    assert_eq!(PaperSize::A4.dimensions_mm(), (210.0, 297.0));
    assert_eq!(PaperSize::Letter.dimensions_mm(), (215.9, 279.4));
    assert_eq!(
        PaperSize::Custom(100.0, 200.0).dimensions_mm(),
        (100.0, 200.0)
    );
}

#[test]
fn paper_size_labels() {
    assert_eq!(PaperSize::A4.label(), "A4");
    assert_eq!(PaperSize::Letter.label(), "Letter");
    assert_eq!(PaperSize::Custom(1.0, 2.0).label(), "Custom");
}

#[test]
fn orientation_labels() {
    assert_eq!(Orientation::Portrait.label(), "Portrait");
    assert_eq!(Orientation::Landscape.label(), "Landscape");
}

#[test]
fn default_margins() {
    let m = Margins::default();
    assert!((m.top - 19.05).abs() < f64::EPSILON);
    assert!((m.left - 17.78).abs() < f64::EPSILON);
}

// ----- 10.2 Print Preview tests -----

#[test]
fn compute_print_pages_produces_pages() {
    let mut state = state_with_mock();
    state.compute_print_pages();
    // Default grid is 8 rows x 5 cols; A4 portrait should produce at least 1 page
    assert!(!state.print_preview.pages.is_empty());
    let first = &state.print_preview.pages[0];
    assert_eq!(first.page_number, 1);
}

#[test]
fn print_preview_navigation() {
    let mut pvs = PrintPreviewState {
        visible: true,
        pages: vec![
            PrintPage {
                rows: (0, 5),
                cols: (0, 2),
                page_number: 1,
            },
            PrintPage {
                rows: (0, 5),
                cols: (3, 4),
                page_number: 2,
            },
            PrintPage {
                rows: (6, 7),
                cols: (0, 2),
                page_number: 3,
            },
        ],
        current_page: 0,
        zoom: 1.0,
    };
    assert!(pvs.next_page());
    assert_eq!(pvs.current_page, 1);
    assert!(pvs.next_page());
    assert_eq!(pvs.current_page, 2);
    assert!(!pvs.next_page()); // at end
    assert!(pvs.prev_page());
    assert_eq!(pvs.current_page, 1);
}

#[test]
fn print_preview_zoom() {
    let mut pvs = PrintPreviewState::default();
    assert!(pvs.zoom_in());
    assert!((pvs.zoom - 1.25).abs() < f64::EPSILON);
    assert!(pvs.zoom_out());
    assert!((pvs.zoom - 1.0).abs() < f64::EPSILON);
}

#[test]
fn print_area_limits_pages() {
    let mut state = SheetsState::new();
    // Set a small print area
    state.set_print_area(CellRange {
        start_row: 0,
        start_col: 0,
        end_row: 1,
        end_col: 1,
    });
    state.compute_print_pages();
    // With a 2x2 area, there should be exactly 1 page
    assert_eq!(state.print_preview.pages.len(), 1);
    let page = &state.print_preview.pages[0];
    assert_eq!(page.rows, (0, 1));
    assert_eq!(page.cols, (0, 1));
}
