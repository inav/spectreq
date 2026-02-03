//! Spectre - HTTP client with browser impersonation
//!
//! This library provides an HTTP client that mimics real browser fingerprints
//! for anti-bot detection evasion.
//!
//! # Features
//!
//! - **Browser Profiles**: Pre-configured profiles for Chrome, Firefox, Safari, and Edge
//! - **TLS Fingerprinting**: JA4 TLS fingerprint support with customizable cipher suites
//! - **HTTP/2 Settings**: Browser-specific HTTP/2 parameters
//! - **TCP Configuration**: Socket options for fingerprint accuracy
//! - **Client Hints**: Sec-CH-UA-* header generation
//! - **Fetch Headers**: Sec-Fetch-* header generation
//! - **HTTP Caching**: In-memory cache with ETag and Last-Modified validation
//! - **Cookie Management**: Automatic cookie storage and inclusion in requests
//! - **Proxy Support**: HTTP CONNECT and SOCKS5 proxy support
//! - **Certificate Pinning**: SPKI hash-based certificate verification
//! - **Session Persistence**: Save/load sessions with cookies and TLS tickets
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use spectreq::{Client, Profile};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client with Chrome 143 profile
//!     let client = Client::new(Profile::chrome_143_windows()).await?;
//!
//!     // Make a GET request
//!     let response = client.get("https://httpbin.org/get").await?;
//!     println!("Status: {}", response.status);
//!     println!("Body: {}", response.text()?);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Modules
//!
//! - [`core`] - Core types, profiles, TLS, errors
//! - [`client`] - HTTP client with all features

pub mod client;
pub mod core;

// Note: Python bindings are in the spectreq-py crate, not in the main crate

// Re-export core types
pub use core::{
    apply_tcp_options, build_tls_config, calculate_ja4, calculate_ja4_from_handshake,
    create_tcp_socket, domain_supports_ech, fetch_ech_configs, generate_client_hints,
    generate_sec_fetch_headers, get_ja4_components, hashmap_to_ordered, merge_ordered_headers,
    parse_ech_config, supports_post_quantum, BrowserName, ClientHints, EchConfig, EchFetchResult,
    FetchDest, FetchMode, FetchSite, Headers, Http2Settings, Ja4Components, Ja4Fingerprint,
    Ja4RawComponents, Ja4hRawComponents, OrderedHeaders, Profile, RequestContext, Result,
    SpectreError, TcpConfig, TlsConfig, OS,
};

// Re-export client types
pub use client::{
    enable_http3, export_pins_to_json, extract_nonce, extract_opaque, extract_qop, extract_realm,
    generate_pin_from_cert, load_pins_from_json, metrics_reporting_task,
    streaming_response_from_bytes, supports_http3, AuthConfig, BasicAuth, BasicAuthCache,
    BearerToken, BearerTokenManager, Cache, CertPinner, CircuitBreaker, CircuitState, Client,
    CompressionType, ConnectionPool, CookieJar, Decompress, DigestAuth, Hooks, HttpResponse,
    HttpVersion, MetricsCollector, MetricsPercentiles, MetricsStats, Middleware, MiddlewareChain,
    MiddlewareChainBuilder, MiddlewareContext, NtlmAuth, PinError, PoolConfig, PoolStats,
    PooledConnection, ProfileData, ProxyRotator, ProxyStatus, RateLimiter, RedirectConfig,
    RequestInfo, RequestLogger, RequestMetrics, RequestTimer, RequestTiming, ResponseInfo,
    RetryConfig, RotationConfig, SerializedSession, SessionError, SessionManager, SharedHooks,
    SliceReader, Socks5Config, Socks5Connector, Socks5DnsResolve, StreamingResponse, TimeoutConfig,
};

// HTTP/3 support is feature-gated
// HTTP/3 support is feature-gated
#[cfg(feature = "http3")]
pub use client::Http3Response;

pub use http::Method;
