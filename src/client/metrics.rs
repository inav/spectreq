//! Metrics collection for Spectre client
//!
//! This module provides built-in request/response metrics collection for observability.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Request/response metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Status code
    pub status: u16,
    /// Request method
    pub method: String,
    /// Request URL
    pub url: String,
    /// Response time
    pub response_time_ms: u64,
    /// DNS lookup time in microseconds
    pub dns_time_us: u64,
    /// TCP connect time in microseconds
    pub tcp_time_us: u64,
    /// TLS handshake time in microseconds
    pub tls_time_us: u64,
    /// Time to first byte (TTFB) in microseconds
    pub ttfb_us: u64,
    /// Request size in bytes
    pub request_size: usize,
    /// Response size in bytes
    pub response_size: usize,
    /// Whether response was from cache
    pub from_cache: bool,
    /// Number of retries
    pub retries: usize,
    /// Timestamp
    pub timestamp: u64,
}

impl RequestMetrics {
    /// Create a new request metrics object
    pub fn new() -> Self {
        Self {
            status: 0,
            method: String::new(),
            url: String::new(),
            response_time_ms: 0,
            dns_time_us: 0,
            tcp_time_us: 0,
            tls_time_us: 0,
            ttfb_us: 0,
            request_size: 0,
            response_size: 0,
            from_cache: false,
            retries: 0,
            timestamp: Instant::now().duration_since(Instant::now()).as_micros() as u64,
        }
    }

    /// Get the success rate (1xx, 2xx, 3xx are different categories)
    pub fn success(&self) -> bool {
        self.status >= 200 && self.status < 400
    }

    /// Get the status class
    pub fn status_class(&self) -> &'static str {
        match self.status {
            100..=199 => "1xx",
            200..=299 => "2xx",
            300..=399 => "3xx",
            400..=499 => "4xx",
            500..=599 => "5xx",
            _ => "unknown",
        }
    }
}

impl Default for RequestMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregate metrics statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStats {
    /// Total number of requests
    pub total_requests: u64,
    /// Number of successful requests (2xx, 3xx)
    pub successful_requests: u64,
    /// Number of failed requests (4xx, 5xx)
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Average TTFB in microseconds
    pub avg_ttfb_us: f64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Requests by status code
    pub status_codes: HashMap<u16, u64>,
    /// Requests by method
    pub methods: HashMap<String, u64>,
}

impl Default for MetricsStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            avg_ttfb_us: 0.0,
            bytes_sent: 0,
            bytes_received: 0,
            cache_hit_rate: 0.0,
            status_codes: HashMap::new(),
            methods: HashMap::new(),
        }
    }
}

