//! Image caching with LRU eviction.
//!
//! The image cache stores decoded `peniko::ImageData` objects keyed by a
//! user-provided identifier (typically a file path or URL). When the cache
//! exceeds its capacity, the least-recently-used entries are evicted to
//! reclaim memory.

use std::collections::HashMap;

/// Key used to look up cached images. Typically a file path or URL.
pub type ImageKey = String;

/// A slot in the LRU cache tracking access order.
struct CacheEntry {
    /// Decoded image data ready for rendering.
    data: peniko::ImageData,
    /// Monotonically increasing access counter for LRU ordering.
    last_access: u64,
}

/// LRU image cache that bounds GPU-side memory usage.
///
/// The cache stores decoded `peniko::ImageData` entries. Vello internally
/// manages the GPU textures — the cache simply avoids re-decoding the same
/// image on every frame.
///
/// # Memory management
///
/// The cache tracks the total byte size of all stored images and evicts
/// least-recently-used entries when `max_bytes` would be exceeded.
///
/// # Async loading
///
/// The cache itself is synchronous. For non-blocking image loading, decode
/// images on a background thread and insert them via
/// [`ImageCache::insert`] when ready.
pub struct ImageCache {
    entries: HashMap<ImageKey, CacheEntry>,
    access_counter: u64,
    max_bytes: usize,
    current_bytes: usize,
}

impl ImageCache {
    /// Creates a new image cache with the given memory budget (in bytes).
    ///
    /// A reasonable default for a document editor is 256 MiB.
    pub fn new(max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_counter: 0,
            max_bytes,
            current_bytes: 0,
        }
    }

    /// Creates a cache with a 256 MiB default budget.
    pub fn default_capacity() -> Self {
        Self::new(256 * 1024 * 1024)
    }

    /// Returns the cached image for `key`, if present.
    ///
    /// Accessing an image updates its LRU timestamp so that frequently
    /// used images survive eviction.
    pub fn get(&mut self, key: &str) -> Option<&peniko::ImageData> {
        let counter = &mut self.access_counter;
        let entry = self.entries.get_mut(key)?;
        *counter += 1;
        entry.last_access = *counter;
        Some(&entry.data)
    }

    /// Inserts a decoded image into the cache.
    ///
    /// If the insertion would exceed `max_bytes`, LRU entries are evicted
    /// until there is enough room. If `key` already exists, the old entry
    /// is replaced.
    pub fn insert(&mut self, key: ImageKey, data: peniko::ImageData) {
        let entry_bytes = Self::image_byte_size(&data);

        // If the key already exists, remove the old entry first.
        if let Some(old) = self.entries.remove(&key) {
            self.current_bytes -= Self::image_byte_size(&old.data);
        }

        // Evict LRU entries until we have room.
        while self.current_bytes + entry_bytes > self.max_bytes && !self.entries.is_empty() {
            self.evict_lru();
        }

        // If a single image is larger than the budget, still insert it
        // (the cache will be at or slightly over budget).
        self.access_counter += 1;
        self.current_bytes += entry_bytes;
        self.entries.insert(
            key,
            CacheEntry {
                data,
                last_access: self.access_counter,
            },
        );
    }

    /// Returns `true` if the cache contains an entry for `key`.
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Removes a specific entry from the cache.
    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(entry) = self.entries.remove(key) {
            self.current_bytes -= Self::image_byte_size(&entry.data);
            true
        } else {
            false
        }
    }

    /// Returns the number of cached images.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the total bytes consumed by cached images.
    pub fn current_bytes(&self) -> usize {
        self.current_bytes
    }

    /// Clears all cached images.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_bytes = 0;
    }

    /// Evicts the least-recently-used entry.
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.last_access)
            .map(|(k, _)| k.clone())
        {
            if let Some(entry) = self.entries.remove(&lru_key) {
                self.current_bytes -= Self::image_byte_size(&entry.data);
            }
        }
    }

    /// Computes the byte size of an `ImageData`.
    ///
    /// Assumes RGBA8 format (4 bytes per pixel).
    fn image_byte_size(data: &peniko::ImageData) -> usize {
        (data.width as usize) * (data.height as usize) * 4
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::default_capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image(w: u32, h: u32) -> peniko::ImageData {
        let pixels = vec![0u8; (w * h * 4) as usize];
        peniko::ImageData {
            data: pixels.into(),
            format: peniko::ImageFormat::Rgba8,
            alpha_type: peniko::ImageAlphaType::AlphaPremultiplied,
            width: w,
            height: h,
        }
    }

    #[test]
    fn insert_and_get() {
        let mut cache = ImageCache::new(1024 * 1024);
        let img = make_test_image(10, 10);
        cache.insert("test".into(), img);
        assert!(cache.contains("test"));
        assert!(cache.get("test").is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn lru_eviction() {
        // 400 bytes budget, each image is 100 bytes (10x10 * 4)
        let mut cache = ImageCache::new(400);
        for i in 0..5 {
            cache.insert(format!("img{i}"), make_test_image(10, 10));
        }
        // Should have evicted some entries to stay under budget.
        assert!(cache.current_bytes() <= 400);
    }

    #[test]
    fn remove_entry() {
        let mut cache = ImageCache::new(1024 * 1024);
        cache.insert("test".into(), make_test_image(10, 10));
        assert!(cache.remove("test"));
        assert!(!cache.contains("test"));
    }

    #[test]
    fn clear_empties_cache() {
        let mut cache = ImageCache::new(1024 * 1024);
        cache.insert("a".into(), make_test_image(10, 10));
        cache.insert("b".into(), make_test_image(10, 10));
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.current_bytes(), 0);
    }
}
