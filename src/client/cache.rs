//! HTTP response caching
//!
//! This module provides an in-memory cache for HTTP responses with
//! support for ETag and Last-Modified validation.
//!
//! # Examples
//!
//! ```rust,ignore
//! use crate::client::Cache;
//!
//! let cache = Cache::new();
//!
//! // The cache is automatically used by Client
//! // but can also be used directly if needed
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Cache entry for HTTP responses
///
/// Represents a cached HTTP response with metadata for freshness validation.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// ETag value
    pub etag: Option<String>,
    /// Last-Modified value
    pub last_modified: Option<String>,
    /// Cached response body
    pub body: Option<Vec<u8>>,
    /// Cache-control max-age
    pub max_age: Option<Duration>,
    /// When this entry was cached
    pub cached_at: SystemTime,
    /// Content type
    pub content_type: Option<String>,
}

impl CacheEntry {
    /// Check if this cache entry is still fresh
    pub fn is_fresh(&self) -> bool {
        if let Some(max_age) = self.max_age {
            self.cached_at
                .elapsed()
                .is_ok_and(|elapsed| elapsed < max_age)
        } else {
            // Default cache time: 5 minutes
            self.cached_at
                .elapsed()
                .is_ok_and(|elapsed| elapsed < Duration::from_secs(300))
        }
    }

    /// Create a new cache entry from response metadata
    pub fn from_response(
        etag: Option<String>,
        last_modified: Option<String>,
        body: Option<Vec<u8>>,
        cache_control: Option<&str>,
        content_type: Option<String>,
    ) -> Self {
        let max_age = cache_control.and_then(|cc| {
            cc.split(',')
                .find_map(|part| {
                    let part = part.trim();
                    if let Some(stripped) = part.strip_prefix("max-age=") {
                        stripped.parse().ok()
                    } else {
                        None
                    }
                })
                .map(Duration::from_secs)
        });

        Self {
            etag,
            last_modified,
            body,
            max_age,
            cached_at: SystemTime::now(),
            content_type,
        }
    }
}

/// In-memory cache for HTTP responses
#[derive(Debug, Clone)]
pub struct Cache {
    inner: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Generate a cache key from URL and method
    pub fn cache_key(url: &str, method: &str) -> String {
        format!("{}:{}", method.to_uppercase(), url)
    }

    /// Get a cache entry
    pub fn get(&self, url: &str, method: &str) -> Option<CacheEntry> {
        let key = Self::cache_key(url, method);
        let inner = self.inner.lock().ok()?;
        inner.get(&key).filter(|e| e.is_fresh()).cloned()
    }

    /// Put a cache entry
    #[allow(clippy::too_many_arguments)]
    pub fn put(
        &self,
        url: &str,
        method: &str,
        etag: Option<String>,
        last_modified: Option<String>,
        body: Option<Vec<u8>>,
        cache_control: Option<&str>,
        content_type: Option<String>,
    ) {
        let key = Self::cache_key(url, method);
        let entry =
            CacheEntry::from_response(etag, last_modified, body, cache_control, content_type);

        if let Ok(mut inner) = self.inner.lock() {
            inner.insert(key, entry);
        }
    }

    /// Remove a cache entry
    pub fn remove(&self, url: &str, method: &str) {
        let key = Self::cache_key(url, method);
        if let Ok(mut inner) = self.inner.lock() {
            inner.remove(&key);
        }
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.clear();
        }
    }

    /// Get the ETag for a cached URL
    pub fn get_etag(&self, url: &str, method: &str) -> Option<String> {
        self.get(url, method)?.etag
    }

    /// Get the Last-Modified for a cached URL
    pub fn get_last_modified(&self, url: &str, method: &str) -> Option<String> {
        self.get(url, method)?.last_modified
    }

    /// Get the cached body for a URL
    pub fn get_body(&self, url: &str, method: &str) -> Option<Vec<u8>> {
        self.get(url, method)?.body
    }

    /// Check if a URL is cached and fresh
    pub fn is_cached(&self, url: &str, method: &str) -> bool {
        self.get(url, method).is_some()
    }

    /// Remove stale entries from the cache
    pub fn cleanup(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.retain(|_, entry| entry.is_fresh());
        }
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.inner.lock().map(|inner| inner.len()).unwrap_or(0)
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key() {
        assert_eq!(
            Cache::cache_key("https://example.com", "GET"),
            "GET:https://example.com"
        );
    }

    #[test]
    fn test_cache_put_get() {
        let cache = Cache::new();
        cache.put(
            "https://example.com",
            "GET",
            Some("\"12345\"".to_string()),
            Some("Wed, 21 Oct 2015 07:28:00 GMT".to_string()),
            Some(b"hello".to_vec()),
            None,
            Some("text/html".to_string()),
        );

        let entry = cache.get("https://example.com", "GET");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().etag, Some("\"12345\"".to_string()));
    }

    #[test]
    fn test_cache_remove() {
        let cache = Cache::new();
        cache.put(
            "https://example.com",
            "GET",
            Some("\"12345\"".to_string()),
            None,
            None,
            None,
            None,
        );

        assert!(cache.is_cached("https://example.com", "GET"));
        cache.remove("https://example.com", "GET");
        assert!(!cache.is_cached("https://example.com", "GET"));
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::new();
        cache.put(
            "https://example.com",
            "GET",
            Some("\"12345\"".to_string()),
            None,
            None,
            None,
            None,
        );
        cache.put(
            "https://example.org",
            "GET",
            Some("\"67890\"".to_string()),
            None,
            None,
            None,
            None,
        );

        assert_eq!(cache.len(), 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_entry_is_fresh() {
        let entry = CacheEntry::from_response(
            Some("\"12345\"".to_string()),
            None,
            None,
            Some("max-age=3600"),
            None,
        );
        assert!(entry.is_fresh());
    }

    #[test]
    fn test_get_etag() {
        let cache = Cache::new();
        cache.put(
            "https://example.com",
            "GET",
            Some("\"abc123\"".to_string()),
            None,
            None,
            None,
            None,
        );

        assert_eq!(
            cache.get_etag("https://example.com", "GET"),
            Some("\"abc123\"".to_string())
        );
    }
}
