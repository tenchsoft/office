//! High-level helpers for CI E2E tests and agent UI inspection.
//!
//! These utilities turn raw automation captures into concise assertions and
//! readable reports. Product tests should prefer these helpers over manually
//! decoding PNG bytes or hardcoding pointer coordinates.

use std::path::PathBuf;
use std::sync::OnceLock;

use tench_ui_automation_core::{
    find_node, format_capture_report, node_inventory, UiAutomationCapture, UiAutomationNode,
    UiAutomationNodeSummary, UiAutomationPoint, UiAutomationRect, UiAutomationSelector,
};

use crate::snapshot::{self, DiffResult, SnapshotComparator};

pub const DEFAULT_NONBLANK_MIN_PIXELS: u64 = 32;

pub trait CaptureAssertions {
    fn assert_png_valid(&self) -> image::RgbaImage;
    fn assert_png_size(&self, width: u32, height: u32) -> image::RgbaImage;
    fn assert_nonblank(&self) -> image::RgbaImage;
    fn assert_selector_present(&self, selector: &UiAutomationSelector) -> &UiAutomationNode;
    fn assert_selector_absent(&self, selector: &UiAutomationSelector);
    fn selector_bounds(&self, selector: &UiAutomationSelector) -> UiAutomationRect;
    fn selector_center(&self, selector: &UiAutomationSelector) -> UiAutomationPoint;
    fn node_inventory(&self) -> Vec<UiAutomationNodeSummary>;
    fn automation_report(&self) -> String;
    fn assert_matches_baseline(&self, name: &str);
    fn assert_matches_baseline_with_tolerance(
        &self,
        name: &str,
        max_diff_pixels: u64,
        max_channel_diff: u8,
    );
}

impl CaptureAssertions for UiAutomationCapture {
    fn assert_png_valid(&self) -> image::RgbaImage {
        assert_capture_png_valid(self)
    }

    fn assert_png_size(&self, width: u32, height: u32) -> image::RgbaImage {
        assert_capture_png_size(self, width, height)
    }

    fn assert_nonblank(&self) -> image::RgbaImage {
        assert_capture_nonblank(self)
    }

    fn assert_selector_present(&self, selector: &UiAutomationSelector) -> &UiAutomationNode {
        assert_selector_present(self, selector)
    }

    fn assert_selector_absent(&self, selector: &UiAutomationSelector) {
        assert_selector_absent(self, selector);
    }

    fn selector_bounds(&self, selector: &UiAutomationSelector) -> UiAutomationRect {
        assert_selector_present(self, selector).bounds
    }

    fn selector_center(&self, selector: &UiAutomationSelector) -> UiAutomationPoint {
        assert_selector_present(self, selector).center()
    }

    fn node_inventory(&self) -> Vec<UiAutomationNodeSummary> {
        self.ui_tree
            .as_ref()
            .map(node_inventory)
            .unwrap_or_default()
    }

    fn automation_report(&self) -> String {
        format_capture_report(self)
    }

    fn assert_matches_baseline(&self, name: &str) {
        assert_capture_matches_baseline(self, name, 0, 0);
    }

    fn assert_matches_baseline_with_tolerance(
        &self,
        name: &str,
        max_diff_pixels: u64,
        max_channel_diff: u8,
    ) {
        assert_capture_matches_baseline(self, name, max_diff_pixels, max_channel_diff);
    }
}

pub fn assert_capture_png_valid(capture: &UiAutomationCapture) -> image::RgbaImage {
    assert!(
        !capture.png_bytes.is_empty(),
        "automation capture did not include PNG bytes; request include_png=true"
    );
    assert!(
        capture.png_bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "automation capture bytes are not a PNG stream"
    );

    let image = image::load_from_memory(&capture.png_bytes)
        .unwrap_or_else(|error| panic!("failed to decode automation PNG: {error}"))
        .into_rgba8();
    assert_eq!(
        (image.width(), image.height()),
        (capture.width, capture.height),
        "decoded PNG dimensions differ from capture metadata"
    );
    image
}

