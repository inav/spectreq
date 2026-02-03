//! Browser fingerprinting profiles
//!
//! This module provides pre-configured browser profiles that mimic real browsers
//! for anti-bot detection evasion. Each profile includes:
//!
//! - TLS fingerprinting (cipher suites, extensions, ALPN)
//! - HTTP/2 settings (window sizes, concurrent streams)
//! - TCP configuration (keepalive, SACK, window size)
//! - Ordered headers for JA4H fingerprinting
//! - Client Hints (Sec-CH-UA-*)
//! - Request context for Sec-Fetch-* headers
//!
//! # Pre-configured Profiles
//!
//! The module includes pre-configured profiles for:
//!
//! - Chrome 120, 131, 133, 141, 143 on Windows, macOS, Linux, Android
//! - Firefox 121 on Windows
//! - Safari 17 on macOS
//! - Edge 120 on Windows
//!
//! # Examples
//!
//! ```rust
//! use spectreq::{Profile, BrowserName, OS};
//!
//! // Use a pre-configured profile
//! let profile = Profile::chrome_143_windows();
//! println!("User-Agent: {}", profile.user_agent);
//!
//! // Build a custom profile
//! let custom = Profile::builder()
//!     .browser(BrowserName::Chrome)
//!     .os(OS::Linux)
//!     .version("143.0")
//!     .http2_initial_window_size(6291456)
//!     .build();
//!
//! // Get ordered headers for JA4H fingerprinting
//! let headers = profile.get_ordered_headers();
//! for (key, value) in headers.iter() {
//!     println!("{}: {}", key, value);
//! }
//! ```

use crate::core::headers::{
    generate_sec_fetch_headers, ClientHints, OrderedHeaders, RequestContext,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Supported browser names for impersonation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrowserName {
    Chrome,
    Firefox,
    Safari,
    Edge,
}

impl fmt::Display for BrowserName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowserName::Chrome => write!(f, "Chrome"),
            BrowserName::Firefox => write!(f, "Firefox"),
            BrowserName::Safari => write!(f, "Safari"),
            BrowserName::Edge => write!(f, "Edge"),
        }
    }
}

/// Supported operating systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OS {
    Windows,
    MacOS,
    Linux,
    Android,
    IOs,
}

impl fmt::Display for OS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OS::Windows => write!(f, "Windows"),
            OS::MacOS => write!(f, "macOS"),
            OS::Linux => write!(f, "Linux"),
            OS::Android => write!(f, "Android"),
            OS::IOs => write!(f, "iOS"),
        }
    }
}

/// HTTP/2 settings that mimic specific browsers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Http2Settings {
    /// Initial window size (bytes)
    pub initial_window_size: u32,
    /// Max concurrent streams
    pub max_concurrent_streams: u32,
    /// Max frame size (bytes)
    pub max_frame_size: u32,
    /// Header table size (bytes)
    pub header_table_size: u32,
    /// Enable push
    pub enable_push: bool,
}

impl Default for Http2Settings {
    fn default() -> Self {
        Self {
            initial_window_size: 65535,
            max_concurrent_streams: 100,
            max_frame_size: 16384,
            header_table_size: 4096,
            enable_push: false,
        }
    }
}

/// TLS configuration for browser impersonation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// List of cipher suites in preference order
    pub cipher_suites: Vec<String>,
    /// TLS extensions to send
    pub extensions: Vec<String>,
    /// Enable GREASE (Generate Random Extensions And Sustain Extensibility)
    pub grease: bool,
    /// TLS version (1.2 or 1.3)
    pub min_version: Option<String>,
    pub max_version: Option<String>,
    /// ALPN protocols
    pub alpn: Vec<String>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            cipher_suites: vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ],
            extensions: vec![
                "server_name".to_string(),
                "status_request".to_string(),
                "supported_groups".to_string(),
                "ec_point_formats".to_string(),
                "signature_algorithms".to_string(),
                "application_layer_protocol_negotiation".to_string(),
                "key_share".to_string(),
            ],
            grease: true,
            min_version: Some("1.2".to_string()),
            max_version: Some("1.3".to_string()),
            alpn: vec!["h2".to_string(), "http/1.1".to_string()],
        }
    }
}

