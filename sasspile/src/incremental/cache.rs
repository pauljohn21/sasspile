//! Cache layer — source-span indexed value cache.
//!
//! Caches evaluated results by `SourceSpan` so that identical source ranges
//! with identical environments reuse previously computed values via `Arc`.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use tracing::{debug, instrument};

use crate::source::SourceSpan;
use crate::value::Value;

/// Cache key combining a source span and environment fingerprint.
#[derive(Debug, Clone)]
struct CacheKey {
    span: SourceSpan,
    /// Simple fingerprint of the referencing environment
    /// (variable values at evaluation time).
    env_fp: u64,
}

impl CacheKey {
    fn new(span: SourceSpan, env_fp: u64) -> Self {
        Self { span, env_fp }
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.span == other.span && self.env_fp == other.env_fp
    }
}

impl Eq for CacheKey {}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.span.start.hash(state);
        self.span.end.hash(state);
        self.env_fp.hash(state);
    }
}

/// Cache of evaluated values indexed by source span.
///
/// When the same span is evaluated in the same environment, returns
/// the cached `Arc<Value>` instead of recomputing.
#[derive(Debug, Default)]
pub struct SpanCache {
    /// Map from cache key to cached value.
    entries: HashMap<CacheKey, Arc<Value>>,
    /// Tracks hit/miss statistics.
    hits: u64,
    misses: u64,
}

impl SpanCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a cached value for the given span and environment fingerprint.
    #[instrument(skip(self))]
    pub fn get(&mut self, span: &SourceSpan, env_fp: u64) -> Option<Arc<Value>> {
        let key = CacheKey::new(*span, env_fp);
        if let Some(val) = self.entries.get(&key) {
            self.hits += 1;
            debug!(hit = true, hits = self.hits, "cache lookup");
            Some(Arc::clone(val))
        } else {
            self.misses += 1;
            debug!(hit = false, misses = self.misses, "cache lookup");
            None
        }
    }

    /// Insert a computed value into the cache.
    #[instrument(skip(self, value))]
    pub fn put(&mut self, span: SourceSpan, env_fp: u64, value: Value) {
        let key = CacheKey::new(span, env_fp);
        self.entries.insert(key, Arc::new(value));
        debug!(cache_size = self.entries.len(), "cache insert");
    }

    /// Invalidate cache entries that overlap with the given span range.
    ///
    /// Called when source editing changes a region — any cached result
    /// whose span is contained in the edit range is stale.
    #[instrument(skip(self))]
    pub fn invalidate_range(&mut self, start: u32, end: u32) {
        let old_len = self.entries.len();
        self.entries.retain(|key, _| {
            let s = &key.span;
            // Remove if span overlaps with [start, end).
            !(s.start < end && s.end > start)
        });
        let removed = old_len - self.entries.len();
        debug!(removed, remaining = self.entries.len(), "invalidated cache range");
    }

    /// Clear all cache entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
        debug!("cache cleared");
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Cache hit count.
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Cache miss count.
    pub fn misses(&self) -> u64 {
        self.misses
    }

    /// Total lookups (hits + misses).
    pub fn total_lookups(&self) -> u64 {
        self.hits + self.misses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_put_and_get() {
        let mut cache = SpanCache::new();
        let span = SourceSpan::new(0, 10);
        let value = Value::Number(crate::value::Number::new(42.0, crate::value::Unit::None));

        cache.put(span, 0, value.clone());
        let result = cache.get(&span, 0);

        assert!(result.is_some());
        assert_eq!(*result.unwrap(), value);
        assert_eq!(cache.hits(), 1);
    }

    #[test]
    fn cache_miss() {
        let mut cache = SpanCache::new();
        let span = SourceSpan::new(0, 10);

        let result = cache.get(&span, 0);
        assert!(result.is_none());
        assert_eq!(cache.misses(), 1);
    }

    #[test]
    fn cache_invalidate_range() {
        let mut cache = SpanCache::new();
        let span_a = SourceSpan::new(0, 10);
        let span_b = SourceSpan::new(20, 30);

        cache.put(span_a, 0, Value::Boolean(true));
        cache.put(span_b, 0, Value::Boolean(false));
        assert_eq!(cache.len(), 2);

        // Invalidate only span_a's range.
        cache.invalidate_range(0, 10);
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&span_a, 0).is_none());
        assert!(cache.get(&span_b, 0).is_some());
    }

    #[test]
    fn cache_env_fingerprint_affects_key() {
        let mut cache = SpanCache::new();
        let span = SourceSpan::new(0, 10);

        cache.put(span, 100, Value::Number(crate::value::Number::new(1.0, crate::value::Unit::None)));
        cache.put(span, 200, Value::Number(crate::value::Number::new(2.0, crate::value::Unit::None)));

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn cache_clear() {
        let mut cache = SpanCache::new();
        cache.put(SourceSpan::new(0, 10), 0, Value::Null);
        cache.put(SourceSpan::new(10, 20), 0, Value::Null);

        assert_eq!(cache.len(), 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
    }
}
