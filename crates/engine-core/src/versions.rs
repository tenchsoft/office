//! Model Version Management (#466)
//!
//! Tracks model versions, supports pinning, and checks for updates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A specific version of a model.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelVersion {
    pub model_id: String,
    pub version: String,
    pub released_at: u64,
    pub changelog: Option<String>,
    pub deprecated: bool,
}

/// Pin policy for a model.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PinPolicy {
    /// Always use the latest available version.
    Latest,
    /// Pin to a specific version string.
    Pinned,
    /// Pin to the latest, but require manual confirmation to upgrade.
    Manual,
}

/// Version management state for a model.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelVersionState {
    pub model_id: String,
    pub current_version: String,
    pub pinned_version: Option<String>,
    pub pin_policy: PinPolicy,
    pub available_versions: Vec<ModelVersion>,
    pub last_checked_at: u64,
}

impl ModelVersionState {
    pub fn new(model_id: String, current_version: String) -> Self {
        Self {
            model_id,
            current_version,
            pinned_version: None,
            pin_policy: PinPolicy::Latest,
            available_versions: Vec::new(),
            last_checked_at: 0,
        }
    }

    /// Check if an update is available based on pin policy.
    pub fn update_available(&self) -> Option<&ModelVersion> {
        if self.pin_policy == PinPolicy::Pinned {
            return None;
        }
        self.available_versions
            .iter()
            .filter(|v| !v.deprecated)
            .find(|v| v.version != self.current_version)
    }
}

/// Manages version state across all models.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct VersionManager {
    states: HashMap<String, ModelVersionState>,
}

impl VersionManager {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Register a model with its current version.
    pub fn register(&mut self, model_id: String, current_version: String) {
        self.states.insert(
            model_id.clone(),
            ModelVersionState::new(model_id, current_version),
        );
    }

    /// Pin a model to a specific version.
    pub fn pin(&mut self, model_id: &str, version: String) {
        if let Some(state) = self.states.get_mut(model_id) {
            state.pin_policy = PinPolicy::Pinned;
            state.pinned_version = Some(version);
        }
    }

    /// Set a model to track the latest version.
    pub fn track_latest(&mut self, model_id: &str) {
        if let Some(state) = self.states.get_mut(model_id) {
            state.pin_policy = PinPolicy::Latest;
            state.pinned_version = None;
        }
    }

    /// Update available versions for a model (e.g. after a registry check).
    pub fn set_available_versions(&mut self, model_id: &str, versions: Vec<ModelVersion>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Some(state) = self.states.get_mut(model_id) {
            state.available_versions = versions;
            state.last_checked_at = now;
        }
    }

    /// Get the version state for a model.
    pub fn get(&self, model_id: &str) -> Option<&ModelVersionState> {
        self.states.get(model_id)
    }

    /// List all models that have updates available.
    pub fn models_with_updates(&self) -> Vec<&ModelVersionState> {
        self.states
            .values()
            .filter(|s| s.update_available().is_some())
            .collect()
    }

    /// Apply an update to a model.
    pub fn apply_update(&mut self, model_id: &str, new_version: &str) {
        if let Some(state) = self.states.get_mut(model_id) {
            state.current_version = new_version.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_prevents_update_detection() {
        let mut mgr = VersionManager::new();
        mgr.register("gpt-4".into(), "2024-01".into());
        mgr.pin("gpt-4", "2024-01".into());

        mgr.set_available_versions(
            "gpt-4",
            vec![ModelVersion {
                model_id: "gpt-4".into(),
                version: "2024-06".into(),
                released_at: 0,
                changelog: None,
                deprecated: false,
            }],
        );

        let state = mgr.get("gpt-4").unwrap();
        assert!(state.update_available().is_none());
    }

    #[test]
    fn latest_detects_update() {
        let mut mgr = VersionManager::new();
        mgr.register("gpt-4".into(), "2024-01".into());
        mgr.set_available_versions(
            "gpt-4",
            vec![ModelVersion {
                model_id: "gpt-4".into(),
                version: "2024-06".into(),
                released_at: 0,
                changelog: None,
                deprecated: false,
            }],
        );

        let state = mgr.get("gpt-4").unwrap();
        assert!(state.update_available().is_some());
    }
}
