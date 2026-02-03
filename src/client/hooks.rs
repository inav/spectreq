//! Request/response hooks for the Spectre client
//!
//! Hooks allow you to execute custom code before a request is sent
//! or after a response is received. This is useful for:
//! - Logging requests/responses
//! - Modifying headers dynamically
//! - Implementing custom caching logic
//! - Adding authentication tokens
//! - Rate limiting
//! - And more...

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, Request};
use crate::core::SpectreError;
use std::sync::{Arc, RwLock};

/// Request information for pre-request hooks
#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub has_body: bool,
}

impl RequestInfo {
    /// Create a new RequestInfo from a hyper request
    pub fn from_request(req: &Request<Full<Bytes>>) -> Self {
        // For GET/HEAD requests, there's typically no body
        let has_body = !matches!(req.method(), &Method::GET | &Method::HEAD);
        Self {
            method: req.method().clone(),
            url: req.uri().to_string(),
            headers: req
                .headers()
                .iter()
                .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
                .collect(),
            has_body,
        }
    }
}

/// Response information for post-response hooks
#[derive(Debug, Clone)]
pub struct ResponseInfo {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body_size: usize,
    pub url: String,
}

impl ResponseInfo {
    /// Create a new ResponseInfo
    pub fn new(status: u16, headers: Vec<(String, String)>, body_size: usize, url: String) -> Self {
        Self {
            status,
            headers,
            body_size,
            url,
        }
    }
}

/// Pre-request hook type
///
/// This function is called before a request is sent.
/// It can modify the request or return an error to cancel the request.
pub type PreRequestHook =
    Arc<dyn Fn(&mut Request<Full<Bytes>>) -> Result<(), SpectreError> + Send + Sync>;

/// Post-response hook type
///
/// This function is called after a response is received.
/// It can access the response information but cannot modify the response.
pub type PostResponseHook = Arc<dyn Fn(&ResponseInfo) -> Result<(), SpectreError> + Send + Sync>;

/// Hooks container for the Spectre client
#[derive(Clone, Default)]
pub struct Hooks {
    pre_request: Vec<PreRequestHook>,
    post_response: Vec<PostResponseHook>,
}

impl Hooks {
    /// Create a new empty hooks container
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pre-request hook
    pub fn add_pre_request<F>(&mut self, hook: F)
    where
        F: Fn(&mut Request<Full<Bytes>>) -> Result<(), SpectreError> + Send + Sync + 'static,
    {
        self.pre_request.push(Arc::new(hook));
    }

    /// Add a post-response hook
    pub fn add_post_response<F>(&mut self, hook: F)
    where
        F: Fn(&ResponseInfo) -> Result<(), SpectreError> + Send + Sync + 'static,
    {
        self.post_response.push(Arc::new(hook));
    }

    /// Execute all pre-request hooks
    pub fn execute_pre_request(&self, req: &mut Request<Full<Bytes>>) -> Result<(), SpectreError> {
        for hook in &self.pre_request {
            hook(req)?;
        }
        Ok(())
    }

    /// Execute all post-response hooks
    pub fn execute_post_response(&self, info: &ResponseInfo) -> Result<(), SpectreError> {
        for hook in &self.post_response {
            hook(info)?;
        }
        Ok(())
    }

    /// Check if there are any pre-request hooks
    pub fn has_pre_request_hooks(&self) -> bool {
        !self.pre_request.is_empty()
    }

    /// Check if there are any post-response hooks
    pub fn has_post_response_hooks(&self) -> bool {
        !self.post_response.is_empty()
    }

    /// Get the number of pre-request hooks
    pub fn pre_request_count(&self) -> usize {
        self.pre_request.len()
    }

    /// Get the number of post-response hooks
    pub fn post_response_count(&self) -> usize {
        self.post_response.len()
    }

    /// Clear all hooks
    pub fn clear(&mut self) {
        self.pre_request.clear();
        self.post_response.clear();
    }
}