/// TCP/IP configuration for browser impersonation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    /// Time to Live (TTL)
    pub ttl: Option<u32>,
    /// TCP window size
    pub window_size: Option<u32>,
    /// Enable Selective Acknowledgment
    pub sack: bool,
    /// Keepalive settings
    pub keepalive: bool,
    pub keepalive_time_secs: Option<u32>,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            ttl: Some(64),
            window_size: None,
            sack: true,
            keepalive: false,
            keepalive_time_secs: None,
        }
    }
}

/// User agent string
pub type UserAgent = String;

/// HTTP headers to include with each request
pub type Headers = HashMap<String, String>;

/// Complete browser profile for impersonation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Browser name
    pub browser: BrowserName,
    /// Operating system
    pub os: OS,
    /// Browser version (e.g., "120.0.6099.109")
    pub version: String,
    /// User agent string
    pub user_agent: UserAgent,
    /// HTTP/2 settings
    pub http2: Http2Settings,
    /// TLS configuration
    pub tls: TlsConfig,
    /// TCP configuration
    pub tcp: TcpConfig,
    /// Default headers to include (legacy, unordered)
    pub headers: Headers,
    /// Ordered headers (critical for JA4H fingerprinting)
    pub header_order: OrderedHeaders,
    /// Request context for Sec-Fetch-* headers generation
    pub request_context: RequestContext,
    /// Client Hints for Sec-CH-UA-* headers
    pub client_hints: ClientHints,
    /// Session seed for GREASE shuffling consistency
    pub session_seed: u64,
    /// Accept encoding
    pub accept_encoding: String,
    /// Override TLS SNI with different hostname (for domain fronting)
    ///
    /// When set, this hostname will be used in the TLS SNI extension
    /// instead of the actual destination hostname.
    /// This is useful for domain fronting through CDNs.
    pub sni_override: Option<String>,
    /// Connect to different host (for domain fronting)
    ///
    /// When set, the connection will be made to this host instead of
    /// the URL's hostname, while the Host header and SNI will use
    /// the original hostname from the URL.
    /// This is useful for domain fronting through CDNs.
    pub connect_to: Option<String>,
}

impl Profile {
    /// Create a new custom profile
    pub fn new(
        browser: BrowserName,
        os: OS,
        version: impl Into<String>,
        user_agent: impl Into<UserAgent>,
    ) -> Self {
        let version = version.into();
        let user_agent = user_agent.into();

        // Generate Client Hints based on browser, OS, and version
        let client_hints = crate::core::headers::generate_client_hints(browser, os, &version);

        Self {
            browser,
            os,
            version,
            user_agent,
            http2: Http2Settings::default(),
            tls: TlsConfig::default(),
            tcp: TcpConfig::default(),
            headers: Headers::new(),
            header_order: OrderedHeaders::new(),
            request_context: RequestContext::default(),
            client_hints,
            session_seed: 0,
            accept_encoding: "gzip, deflate, br, zstd".to_string(),
            sni_override: None,
            connect_to: None,
        }
    }

    /// Create a builder for this profile
    pub fn builder() -> ProfileBuilder {
        ProfileBuilder::new()
    }

