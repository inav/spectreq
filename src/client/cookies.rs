//! HTTP cookie management
//!
//! This module provides a thread-safe cookie jar that automatically
//! handles cookie storage and inclusion in requests.
//!
//! # Examples
//!
//! ```rust,ignore
//! use crate::client::CookieJar;
//!
//! let jar = CookieJar::new();
//! // Automatically managed by Client
//! ```

use cookie_store::{CookieStore, RawCookie};
use std::sync::{Arc, Mutex};
use url::Url;

/// Thread-safe cookie jar
///
/// Stores HTTP cookies and automatically includes them in requests
/// to matching domains. Implements standard cookie handling including
/// path, secure, and domain attributes.
#[derive(Debug, Clone)]
pub struct CookieJar {
    inner: Arc<Mutex<CookieStore>>,
}

impl Default for CookieJar {
    fn default() -> Self {
        Self::new()
    }
}

impl CookieJar {
    /// Create a new empty cookie jar
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(CookieStore::new())),
        }
    }

    /// Add cookies from a Set-Cookie header
    pub fn set_cookies(&self, cookie_str: &[&str], url: &Url) {
        if let Ok(mut jar) = self.inner.lock() {
            let parsed_url = url::Url::parse(url.as_str()).unwrap_or_else(|_| url.clone());
            for cookie in cookie_str {
                if let Ok(raw_cookie) = RawCookie::parse(*cookie) {
                    let _ = jar.insert_raw(&raw_cookie, &parsed_url);
                }
            }
        }
    }

    /// Get cookies for a URL as a Cookie header value
    pub fn get_cookie_value(&self, url: &Url) -> Option<String> {
        self.inner.lock().ok().and_then(|jar| {
            let values: Vec<_> = jar
                .get_request_values(url)
                .map(|(name, value)| format!("{}={}", name, value))
                .collect();
            if values.is_empty() {
                None
            } else {
                Some(values.join("; "))
            }
        })
    }

    /// Clear all cookies
    pub fn clear(&self) {
        if let Ok(mut jar) = self.inner.lock() {
            *jar = CookieStore::new();
        }
    }

    /// Get the number of cookies in the jar
    pub fn len(&self) -> usize {
        self.inner
            .lock()
            .map(|jar| jar.iter_any().count())
            .unwrap_or(0)
    }

    /// Check if the jar is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove cookies for a specific domain
    pub fn remove_for_domain(&self, domain: &str) {
        if let Ok(mut jar) = self.inner.lock() {
            // Create a new jar with cookies not matching the domain
            let new_store = CookieStore::new();
            let old_store = std::mem::replace(&mut *jar, new_store);

            for cookie in old_store.iter_any() {
                if cookie.domain() != Some(domain) {
                    let url = if let Some(cookie_domain) = cookie.domain() {
                        let scheme = if cookie.secure().unwrap_or(false) {
                            "https"
                        } else {
                            "http"
                        };
                        Url::parse(&format!("{}://{}/", scheme, cookie_domain))
                    } else {
                        Url::parse("http://example.com/")
                    };

                    if let Ok(url) = url {
                        let raw = RawCookie::from(cookie.clone());
                        let _ = jar.insert_raw(&raw, &url);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_jar_new() {
        let jar = CookieJar::new();
        assert_eq!(jar.len(), 0);
        assert!(jar.is_empty());
    }

    #[test]
    fn test_set_get_cookies() {
        let jar = CookieJar::new();
        let url = Url::parse("https://example.com").unwrap();

        jar.set_cookies(&["session=abc123; Path=/; Secure"], &url);

        let cookie_value = jar.get_cookie_value(&url);
        assert_eq!(cookie_value, Some("session=abc123".to_string()));
    }

    #[test]
    fn test_clear_cookies() {
        let jar = CookieJar::new();
        let url = Url::parse("https://example.com").unwrap();

        jar.set_cookies(&["session=abc123"], &url);
        assert_eq!(jar.len(), 1);

        jar.clear();
        assert_eq!(jar.len(), 0);
    }

    #[test]
    fn test_multiple_cookies() {
        let jar = CookieJar::new();
        let url = Url::parse("https://example.com").unwrap();

        jar.set_cookies(
            &[
                "session=abc123; Path=/",
                "user=john; Domain=example.com",
                "theme=dark",
            ],
            &url,
        );

        let cookie_value = jar.get_cookie_value(&url);
        assert!(cookie_value.is_some());
        assert!(cookie_value.unwrap().contains("session=abc123"));
    }
}
