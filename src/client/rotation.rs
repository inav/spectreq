//! Smart proxy rotation for Spectre client
//!
//! This module provides automatic proxy rotation with health checking
//! and backoff strategy for failed proxies.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Proxy rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationConfig {
    /// Number of failures before marking a proxy as unhealthy
    pub failure_threshold: usize,
    /// Backoff duration before retrying a failed proxy
    pub backoff_duration: Duration,
    /// Maximum backoff duration
    pub max_backoff_duration: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Health check timeout
    pub health_check_timeout: Duration,
    /// Enable automatic rotation
    pub enabled: bool,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            backoff_duration: Duration::from_secs(60),
            max_backoff_duration: Duration::from_secs(3600),
            health_check_interval: Duration::from_secs(300),
            health_check_timeout: Duration::from_secs(10),
            enabled: true,
        }
    }
}

impl RotationConfig {
    /// Create a new rotation config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the failure threshold
    pub fn failure_threshold(mut self, threshold: usize) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the backoff duration
    pub fn backoff_duration(mut self, duration: Duration) -> Self {
        self.backoff_duration = duration;
        self
    }

    /// Set the maximum backoff duration
    pub fn max_backoff_duration(mut self, duration: Duration) -> Self {
        self.max_backoff_duration = duration;
        self
    }

    /// Set the health check interval
    pub fn health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Set the health check timeout
    pub fn health_check_timeout(mut self, timeout: Duration) -> Self {
        self.health_check_timeout = timeout;
        self
    }

    /// Enable or disable automatic rotation
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Proxy entry with health status
#[derive(Debug, Clone)]
struct ProxyEntry {
    /// Proxy URL
    url: String,
    /// Number of consecutive failures
    failures: usize,
    /// Time when the proxy can be retried
    retry_after: Option<Instant>,
    /// Last health check time
    #[allow(dead_code)]
    last_health_check: Option<Instant>,
    /// Whether the proxy is healthy
    is_healthy: bool,
    /// Total number of successful requests
    success_count: usize,
    /// Total number of failed requests
    total_failures: usize,
}

impl ProxyEntry {
    fn new(url: String) -> Self {
        Self {
            url,
            failures: 0,
            retry_after: None,
            last_health_check: None,
            is_healthy: true,
            success_count: 0,
            total_failures: 0,
        }
    }

    /// Record a successful request
    fn record_success(&mut self) {
        self.failures = 0;
        self.retry_after = None;
        self.is_healthy = true;
        self.success_count += 1;
    }

    /// Record a failed request
    fn record_failure(&mut self, config: &RotationConfig) {
        self.failures += 1;
        self.total_failures += 1;

        if self.failures >= config.failure_threshold {
            self.is_healthy = false;
            // Calculate backoff with exponential increase
            let backoff_secs = config.backoff_duration.as_secs()
                * 2_u64.pow((self.failures - config.failure_threshold) as u32);
            let backoff = Duration::from_secs(backoff_secs).min(config.max_backoff_duration);
            self.retry_after = Some(Instant::now() + backoff);
        }
    }

    /// Check if the proxy can be used
    fn can_use(&self) -> bool {
        if !self.is_healthy {
            if let Some(retry_after) = self.retry_after {
                return Instant::now() >= retry_after;
            }
        }
        true
    }

    /// Get the success rate
    fn success_rate(&self) -> f64 {
        let total = self.success_count + self.total_failures;
        if total == 0 {
            1.0
        } else {
            self.success_count as f64 / total as f64
        }
    }
}

/// Smart proxy rotator
pub struct ProxyRotator {
    /// List of proxies in rotation order
    proxies: Arc<RwLock<VecDeque<ProxyEntry>>>,
    /// Current proxy index
    current_index: Arc<RwLock<usize>>,
    /// Rotation configuration
    config: RotationConfig,
}

impl ProxyRotator {
    /// Create a new proxy rotator with the given configuration
    pub fn new(config: RotationConfig) -> Self {
        Self {
            proxies: Arc::new(RwLock::new(VecDeque::new())),
            current_index: Arc::new(RwLock::new(0)),
            config,
        }
    }

    /// Create a proxy rotator with default configuration
    pub fn with_defaults() -> Self {
        Self::new(RotationConfig::default())
    }

    /// Add a proxy to the rotation list
    pub async fn add_proxy(&self, proxy: String) {
        let mut proxies = self.proxies.write().await;
        proxies.push_back(ProxyEntry::new(proxy));
    }

