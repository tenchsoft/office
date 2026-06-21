//! Widget tree assertion helpers.
//!
//! Provides a trait and functions for asserting on widget tree state,
//! including visibility, focus, layout bounds, and hit-testing.

use kurbo::{Point, Rect, Size};
use tench_ui::core::types::WidgetId;
use tench_ui::core::widget::WidgetPod;

use crate::harness::TestHarness;

/// Extension trait for asserting on widget tree state via a `TestHarness`.
pub trait WidgetAssertions {
    /// Asserts that a widget with the given ID is visible (has non-zero size).
    fn assert_widget_visible(&self, id: WidgetId);

    /// Asserts that a widget with the given ID is at the expected position.
    fn assert_widget_position(&self, id: WidgetId, expected: Point);

    /// Asserts that a widget with the given ID has the expected size.
    fn assert_widget_size(&self, id: WidgetId, expected: Size);

    /// Asserts that a widget's bounding box is contained within its parent.
    fn assert_widget_within_bounds(&self, id: WidgetId, parent_bounds: Rect);

    /// Asserts that the given point hits a widget (any widget).
    fn assert_hit(&mut self, pos: Point);

    /// Asserts that the given point hits a specific widget.
    fn assert_hit_widget(&mut self, pos: Point, expected_id: WidgetId);

    /// Asserts that the given point does NOT hit any widget.
    fn assert_no_hit(&mut self, pos: Point);

    /// Asserts that a specific widget is focused.
    fn assert_focus_on(&self, id: WidgetId);

    /// Asserts that no widget is focused.
    fn assert_no_focus(&self);

    /// Asserts that the root widget has non-zero size (is laid out).
    fn assert_root_laid_out(&self);

    /// Asserts that the root widget's bounds are within the viewport.
    fn assert_root_within_viewport(&self);

    /// Asserts that a widget's laid-out bounds fit within the expected text
    /// content area. Text-specific glyph metrics are validated by render
    /// snapshots; this assertion catches layout overflow before paint.
    fn assert_no_overflow(&self, _id: WidgetId, _bounds: Rect);
}

impl WidgetAssertions for TestHarness {
    fn assert_widget_visible(&self, id: WidgetId) {
        if self.root_id() == id {
            let size = self.root_size();
            assert!(
                size.width > 0.0 && size.height > 0.0,
                "Widget {} is not visible (size: {:?})",
                id,
                size
            );
        } else {
            panic!(
                "assert_widget_visible: non-root widget assertions are not yet supported \
                 (requested widget {}, root is {})",
                id,
                self.root_id()
            );
        }
    }

    fn assert_widget_position(&self, id: WidgetId, expected: Point) {
        if self.root_id() == id {
            let pos = self.root().state.position;
            assert!(
                (pos.x - expected.x).abs() < 0.01 && (pos.y - expected.y).abs() < 0.01,
                "Widget {} position: expected {:?}, got {:?}",
                id,
                expected,
                pos
            );
        } else {
            panic!(
                "assert_widget_position: non-root widget assertions are not yet supported \
                 (requested widget {}, root is {})",
                id,
                self.root_id()
            );
        }
    }

    fn assert_widget_size(&self, id: WidgetId, expected: Size) {
        if self.root_id() == id {
            let size = self.root_size();
            assert!(
                (size.width - expected.width).abs() < 0.01
                    && (size.height - expected.height).abs() < 0.01,
                "Widget {} size: expected {:?}, got {:?}",
                id,
                expected,
                size
            );
        } else {
            panic!(
                "assert_widget_size: non-root widget assertions are not yet supported \
                 (requested widget {}, root is {})",
                id,
                self.root_id()
            );
        }
    }

    fn assert_widget_within_bounds(&self, id: WidgetId, parent_bounds: Rect) {
        if self.root_id() == id {
            let widget_bounds = self.root_bounds();
            assert!(
                parent_bounds.contains_rect(widget_bounds),
                "Widget {} bounds {:?} exceed parent bounds {:?}",
                id,
                widget_bounds,
                parent_bounds
            );
        } else {
            panic!(
                "assert_widget_within_bounds: non-root widget assertions are not yet supported \
                 (requested widget {}, root is {})",
                id,
                self.root_id()
            );
        }
    }