    /// Chrome 120 on Windows 11
    pub fn chrome_120_windows() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::Windows)
            .version("120.0.6099.109")
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .http2_initial_window_size(65536)
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".to_string(),
                "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(128)
            .build()
    }

    /// Firefox 121 on Windows 11
    pub fn firefox_121_windows() -> Self {
        Self::builder()
            .browser(BrowserName::Firefox)
            .os(OS::Windows)
            .version("121.0")
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
            )
            .http2_initial_window_size(65535)
            .http2_max_concurrent_streams(100)
            .http2_header_table_size(4096)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(false)
            .tcp_ttl(64)
            .build()
    }

    /// Safari 17 on macOS Sonoma
    pub fn safari_17_macos() -> Self {
        Self::builder()
            .browser(BrowserName::Safari)
            .os(OS::MacOS)
            .version("17.0")
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15")
            .http2_initial_window_size(65536)
            .http2_max_concurrent_streams(100)
            .http2_header_table_size(4096)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".to_string(),
                "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".to_string(),
            ])
            .tls_grease(false)
            .tcp_ttl(64)
            .build()
    }

    /// Edge 120 on Windows 11
    pub fn edge_120_windows() -> Self {
        Self::builder()
            .browser(BrowserName::Edge)
            .os(OS::Windows)
            .version("120.0.2210.61")
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.2210.61")
            .http2_initial_window_size(65536)
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(128)
            .build()
    }

    /// Chrome 120 on macOS
    pub fn chrome_120_macos() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::MacOS)
            .version("120.0.6099.109")
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .http2_initial_window_size(65536)
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(64)
            .build()
    }

    /// Chrome 120 on Linux
    pub fn chrome_120_linux() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::Linux)
            .version("120.0.6099.109")
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .http2_initial_window_size(65536)
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(64)
            .build()
    }

    /// Chrome 120 on Android
    pub fn chrome_120_android() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::Android)
            .version("120.0.6099.43")
            .user_agent("Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.6099.43 Mobile Safari/537.36")
            .http2_initial_window_size(65536)
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(64)
            .build()
    }

    /// Chrome 131 on Windows 11
    /// Chrome 131+ uses ~6MB window size for HTTP/2
    pub fn chrome_131_windows() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::Windows)
            .version("131.0.0.0")
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .http2_initial_window_size(6291456) // 6MB window size
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(128)
            .build()
    }

    /// Chrome 133 on Windows 11
    pub fn chrome_133_windows() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::Windows)
            .version("133.0.0.0")
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36")
            .http2_initial_window_size(6291456) // 6MB window size
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(128)
            .build()
    }

    /// Chrome 141 on Windows 11
    pub fn chrome_141_windows() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::Windows)
            .version("141.0.0.0")
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36")
            .http2_initial_window_size(6291456) // 6MB window size
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(128)
            .build()
    }

    /// Chrome 143 on Windows 11
    pub fn chrome_143_windows() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::Windows)
            .version("143.0.0.0")
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36")
            .http2_initial_window_size(6291456) // 6MB window size
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(128)
            .build()
    }

    /// Chrome 143 on macOS
    pub fn chrome_143_macos() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::MacOS)
            .version("143.0.0.0")
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36")
            .http2_initial_window_size(6291456)
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(64)
            .build()
    }

    /// Chrome 143 on Linux
    pub fn chrome_143_linux() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::Linux)
            .version("143.0.0.0")
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36")
            .http2_initial_window_size(6291456)
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(64)
            .build()
    }

    /// Chrome 143 on Android
    pub fn chrome_143_android() -> Self {
        Self::builder()
            .browser(BrowserName::Chrome)
            .os(OS::Android)
            .version("143.0.6099.43")
            .user_agent("Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.6099.43 Mobile Safari/537.36")
            .http2_initial_window_size(6291456)
            .http2_max_concurrent_streams(256)
            .http2_header_table_size(65536)
            .tls_cipher_suites(vec![
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            ])
            .tls_grease(true)
            .tcp_ttl(64)
            .build()
    }

    /// Get ordered headers for this profile
    ///
    /// This combines:
    /// 1. Default headers (accept, accept-encoding, user-agent, etc.)
    /// 2. Sec-Fetch-* headers based on request context
    /// 3. Client Hints (Sec-CH-UA-*)
    /// 4. Any profile-specific custom headers
    pub fn get_ordered_headers(&self) -> OrderedHeaders {
        let mut headers = OrderedHeaders::new();

        // Standard headers in the order Chrome sends them
        headers.insert("accept".to_string(), "*/*".to_string());
        headers.insert("accept-encoding".to_string(), self.accept_encoding.clone());
        headers.insert("accept-language".to_string(), "en-US,en;q=0.9".to_string());
        headers.insert("user-agent".to_string(), self.user_agent.clone());

        // Add Sec-Fetch-* headers
        let sec_fetch = generate_sec_fetch_headers(&self.request_context);
        for (key, value) in sec_fetch.iter() {
            headers.insert(key.clone(), value.clone());
        }

        // Add Client Hints
        for (key, value) in self.client_hints.to_headers().iter() {
            headers.insert(key.clone(), value.clone());
        }

        // Merge with profile's custom ordered headers
        for (key, value) in self.header_order.iter() {
            headers.insert(key.clone(), value.clone());
        }

        // Merge legacy headers (for backward compatibility)
        for (key, value) in self.headers.iter() {
            if !headers.contains_key(key) {
                headers.insert(key.clone(), value.clone());
            }
        }

        headers
    }

    /// Set the request context for this profile
    pub fn with_request_context(mut self, ctx: RequestContext) -> Self {
        self.request_context = ctx;
        self
    }

    /// Set a custom header (in ordered headers)
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.header_order.insert(key.into(), value.into());
        self
    }

    /// Set the session seed for GREASE shuffling
    pub fn with_session_seed(mut self, seed: u64) -> Self {
        self.session_seed = seed;
        self
    }

    /// Set SNI override for domain fronting
    ///
    /// When set, this hostname will be used in the TLS SNI extension
    /// instead of the actual destination hostname.
    /// This is useful for domain fronting through CDNs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spectreq::Profile;
    ///
    /// // Connect to example.com but use cloudflare.com in SNI
    /// let profile = Profile::chrome_143_windows()
    ///     .with_sni_override("cloudflare.com");
    /// ```
    pub fn with_sni_override(mut self, sni: impl Into<String>) -> Self {
        self.sni_override = Some(sni.into());
        self
    }

    /// Set connect target for domain fronting
    ///
    /// When set, the connection will be made to this host instead of
    /// the URL's hostname, while the Host header and SNI will use
    /// the original hostname from the URL.
    /// This is useful for domain fronting through CDNs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spectreq::Profile;
    ///
    /// // Connect to cloudflare.com but use example.com in Host header and SNI
    /// let profile = Profile::chrome_143_windows()
    ///     .with_connect_to("cloudflare.com");
    /// ```
    pub fn with_connect_to(mut self, host: impl Into<String>) -> Self {
        self.connect_to = Some(host.into());
        self
    }
}