pub fn assert_capture_png_size(
    capture: &UiAutomationCapture,
    width: u32,
    height: u32,
) -> image::RgbaImage {
    assert_eq!(
        (capture.width, capture.height),
        (width, height),
        "automation capture dimensions differ from expected viewport"
    );
    let image = assert_capture_png_valid(capture);
    assert_eq!(
        (image.width(), image.height()),
        (width, height),
        "decoded PNG dimensions differ from expected viewport"
    );
    image
}

pub fn assert_capture_nonblank(capture: &UiAutomationCapture) -> image::RgbaImage {
    assert_capture_nonblank_with_min(capture, DEFAULT_NONBLANK_MIN_PIXELS)
}

pub fn assert_capture_nonblank_with_min(
    capture: &UiAutomationCapture,
    min_unique_pixels: u64,
) -> image::RgbaImage {
    let image = assert_capture_png_valid(capture);
    assert!(
        snapshot::is_nonblank(&image, min_unique_pixels),
        "automation capture is blank or nearly blank: {}x{}, min_unique_pixels={}",
        capture.width,
        capture.height,
        min_unique_pixels
    );
    image
}

pub fn assert_capture_changed(
    before: &UiAutomationCapture,
    after: &UiAutomationCapture,
) -> DiffResult {
    assert_capture_changed_by(before, after, 1)
}

pub fn assert_capture_changed_by(
    before: &UiAutomationCapture,
    after: &UiAutomationCapture,
    min_changed_pixels: u64,
) -> DiffResult {
    assert_eq!(
        (before.width, before.height),
        (after.width, after.height),
        "cannot compare captures with different dimensions"
    );
    let before_image = assert_capture_png_valid(before);
    let after_image = assert_capture_png_valid(after);
    let diff = SnapshotComparator::new().compare_images(&before_image, &after_image);
    assert!(
        diff.different_pixels >= min_changed_pixels,
        "expected automation capture to change by at least {} pixels, changed {} of {} pixels",
        min_changed_pixels,
        diff.different_pixels,
        diff.total_pixels
    );
    diff
}

pub fn assert_selector_present<'a>(
    capture: &'a UiAutomationCapture,
    selector: &UiAutomationSelector,
) -> &'a UiAutomationNode {
    let root = capture_tree(capture);
    find_node(root, selector).unwrap_or_else(|| {
        let report = format_capture_report(capture);
        let truncated = truncate_report(&report, 4096);
        panic!(
            "selector not found: {}\n{}",
            selector.description(),
            truncated
        )
    })
}

/// Truncates a report string to `max_len` bytes, appending an ellipsis if
/// truncated, so that assertion failure messages remain readable in CI logs.
pub fn truncate_report(report: &str, max_len: usize) -> String {
    if report.len() <= max_len {
        report.to_string()
    } else {
        let mut truncated = report[..report.floor_char_boundary(max_len)].to_string();
        truncated.push_str("\n... (report truncated)");
        truncated
    }
}

pub fn assert_selector_absent(capture: &UiAutomationCapture, selector: &UiAutomationSelector) {
    let root = capture_tree(capture);
    assert!(
        find_node(root, selector).is_none(),
        "selector should be absent but was found: {}\n{}",
        selector.description(),
        format_capture_report(capture)
    );
}

pub fn capture_tree(capture: &UiAutomationCapture) -> &UiAutomationNode {
    capture.ui_tree.as_ref().unwrap_or_else(|| {
        panic!("automation capture did not include UI tree; request include_tree=true")
    })
}

pub fn assert_node_bounds_within_capture(capture: &UiAutomationCapture, node: &UiAutomationNode) {
    let viewport = UiAutomationRect {
        x: 0.0,
        y: 0.0,
        width: capture.width as f64,
        height: capture.height as f64,
    };
    assert!(
        viewport.contains_rect(node.bounds),
        "node bounds exceed capture viewport: selector={}, bounds={:?}, viewport={:?}",
        node.selector_hint()
            .map(|selector| selector.description())
            .unwrap_or_else(|| format!("id={}", node.id)),
        node.bounds,
        viewport
    );
}