/// Shared hooks container that can be cloned and used across clients
#[derive(Clone, Default)]
pub struct SharedHooks {
    inner: Arc<RwLock<Hooks>>,
}

impl SharedHooks {
    /// Create a new shared hooks container
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pre-request hook
    pub fn add_pre_request<F>(&self, hook: F)
    where
        F: Fn(&mut Request<Full<Bytes>>) -> Result<(), SpectreError> + Send + Sync + 'static,
    {
        let mut inner = self.inner.write().unwrap();
        inner.add_pre_request(hook);
    }

    /// Add a post-response hook
    pub fn add_post_response<F>(&self, hook: F)
    where
        F: Fn(&ResponseInfo) -> Result<(), SpectreError> + Send + Sync + 'static,
    {
        let mut inner = self.inner.write().unwrap();
        inner.add_post_response(hook);
    }

    /// Execute all pre-request hooks
    pub fn execute_pre_request(&self, req: &mut Request<Full<Bytes>>) -> Result<(), SpectreError> {
        let inner = self.inner.read().unwrap();
        inner.execute_pre_request(req)
    }

    /// Execute all post-response hooks
    pub fn execute_post_response(&self, info: &ResponseInfo) -> Result<(), SpectreError> {
        let inner = self.inner.read().unwrap();
        inner.execute_post_response(info)
    }

    /// Clear all hooks
    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_new() {
        let hooks = Hooks::new();
        assert!(!hooks.has_pre_request_hooks());
        assert!(!hooks.has_post_response_hooks());
    }

    #[test]
    fn test_hooks_add_pre_request() {
        let mut hooks = Hooks::new();
        hooks.add_pre_request(|_req| Ok(()));
        assert!(hooks.has_pre_request_hooks());
        assert_eq!(hooks.pre_request_count(), 1);
    }

    #[test]
    fn test_hooks_add_post_response() {
        let mut hooks = Hooks::new();
        hooks.add_post_response(|_info| Ok(()));
        assert!(hooks.has_post_response_hooks());
        assert_eq!(hooks.post_response_count(), 1);
    }

    #[test]
    fn test_hooks_execute_pre_request() {
        let mut hooks = Hooks::new();
        let called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let called_clone = called.clone();
        hooks.add_pre_request(move |_req| {
            *called_clone.lock().unwrap() = true;
            Ok(())
        });

        let mut req = Request::builder()
            .uri("https://example.com")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(hooks.execute_pre_request(&mut req).is_ok());
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_hooks_execute_post_response() {
        let mut hooks = Hooks::new();
        let called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let called_clone = called.clone();
        hooks.add_post_response(move |_info| {
            *called_clone.lock().unwrap() = true;
            Ok(())
        });

        let info = ResponseInfo::new(200, vec![], 0, "https://example.com".to_string());
        assert!(hooks.execute_post_response(&info).is_ok());
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_hooks_clear() {
        let mut hooks = Hooks::new();
        hooks.add_pre_request(|_req| Ok(()));
        hooks.add_post_response(|_info| Ok(()));
        hooks.clear();
        assert!(!hooks.has_pre_request_hooks());
        assert!(!hooks.has_post_response_hooks());
    }

    #[test]
    fn test_request_info_from_request() {
        let req = Request::builder()
            .method("GET")
            .uri("https://example.com/test")
            .header("X-Custom", "value")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let info = RequestInfo::from_request(&req);
        assert_eq!(info.method, Method::GET);
        assert_eq!(info.url, "https://example.com/test");
        assert!(!info.has_body);
    }

    #[test]
    fn test_shared_hooks() {
        let hooks = SharedHooks::new();
        hooks.add_pre_request(|_req| Ok(()));

        let mut req = Request::builder()
            .uri("https://example.com")
            .body(Full::new(Bytes::new()))
            .unwrap();

        assert!(hooks.execute_pre_request(&mut req).is_ok());
    }
}