/// Builder for creating custom [`Profile`] instances
///
/// Provides a fluent interface for constructing browser profiles with
/// custom settings for TLS, HTTP/2, TCP, and headers.
///
/// # Examples
///
/// ```rust
/// use spectreq::{Profile, BrowserName, OS};
///
/// let profile = Profile::builder()
///     .browser(BrowserName::Chrome)
///     .os(OS::Linux)
///     .version("143.0")
///     .http2_initial_window_size(6291456)
///     .tls_grease(true)
///     .build();
/// ```
pub struct ProfileBuilder {
    profile: Profile,
}

impl ProfileBuilder {
    /// Create a new profile builder with default values
    ///
    /// The builder starts with Chrome on Windows as defaults.
    pub fn new() -> Self {
        Self {
            profile: Profile {
                browser: BrowserName::Chrome,
                os: OS::Windows,
                version: "1.0".to_string(),
                user_agent: String::new(),
                http2: Http2Settings::default(),
                tls: TlsConfig::default(),
                tcp: TcpConfig::default(),
                headers: Headers::new(),
                header_order: OrderedHeaders::new(),
                request_context: RequestContext::default(),
                client_hints: ClientHints::default(),
                session_seed: 0,
                accept_encoding: "gzip, deflate, br, zstd".to_string(),
                sni_override: None,
                connect_to: None,
            },
        }
    }

    /// Set the browser to impersonate
    ///
    /// This will automatically regenerate client hints for the new browser.
    pub fn browser(mut self, browser: BrowserName) -> Self {
        self.profile.browser = browser;
        // Regenerate client hints for new browser
        self.profile.client_hints = crate::core::headers::generate_client_hints(
            browser,
            self.profile.os,
            &self.profile.version.clone(),
        );
        self
    }

    /// Set the operating system to impersonate
    ///
    /// This will automatically regenerate client hints for the new OS.
    pub fn os(mut self, os: OS) -> Self {
        self.profile.os = os;
        // Regenerate client hints for new OS
        self.profile.client_hints = crate::core::headers::generate_client_hints(
            self.profile.browser,
            os,
            &self.profile.version.clone(),
        );
        self
    }

