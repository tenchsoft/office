//! Model Registry & Discovery (#463)
//!
//! Central registry of all known models across providers,
//! with metadata lookup and capability filtering.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use tench_shared_types::{ModelDescriptor, ModelLocation};

/// Extended metadata about a model beyond the basic descriptor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelMeta {
    pub descriptor: ModelDescriptor,
    pub context_window: usize,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub cost_per_million_input: Option<f64>,
    pub cost_per_million_output: Option<f64>,
    pub tags: Vec<String>,
}

impl ModelMeta {
    pub fn from_descriptor(d: ModelDescriptor) -> Self {
        Self {
            descriptor: d,
            context_window: 4096,
            supports_streaming: true,
            supports_tools: false,
            supports_vision: false,
            cost_per_million_input: None,
            cost_per_million_output: None,
            tags: Vec::new(),
        }
    }
}

/// The model registry stores metadata for all known models.
pub struct ModelRegistry {
    models: RwLock<HashMap<String, ModelMeta>>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: RwLock::new(HashMap::new()),
        }
    }

    /// Register a model with extended metadata.
    pub fn register(&self, meta: ModelMeta) {
        let mut models = self
            .models
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        models.insert(meta.descriptor.id.clone(), meta);
    }

    /// Register a basic ModelDescriptor with default metadata.
    pub fn register_descriptor(&self, d: ModelDescriptor) {
        self.register(ModelMeta::from_descriptor(d));
    }

    /// Look up a model by ID.
    pub fn get(&self, model_id: &str) -> Option<ModelMeta> {
        self.models
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(model_id)
            .cloned()
    }

    /// List all registered models.
    pub fn list_all(&self) -> Vec<ModelMeta> {
        self.models
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .cloned()
            .collect()
    }

    /// List models for a specific provider.
    pub fn list_by_provider(&self, provider: &str) -> Vec<ModelMeta> {
        self.models
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .filter(|m| m.descriptor.provider == provider)
            .cloned()
            .collect()
    }

    /// List models by location (Local, Cloud, etc.).
    pub fn list_by_location(&self, location: ModelLocation) -> Vec<ModelMeta> {
        self.models
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .filter(|m| m.descriptor.location == location)
            .cloned()
            .collect()
    }

    /// Find models that support a specific capability tag.
    pub fn find_by_tag(&self, tag: &str) -> Vec<ModelMeta> {
        self.models
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .filter(|m| m.tags.iter().any(|t| t == tag))
            .cloned()
            .collect()
    }

    /// Remove a model from the registry.
    pub fn unregister(&self, model_id: &str) {
        self.models
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(model_id);
    }

    /// Number of registered models.
    pub fn len(&self) -> usize {
        self.models
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.models
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty()
    }

    /// Bulk-register models from a provider's list.
    pub fn register_from_provider(&self, models: Vec<ModelDescriptor>) {
        for d in models {
            self.register_descriptor(d);
        }
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_descriptor(id: &str, provider: &str) -> ModelDescriptor {
        ModelDescriptor {
            id: id.to_string(),
            display_name: id.to_string(),
            provider: provider.to_string(),
            capability: "chat".to_string(),
            location: ModelLocation::Cloud,
        }
    }

    #[test]
    fn register_and_lookup() {
        let registry = ModelRegistry::new();
        let d = test_descriptor("gpt-4", "openai");
        registry.register_descriptor(d);
        assert!(registry.get("gpt-4").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn list_by_provider_filters() {
        let registry = ModelRegistry::new();
        registry.register_descriptor(test_descriptor("gpt-4", "openai"));
        registry.register_descriptor(test_descriptor("claude-3", "anthropic"));
        registry.register_descriptor(test_descriptor("gpt-3.5", "openai"));

        let openai = registry.list_by_provider("openai");
        assert_eq!(openai.len(), 2);
        let anthropic = registry.list_by_provider("anthropic");
        assert_eq!(anthropic.len(), 1);
    }

    #[test]
    fn find_by_tag() {
        let registry = ModelRegistry::new();
        let mut meta = ModelMeta::from_descriptor(test_descriptor("gpt-4-vision", "openai"));
        meta.tags.push("vision".to_string());
        registry.register(meta);
        registry.register_descriptor(test_descriptor("gpt-3.5", "openai"));

        let vision = registry.find_by_tag("vision");
        assert_eq!(vision.len(), 1);
        assert_eq!(vision[0].descriptor.id, "gpt-4-vision");
    }
}
