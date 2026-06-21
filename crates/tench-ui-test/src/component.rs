//! Fluent Component API for test automation.

use tench_ui_automation_core::{
    find_node, UiAutomationAction, UiAutomationCapture, UiAutomationCaptureRequest,
    UiAutomationModifiers, UiAutomationSelector,
};

use crate::harness::TestHarness;

/// A fluent handle to a single automation node, identified by `debug_id`.
///
/// Each method dispatches an action and returns `&mut Self` for chaining.
pub struct Component<'h> {
    harness: &'h mut TestHarness,
    debug_id: String,
}

impl<'h> Component<'h> {
    pub(crate) fn new(harness: &'h mut TestHarness, debug_id: String) -> Self {
        Self { harness, debug_id }
    }

    /// Returns the selector for this component.
    fn selector(&self) -> UiAutomationSelector {
        UiAutomationSelector::debug_id(&self.debug_id)
    }

    // --- Actions ---

    /// Clicks this component.
    pub fn click(&mut self) -> &mut Self {
        self.harness
            .automation_action(UiAutomationAction::Click {
                selector: self.selector(),
                modifiers: UiAutomationModifiers::default(),
            })
            .unwrap_or_else(|e| panic!("click on '{}' failed: {e:?}", self.debug_id));
        self
    }

    /// Clicks this component with modifiers held.
    pub fn click_with(&mut self, modifiers: UiAutomationModifiers) -> &mut Self {
        self.harness
            .automation_action(UiAutomationAction::Click {
                selector: self.selector(),
                modifiers,
            })
            .unwrap_or_else(|e| panic!("click_with on '{}' failed: {e:?}", self.debug_id));
        self
    }

    /// Right-clicks this component.
    pub fn right_click(&mut self) -> &mut Self {
        self.harness
            .automation_action(UiAutomationAction::RightClick {
                selector: self.selector(),
                modifiers: UiAutomationModifiers::default(),
            })
            .unwrap_or_else(|e| panic!("right_click on '{}' failed: {e:?}", self.debug_id));
        self
    }

    /// Double-clicks this component.
    pub fn double_click(&mut self) -> &mut Self {
        self.harness
            .automation_action(UiAutomationAction::DoubleClick {
                selector: self.selector(),
                modifiers: UiAutomationModifiers::default(),
            })
            .unwrap_or_else(|e| panic!("double_click on '{}' failed: {e:?}", self.debug_id));
        self
    }

    /// Hovers over this component.
    pub fn hover(&mut self) -> &mut Self {
        self.harness
            .automation_action(UiAutomationAction::Hover {
                selector: self.selector(),
            })
            .unwrap_or_else(|e| panic!("hover on '{}' failed: {e:?}", self.debug_id));
        self
    }

    /// Types text into this component (clicks first to focus, then types).
    pub fn type_text(&mut self, text: &str) -> &mut Self {
        self.harness
            .automation_action(UiAutomationAction::TypeText {
                selector: self.selector(),
                text: text.to_string(),
            })
            .unwrap_or_else(|e| panic!("type_text on '{}' failed: {e:?}", self.debug_id));
        self
    }

    // --- Queries ---

    /// Captures the current state and returns a fresh capture.
    pub fn capture(&mut self) -> UiAutomationCapture {
        self.harness
            .automation_capture(UiAutomationCaptureRequest::default())
    }

    /// Returns the label of this component, or `None` if not found.
    pub fn label(&mut self) -> Option<String> {
        let cap = self.capture();
        let tree = cap.ui_tree.as_ref()?;
        find_node(tree, &self.selector())?.label.clone()
    }

    /// Returns the value of this component, or `None`.
    pub fn value(&mut self) -> Option<String> {
        let cap = self.capture();
        let tree = cap.ui_tree.as_ref()?;
        find_node(tree, &self.selector())?.value.clone()
    }

    // --- Assertions ---

    /// Asserts that this component exists in the UI tree.
    pub fn assert_present(&mut self) -> &mut Self {
        let cap = self.capture();
        crate::assert_selector_present(&cap, &self.selector());
        self
    }

    /// Asserts that this component does NOT exist in the UI tree.
    pub fn assert_absent(&mut self) -> &mut Self {
        let cap = self.capture();
        crate::assert_selector_absent(&cap, &self.selector());
        self
    }

    /// Asserts that this component's label exactly equals `expected`.
    ///
    /// On failure, includes a truncated automation report for debugging.
    pub fn assert_label(&mut self, expected: &str) -> &mut Self {
        let cap = self.capture();
        let tree = cap.ui_tree.as_ref();
        let node = tree.and_then(|t| find_node(t, &self.selector()));
        match node {
            Some(node) => {
                let label = node.label.as_deref().unwrap_or("");
                if label != expected {
                    let report = tench_ui_automation_core::format_capture_report(&cap);
                    panic!(
                        "label mismatch for '{}': expected {expected:?}, got {label:?}\n{}",
                        self.debug_id,
                        crate::automation::truncate_report(&report, 4096)
                    );
                }
            }
            None => {
                let report = tench_ui_automation_core::format_capture_report(&cap);
                panic!(
                    "component '{}' not found in tree\n{}",
                    self.debug_id,
                    crate::automation::truncate_report(&report, 4096)
                );
            }
        }
        self
    }

    /// Asserts that this component's label contains `expected`.
    ///
    /// On failure, includes a truncated automation report for debugging.
    pub fn assert_label_contains(&mut self, expected: &str) -> &mut Self {
        let cap = self.capture();
        let tree = cap.ui_tree.as_ref();
        let node = tree.and_then(|t| find_node(t, &self.selector()));
        match node {
            Some(node) => {
                let label = node.label.as_deref().unwrap_or("");
                if !label.contains(expected) {
                    let report = tench_ui_automation_core::format_capture_report(&cap);
                    panic!(
                        "label {label:?} for '{}' does not contain {expected:?}\n{}",
                        self.debug_id,
                        crate::automation::truncate_report(&report, 4096)
                    );
                }
            }
            None => {
                let report = tench_ui_automation_core::format_capture_report(&cap);
                panic!(
                    "component '{}' not found in tree\n{}",
                    self.debug_id,
                    crate::automation::truncate_report(&report, 4096)
                );
            }
        }
        self
    }

    /// Asserts that this component's value exactly equals `expected`.
    ///
    /// On failure, includes a truncated automation report for debugging.
    pub fn assert_value(&mut self, expected: &str) -> &mut Self {
        let cap = self.capture();
        let tree = cap.ui_tree.as_ref();
        let node = tree.and_then(|t| find_node(t, &self.selector()));
        match node {
            Some(node) => {
                let value = node.value.as_deref().unwrap_or("");
                if value != expected {
                    let report = tench_ui_automation_core::format_capture_report(&cap);
                    panic!(
                        "value mismatch for '{}': expected {expected:?}, got {value:?}\n{}",
                        self.debug_id,
                        crate::automation::truncate_report(&report, 4096)
                    );
                }
            }
            None => {
                let report = tench_ui_automation_core::format_capture_report(&cap);
                panic!(
                    "component '{}' not found in tree\n{}",
                    self.debug_id,
                    crate::automation::truncate_report(&report, 4096)
                );
            }
        }
        self
    }

    /// Returns the `debug_id` string.
    pub fn debug_id(&self) -> &str {
        &self.debug_id
    }
}