    /// Set the browser version string
    ///
    /// This will automatically regenerate client hints for the new version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        let version = version.into();
        // Regenerate client hints for new version
        self.profile.client_hints = crate::core::headers::generate_client_hints(
            self.profile.browser,
            self.profile.os,
            &version,
        );
        self.profile.version = version;
        self
    }

    /// Set the user agent string
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.profile.user_agent = user_agent.into();
        self
    }

    /// Set the HTTP/2 initial window size (in bytes)
    pub fn http2_initial_window_size(mut self, size: u32) -> Self {
        self.profile.http2.initial_window_size = size;
        self
    }

    /// Set the HTTP/2 maximum concurrent streams
    pub fn http2_max_concurrent_streams(mut self, streams: u32) -> Self {
        self.profile.http2.max_concurrent_streams = streams;
        self
    }

    /// Set the HTTP/2 maximum frame size (in bytes)
    pub fn http2_max_frame_size(mut self, size: u32) -> Self {
        self.profile.http2.max_frame_size = size;
        self
    }

    /// Set the HTTP/2 header table size (in bytes)
    pub fn http2_header_table_size(mut self, size: u32) -> Self {
        self.profile.http2.header_table_size = size;
        self
    }

    /// Enable or disable HTTP/2 server push
    pub fn http2_enable_push(mut self, enable: bool) -> Self {
        self.profile.http2.enable_push = enable;
        self
    }

    /// Set the TLS cipher suites (in preference order)
    pub fn tls_cipher_suites(mut self, suites: Vec<String>) -> Self {
        self.profile.tls.cipher_suites = suites;
        self
    }

    /// Set the TLS extensions to send
    pub fn tls_extensions(mut self, extensions: Vec<String>) -> Self {
        self.profile.tls.extensions = extensions;
        self
    }

    /// Enable or disable GREASE (Generate Random Extensions And Sustain Extensibility)
    pub fn tls_grease(mut self, grease: bool) -> Self {
        self.profile.tls.grease = grease;
        self
    }

    /// Set the ALPN protocols
    pub fn tls_alpn(mut self, alpn: Vec<String>) -> Self {
        self.profile.tls.alpn = alpn;
        self
    }

    /// Set the TCP Time to Live (TTL)
    pub fn tcp_ttl(mut self, ttl: u32) -> Self {
        self.profile.tcp.ttl = Some(ttl);
        self
    }

    /// Set the TCP window size
    pub fn tcp_window_size(mut self, size: u32) -> Self {
        self.profile.tcp.window_size = Some(size);
        self
    }

    /// Enable or disable Selective Acknowledgment (SACK)
    pub fn tcp_sack(mut self, sack: bool) -> Self {
        self.profile.tcp.sack = sack;
        self
    }

    /// Enable or disable TCP keepalive
    pub fn tcp_keepalive(mut self, keepalive: bool) -> Self {
        self.profile.tcp.keepalive = keepalive;
        self
    }

    /// Set the TCP keepalive time in seconds
    pub fn tcp_keepalive_time_secs(mut self, secs: u32) -> Self {
        self.profile.tcp.keepalive_time_secs = Some(secs);
        self
    }

    /// Set the Accept-Encoding header value
    pub fn accept_encoding(mut self, encoding: impl Into<String>) -> Self {
        self.profile.accept_encoding = encoding.into();
        self
    }

    /// Add a custom header (unordered, for backward compatibility)
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.profile.headers.insert(key.into(), value.into());
        self
    }

    /// Set an ordered header (for JA4H fingerprinting)
    ///
    /// Ordered headers preserve the order they are sent, which is important
    /// for JA4H fingerprint calculation.
    pub fn ordered_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.profile.header_order.insert(key.into(), value.into());
        self
    }

    /// Set the request context for Sec-Fetch-* header generation
    pub fn request_context(mut self, ctx: RequestContext) -> Self {
        self.profile.request_context = ctx;
        self
    }

    /// Set client hints directly (overrides automatic generation)
    pub fn client_hints(mut self, hints: ClientHints) -> Self {
        self.profile.client_hints = hints;
        self
    }

    /// Set the session seed for GREASE shuffling consistency
    pub fn session_seed(mut self, seed: u64) -> Self {
        self.profile.session_seed = seed;
        self
    }

    /// Set SNI override for domain fronting
    ///
    /// When set, this hostname will be used in the TLS SNI extension
    /// instead of the actual destination hostname.
    pub fn sni_override(mut self, sni: impl Into<String>) -> Self {
        self.profile.sni_override = Some(sni.into());
        self
    }

    /// Set connect target for domain fronting
    ///
    /// When set, the connection will be made to this host instead of
    /// the URL's hostname, while the Host header and SNI will use
    /// the original hostname from the URL.
    pub fn connect_to(mut self, host: impl Into<String>) -> Self {
        self.profile.connect_to = Some(host.into());
        self
    }

    /// Build the profile
    ///
    /// Consumes the builder and returns the configured [`Profile`].
    pub fn build(self) -> Profile {
        self.profile
    }
}

impl Default for ProfileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Dynamic Profile Loading
// ============================================================================

