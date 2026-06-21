use tench_sheets_lib::ui::SheetsApp;
use tench_ui_automation_core::{
    find_node, UiAutomationAction, UiAutomationCapture, UiAutomationCaptureRequest,
    UiAutomationKey, UiAutomationModifiers, UiAutomationNode, UiAutomationRect,
    UiAutomationSelector,
};
use tench_ui_test::{harness::HarnessConfig, snapshot::is_nonblank, TestHarness};

fn harness() -> TestHarness {
    TestHarness::with_config(
        SheetsApp::new(),
        HarnessConfig::with_viewport(1280.0, 720.0),
    )
}

fn decode_png(capture: &UiAutomationCapture) -> image::RgbaImage {
    assert!(capture.png_bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    let image = image::load_from_memory(&capture.png_bytes)
        .expect("valid automation png")
        .to_rgba8();
    assert_eq!(image.width(), capture.width);
    assert_eq!(image.height(), capture.height);
    assert!(
        is_nonblank(&image, 64),
        "capture should be visually non-blank"
    );
    image
}

fn tree(capture: &UiAutomationCapture) -> &UiAutomationNode {
    capture.ui_tree.as_ref().expect("automation tree")
}

fn selector(debug_id: &str) -> UiAutomationSelector {
    UiAutomationSelector::ByDebugId {
        debug_id: debug_id.to_string(),
    }
}

fn assert_selector(capture: &UiAutomationCapture, debug_id: &str) {
    assert!(
        find_node(tree(capture), &selector(debug_id)).is_some(),
        "missing selector {debug_id}"
    );
}

fn assert_absent(capture: &UiAutomationCapture, debug_id: &str) {
    assert!(
        find_node(tree(capture), &selector(debug_id)).is_none(),
        "unexpected selector {debug_id}"
    );
}

fn assert_bounds_inside(capture: &UiAutomationCapture, debug_id: &str) {
    let node = find_node(tree(capture), &selector(debug_id)).expect("selector node");
    assert_rect_inside(
        &node.bounds,
        capture.width as f64,
        capture.height as f64,
        debug_id,
    );
}

fn assert_rect_inside(rect: &UiAutomationRect, width: f64, height: f64, label: &str) {
    assert!(
        rect.width > 0.0,
        "{label} width should be positive: {rect:?}"
    );
    assert!(
        rect.height > 0.0,
        "{label} height should be positive: {rect:?}"
    );
    assert!(rect.x >= 0.0, "{label} x should be on-screen: {rect:?}");
    assert!(rect.y >= 0.0, "{label} y should be on-screen: {rect:?}");
    assert!(
        rect.x + rect.width <= width,
        "{label} should fit viewport width: {rect:?} / {width}"
    );
    assert!(
        rect.y + rect.height <= height,
        "{label} should fit viewport height: {rect:?} / {height}"
    );
}

fn click(harness: &mut TestHarness, debug_id: &str) -> UiAutomationCapture {
    harness
        .automation_action(UiAutomationAction::Click {
            selector: selector(debug_id),
            modifiers: Default::default(),
        })
        .unwrap_or_else(|err| panic!("click {debug_id}: {err:?}"))
}

fn key(
    harness: &mut TestHarness,
    key: UiAutomationKey,
    modifiers: UiAutomationModifiers,
) -> UiAutomationCapture {
    harness
        .automation_action(UiAutomationAction::KeyPress { key, modifiers })
        .expect("key press")
}

fn open_menu_item(
    harness: &mut TestHarness,
    menu_id: &str,
    item_id: &str,
    expected_id: &str,
) -> UiAutomationCapture {
    let menu = click(harness, menu_id);
    decode_png(&menu);
    assert_selector(&menu, item_id);
    let opened = click(harness, item_id);
    decode_png(&opened);
    assert_selector(&opened, expected_id);
    assert_bounds_inside(&opened, expected_id);
    opened
}

#[test]
fn sheets_plan_controls_are_visible_and_clickable_ui_e2e() {
    let mut harness = harness();
    let capture = harness.automation_capture(UiAutomationCaptureRequest::default());
    decode_png(&capture);

    for debug_id in [
        "sheets.menu.file",
        "sheets.menu.edit",
        "sheets.menu.view",
        "sheets.menu.insert",
        "sheets.menu.format",
        "sheets.menu.data",
        "sheets.menu.tools",
        "sheets.menu.help",
        "sheets.toolbar.bold",
        "sheets.toolbar.italic",
        "sheets.toolbar.underline",
        "sheets.toolbar.align_left",
        "sheets.toolbar.align_center",
        "sheets.toolbar.align_right",
        "sheets.toolbar.number_format",
        "sheets.toolbar.format_painter",
        "sheets.toolbar.merge_cells",
        "sheets.formula.input",
        "sheets.grid.cell.1.1",
        "sheets.sheet_nav.first",
        "sheets.sheet_nav.previous",
        "sheets.sheet_nav.next",
        "sheets.sheet_nav.last",
        "sheets.sheet.add",
        "sheets.status.zoom_in",
        "sheets.status.zoom_out",
        "sheets.status.zoom_reset",
        "sheets.status.zoom_slider",
    ] {
        assert_selector(&capture, debug_id);
        assert_bounds_inside(&capture, debug_id);
    }

    let after_file_click = harness
        .automation_action(UiAutomationAction::Click {
            selector: selector("sheets.menu.file"),
            modifiers: Default::default(),
        })
        .expect("click file menu");
    decode_png(&after_file_click);
    assert_selector(&after_file_click, "sheets.menu.file.item.new_workbook");
    assert_selector(&after_file_click, "sheets.menu.file.item.page_setup");
    assert_bounds_inside(&after_file_click, "sheets.menu.file.item.new_workbook");

    let after_edit_click = harness
        .automation_action(UiAutomationAction::Click {
            selector: selector("sheets.menu.edit"),
            modifiers: Default::default(),
        })
        .expect("click edit menu");
    decode_png(&after_edit_click);
    assert_absent(&after_edit_click, "sheets.menu.file.item.new_workbook");
    assert_selector(&after_edit_click, "sheets.menu.edit.item.find");

    let after_find_click = harness
        .automation_action(UiAutomationAction::Click {
            selector: selector("sheets.menu.edit.item.find"),
            modifiers: Default::default(),
        })
        .expect("click find menu item");
    decode_png(&after_find_click);
    assert_absent(&after_find_click, "sheets.menu.edit.item.find");
    assert_selector(&after_find_click, "sheets.dialog.find_replace");
}

#[test]
fn sheets_plan_toolbar_and_automatic_status_update_via_real_events_ui_e2e() {
    let mut harness = harness();

    let before = harness.automation_capture(UiAutomationCaptureRequest::default());
    decode_png(&before);

    let after_bold = click(&mut harness, "sheets.toolbar.bold");
    let before_image = decode_png(&before);
    let after_image = decode_png(&after_bold);
    assert!(
        before_image.as_raw() != after_image.as_raw(),
        "bold click should produce a visible toolbar state change"
    );
    assert_selector(&after_bold, "sheets.toolbar.bold");

    let after_zoom = click(&mut harness, "sheets.status.zoom_in");
    let zoom_image = decode_png(&after_zoom);
    assert!(
        after_image.as_raw() != zoom_image.as_raw(),
        "zoom-in click should update automatic status/canvas rendering"
    );
    assert_selector(&after_zoom, "sheets.status.zoom_slider");

    let before_shortcut = after_zoom;
    let after_find_shortcut = key(
        &mut harness,
        UiAutomationKey::Character("f".to_string()),
        UiAutomationModifiers {
            control: true,
            ..UiAutomationModifiers::default()
        },
    );
    let before_shortcut_image = decode_png(&before_shortcut);
    let find_shortcut_image = decode_png(&after_find_shortcut);
    assert!(
        before_shortcut_image.as_raw() != find_shortcut_image.as_raw(),
        "find shortcut should display a dialog"
    );
    assert_selector(&after_find_shortcut, "sheets.dialog.find_replace");
}

#[test]
fn sheets_plan_menu_dialog_controls_use_real_clicks_ui_e2e() {
    for (menu_id, item_id, expected_ids) in [
        (
            "sheets.menu.file",
            "sheets.menu.file.item.page_setup",
            &[
                "sheets.dialog.page_setup",
                "sheets.page_setup.orientation",
                "sheets.page_setup.ok",
            ][..],
        ),
        (
            "sheets.menu.file",
            "sheets.menu.file.item.print_preview",
            &["sheets.print_preview"][..],
        ),
        (
            "sheets.menu.edit",
            "sheets.menu.edit.item.paste_special",
            &[
                "sheets.dialog.paste_special",
                "sheets.paste_special.mode.values",
                "sheets.paste_special.ok",
            ][..],
        ),
        (
            "sheets.menu.insert",
            "sheets.menu.insert.item.function",
            &[
                "sheets.dialog.insert_function",
                "sheets.insert_function.row.0",
                "sheets.insert_function.insert",
            ][..],
        ),
        (
            "sheets.menu.insert",
            "sheets.menu.insert.item.chart",
            &[
                "sheets.dialog.chart_wizard",
                "sheets.chart_wizard.data_range",
                "sheets.chart_wizard.next",
            ][..],
        ),
        (
            "sheets.menu.format",
            "sheets.menu.format.item.format_cells",
            &[
                "sheets.dialog.format_cells",
                "sheets.format_cells.tab.number",
                "sheets.format_cells.ok",
            ][..],
        ),
        (
            "sheets.menu.format",
            "sheets.menu.format.item.conditional_format",
            &[
                "sheets.dialog.conditional_format",
                "sheets.conditional_format.operator",
                "sheets.conditional_format.ok",
            ][..],
        ),
        (
            "sheets.menu.data",
            "sheets.menu.data.item.sort",
            &["sheets.dialog.sort", "sheets.sort.column", "sheets.sort.ok"][..],
        ),
        (
            "sheets.menu.data",
            "sheets.menu.data.item.data_validation",
            &[
                "sheets.dialog.data_validation",
                "sheets.data_validation.type",
                "sheets.data_validation.ok",
            ][..],
        ),
        (
            "sheets.menu.data",
            "sheets.menu.data.item.pivot_table",
            &["sheets.dialog.pivot_table"][..],
        ),
        (
            "sheets.menu.tools",
            "sheets.menu.tools.item.settings",
            &["sheets.dialog.settings"][..],
        ),
    ] {
        let mut harness = harness();
        let capture = open_menu_item(&mut harness, menu_id, item_id, expected_ids[0]);
        for expected_id in expected_ids {
            assert_selector(&capture, expected_id);
            assert_bounds_inside(&capture, expected_id);
        }
    }
}

#[test]
fn sheets_plan_sheet_tabs_and_keyboard_navigation_use_real_events_ui_e2e() {
    let mut harness = harness();

    let added = click(&mut harness, "sheets.sheet.add");
    decode_png(&added);
    assert_selector(&added, "sheets.sheet_tab.1");
    assert_bounds_inside(&added, "sheets.sheet_tab.1");

    let last = click(&mut harness, "sheets.sheet_nav.last");
    decode_png(&last);
    assert_selector(&last, "sheets.sheet_tab.1");

    let first = click(&mut harness, "sheets.sheet_nav.first");
    decode_png(&first);
    assert_selector(&first, "sheets.sheet_tab.0");

    let grid_before = harness.automation_capture(UiAutomationCaptureRequest::default());
    let moved = key(
        &mut harness,
        UiAutomationKey::ArrowRight,
        UiAutomationModifiers::default(),
    );
    decode_png(&moved);
    assert_selector(&moved, "sheets.grid.cell.1.2");
    assert!(
        decode_png(&grid_before).as_raw() != decode_png(&moved).as_raw(),
        "arrow key selection should visibly move the active cell"
    );
}