    fn assert_hit(&mut self, pos: Point) {
        let hit = self.hit_test(pos);
        assert!(hit.is_some(), "Expected a widget hit at {:?}", pos);
    }

    fn assert_hit_widget(&mut self, pos: Point, expected_id: WidgetId) {
        let hit = self.hit_test(pos);
        assert_eq!(
            hit,
            Some(expected_id),
            "Expected hit at {:?} to be widget {}, got {:?}",
            pos,
            expected_id,
            hit
        );
    }

    fn assert_no_hit(&mut self, pos: Point) {
        let hit = self.hit_test(pos);
        assert!(
            hit.is_none(),
            "Expected no widget hit at {:?}, but hit widget {:?}",
            pos,
            hit
        );
    }

    fn assert_focus_on(&self, id: WidgetId) {
        let focused = self.focused_widget();
        assert_eq!(
            focused,
            Some(id),
            "Expected focus on widget {}, but focus is on {:?}",
            id,
            focused
        );
    }

    fn assert_no_focus(&self) {
        let focused = self.focused_widget();
        assert!(
            focused.is_none(),
            "Expected no focused widget, but focus is on {:?}",
            focused
        );
    }

    fn assert_root_laid_out(&self) {
        let size = self.root_size();
        assert!(
            size.width > 0.0 && size.height > 0.0,
            "Root widget is not laid out (size: {:?})",
            size
        );
    }

    fn assert_root_within_viewport(&self) {
        let viewport = Rect::from_origin_size(Point::ZERO, self.config().viewport);
        let root_bounds = self.root_bounds();
        assert!(
            viewport.contains_rect(root_bounds),
            "Root widget bounds {:?} exceed viewport {:?}",
            root_bounds,
            viewport
        );
    }

    fn assert_no_overflow(&self, _id: WidgetId, _bounds: Rect) {
        if self.root_id() != _id {
            panic!(
                "assert_no_overflow: non-root widget assertions are not yet supported \
                 (requested widget {}, root is {})",
                _id,
                self.root_id()
            );
        }
        let root_bounds = self.root_bounds();
        assert!(
            root_bounds.x0.is_finite()
                && root_bounds.y0.is_finite()
                && root_bounds.x1.is_finite()
                && root_bounds.y1.is_finite(),
            "Widget {} bounds are not finite: {:?}",
            _id,
            root_bounds
        );
        assert!(
            _bounds.contains_rect(root_bounds),
            "Widget {} content bounds {:?} exceed allowed text bounds {:?}",
            _id,
            root_bounds,
            _bounds
        );
    }
}

// --- Standalone assertion helpers ---

/// Asserts that a widget pod has non-zero size.
pub fn assert_visible(pod: &WidgetPod) {
    let size = pod.state.size;
    assert!(
        size.width > 0.0 && size.height > 0.0,
        "Widget {} is not visible (size: {:?})",
        pod.id(),
        size
    );
}

/// Asserts that a widget pod's bounding box is within the given rect.
pub fn assert_within_bounds(pod: &WidgetPod, bounds: Rect) {
    let widget_bounds = pod.state.bounding_box();
    assert!(
        bounds.contains_rect(widget_bounds),
        "Widget {} bounds {:?} exceed {:?}",
        pod.id(),
        widget_bounds,
        bounds
    );
}

/// Asserts that a widget pod is focused.
pub fn assert_focused(pod: &WidgetPod) {
    assert!(pod.state.is_focused, "Widget {} is not focused", pod.id());
}

/// Asserts that a widget pod is not focused.
pub fn assert_not_focused(pod: &WidgetPod) {
    assert!(
        !pod.state.is_focused,
        "Widget {} is unexpectedly focused",
        pod.id()
    );
}

/// Asserts that a widget pod is hovered.
pub fn assert_hovered(pod: &WidgetPod) {
    assert!(pod.state.is_hovered, "Widget {} is not hovered", pod.id());
}

/// Asserts that a widget pod is disabled.
pub fn assert_disabled(pod: &WidgetPod) {
    assert!(pod.state.is_disabled, "Widget {} is not disabled", pod.id());
}

/// Asserts that a widget pod is NOT disabled.
pub fn assert_enabled(pod: &WidgetPod) {
    assert!(
        !pod.state.is_disabled,
        "Widget {} is unexpectedly disabled",
        pod.id()
    );
}