impl Profile {
    /// Load a profile from a JSON file
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use spectreq::Profile;
    ///
    /// let profile = Profile::from_json_file("profiles/chrome_143.json")?;
    /// ```
    pub fn from_json_file(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, crate::core::SpectreError> {
        let contents = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            crate::core::SpectreError::Config(format!("Failed to read profile file: {}", e))
        })?;
        Self::from_json(&contents)
    }

    /// Load a profile from a JSON string
    pub fn from_json(json: &str) -> Result<Self, crate::core::SpectreError> {
        serde_json::from_str(json).map_err(|e| {
            crate::core::SpectreError::Config(format!("Failed to parse profile JSON: {}", e))
        })
    }

    /// Load a profile from a YAML file
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use spectreq::Profile;
    ///
    /// let profile = Profile::from_yaml_file("profiles/chrome_143.yaml")?;
    /// ```
    pub fn from_yaml_file(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, crate::core::SpectreError> {
        let contents = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            crate::core::SpectreError::Config(format!("Failed to read profile file: {}", e))
        })?;
        Self::from_yaml(&contents)
    }

    /// Load a profile from a YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self, crate::core::SpectreError> {
        serde_yaml::from_str(yaml).map_err(|e| {
            crate::core::SpectreError::Config(format!("Failed to parse profile YAML: {}", e))
        })
    }

    /// Export profile to JSON string
    pub fn to_json(&self) -> Result<String, crate::core::SpectreError> {
        serde_json::to_string_pretty(self).map_err(|e| {
            crate::core::SpectreError::Config(format!("Failed to serialize profile: {}", e))
        })
    }

    /// Export profile to YAML string
    pub fn to_yaml(&self) -> Result<String, crate::core::SpectreError> {
        serde_yaml::to_string(self).map_err(|e| {
            crate::core::SpectreError::Config(format!("Failed to serialize profile: {}", e))
        })
    }

    /// Get a random profile from predefined list
    ///
    /// Useful for anti-detection by rotating browser fingerprints.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spectreq::Profile;
    ///
    /// let profile = Profile::random();
    /// println!("Using: {} on {}", profile.browser, profile.os);
    /// ```
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let profiles: Vec<fn() -> Self> = vec![
            Self::chrome_143_windows,
            Self::chrome_143_macos,
            Self::chrome_143_linux,
            Self::chrome_120_windows,
            Self::chrome_131_windows,
            Self::firefox_121_windows,
            Self::safari_17_macos,
            Self::edge_120_windows,
        ];
        let idx = rng.random_range(0..profiles.len());
        profiles[idx]()
    }

    /// Get a random Chrome profile
    pub fn random_chrome() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let profiles: Vec<fn() -> Self> = vec![
            Self::chrome_143_windows,
            Self::chrome_143_macos,
            Self::chrome_143_linux,
            Self::chrome_143_android,
            Self::chrome_131_windows,
            Self::chrome_120_windows,
            Self::chrome_120_macos,
            Self::chrome_120_linux,
        ];
        let idx = rng.random_range(0..profiles.len());
        profiles[idx]()
    }

    /// Randomize session-specific values for anti-detection
    ///
    /// This randomizes:
    /// - Session seed (for GREASE shuffling)
    /// - Minor timing variations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spectreq::Profile;
    ///
    /// let profile = Profile::chrome_143_windows().randomize();
    /// ```
    pub fn randomize(mut self) -> Self {
        use rand::Rng;
        let mut rng = rand::rng();

        // Randomize session seed for GREASE shuffling
        self.session_seed = rng.random();

        // Slight variations in HTTP/2 settings (within normal range)
        if self.http2.initial_window_size > 65536 {
            // For Chrome 131+ with 6MB window, vary by +/- 32KB
            let delta: i32 = rng.random_range(-32768..32768);
            self.http2.initial_window_size =
                (self.http2.initial_window_size as i64 + delta as i64) as u32;
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_builder() {
        let profile = Profile::builder()
            .browser(BrowserName::Chrome)
            .os(OS::Windows)
            .version("120.0")
            .user_agent("test-ua")
            .build();

        assert_eq!(profile.browser, BrowserName::Chrome);
        assert_eq!(profile.os, OS::Windows);
        assert_eq!(profile.version, "120.0");
        assert_eq!(profile.user_agent, "test-ua");
    }

    #[test]
    fn test_predefined_profiles() {
        let chrome = Profile::chrome_120_windows();
        assert_eq!(chrome.browser, BrowserName::Chrome);
        assert_eq!(chrome.os, OS::Windows);

        let firefox = Profile::firefox_121_windows();
        assert_eq!(firefox.browser, BrowserName::Firefox);
        assert_eq!(firefox.os, OS::Windows);

        let safari = Profile::safari_17_macos();
        assert_eq!(safari.browser, BrowserName::Safari);
        assert_eq!(safari.os, OS::MacOS);
    }
}
