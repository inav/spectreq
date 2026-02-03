//! Middleware chain for Spectre client
//!
//! This module provides a composable middleware system for request/response processing
//! with common middlewares like rate limiting, logging, caching, retry, and circuit breaker.

use crate::core::{Result, SpectreError};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub use bytes::Bytes;
/// HTTP request and response types for middleware
pub use http::{Method, Request, Response, StatusCode, Uri};
pub use http_body_util::{BodyExt, Full};

/// Middleware context
#[derive(Debug, Clone)]
pub struct MiddlewareContext {
    /// Start time of the request
    pub start_time: Instant,
    /// Request method
    pub method: Method,
    /// Request URL
    pub url: String,
    /// Custom data for middleware communication
    pub data: Arc<RwLock<std::collections::HashMap<String, String>>>,
}

impl MiddlewareContext {
    /// Create a new middleware context
    pub fn new(method: Method, url: String) -> Self {
        Self {
            start_time: Instant::now(),
            method,
            url,
            data: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get the elapsed time since the request started
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Set custom data
    pub async fn set_data(&self, key: &str, value: &str) {
        let mut data = self.data.write().await;
        data.insert(key.to_string(), value.to_string());
    }

    /// Get custom data
    pub async fn get_data(&self, key: &str) -> Option<String> {
        let data = self.data.read().await;
        data.get(key).cloned()
    }
}

/// Middleware trait for processing requests and responses
pub trait Middleware: Send + Sync {
    /// Process the request before sending
    fn process_request(
        &self,
        _req: &mut Request<Full<Bytes>>,
        _ctx: &MiddlewareContext,
    ) -> Result<()> {
        Ok(())
    }

    /// Process the response after receiving
    fn process_response(
        &self,
        _resp: &mut Response<Full<Bytes>>,
        _ctx: &MiddlewareContext,
    ) -> Result<()> {
        Ok(())
    }

    /// Handle errors during request processing
    fn process_error(&self, _error: &mut SpectreError, _ctx: &MiddlewareContext) -> Result<()> {
        Ok(())
    }
}

/// Next middleware in the chain
pub struct Next<'a> {
    index: usize,
    middleware: &'a [Arc<dyn Middleware>],
}

impl<'a> Next<'a> {
    #[allow(dead_code)]
    fn new(index: usize, middleware: &'a [Arc<dyn Middleware>]) -> Self {
        Self { index, middleware }
    }

    /// Call the next middleware in the chain
    pub async fn call(
        &self,
        req: &mut Request<Full<Bytes>>,
        ctx: &MiddlewareContext,
    ) -> Result<()> {
        if let Some(middleware) = self.middleware.get(self.index) {
            middleware.process_request(req, ctx)?;
        }
        Ok(())
    }
}

/// Rate limiting middleware
#[derive(Debug, Clone)]
pub struct RateLimiter {
    max_requests: usize,
    window: Duration,
    requests: Arc<RwLock<Vec<Instant>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    ///
    /// * `max_requests` - Maximum number of requests allowed in the window
    /// * `window` - Time window for rate limiting
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            requests: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Clean up old timestamps outside the current window
    async fn cleanup_old_requests(&self) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        requests.retain(|&ts| now.duration_since(ts) < self.window);
    }

    /// Check if a request is allowed
    pub async fn check_rate_limit(&self) -> bool {
        self.cleanup_old_requests().await;
        let requests = self.requests.read().await;
        requests.len() < self.max_requests
    }

    /// Record a request
    pub async fn record_request(&self) {
        let mut requests = self.requests.write().await;
        requests.push(Instant::now());
    }
}

impl Middleware for RateLimiter {
    fn process_request(
        &self,
        _req: &mut Request<Full<Bytes>>,
        _ctx: &MiddlewareContext,
    ) -> Result<()> {
        // Note: This is a synchronous check
        // In async context, use the async methods directly
        if !self
            .requests
            .try_read()
            .is_ok_and(|r| r.len() < self.max_requests)
        {
            return Err(SpectreError::Http("Rate limit exceeded".to_string()));
        }
        Ok(())
    }
}

/// Request logging middleware
#[derive(Debug, Clone)]
pub struct RequestLogger {
    log_headers: bool,
    log_body: bool,
}

impl RequestLogger {
    /// Create a new request logger
    ///
    /// # Arguments
    ///
    /// * `log_headers` - Whether to log request headers
    /// * `log_body` - Whether to log request body
    pub fn new(log_headers: bool, log_body: bool) -> Self {
        Self {
            log_headers,
            log_body,
        }
    }
}