/// Metrics collector for tracking request/response metrics
pub struct MetricsCollector {
    /// All recorded metrics
    metrics: Arc<RwLock<Vec<RequestMetrics>>>,
    /// Aggregated statistics
    stats: Arc<RwLock<MetricsStats>>,
    /// Maximum number of metrics to keep
    max_metrics: usize,
    /// Counters for atomic operations
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    failed_requests: Arc<AtomicU64>,
    total_response_time_ms: Arc<AtomicU64>,
    total_ttfb_us: Arc<AtomicU64>,
    total_bytes_sent: Arc<AtomicU64>,
    total_bytes_received: Arc<AtomicU64>,
    cache_hits: Arc<AtomicU64>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(MetricsStats::default())),
            max_metrics: 10000,
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            total_response_time_ms: Arc::new(AtomicU64::new(0)),
            total_ttfb_us: Arc::new(AtomicU64::new(0)),
            total_bytes_sent: Arc::new(AtomicU64::new(0)),
            total_bytes_received: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create a new metrics collector with a max metrics limit
    pub fn with_max_metrics(max: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(MetricsStats::default())),
            max_metrics: max,
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            total_response_time_ms: Arc::new(AtomicU64::new(0)),
            total_ttfb_us: Arc::new(AtomicU64::new(0)),
            total_bytes_sent: Arc::new(AtomicU64::new(0)),
            total_bytes_received: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a request/response
    pub async fn record(&self, metrics: RequestMetrics) {
        // Update atomic counters
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        if metrics.success() {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        self.total_response_time_ms
            .fetch_add(metrics.response_time_ms, Ordering::Relaxed);
        self.total_ttfb_us
            .fetch_add(metrics.ttfb_us, Ordering::Relaxed);
        self.total_bytes_sent
            .fetch_add(metrics.request_size as u64, Ordering::Relaxed);
        self.total_bytes_received
            .fetch_add(metrics.response_size as u64, Ordering::Relaxed);

        if metrics.from_cache {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        }

        // Store metrics
        {
            let mut metrics_store = self.metrics.write().await;
            if metrics_store.len() >= self.max_metrics {
                metrics_store.remove(0);
            }
            metrics_store.push(metrics);
        } // Drop write lock before calling update_stats

        // Update aggregated stats (needs read lock on metrics)
        self.update_stats().await;
    }

    /// Update aggregated statistics
    async fn update_stats(&self) {
        let metrics = self.metrics.read().await;
        let total = self.total_requests.load(Ordering::Relaxed);

        if total == 0 {
            return;
        }

        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_time = self.total_response_time_ms.load(Ordering::Relaxed);
        let total_ttfb = self.total_ttfb_us.load(Ordering::Relaxed);
        let bytes_sent = self.total_bytes_sent.load(Ordering::Relaxed);
        let bytes_received = self.total_bytes_received.load(Ordering::Relaxed);
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);

        let mut status_codes = HashMap::new();
        let mut methods = HashMap::new();

        for m in metrics.iter() {
            *status_codes.entry(m.status).or_insert(0) += 1;
            *methods.entry(m.method.clone()).or_insert(0) += 1;
        }

        let stats = MetricsStats {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            avg_response_time_ms: total_time as f64 / total as f64,
            avg_ttfb_us: total_ttfb as f64 / total as f64,
            bytes_sent,
            bytes_received,
            cache_hit_rate: cache_hits as f64 / total as f64,
            status_codes,
            methods,
        };

        *self.stats.write().await = stats;
    }

    /// Get the aggregated statistics
    pub async fn stats(&self) -> MetricsStats {
        self.stats.read().await.clone()
    }

    /// Get all recorded metrics
    pub async fn all_metrics(&self) -> Vec<RequestMetrics> {
        self.metrics.read().await.clone()
    }

    /// Get the last N metrics
    pub async fn last_metrics(&self, n: usize) -> Vec<RequestMetrics> {
        let metrics = self.metrics.read().await;
        let start = if metrics.len() > n {
            metrics.len() - n
        } else {
            0
        };
        metrics[start..].to_vec()
    }

    /// Clear all metrics
    pub async fn clear(&self) {
        self.metrics.write().await.clear();
        self.reset_counters();
        // Directly set stats to default since update_stats would early-return
        *self.stats.write().await = MetricsStats::default();
    }

    /// Reset all counters
    fn reset_counters(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.total_response_time_ms.store(0, Ordering::Relaxed);
        self.total_ttfb_us.store(0, Ordering::Relaxed);
        self.total_bytes_sent.store(0, Ordering::Relaxed);
        self.total_bytes_received.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
    }

    /// Get metrics by host
    pub async fn metrics_by_host(&self, host: &str) -> Vec<RequestMetrics> {
        let metrics = self.metrics.read().await;
        metrics
            .iter()
            .filter(|m| m.url.contains(host))
            .cloned()
            .collect()
    }

    /// Get metrics by status code
    pub async fn metrics_by_status(&self, status: u16) -> Vec<RequestMetrics> {
        let metrics = self.metrics.read().await;
        metrics
            .iter()
            .filter(|m| m.status == status)
            .cloned()
            .collect()
    }

    /// Get metrics for the last N minutes
    pub async fn metrics_since(&self, duration: Duration) -> Vec<RequestMetrics> {
        let metrics = self.metrics.read().await;
        let cutoff = Instant::now() - duration;
        metrics
            .iter()
            .filter(|m| {
                let timestamp = m.timestamp;
                let now = Instant::now()
                    .duration_since(Instant::now())
                    .saturating_sub(Duration::from_micros(timestamp));
                cutoff.elapsed() >= now
            })
            .cloned()
            .collect()
    }

    /// Get performance percentiles
    pub async fn percentiles(&self) -> MetricsPercentiles {
        let metrics = self.metrics.read().await;
        if metrics.is_empty() {
            return MetricsPercentiles::default();
        }

        let mut response_times: Vec<u64> = metrics.iter().map(|m| m.response_time_ms).collect();
        let mut ttfb_times: Vec<u64> = metrics.iter().map(|m| m.ttfb_us).collect();

        response_times.sort();
        ttfb_times.sort();

        MetricsPercentiles {
            p50_response_time_ms: percentile(&response_times, 0.50),
            p90_response_time_ms: percentile(&response_times, 0.90),
            p95_response_time_ms: percentile(&response_times, 0.95),
            p99_response_time_ms: percentile(&response_times, 0.99),
            p50_ttfb_us: percentile(&ttfb_times, 0.50),
            p90_ttfb_us: percentile(&ttfb_times, 0.90),
            p95_ttfb_us: percentile(&ttfb_times, 0.95),
            p99_ttfb_us: percentile(&ttfb_times, 0.99),
        }
    }

    /// Get the number of recorded metrics
    pub async fn len(&self) -> usize {
        self.metrics.read().await.len()
    }

    /// Check if there are any metrics
    pub async fn is_empty(&self) -> bool {
        self.metrics.read().await.is_empty()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics percentiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsPercentiles {
    /// 50th percentile response time (median)
    pub p50_response_time_ms: u64,
    /// 90th percentile response time
    pub p90_response_time_ms: u64,
    /// 95th percentile response time
    pub p95_response_time_ms: u64,
    /// 99th percentile response time
    pub p99_response_time_ms: u64,
    /// 50th percentile TTFB
    pub p50_ttfb_us: u64,
    /// 90th percentile TTFB
    pub p90_ttfb_us: u64,
    /// 95th percentile TTFB
    pub p95_ttfb_us: u64,
    /// 99th percentile TTFB
    pub p99_ttfb_us: u64,
}

impl Default for MetricsPercentiles {
    fn default() -> Self {
        Self {
            p50_response_time_ms: 0,
            p90_response_time_ms: 0,
            p95_response_time_ms: 0,
            p99_response_time_ms: 0,
            p50_ttfb_us: 0,
            p90_ttfb_us: 0,
            p95_ttfb_us: 0,
            p99_ttfb_us: 0,
        }
    }
}

/// Calculate percentile from a sorted array
fn percentile(sorted_data: &[u64], percentile: f64) -> u64 {
    if sorted_data.is_empty() {
        return 0;
    }

    let index = ((sorted_data.len() - 1) as f64 * percentile).round() as usize;
    sorted_data[index]
}

/// Timer for measuring request duration
pub struct RequestTimer {
    start: Instant,
    dns_start: Option<Instant>,
    dns_end: Option<Instant>,
    tcp_start: Option<Instant>,
    tcp_end: Option<Instant>,
    tls_start: Option<Instant>,
    tls_end: Option<Instant>,
    ttfb_start: Option<Instant>,
    ttfb_end: Option<Instant>,
}

impl RequestTimer {
    /// Create a new request timer
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            dns_start: None,
            dns_end: None,
            tcp_start: None,
            tcp_end: None,
            tls_start: None,
            tls_end: None,
            ttfb_start: None,
            ttfb_end: None,
        }
    }

    /// Start DNS lookup timing
    pub fn start_dns(&mut self) {
        self.dns_start = Some(Instant::now());
    }

    /// End DNS lookup timing
    pub fn end_dns(&mut self) -> Duration {
        self.dns_end = Some(Instant::now());
        self.dns_start.map_or(Duration::ZERO, |start| {
            self.dns_end
                .map_or(Duration::ZERO, |end| end.duration_since(start))
        })
    }

    /// Start TCP connection timing
    pub fn start_tcp(&mut self) {
        self.tcp_start = Some(Instant::now());
    }

    /// End TCP connection timing
    pub fn end_tcp(&mut self) -> Duration {
        self.tcp_end = Some(Instant::now());
        self.tcp_start.map_or(Duration::ZERO, |start| {
            self.tcp_end
                .map_or(Duration::ZERO, |end| end.duration_since(start))
        })
    }

    /// Start TLS handshake timing
    pub fn start_tls(&mut self) {
        self.tls_start = Some(Instant::now());
    }

    /// End TLS handshake timing
    pub fn end_tls(&mut self) -> Duration {
        self.tls_end = Some(Instant::now());
        self.tls_start.map_or(Duration::ZERO, |start| {
            self.tls_end
                .map_or(Duration::ZERO, |end| end.duration_since(start))
        })
    }

    /// Start TTFB timing
    pub fn start_ttfb(&mut self) {
        self.ttfb_start = Some(Instant::now());
    }

    /// End TTFB timing
    pub fn end_ttfb(&mut self) -> Duration {
        self.ttfb_end = Some(Instant::now());
        self.ttfb_start.map_or(Duration::ZERO, |start| {
            self.ttfb_end
                .map_or(Duration::ZERO, |end| end.duration_since(start))
        })
    }

    /// Get the total elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get DNS lookup duration
    ///
    /// Returns the time taken for DNS resolution, or `Duration::ZERO` if not measured.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut timer = RequestTimer::new();
    /// timer.start_dns();
    /// // ... DNS lookup happens ...
    /// timer.end_dns();
    ///
    /// println!("DNS lookup took: {:?}", timer.dns_duration());
    /// ```
    pub fn dns_duration(&self) -> Duration {
        self.dns_start.map_or(Duration::ZERO, |start| {
            self.dns_end
                .map_or(Duration::ZERO, |end| end.duration_since(start))
        })
    }

    /// Get TCP connection duration
    ///
    /// Returns the time taken for TCP connection, or `Duration::ZERO` if not measured.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut timer = RequestTimer::new();
    /// timer.start_tcp();
    /// // ... TCP connection happens ...
    /// timer.end_tcp();
    ///
    /// println!("TCP connect took: {:?}", timer.tcp_duration());
    /// ```
    pub fn tcp_duration(&self) -> Duration {
        self.tcp_start.map_or(Duration::ZERO, |start| {
            self.tcp_end
                .map_or(Duration::ZERO, |end| end.duration_since(start))
        })
    }

    /// Get TLS handshake duration
    ///
    /// Returns the time taken for TLS handshake, or `Duration::ZERO` if not measured.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut timer = RequestTimer::new();
    /// timer.start_tls();
    /// // ... TLS handshake happens ...
    /// timer.end_tls();
    ///
    /// println!("TLS handshake took: {:?}", timer.tls_duration());
    /// ```
    pub fn tls_duration(&self) -> Duration {
        self.tls_start.map_or(Duration::ZERO, |start| {
            self.tls_end
                .map_or(Duration::ZERO, |end| end.duration_since(start))
        })
    }

    /// Get Time To First Byte (TTFB) duration
    ///
    /// Returns the time from request sent to first byte received, or `Duration::ZERO` if not measured.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut timer = RequestTimer::new();
    /// timer.start_ttfb();
    /// // ... waiting for first byte ...
    /// timer.end_ttfb();
    ///
    /// println!("TTFB: {:?}", timer.ttfb_duration());
    /// ```
    pub fn ttfb_duration(&self) -> Duration {
        self.ttfb_start.map_or(Duration::ZERO, |start| {
            self.ttfb_end
                .map_or(Duration::ZERO, |end| end.duration_since(start))
        })
    }

    /// Build request metrics from the timer data
    pub fn build_metrics(
        &self,
        method: &str,
        url: &str,
        status: u16,
        request_size: usize,
        response_size: usize,
        from_cache: bool,
        retries: usize,
    ) -> RequestMetrics {
        RequestMetrics {
            status,
            method: method.to_string(),
            url: url.to_string(),
            response_time_ms: self.elapsed().as_millis() as u64,
            dns_time_us: self
                .dns_start
                .map_or(Duration::ZERO, |start| {
                    self.dns_end
                        .map_or(Duration::ZERO, |end| end.duration_since(start))
                })
                .as_micros() as u64,
            tcp_time_us: self
                .tcp_start
                .map_or(Duration::ZERO, |start| {
                    self.tcp_end
                        .map_or(Duration::ZERO, |end| end.duration_since(start))
                })
                .as_micros() as u64,
            tls_time_us: self
                .tls_start
                .map_or(Duration::ZERO, |start| {
                    self.tls_end
                        .map_or(Duration::ZERO, |end| end.duration_since(start))
                })
                .as_micros() as u64,
            ttfb_us: self
                .ttfb_start
                .map_or(Duration::ZERO, |start| {
                    self.ttfb_end
                        .map_or(Duration::ZERO, |end| end.duration_since(start))
                })
                .as_micros() as u64,
            request_size,
            response_size,
            from_cache,
            retries,
            timestamp: Instant::now().duration_since(self.start).as_micros() as u64,
        }
    }
}

