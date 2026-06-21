use crate::config::CacheConfig;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;

/// A cached response entry with TTL.
#[derive(Clone, Debug)]
pub struct CacheEntry {
    #[allow(dead_code)]
    pub key: String,
    pub response: String,
    pub created_at: Instant,
    pub ttl: Duration,
    pub hit_count: u32,
    pub model: String,
    pub token_count: u32,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

// CacheConfig is defined in config.rs

/// Hash-based response cache with TTL and LRU eviction.
#[derive(Clone, Debug)]
pub struct ResponseCache {
    entries: Arc<Mutex<HashMap<String, CacheEntry>>>,
    config: CacheConfig,
}

impl ResponseCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Generate a cache key from provider, model, and prompt.
    pub fn make_key(provider: &str, model: &str, prompt: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        provider.hash(&mut hasher);
        model.hash(&mut hasher);
        prompt.hash(&mut hasher);
        format!("{provider}:{model}:{:016x}", hasher.finish())
    }

    /// Get a cached response if it exists and is not expired.
    pub fn get(&self, key: &str) -> Option<String> {
        if !self.config.enabled {
            return None;
        }
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired() {
                entries.remove(key);
                return None;
            }
            entry.hit_count += 1;
            return Some(entry.response.clone());
        }
        None
    }

    /// Store a response in the cache.
    pub fn put(&self, key: String, response: String, model: String, token_count: u32) {
        if !self.config.enabled {
            return;
        }
        if response.len() > self.config.max_entry_size_bytes {
            return;
        }
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Evict expired entries first
        let expired_keys: Vec<String> = entries
            .iter()
            .filter(|(_, v)| v.is_expired())
            .map(|(k, _)| k.clone())
            .collect();
        for k in expired_keys {
            entries.remove(&k);
        }

        // LRU eviction if at capacity
        while entries.len() >= self.config.max_entries {
            if let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, v)| v.created_at)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest_key);
            }
        }

        entries.insert(
            key.clone(),
            CacheEntry {
                key,
                response,
                created_at: Instant::now(),
                ttl: Duration::from_secs(self.config.default_ttl_secs),
                hit_count: 0,
                model,
                token_count,
            },
        );
    }

    /// Store with a custom TTL.
    pub fn put_with_ttl(
        &self,
        key: String,
        response: String,
        model: String,
        token_count: u32,
        ttl: Duration,
    ) {
        if !self.config.enabled {
            return;
        }
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        entries.insert(
            key.clone(),
            CacheEntry {
                key,
                response,
                created_at: Instant::now(),
                ttl,
                hit_count: 0,
                model,
                token_count,
            },
        );
    }

    /// Invalidate a specific cache entry.
    pub fn invalidate(&self, key: &str) -> bool {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(key)
            .is_some()
    }

    /// Invalidate all entries for a given model.
    pub fn invalidate_model(&self, model: &str) -> usize {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let keys_to_remove: Vec<String> = entries
            .iter()
            .filter(|(_, v)| v.model == model)
            .map(|(k, _)| k.clone())
            .collect();
        let count = keys_to_remove.len();
        for k in keys_to_remove {
            entries.remove(&k);
        }
        count
    }

    /// Clear all cache entries.
    pub fn clear(&self) {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let total = entries.len();
        let expired = entries.values().filter(|e| e.is_expired()).count();
        let total_hits: u32 = entries.values().map(|e| e.hit_count).sum();
        let total_tokens: u32 = entries.values().map(|e| e.token_count).sum();
        let total_size: usize = entries.values().map(|e| e.response.len()).sum();
        CacheStats {
            entry_count: total,
            expired_count: expired,
            total_hits,
            total_tokens,
            total_size_bytes: total_size,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CacheStats {
    pub entry_count: usize,
    pub expired_count: usize,
    pub total_hits: u32,
    pub total_tokens: u32,
    pub total_size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_put_and_get() {
        let cache = ResponseCache::with_defaults();
        let key = ResponseCache::make_key("openai", "gpt-4", "hello");
        cache.put(key.clone(), "world".into(), "gpt-4".into(), 10);
        assert_eq!(cache.get(&key), Some("world".into()));
    }

    #[test]
    fn cache_expired_returns_none() {
        let cache = ResponseCache::new(CacheConfig {
            default_ttl_secs: 0,
            ..Default::default()
        });
        let key = "test-key".to_string();
        cache.put_with_ttl(
            key.clone(),
            "value".into(),
            "model".into(),
            5,
            Duration::from_millis(1),
        );
        std::thread::sleep(Duration::from_millis(5));
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn cache_lru_eviction() {
        let cache = ResponseCache::new(CacheConfig {
            max_entries: 2,
            ..Default::default()
        });
        cache.put("k1".into(), "v1".into(), "m".into(), 1);
        cache.put("k2".into(), "v2".into(), "m".into(), 1);
        cache.put("k3".into(), "v3".into(), "m".into(), 1);
        assert!(cache.get("k1").is_none()); // evicted
        assert_eq!(cache.get("k2"), Some("v2".into()));
        assert_eq!(cache.get("k3"), Some("v3".into()));
    }

    #[test]
    fn cache_invalidate_model() {
        let cache = ResponseCache::with_defaults();
        cache.put("k1".into(), "v1".into(), "gpt-4".into(), 1);
        cache.put("k2".into(), "v2".into(), "claude".into(), 1);
        let removed = cache.invalidate_model("gpt-4");
        assert_eq!(removed, 1);
        assert!(cache.get("k1").is_none());
        assert_eq!(cache.get("k2"), Some("v2".into()));
    }

    #[test]
    fn cache_key_deterministic() {
        let k1 = ResponseCache::make_key("openai", "gpt-4", "hello");
        let k2 = ResponseCache::make_key("openai", "gpt-4", "hello");
        let k3 = ResponseCache::make_key("openai", "gpt-4", "world");
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }
}