impl Middleware for RequestLogger {
    fn process_request(
        &self,
        req: &mut Request<Full<Bytes>>,
        _ctx: &MiddlewareContext,
    ) -> Result<()> {
        println!("[Request] {} {}", req.method(), req.uri());

        if self.log_headers {
            for (name, value) in req.headers().iter() {
                println!("  {}: {}", name, value.to_str().unwrap_or(""));
            }
        }

        if self.log_body {
            println!("  Body: <present>");
        }

        Ok(())
    }

    fn process_response(
        &self,
        resp: &mut Response<Full<Bytes>>,
        ctx: &MiddlewareContext,
    ) -> Result<()> {
        println!(
            "[Response] Status: {} (took {:?})",
            resp.status(),
            ctx.elapsed()
        );
        Ok(())
    }
}

/// Retry middleware
#[derive(Debug, Clone)]
pub struct RetryMiddleware {
    max_retries: u32,
    retry_on_status: Vec<u16>,
    wait_min: Duration,
    wait_max: Duration,
}

impl RetryMiddleware {
    /// Create a new retry middleware
    ///
    /// # Arguments
    ///
    /// * `max_retries` - Maximum number of retry attempts
    /// * `retry_on_status` - HTTP status codes that trigger a retry
    /// * `wait_min` - Minimum wait time before retry
    /// * `wait_max` - Maximum wait time before retry
    pub fn new(
        max_retries: u32,
        retry_on_status: Vec<u16>,
        wait_min: Duration,
        wait_max: Duration,
    ) -> Self {
        Self {
            max_retries,
            retry_on_status,
            wait_min,
            wait_max,
        }
    }

    /// Check if a response should be retried
    pub fn should_retry(&self, status: u16, attempt: u32) -> bool {
        attempt < self.max_retries && self.retry_on_status.contains(&status)
    }

    /// Calculate wait time with exponential backoff
    pub fn calculate_wait(&self, attempt: u32) -> Duration {
        let base = self.wait_min.as_millis() as f64;
        let delay = base * 2_f64.powi(attempt as i32);
        let delay = delay.min(self.wait_max.as_millis() as f64);
        Duration::from_millis(delay as u64)
    }
}

