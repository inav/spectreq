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

pub mod core;
pub mod client;

// Note: Python bindings are in the spectreq-py crate, not in the main crate

// Re-export core types
pub use core::{
    Profile, BrowserName, OS, SpectreError, Result,
    Headers, OrderedHeaders, ClientHints,
    FetchDest, FetchMode, FetchSite, RequestContext,
    generate_client_hints, generate_sec_fetch_headers,
    hashmap_to_ordered, merge_ordered_headers,
    TcpConfig, Http2Settings, TlsConfig,
    apply_tcp_options, create_tcp_socket,
    build_tls_config, get_ja4_components, supports_post_quantum, Ja4Components,
    calculate_ja4, calculate_ja4_from_handshake, Ja4Fingerprint, Ja4RawComponents, Ja4hRawComponents,
    domain_supports_ech, fetch_ech_configs, parse_ech_config, EchConfig, EchFetchResult,
};

// Re-export client types
pub use client::{
    Client, HttpResponse, Cache, CookieJar, SessionManager,
    HttpVersion, RequestTiming, RetryConfig, RedirectConfig, TimeoutConfig,
    CompressionType, Decompress,
    Hooks, RequestInfo, ResponseInfo, SharedHooks,
    CertPinner, PinError, export_pins_to_json, generate_pin_from_cert, load_pins_from_json,
    ProfileData, SerializedSession, SessionError,
    Socks5Config, Socks5Connector, Socks5DnsResolve,
    enable_http3, supports_http3,
    AuthConfig, BasicAuth, BasicAuthCache, BearerToken, BearerTokenManager, DigestAuth, NtlmAuth,
    extract_nonce, extract_opaque, extract_qop, extract_realm,
    MetricsCollector, MetricsPercentiles, MetricsStats, RequestMetrics, RequestTimer,
    metrics_reporting_task,
    CircuitBreaker, CircuitState, RateLimiter, RequestLogger,
    Middleware, MiddlewareChain, MiddlewareChainBuilder, MiddlewareContext,
    ConnectionPool, PoolConfig, PoolStats, PooledConnection,
    ProxyRotator, ProxyStatus, RotationConfig,
    SliceReader, StreamingResponse, streaming_response_from_bytes,
};

// HTTP/3 support is feature-gated
// HTTP/3 support is feature-gated
#[cfg(feature = "http3")]
pub use client::Http3Response;

pub use http::Method;
