//! License automation nodes — exposes the local credential store state to
//! the UI test harness via four `Label` debug_id nodes:
//!
//! | debug_id | value |
//! |---|---|
//! | `<product>.license.state` | `"unactivated"` / `"active"` / `"expired"` |
//! | `<product>.license.device_id` | 64-char hex SHA-256 |
//! | `<product>.license.license_key` | `"TENCH-..."` or `""` |
//! | `<product>.license.expires_at` | RFC 3339 string or `""` |
//!
//! See `plans/background/docs/license-persistence.md` Observability table.

use std::sync::Arc;
use tench_ui::{UiAutomationNode, UiAutomationRect};
use tench_license_store::{LicenseStatus, LicenseStore};

/// Pushes the 4 license automation nodes for the given store. `base_id` is
/// the next free node id (callers usually pass `last_node.id + 1`).
pub(crate) fn push_license_nodes(
    nodes: &mut Vec<UiAutomationNode>,
    mut next_id: u64,
    store: &Arc<LicenseStore>,
    product: &str,
) {
    let state = store.state();
    let status_str = match state.status() {
        LicenseStatus::Unactivated => "unactivated",
        LicenseStatus::Active => "active",
        LicenseStatus::Expired => "expired",
    };
    let license_key = state.license_key.clone().unwrap_or_default();
    let expires_at = state.token_expires_at.clone().unwrap_or_default();

    push(
        nodes,
        &mut next_id,
        format!("{product}.license.state"),
        status_str,
    );
    push(
        nodes,
        &mut next_id,
        format!("{product}.license.device_id"),
        &state.device_id,
    );
    push(
        nodes,
        &mut next_id,
        format!("{product}.license.license_key"),
        &license_key,
    );
    push(
        nodes,
        &mut next_id,
        format!("{product}.license.expires_at"),
        &expires_at,
    );
}

fn push(
    nodes: &mut Vec<UiAutomationNode>,
    next_id: &mut u64,
    debug_id: String,
    value: &str,
) {
    *next_id = next_id.saturating_add(1);
    nodes.push(UiAutomationNode {
        id: *next_id,
        debug_id: Some(debug_id),
        role: "label".to_string(),
        label: Some(value.to_string()),
        value: Some(value.to_string()),
        bounds: UiAutomationRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
        enabled: true,
        focused: false,
        hovered: false,
        children: Vec::new(),
    });
}