impl Middleware for RetryMiddleware {
    fn process_response(
        &self,
        _resp: &mut Response<Full<Bytes>>,
        _ctx: &MiddlewareContext,
    ) -> Result<()> {
        // The actual retry logic is handled by the client
        // This middleware just tracks whether a retry should happen
        Ok(())
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker middleware
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<usize>>,
    failure_threshold: usize,
    timeout: Duration,
    last_failure: Arc<RwLock<Option<Instant>>>,
    success_threshold: usize,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    ///
    /// # Arguments
    ///
    /// * `failure_threshold` - Number of failures before opening the circuit
    /// * `timeout` - How long to wait before trying again (half-open state)
    /// * `success_threshold` - Number of successes needed to close the circuit
    pub fn new(failure_threshold: usize, timeout: Duration, success_threshold: usize) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            failure_threshold,
            timeout,
            last_failure: Arc::new(RwLock::new(None)),
            success_threshold,
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;
        let last_failure = self.last_failure.write().await;

        match *state {
            CircuitState::Closed => {
                *failure_count = 0;
            }
            CircuitState::HalfOpen => {
                // Check if we have enough successes to close the circuit
                let successes = self.success_threshold.saturating_sub(*failure_count);
                if successes >= self.success_threshold {
                    *state = CircuitState::Closed;
                    *failure_count = 0;
                }
            }
            CircuitState::Open => {
                // Stay open until timeout expires
                if let Some(failure_time) = *last_failure {
                    if Instant::now().duration_since(failure_time) >= self.timeout {
                        *state = CircuitState::HalfOpen;
                    }
                }
            }
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;
        let mut last_failure = self.last_failure.write().await;

        *failure_count += 1;
        *last_failure = Some(Instant::now());

        if *failure_count >= self.failure_threshold {
            *state = CircuitState::Open;
        }
    }

    /// Check if the circuit allows requests
    pub async fn is_closed(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has expired
                let last_failure = self.last_failure.read().await;
                if let Some(failure_time) = *last_failure {
                    if Instant::now().duration_since(failure_time) >= self.timeout {
                        // Transition to half-open (done by record_success)
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Get the current circuit state
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Reset the circuit breaker to closed state
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;
        *state = CircuitState::Closed;
        *failure_count = 0;
    }
}

impl Middleware for CircuitBreaker {
    fn process_request(
        &self,
        _req: &mut Request<Full<Bytes>>,
        _ctx: &MiddlewareContext,
    ) -> Result<()> {
        // Note: This is a synchronous check
        // In async context, use the async methods directly
        if !self
            .state
            .try_read()
            .map_or(true, |s| *s == CircuitState::Closed)
        {
            return Err(SpectreError::Http("Circuit breaker is open".to_string()));
        }
        Ok(())
    }
}

/// Middleware chain
#[derive(Clone)]
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    /// Create a new middleware chain
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add a middleware to the chain
    pub fn add_middleware(&mut self, middleware: Arc<dyn Middleware>) -> &mut Self {
        self.middlewares.push(middleware);
        self
    }

    /// Get all middlewares
    pub fn middlewares(&self) -> &[Arc<dyn Middleware>] {
        &self.middlewares
    }

    /// Process a request through all middlewares
    pub async fn process_request(
        &self,
        req: &mut Request<Full<Bytes>>,
        ctx: &MiddlewareContext,
    ) -> Result<()> {
        for middleware in &self.middlewares {
            middleware.process_request(req, ctx)?;
        }
        Ok(())
    }

    /// Process a response through all middlewares
    pub async fn process_response(
        &self,
        resp: &mut Response<Full<Bytes>>,
        ctx: &MiddlewareContext,
    ) -> Result<()> {
        for middleware in self.middlewares.iter().rev() {
            middleware.process_response(resp, ctx)?;
        }
        Ok(())
    }

    /// Process an error through all middlewares
    pub async fn process_error(
        &self,
        error: &mut SpectreError,
        ctx: &MiddlewareContext,
    ) -> Result<()> {
        for middleware in self.middlewares.iter().rev() {
            middleware.process_error(error, ctx)?;
        }
        Ok(())
    }

    /// Check if there are any middlewares
    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating middleware chains
pub struct MiddlewareChainBuilder {
    chain: MiddlewareChain,
}

impl MiddlewareChainBuilder {
    /// Create a new middleware chain builder
    pub fn new() -> Self {
        Self {
            chain: MiddlewareChain::new(),
        }
    }

    /// Add a middleware to the chain
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.chain.add_middleware(middleware);
        self
    }

    /// Add a rate limiter
    pub fn rate_limiter(self, max_requests: usize, window: Duration) -> Self {
        self.add(Arc::new(RateLimiter::new(max_requests, window)))
    }

    /// Add a request logger
    pub fn logger(self, log_headers: bool, log_body: bool) -> Self {
        self.add(Arc::new(RequestLogger::new(log_headers, log_body)))
    }

    /// Add a circuit breaker
    pub fn circuit_breaker(self, failure_threshold: usize, timeout: Duration) -> Self {
        self.add(Arc::new(CircuitBreaker::new(failure_threshold, timeout, 2)))
    }

    /// Build the middleware chain
    pub fn build(self) -> MiddlewareChain {
        self.chain
    }
}

impl Default for MiddlewareChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));
        assert!(limiter.requests.try_read().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_rate_limiter_async() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.check_rate_limit().await);
        assert!(limiter.check_rate_limit().await);
        limiter.record_request().await;
        limiter.record_request().await;
        assert!(!limiter.check_rate_limit().await);
    }

    #[test]
    fn test_circuit_breaker() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(30), 2);
        // CircuitBreaker::new creates a new circuit breaker in Closed state
        // We can't check state synchronously without async, so we just verify creation
        assert_eq!(cb.failure_threshold, 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker_async() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100), 1);

        assert!(cb.is_closed().await);

        cb.record_failure().await;
        assert!(cb.is_closed().await);

        cb.record_failure().await;
        assert!(!cb.is_closed().await); // Should be open now

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(cb.is_closed().await); // Should be half-open now

        cb.record_success().await;
        assert!(cb.is_closed().await); // Should be closed now
    }

    #[test]
    fn test_middleware_chain() {
        let chain = MiddlewareChainBuilder::new().logger(true, false).build();

        assert!(!chain.is_empty());
        assert_eq!(chain.middlewares().len(), 1);
    }

    #[test]
    fn test_middleware_context() {
        let ctx = MiddlewareContext::new(Method::GET, "https://example.com".to_string());
        assert_eq!(ctx.method, Method::GET);
        assert_eq!(ctx.url, "https://example.com");
    }
}
