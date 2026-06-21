//! Provider Agnostic Adapter Interface (#471)
//!
//! A unified CloudProvider trait that all cloud adapters implement,
//! with a registry for dynamic dispatch.

use crate::traits::EngineProvider;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about a cloud provider adapter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdapterInfo {
    pub id: String,
    pub display_name: String,
    pub base_url: String,
    pub api_key_env: String,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub max_context_tokens: usize,
}

/// Trait that all cloud adapters must implement for registration.
pub trait CloudAdapter: Send + Sync {
    /// Return adapter metadata.
    fn info(&self) -> AdapterInfo;

    /// Return the underlying EngineProvider for dispatch.
    fn provider(&self) -> &dyn EngineProvider;

    /// Check if this adapter is configured (has API key available).
    fn is_configured(&self) -> bool {
        std::env::var(self.info().api_key_env).is_ok()
    }
}

/// Registry of all available cloud adapters.
pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn CloudAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register a cloud adapter.
    pub fn register(&mut self, adapter: Box<dyn CloudAdapter>) {
        let id = adapter.info().id.clone();
        self.adapters.insert(id, adapter);
    }

    /// Get an adapter by ID.
    pub fn get(&self, id: &str) -> Option<&dyn CloudAdapter> {
        self.adapters.get(id).map(|a| a.as_ref())
    }

    /// List all registered adapter infos.
    pub fn list_adapters(&self) -> Vec<AdapterInfo> {
        self.adapters.values().map(|a| a.info()).collect()
    }

    /// List only configured (API key available) adapters.
    pub fn list_configured(&self) -> Vec<AdapterInfo> {
        self.adapters
            .values()
            .filter(|a| a.is_configured())
            .map(|a| a.info())
            .collect()
    }

    /// Get the EngineProvider for a specific adapter.
    pub fn provider(&self, id: &str) -> Option<&dyn EngineProvider> {
        self.adapters.get(id).map(|a| a.provider())
    }

    /// Number of registered adapters.
    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCloudAdapter;

    impl CloudAdapter for MockCloudAdapter {
        fn info(&self) -> AdapterInfo {
            AdapterInfo {
                id: "mock-cloud".into(),
                display_name: "Mock Cloud".into(),
                base_url: "https://mock.example.com".into(),
                api_key_env: "MOCK_API_KEY".into(),
                supports_streaming: true,
                supports_tools: false,
                supports_vision: false,
                max_context_tokens: 4096,
            }
        }

        fn provider(&self) -> &dyn EngineProvider {
            // In real use, this returns the actual provider.
            // For testing we can't return a reference to a temporary.
            unimplemented!()
        }
    }

    #[test]
    fn registry_lists_adapters() {
        let mut registry = AdapterRegistry::new();
        registry.register(Box::new(MockCloudAdapter));
        assert_eq!(registry.len(), 1);
        let infos = registry.list_adapters();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].id, "mock-cloud");
    }
}