pub fn assert_tree_bounds_within_capture(capture: &UiAutomationCapture) {
    for node in capture_tree(capture).walk() {
        assert_node_bounds_within_capture(capture, node);
    }
}

pub fn nodes_at_point(
    capture: &UiAutomationCapture,
    point: UiAutomationPoint,
) -> Vec<&UiAutomationNode> {
    capture
        .ui_tree
        .as_ref()
        .map(|root| tench_ui_automation_core::nodes_at_point(root, point))
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Baseline snapshot comparison (Section 5.5)
// ---------------------------------------------------------------------------

/// Root directory for baseline snapshot PNG files.
///
/// Defaults to `tests/snapshots/` relative to the current working directory.
/// Override with `TENCH_UI_SNAPSHOT_DIR` environment variable.
static SNAPSHOT_DIR: OnceLock<PathBuf> = OnceLock::new();

fn snapshot_dir() -> &'static PathBuf {
    SNAPSHOT_DIR.get_or_init(|| {
        std::env::var("TENCH_UI_SNAPSHOT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("tests/snapshots"))
    })
}

/// Returns the baseline PNG path for the given snapshot name.
pub fn baseline_path(name: &str) -> PathBuf {
    snapshot_dir().join(format!("{name}.png"))
}

/// Compares the capture's PNG against a baseline snapshot file.
///
/// If `TENCH_UI_UPDATE_SNAPSHOTS=1` is set, the baseline is created or
/// updated instead of compared. On mismatch, a diff PNG is written
/// alongside the baseline (e.g. `tests/snapshots/name.diff.png`).
pub fn assert_capture_matches_baseline(
    capture: &UiAutomationCapture,
    name: &str,
    max_diff_pixels: u64,
    max_channel_diff: u8,
) {
    let actual = assert_capture_png_valid(capture);
    let path = baseline_path(name);

    if std::env::var("TENCH_UI_UPDATE_SNAPSHOTS").unwrap_or_default() == "1" {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        actual.save(&path).unwrap_or_else(|e| {
            panic!(
                "failed to save baseline snapshot to {}: {e}",
                path.display()
            )
        });
        return;
    }

    let expected = image::open(&path)
        .unwrap_or_else(|e| {
            panic!(
            "baseline snapshot not found at {} (set TENCH_UI_UPDATE_SNAPSHOTS=1 to create): {e}",
            path.display()
        )
        })
        .into_rgba8();

    let comparator = SnapshotComparator::new();
    let result = comparator.compare_images(&actual, &expected);

    if !result.matches(max_diff_pixels, max_channel_diff) {
        // Generate diff image for debugging.
        if let Some(diff_image) = comparator.generate_diff_image(&actual, &expected) {
            let diff_path = snapshot_dir().join(format!("{name}.diff.png"));
            let _ = diff_image.save(&diff_path);
        }
        // Save the actual image for easy comparison.
        let actual_path = snapshot_dir().join(format!("{name}.actual.png"));
        let _ = actual.save(&actual_path);

        panic!(
            "baseline snapshot mismatch for '{name}': {}/{} pixels differ (max allowed: {max_diff_pixels}), \
             max channel diff: {} (max allowed: {max_channel_diff}), mean diff: {:.2}\n\
             baseline: {}\n\
             actual:   {}\n\
             diff:     {}",
            result.different_pixels,
            result.total_pixels,
            result.max_channel_diff,
            result.mean_diff,
            path.display(),
            actual_path.display(),
            snapshot_dir().join(format!("{name}.diff.png")).display(),
        );
    }
}

// ---------------------------------------------------------------------------
// Inventory diff (Section 5.10)
// ---------------------------------------------------------------------------

/// Describes the difference between two automation inventories.
#[derive(Debug, Clone)]
pub struct InventoryDiff {
    /// Nodes present in `after` but not in `before`.
    pub added: Vec<UiAutomationNodeSummary>,
    /// Nodes present in `before` but not in `after`.
    pub removed: Vec<UiAutomationNodeSummary>,
    /// Nodes present in both whose label or value changed.
    pub changed: Vec<InventoryChange>,
}

/// A single node that changed between two inventories.
#[derive(Debug, Clone)]
pub struct InventoryChange {
    pub debug_id: Option<String>,
    pub before_label: Option<String>,
    pub after_label: Option<String>,
    pub before_value: Option<String>,
    pub after_value: Option<String>,
}

impl std::fmt::Display for InventoryDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty() {
            write!(f, "inventory diff: no changes")?;
            return Ok(());
        }
        writeln!(f, "inventory diff:")?;
        for node in &self.added {
            let id = node.debug_id.as_deref().unwrap_or("(no debug_id)");
            let text = node
                .label
                .as_deref()
                .or(node.value.as_deref())
                .unwrap_or("");
            writeln!(f, "  + {id} ({}) {text:?})", node.role)?;
        }
        for node in &self.removed {
            let id = node.debug_id.as_deref().unwrap_or("(no debug_id)");
            let text = node
                .label
                .as_deref()
                .or(node.value.as_deref())
                .unwrap_or("");
            writeln!(f, "  - {id} ({}) {text:?})", node.role)?;
        }
        for change in &self.changed {
            let id = change.debug_id.as_deref().unwrap_or("(no debug_id)");
            writeln!(f, "  ~ {id}")?;
            if change.before_label != change.after_label {
                writeln!(
                    f,
                    "      label: {:?} -> {:?}",
                    change.before_label, change.after_label
                )?;
            }
            if change.before_value != change.after_value {
                writeln!(
                    f,
                    "      value: {:?} -> {:?}",
                    change.before_value, change.after_value
                )?;
            }
        }
        Ok(())
    }
}