impl Default for RequestTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Background metrics reporting task
pub async fn metrics_reporting_task(
    collector: MetricsCollector,
    interval: Duration,
    reporter: impl Fn(MetricsStats) + Send + Sync + 'static,
) {
    let mut interval_timer = tokio::time::interval(interval);
    loop {
        interval_timer.tick().await;
        let stats = collector.stats().await;
        reporter(stats);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        let mut metrics = RequestMetrics::new();
        metrics.status = 200;
        metrics.method = "GET".to_string();
        metrics.url = "https://example.com".to_string();
        metrics.response_time_ms = 100;
        metrics.ttfb_us = 50000;

        collector.record(metrics).await;

        let stats = collector.stats().await;
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_request_timer() {
        let mut timer = RequestTimer::new();
        timer.start_dns();
        std::thread::sleep(Duration::from_millis(10));
        let dns_duration = timer.end_dns();
        assert!(dns_duration.as_millis() >= 10);
    }

    #[tokio::test]
    async fn test_metrics_percentiles() {
        let collector = MetricsCollector::new();

        // Add some test metrics
        for i in 0..10 {
            let mut metrics = RequestMetrics::new();
            metrics.status = 200;
            metrics.response_time_ms = i * 100;
            metrics.ttfb_us = i * 1000;
            collector.record(metrics).await;
        }

        let percentiles = collector.percentiles().await;
        assert!(percentiles.p50_response_time_ms > 0);
        assert!(percentiles.p95_response_time_ms > 0);
    }

    #[tokio::test]
    async fn test_metrics_clear() {
        let collector = MetricsCollector::new();

        let mut metrics = RequestMetrics::new();
        metrics.status = 200;
        collector.record(metrics).await;
        assert_eq!(collector.len().await, 1);

        collector.clear().await;
        assert_eq!(collector.len().await, 0);
        assert_eq!(collector.stats().await.total_requests, 0);
    }
}
