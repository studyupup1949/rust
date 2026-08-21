//! In-memory cache for storing key-value pairs.
//!
//! Uses moka's high-performance concurrent cache implementation.

use moka::sync::Cache;

/// Thread-safe in-memory cache with configurable capacity.
///
/// Used for storing:
/// - Environment variables (`MemCache<String, String>`)
/// - Node outputs (`MemCache<NodeId, Vars>`)
///
/// The cache is backed by moka, which provides:
/// - Thread-safe concurrent access
/// - LRU eviction when capacity is exceeded
#[derive(Clone)]
pub struct MemCache<K, V> {
    variables: Cache<K, V>,
}

impl<K, V> MemCache<K, V>
where
    K: std::hash::Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Allocate a new [`MemCache`].
    pub fn new(capacity: usize) -> Self {
        Self {
            variables: Cache::new(capacity as u64),
        }
    }

    /// Set a global variable.
    pub fn set(
        &self,
        key: K,
        value: V,
    ) {
        self.variables.insert(key, value);
    }

    /// Get environment variables through key `&K`.
    pub fn get(
        &self,
        key: &K,
    ) -> Option<V> {
        self.variables.get(key)
    }

    /// Remove environment variables through key `&K`.
    pub fn remove(
        &self,
        key: &K,
    ) {
        self.variables.remove(key);
    }

    /// Return an iterator over the entries of the cache.
    pub fn iter(&self) -> moka::sync::Iter<'_, K, V> {
        self.variables.iter()
    }
}