    /// Add multiple proxies to the rotation list
    pub async fn add_proxies(&self, proxies: Vec<String>) {
        let mut proxy_list = self.proxies.write().await;
        for proxy in proxies {
            proxy_list.push_back(ProxyEntry::new(proxy));
        }
    }

    /// Get the next available proxy
    pub async fn next_proxy(&self) -> Option<String> {
        if !self.config.enabled {
            let proxies = self.proxies.read().await;
            let index = *self.current_index.read().await;
            return proxies.get(index).map(|p| p.url.clone());
        }

        let proxies = self.proxies.read().await;
        if proxies.is_empty() {
            return None;
        }

        let len = proxies.len();
        let start_index = *self.current_index.read().await;

        // Try to find a healthy proxy starting from current index
        for i in 0..len {
            let index = (start_index + i) % len;
            if let Some(proxy_entry) = proxies.get(index) {
                if proxy_entry.can_use() {
                    *self.current_index.write().await = index;
                    return Some(proxy_entry.url.clone());
                }
            }
        }

        // No healthy proxy found, return the current one anyway
        proxies.get(start_index).map(|p| p.url.clone())
    }

    /// Record a successful request for the current proxy
    pub async fn record_success(&self) {
        if !self.config.enabled {
            return;
        }

        let proxies = self.proxies.read().await;
        let index = *self.current_index.read().await;
        if let Some(_proxy_entry) = proxies.get(index) {
            // Need to drop the read lock before getting write lock
            drop(proxies);
            let mut proxies = self.proxies.write().await;
            if let Some(proxy_entry) = proxies.get_mut(index) {
                proxy_entry.record_success();
            }
        }
    }

    /// Record a failed request for the current proxy
    pub async fn record_failure(&self) {
        if !self.config.enabled {
            return;
        }

        let proxies = self.proxies.read().await;
        let index = *self.current_index.read().await;
        if let Some(proxy_entry) = proxies.get(index) {
            // Need to drop the read lock before getting write lock
            let proxy_url = proxy_entry.url.clone();
            drop(proxies);
            let mut proxies = self.proxies.write().await;
            if let Some(proxy_entry) = proxies.get_mut(index) {
                if proxy_entry.url == proxy_url {
                    proxy_entry.record_failure(&self.config);
                }
            }
        }
    }

    /// Get the current proxy
    pub async fn current_proxy(&self) -> Option<String> {
        let proxies = self.proxies.read().await;
        let index = *self.current_index.read().await;
        proxies.get(index).map(|p| p.url.clone())
    }

    /// Get all proxy statuses
    pub async fn proxy_statuses(&self) -> Vec<ProxyStatus> {
        let proxies = self.proxies.read().await;
        proxies
            .iter()
            .map(|p| ProxyStatus {
                url: p.url.clone(),
                is_healthy: p.is_healthy,
                failures: p.failures,
                success_count: p.success_count,
                total_failures: p.total_failures,
                success_rate: p.success_rate(),
                retry_after: p.retry_after,
            })
            .collect()
    }

    /// Remove a proxy from the rotation
    pub async fn remove_proxy(&self, url: &str) -> bool {
        let mut proxies = self.proxies.write().await;
        let original_len = proxies.len();
        proxies.retain(|p| p.url != url);

        // Adjust current index if needed
        let mut index = self.current_index.write().await;
        if *index >= proxies.len() && !proxies.is_empty() {
            *index = 0;
        }

        proxies.len() < original_len
    }

    /// Clear all proxies
    pub async fn clear(&self) {
        let mut proxies = self.proxies.write().await;
        proxies.clear();
        *self.current_index.write().await = 0;
    }

    /// Get the number of proxies
    pub async fn len(&self) -> usize {
        self.proxies.read().await.len()
    }

    /// Check if there are any proxies
    pub async fn is_empty(&self) -> bool {
        self.proxies.read().await.is_empty()
    }

    /// Get the rotation configuration
    pub fn config(&self) -> &RotationConfig {
        &self.config
    }

    /// Manually rotate to the next proxy
    pub async fn rotate(&self) -> Option<String> {
        let len = self.proxies.read().await.len();
        if len == 0 {
            return None;
        }

        let mut index = self.current_index.write().await;
        *index = (*index + 1) % len;
        drop(index);

        self.current_proxy().await
    }

