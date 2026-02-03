//! Spectre Client - HTTP client with browser impersonation
//!
//! This library provides an HTTP client that mimics real browser fingerprints
//! for anti-bot detection evasion. It builds on `spectreq::core` to provide
//! a full-featured HTTP client with caching, cookies, and more.

pub mod auth;
pub mod cache;
#[allow(clippy::module_inception)]
pub mod client;
pub mod compression;
pub mod connector;
pub mod cookies;
pub mod hooks;
pub mod http3;
pub mod metrics;
pub mod middleware;
pub mod pinning;
pub mod pool;
pub mod rotation;
pub mod session;
pub mod socks5;
pub mod streaming;

pub use auth::{extract_nonce, extract_opaque, extract_qop, extract_realm};
pub use auth::{
    AuthConfig, BasicAuth, BasicAuthCache, BearerToken, BearerTokenManager, DigestAuth, NtlmAuth,
};
pub use cache::Cache;
pub use client::{
    Client, HttpResponse, HttpVersion, RedirectConfig, RequestTiming, RetryConfig, TimeoutConfig,
};
pub use compression::{CompressionType, Decompress};
pub use cookies::CookieJar;
pub use hooks::{Hooks, RequestInfo, ResponseInfo, SharedHooks};
#[cfg(feature = "http3")]
pub use http3::Http3Response;
pub use http3::{enable_http3, supports_http3};
pub use metrics::{
    metrics_reporting_task, MetricsCollector, MetricsPercentiles, MetricsStats, RequestMetrics,
    RequestTimer,
};
pub use middleware::{CircuitBreaker, CircuitState, RateLimiter, RequestLogger};
pub use middleware::{Middleware, MiddlewareChain, MiddlewareChainBuilder, MiddlewareContext};
pub use pinning::{
    export_pins_to_json, generate_pin_from_cert, load_pins_from_json, CertPinner, PinError,
};
pub use pool::{ConnectionPool, PoolConfig, PoolStats, PooledConnection};
pub use rotation::{ProxyRotator, ProxyStatus, RotationConfig};
pub use session::{ProfileData, SerializedSession, SessionError, SessionManager};
pub use socks5::{Socks5Config, Socks5Connector, Socks5DnsResolve};
pub use streaming::{streaming_response_from_bytes, SliceReader, StreamingResponse};