/// Computes the diff between two inventories.
///
/// Nodes are matched by `debug_id`. Nodes without a `debug_id` are matched
/// by `(role, label)`.
pub fn inventory_diff(
    before: &[UiAutomationNodeSummary],
    after: &[UiAutomationNodeSummary],
) -> InventoryDiff {
    use std::collections::HashMap;

    let mut before_by_key: HashMap<String, &UiAutomationNodeSummary> = HashMap::new();
    for node in before {
        let key = inventory_key(node);
        before_by_key.entry(key).or_insert(node);
    }

    let mut after_by_key: HashMap<String, &UiAutomationNodeSummary> = HashMap::new();
    for node in after {
        let key = inventory_key(node);
        after_by_key.entry(key).or_insert(node);
    }

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    // Find added and changed nodes.
    for (key, after_node) in &after_by_key {
        if let Some(before_node) = before_by_key.get(key) {
            if before_node.label != after_node.label || before_node.value != after_node.value {
                changed.push(InventoryChange {
                    debug_id: after_node.debug_id.clone(),
                    before_label: before_node.label.clone(),
                    after_label: after_node.label.clone(),
                    before_value: before_node.value.clone(),
                    after_value: after_node.value.clone(),
                });
            }
        } else {
            added.push((*after_node).clone());
        }
    }

    // Find removed nodes.
    for (key, before_node) in &before_by_key {
        if !after_by_key.contains_key(key) {
            removed.push((*before_node).clone());
        }
    }

    InventoryDiff {
        added,
        removed,
        changed,
    }
}

fn inventory_key(node: &UiAutomationNodeSummary) -> String {
    if let Some(debug_id) = &node.debug_id {
        format!("debug_id:{debug_id}")
    } else {
        let label = node.label.as_deref().unwrap_or("");
        format!("role:{}:{label}", node.role)
    }
}

/// Prints the inventory diff between two captures for debugging.
pub fn print_inventory_diff(before: &UiAutomationCapture, after: &UiAutomationCapture) {
    let before_inv = before.node_inventory();
    let after_inv = after.node_inventory();
    let diff = inventory_diff(&before_inv, &after_inv);
    eprintln!("{diff}");
}