    /// Reset failure counts for all proxies (e.g., after network recovery)
    pub async fn reset_failures(&self) {
        let mut proxies = self.proxies.write().await;
        for proxy in proxies.iter_mut() {
            proxy.failures = 0;
            proxy.retry_after = None;
            proxy.is_healthy = true;
        }
    }
}

/// Proxy status information
#[derive(Debug, Clone)]
pub struct ProxyStatus {
    /// Proxy URL
    pub url: String,
    /// Whether the proxy is healthy
    pub is_healthy: bool,
    /// Number of consecutive failures
    pub failures: usize,
    /// Total successful requests
    pub success_count: usize,
    /// Total failed requests
    pub total_failures: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// When the proxy can be retried (if unhealthy)
    pub retry_after: Option<Instant>,
}

/// Background health check task for the proxy rotator
pub async fn health_check_task(rotator: ProxyRotator) {
    let interval = rotator.config().health_check_interval;
    let mut interval_timer = tokio::time::interval(interval);

    loop {
        interval_timer.tick().await;

        // Collect URLs that need health checking
        let urls_to_check = {
            let proxies = rotator.proxies.read().await;
            let mut urls = Vec::new();
            for proxy in proxies.iter() {
                if !proxy.is_healthy {
                    if let Some(retry_after) = proxy.retry_after {
                        if Instant::now() >= retry_after {
                            urls.push(proxy.url.clone());
                        }
                    }
                }
            }
            urls
        };

        // Health check each URL
        for url in urls_to_check {
            if health_check(&url, rotator.config().health_check_timeout).await {
                rotator.record_success().await;
            } else {
                rotator.record_failure().await;
            }
        }
    }
}

/// Perform a health check on a proxy
async fn health_check(proxy_url: &str, timeout: Duration) -> bool {
    // Parse proxy URL
    let url = match url::Url::parse(proxy_url) {
        Ok(u) => u,
        Err(_) => return false,
    };

    let host = match url.host_str() {
        Some(h) => h,
        None => return false,
    };

    let port = url.port().unwrap_or(match url.scheme() {
        "socks5" | "socks5h" => 1080,
        "http" | "https" => 8080,
        _ => return false,
    });

    // Try to connect to the proxy
    let addr = format!("{}:{}", host, port);
    match tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_rotator() {
        let rotator = ProxyRotator::with_defaults();
        rotator
            .add_proxy("socks5://127.0.0.1:1080".to_string())
            .await;
        rotator
            .add_proxy("socks5://127.0.0.1:1081".to_string())
            .await;

        assert_eq!(rotator.len().await, 2);
        assert!(!rotator.is_empty().await);

        let proxy = rotator.next_proxy().await;
        assert_eq!(proxy, Some("socks5://127.0.0.1:1080".to_string()));
    }

    #[tokio::test]
    async fn test_proxy_rotation() {
        let rotator = ProxyRotator::with_defaults();
        rotator
            .add_proxy("socks5://127.0.0.1:1080".to_string())
            .await;
        rotator
            .add_proxy("socks5://127.0.0.1:1081".to_string())
            .await;

        assert_eq!(
            rotator.current_proxy().await,
            Some("socks5://127.0.0.1:1080".to_string())
        );

        rotator.rotate().await;
        assert_eq!(
            rotator.current_proxy().await,
            Some("socks5://127.0.0.1:1081".to_string())
        );

        rotator.rotate().await;
        assert_eq!(
            rotator.current_proxy().await,
            Some("socks5://127.0.0.1:1080".to_string())
        );
    }

    #[tokio::test]
    async fn test_proxy_status() {
        let rotator = ProxyRotator::with_defaults();
        rotator
            .add_proxy("socks5://127.0.0.1:1080".to_string())
            .await;

        let statuses = rotator.proxy_statuses().await;
        assert_eq!(statuses.len(), 1);
        assert!(statuses[0].is_healthy);
        assert_eq!(statuses[0].failures, 0);
    }

    #[tokio::test]
    async fn test_remove_proxy() {
        let rotator = ProxyRotator::with_defaults();
        rotator
            .add_proxy("socks5://127.0.0.1:1080".to_string())
            .await;
        rotator
            .add_proxy("socks5://127.0.0.1:1081".to_string())
            .await;

        assert!(rotator.remove_proxy("socks5://127.0.0.1:1081").await);
        assert_eq!(rotator.len().await, 1);
    }

    #[test]
    fn test_rotation_config() {
        let config = RotationConfig::new()
            .failure_threshold(5)
            .backoff_duration(Duration::from_secs(120));

        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.backoff_duration, Duration::from_secs(120));
    }
}
