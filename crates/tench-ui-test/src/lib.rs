//! # tench-ui-test
//!
//! Test harness for Rust-native UI testing with the tench-ui framework.
//!
//! Provides:
//! - **TestHarness**: Mount a widget tree in a deterministic headless environment.
//! - **Event simulation**: Replay keyboard and pointer events without a real window.
//! - **Widget tree assertions**: Query and assert on widget state, visibility, and layout.
//! - **Snapshot comparison**: Render widgets to bitmaps and compare against baselines.
//! - **Accessibility tree**: Capture and assert on the accessibility tree structure.
//!
//! # Test naming conventions
//!
//! Test files should follow these suffixes:
//! - `*_unit.rs` — pure unit tests
//! - `*_integration.rs` — crate/command integration tests
//! - `*_security_regression.rs` — security regression tests
//! - `*_e2e.rs` — end-to-end tests (includes `ui_e2e`, `product_e2e`, `app_smoke`)
//!
//! # Example
//!
//! ```ignore
//! use tench_ui_test::{TestHarness, EventSimulator, WidgetAssertions};
//!
//! let mut harness = TestHarness::new(my_widget);
//! harness.layout(Size::new(800.0, 600.0));
//!
//! // Simulate a click
//! harness.dispatch_pointer(PointerEvent::Down(PointerButtonEvent {
//!     button: PointerButton::Primary,
//!     pos: Point::new(100.0, 100.0),
//!     buttons: PointerButtons::new(),
//! }));
//!
//! // Assert widget state
//! harness.assert_widget_visible(root_id);
//! harness.assert_focus_on(target_id);
//! ```

pub mod accessibility;
pub mod assertions;
pub mod automation;
pub mod component;
pub mod events;
pub mod harness;
pub mod snapshot;
pub mod test_helpers;

