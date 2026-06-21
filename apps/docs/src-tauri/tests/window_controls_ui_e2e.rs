/// UI automation tests for docs window caption controls (minimize / maximize /
/// close) and the title-bar drag region.
///
/// Verifies that clicking each caption button submits the correct
/// [`tench_ui::core::events::WindowAction`], that clicking empty menu-bar
/// space submits a `StartDrag`, and that clicking a real menu item does NOT
/// start a drag. Actions are inspected via `TestHarness::drain_actions` (the
/// headless harness does not execute them on a real window).
use tench_docs_lib::ui::DocsApp;
use tench_ui::core::events::{Action, WindowAction};
use tench_ui_test::harness::HarnessConfig;
use tench_ui_test::test_helpers::*;
use tench_ui_test::EventSimulator;
use tench_ui_test::TestHarness;

fn make_harness() -> TestHarness {
    TestHarness::with_config(DocsApp::new(), HarnessConfig::with_viewport(1280.0, 820.0))
}

/// Drain submitted actions and return the first `WindowAction`, if any.
fn drained_window_action(harness: &mut TestHarness) -> Option<WindowAction> {
    harness
        .drain_actions()
        .into_iter()
        .find_map(|(action, _)| action.downcast::<WindowAction>())
}

#[test]
fn window_controls_caption_buttons_present() {
    let mut harness = make_harness();
    let cap = capture(&mut harness);
    assert_selector(&cap, "docs.window.minimize");
    assert_selector(&cap, "docs.window.maximize");
    assert_selector(&cap, "docs.window.close");
    // Maximize starts restored in the headless harness (no real window).
    assert_eq!(
        get_node_value(&cap, "docs.window.maximize").as_deref(),
        Some("restored")
    );
}

#[test]
fn window_controls_close_submits_close_action() {
    let mut harness = make_harness();
    click(&mut harness, "docs.window.close");
    assert_eq!(
        drained_window_action(&mut harness),
        Some(WindowAction::Close)
    );
}

#[test]
fn window_controls_minimize_submits_minimize_action() {
    let mut harness = make_harness();
    click(&mut harness, "docs.window.minimize");
    assert_eq!(
        drained_window_action(&mut harness),
        Some(WindowAction::Minimize)
    );
}

#[test]
fn window_controls_maximize_submits_toggle_maximize_action() {
    let mut harness = make_harness();
    click(&mut harness, "docs.window.maximize");
    assert_eq!(
        drained_window_action(&mut harness),
        Some(WindowAction::ToggleMaximize)
    );
}

#[test]
fn window_controls_empty_menu_bar_submits_start_drag() {
    let mut harness = make_harness();
    // Click a point in the menu bar that is neither a menu label nor a
    // caption button. Menu labels end near x≈342; caption buttons start at
    // width - WINDOW_CONTROLS_W (≈1142). Midpoint is safely empty.
    let x = 740.0;
    let y = 18.0;
    for event in EventSimulator::click(tench_ui::Point::new(x, y)) {
        harness.dispatch_pointer(event);
    }
    assert_eq!(
        drained_window_action(&mut harness),
        Some(WindowAction::StartDrag)
    );
}

#[test]
fn window_controls_menu_item_click_does_not_drag() {
    let mut harness = make_harness();
    // Clicking a real menu item must open the menu, NOT start a window drag.
    click(&mut harness, "docs.menu.file");
    assert_eq!(drained_window_action(&mut harness), None);
    // Sanity: the File menu opened (no drag means the click was handled).
    let cap = capture(&mut harness);
    assert_eq!(node_text(&cap, "docs.menu.active"), "File");
}

#[test]
fn window_controls_caption_click_does_not_open_menu() {
    let mut harness = make_harness();
    // Clicking the close button must not be mis-routed to a menu item.
    click(&mut harness, "docs.window.close");
    let cap = capture(&mut harness);
    assert_eq!(node_text(&cap, "docs.menu.active"), "none");
    // Ensure `Action` import is used for downstream downcast consumers.
    let _: Vec<(Action, _)> = harness.drain_actions();
}