pub use accessibility::AccessibilitySnapshot;
pub use assertions::WidgetAssertions;
pub use automation::{
    assert_capture_changed, assert_capture_changed_by, assert_capture_nonblank,
    assert_capture_nonblank_with_min, assert_capture_png_size, assert_capture_png_valid,
    assert_node_bounds_within_capture, assert_selector_absent, assert_selector_present,
    assert_tree_bounds_within_capture, baseline_path, capture_tree, inventory_diff, nodes_at_point,
    print_inventory_diff, truncate_report, CaptureAssertions, InventoryChange, InventoryDiff,
    DEFAULT_NONBLANK_MIN_PIXELS,
};
pub use component::Component;
pub use events::EventSimulator;
pub use harness::TestHarness;
pub use snapshot::{render_scene_to_image, render_scene_to_png, SnapshotComparator};

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use kurbo::{Axis, Rect, Size};
    use tench_ui::core::events::PointerEvent;
    use tench_ui::core::types::Color;
    use tench_ui::core::widget::{
        AccessRole, AccessibilityNode, LayoutCtx, MeasureCtx, PaintCtx, Widget, WidgetState,
    };
    use tench_ui::render::Painter;
    use tench_ui::vello::Scene;
    use tench_ui::{UiAutomationNode, UiAutomationPoint, UiAutomationRect};
    use tench_ui_automation_core::{
        find_node, UiAutomationAction, UiAutomationCaptureRequest, UiAutomationSelector,
    };

    use super::{harness::HarnessConfig, CaptureAssertions, TestHarness, WidgetAssertions};

    struct FixedWidget;

    impl Widget for FixedWidget {
        fn measure(&mut self, _ctx: &mut MeasureCtx<'_>, _axis: Axis, _available: f64) -> f64 {
            64.0
        }

        fn layout(&mut self, ctx: &mut LayoutCtx<'_>, size: Size) {
            ctx.state.size = size;
        }

        fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _scene: &mut Scene) {}
    }

    #[test]
    fn widget_overflow_assertion_checks_root_bounds_ui_e2e() {
        let mut harness = TestHarness::new(FixedWidget);
        harness.layout_size(Size::new(120.0, 40.0));

        harness.assert_no_overflow(harness.root_id(), Rect::new(0.0, 0.0, 120.0, 40.0));
    }

    struct AutomationFixture {
        clicked: Arc<AtomicBool>,
    }

    impl Widget for AutomationFixture {
        fn measure(&mut self, _ctx: &mut MeasureCtx<'_>, _axis: Axis, available: f64) -> f64 {
            available
        }

        fn layout(&mut self, ctx: &mut LayoutCtx<'_>, size: Size) {
            ctx.state.size = size;
        }

        fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _scene: &mut Scene) {}

        fn on_pointer_event(&mut self, _ctx: &mut tench_ui::EventCtx<'_>, event: &PointerEvent) {
            if let PointerEvent::Down(event) = event {
                let hit = Rect::new(10.0, 12.0, 110.0, 44.0);
                if hit.contains(event.pos) {
                    self.clicked.store(true, Ordering::SeqCst);
                }
            }
        }

        fn accessibility_tree(&self, state: &WidgetState) -> AccessibilityNode {
            AccessibilityNode {
                role: AccessRole::Window,
                label: Some("Automation fixture".to_string()),
                value: None,
                focused: state.is_focused,
                disabled: state.is_disabled,
                children: Vec::new(),
            }
        }

        fn automation_children(&self, state: &WidgetState) -> Vec<UiAutomationNode> {
            vec![UiAutomationNode {
                id: state.id.to_raw().saturating_mul(1000).saturating_add(1),
                debug_id: Some("fixture.primary".to_string()),
                role: "button".to_string(),
                label: Some("Primary".to_string()),
                value: None,
                bounds: UiAutomationRect {
                    x: 10.0,
                    y: 12.0,
                    width: 100.0,
                    height: 32.0,
                },
                enabled: true,
                focused: false,
                hovered: false,
                children: Vec::new(),
            }]
        }

        fn debug_id(&self) -> Option<&str> {
            Some("fixture.root")
        }
    }

    #[test]
    fn headless_harness_clicks_selector_node_ui_automation() {
        let clicked = Arc::new(AtomicBool::new(false));
        let mut harness = TestHarness::with_config(
            AutomationFixture {
                clicked: Arc::clone(&clicked),
            },
            HarnessConfig::with_viewport(160.0, 80.0),
        );

        let tree = harness.automation_tree();
        assert!(find_node(
            &tree,
            &UiAutomationSelector::ByDebugId {
                debug_id: "fixture.primary".to_string()
            }
        )
        .is_some());

        let capture = harness
            .automation_action(UiAutomationAction::Click {
                selector: UiAutomationSelector::ByDebugId {
                    debug_id: "fixture.primary".to_string(),
                },
                modifiers: Default::default(),
            })
            .expect("click selector");

        assert!(clicked.load(Ordering::SeqCst));
        capture.assert_png_size(160, 80);
    }

    #[test]
    fn headless_harness_reports_selector_bounds_ui_automation() {
        let clicked = Arc::new(AtomicBool::new(false));
        let mut harness = TestHarness::with_config(
            AutomationFixture {
                clicked: Arc::clone(&clicked),
            },
            HarnessConfig::with_viewport(160.0, 80.0),
        );

        let selector = UiAutomationSelector::debug_id("fixture.primary");
        let bounds = harness.automation_bounds(&selector).expect("button bounds");
        assert_eq!(bounds.center(), UiAutomationPoint { x: 60.0, y: 28.0 });

        let hits = harness.automation_nodes_at_point(bounds.center());
        assert!(
            hits.iter()
                .any(|node| node.debug_id.as_deref() == Some("fixture.primary")),
            "point lookup should include the primary button"
        );

        let inventory = harness.automation_inventory();
        assert!(
            inventory.iter().any(|node| node.selector_hint.as_ref()
                == Some(&UiAutomationSelector::debug_id("fixture.primary"))),
            "inventory should include selector hints"
        );

        let report = harness.automation_report();
        assert!(report.contains("debug_id=fixture.primary"));
        assert!(report.contains("(60.0,28.0)"));
    }

    struct PaintedFixture;

    impl Widget for PaintedFixture {
        fn measure(&mut self, _ctx: &mut MeasureCtx<'_>, _axis: Axis, available: f64) -> f64 {
            available
        }

        fn layout(&mut self, ctx: &mut LayoutCtx<'_>, size: Size) {
            ctx.state.size = size;
        }

        fn paint(&mut self, ctx: &mut PaintCtx<'_>, scene: &mut Scene) {
            let mut painter = Painter::new(scene);
            painter.fill_background(ctx.state.size, Color::rgb8(16, 18, 24));
            painter.fill_rect(Rect::new(12.0, 10.0, 88.0, 50.0), Color::rgb8(52, 211, 153));
        }

        fn debug_id(&self) -> Option<&str> {
            Some("painted.root")
        }
    }

    #[test]
    fn headless_capture_returns_png_bytes_ui_automation() {
        let mut harness =
            TestHarness::with_config(PaintedFixture, HarnessConfig::with_viewport(128.0, 72.0));

        let capture = harness
            .try_automation_capture(UiAutomationCaptureRequest {
                include_png: true,
                include_tree: true,
            })
            .expect("headless PNG capture");

        assert_eq!(capture.width, 128);
        assert_eq!(capture.height, 72);
        assert!(
            capture.png_bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            "capture should return PNG bytes"
        );
        assert!(capture.ui_tree.is_some(), "capture should include UI tree");

        capture.assert_png_size(128, 72);
        capture.assert_nonblank();
    }
}
