//! HTTP client with browser impersonation
//!
//! This module provides the main [`Client`] type for making HTTP requests
//! that mimic real browser fingerprints.
//!
//! # Examples
//!
//! ```rust,ignore
//! use spectreq::Client;
//! use spectreq::Profile;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new(Profile::chrome_143_windows()).await?;
//!     let response = client.get("https://example.com").await?;
//!     println!("{}", response.text()?);
//!     Ok(())
//! }
//! ```
//!
//! # Using the Builder
//!
//! ```rust,ignore
//! use spectreq::Client;
//! use spectreq::Profile;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::builder()
//!         .profile(Profile::chrome_143_windows())
//!         .enable_cache(true)
//!         .enable_cookies(true)
//!         .proxy("socks5://127.0.0.1:1080")
//!         .build()
//!         .await?;
//!     Ok(())
//! }
//! ```

use crate::client::cache::Cache;
use crate::client::compression::Decompress;
use crate::client::connector::ImpersonateConnector;
use crate::client::cookies::CookieJar;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request};
use hyper_util::client::legacy::{Client as HyperClient, Error as HyperLegacyError};
use hyper_util::rt::TokioExecutor;
use crate::core::{Headers, Profile, Result, SpectreError};
use std::time::Duration;
use url::Url;

/// HTTP protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVersion {
    /// HTTP/1.1
    H1,
    /// HTTP/2
    H2,
    /// HTTP/3 (QUIC)
    H3,
}

impl HttpVersion {
    /// Get the ALPN protocol string for this version
    pub fn alpn(&self) -> &'static str {
        match self {
            HttpVersion::H1 => "http/1.1",
            HttpVersion::H2 => "h2",
            HttpVersion::H3 => "h3",
        }
    }

    /// Get the version string
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpVersion::H1 => "HTTP/1.1",
            HttpVersion::H2 => "HTTP/2",
            HttpVersion::H3 => "HTTP/3",
        }
    }
}

/// Request timing metrics
///
/// Contains detailed timing information for each stage of an HTTP request.
/// This is useful for performance analysis and debugging.
///
/// # Examples
///
/// ```rust,ignore
/// let resp = client.get("https://example.com").await?;
///
/// println!("DNS lookup: {:?}", resp.timing.dns_lookup);
/// println!("TCP connect: {:?}", resp.timing.tcp_connect);
/// println!("TLS handshake: {:?}", resp.timing.tls_handshake);
/// println!("TTFB: {:?}", resp.timing.ttfb);
/// println!("Total: {:?}", resp.timing.total);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct RequestTiming {
    /// Time taken for DNS lookup
    pub dns_lookup: Duration,
    /// Time taken for TCP connection
    pub tcp_connect: Duration,
    /// Time taken for TLS handshake
    pub tls_handshake: Duration,
    /// Time to first byte (TTFB)
    pub ttfb: Duration,
    /// Total time for the request
    pub total: Duration,
}

impl RequestTiming {
    /// Create a new RequestTiming with all fields set to zero
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the total time excluding DNS lookup
    pub fn connection_time(&self) -> Duration {
        self.tcp_connect + self.tls_handshake
    }

    /// Get the server processing time (TTFB - connection time)
    pub fn server_time(&self) -> Duration {
        self.ttfb.saturating_sub(self.connection_time())
    }
}

/// Convert hyper_util error to SpectreError
fn convert_hyper_error(err: HyperLegacyError) -> SpectreError {
    SpectreError::Hyper(err.to_string())
}

/// HTTP response with metadata
///
/// Contains the status code, headers, body, and metadata about the response
/// such as wire size, content type, ETag, timing information, and whether it was served from cache.
///
/// # Examples
///
/// ```rust,ignore
/// use spectreq::Client;
///
/// let client = Client::new(profile).await?;
/// let resp = client.get("https://example.com").await?;
///
/// if resp.ok() {
///     println!("Status: {}", resp.status);
///     println!("Content-Type: {:?}", resp.content_type);
///     println!("Body: {}", resp.text()?);
///     println!("Total time: {:?}", resp.timing.total);
/// }
///
/// // Parse JSON response
/// if resp.content_type.as_ref().map_or(false, |ct| ct.contains("json")) {
///     let json: serde_json::Value = resp.json()?;
///     println!("JSON: {}", json);
/// }
/// ```
#[derive(Debug)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: Vec<(String, String)>,
    /// Response body (decompressed)
    pub body: Vec<u8>,
    /// Wire size (compressed size from network)
    pub wire_size: usize,
    /// Content type
    pub content_type: Option<String>,
    /// ETag if present
    pub etag: Option<String>,
    /// Last-Modified if present
    pub last_modified: Option<String>,
    /// Whether this response was from cache (304)
    pub from_cache: bool,
    /// Request timing information
    pub timing: RequestTiming,
}

impl HttpResponse {
    /// Get the response body as text
    ///
    /// # Errors
    ///
    /// Returns an error if the response body is not valid UTF-8.
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| SpectreError::Http(format!("Invalid UTF-8: {}", e)))
    }

    /// Get the response body as bytes
    ///
    /// Returns a slice reference to the response body.
    pub fn bytes(&self) -> &[u8] {
        &self.body
    }

    /// Parse response body as JSON
    ///
    /// # Errors
    ///
    /// Returns an error if the response body is not valid JSON.
    pub fn json(&self) -> Result<serde_json::Value> {
        serde_json::from_slice(&self.body)
            .map_err(|e| SpectreError::Http(format!("Invalid JSON: {}", e)))
    }

    /// Get a specific header value
    ///
    /// Header names are case-insensitive.
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
    }

    /// Check if request was successful (2xx or 3xx status code)
    ///
    /// Returns `true` for status codes 200-399.
    pub fn ok(&self) -> bool {
        self.status >= 200 && self.status < 400
    }
}

/// Retry configuration for requests
///
/// # Examples
///
/// ```rust
/// use spectreq::RetryConfig;
/// use std::time::Duration;
///
/// let config = RetryConfig::new()
///     .max_retries(5)
///     .retry_on_status(vec![408, 429, 500, 502, 503, 504])
///     .wait_bounds(Duration::from_millis(100), Duration::from_secs(30));
/// ```
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Status codes to retry on
    pub retry_on_status: Vec<u16>,
    /// Minimum wait time between retries
    pub wait_min: Duration,
    /// Maximum wait time between retries
    pub wait_max: Duration,
    /// Backoff exponent
    pub backoff_exponent: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_on_status: vec![408, 429, 500, 502, 503, 504],
            wait_min: Duration::from_millis(100),
            wait_max: Duration::from_secs(30),
            backoff_exponent: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of retries
    pub fn max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Set the status codes to retry on
    pub fn retry_on_status(mut self, codes: Vec<u16>) -> Self {
        self.retry_on_status = codes;
        self
    }

    /// Set the minimum and maximum wait time between retries
    pub fn wait_bounds(mut self, min: Duration, max: Duration) -> Self {
        self.wait_min = min;
        self.wait_max = max;
        self
    }

    /// Calculate the backoff delay for a given attempt
    ///
    /// Uses exponential backoff with the configured exponent.
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let base = self.wait_min.as_millis() as f64;
        let delay = base * self.backoff_exponent.powi(attempt as i32);
        let delay = delay.min(self.wait_max.as_millis() as f64);
        Duration::from_millis(delay as u64)
    }
}

/// Redirect configuration for requests
///
/// # Examples
///
/// ```rust
/// use spectreq::RedirectConfig;
///
/// let config = RedirectConfig::new()
///     .follow(true)
///     .max_redirects(10);
/// ```
#[derive(Debug, Clone)]
pub struct RedirectConfig {
    /// Whether to follow redirects
    pub follow: bool,
    /// Maximum number of redirects to follow
    pub max_redirects: usize,
}

impl Default for RedirectConfig {
    fn default() -> Self {
        Self {
            follow: true,
            max_redirects: 20,
        }
    }
}

impl RedirectConfig {
    /// Create a new redirect config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to follow redirects
    pub fn follow(mut self, follow: bool) -> Self {
        self.follow = follow;
        self
    }

    /// Set the maximum number of redirects to follow
    pub fn max_redirects(mut self, max: usize) -> Self {
        self.max_redirects = max;
        self
    }
}

/// Timeout configuration for requests
///
/// # Examples
///
/// ```rust
/// use spectreq::TimeoutConfig;
/// use std::time::Duration;
///
/// let config = TimeoutConfig::new()
///     .connect(Duration::from_secs(5))
///     .read(Duration::from_secs(15))
///     .total(Duration::from_secs(30));
/// ```
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Connect timeout
    pub connect: Duration,
    /// Read timeout
    pub read: Duration,
    /// Total request timeout
    pub total: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(10),
            read: Duration::from_secs(30),
            total: Duration::from_secs(60),
        }
    }
}

impl TimeoutConfig {
    /// Create a new timeout config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the connection timeout
    pub fn connect(mut self, timeout: Duration) -> Self {
        self.connect = timeout;
        self
    }

    /// Set the read timeout
    pub fn read(mut self, timeout: Duration) -> Self {
        self.read = timeout;
        self
    }

    /// Set the total request timeout
    pub fn total(mut self, timeout: Duration) -> Self {
        self.total = timeout;
        self
    }
}

/// HTTP client with browser impersonation
///
/// The main client type for making HTTP requests that mimic real browser fingerprints.
///
/// # Examples
///
/// ```rust,ignore
/// use spectreq::Client;
/// use spectreq::Profile;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::new(Profile::chrome_143_windows()).await?;
///
///     // GET request
///     let resp = client.get("https://httpbin.org/get").await?;
///     println!("{}", resp.text()?);
///
///     // POST request
///     let data = r#"{"key": "value"}"#;
///     let resp = client.post("https://httpbin.org/post", data.as_bytes()).await?;
///     println!("{}", resp.text()?);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    profile: Profile,
    cache: Cache,
    cookie_jar: CookieJar,
    proxy: Option<String>,
    /// TCP proxy for HTTP/1.1 and HTTP/2 (different from UDP proxy for HTTP/3)
    tcp_proxy: Option<String>,
    /// UDP proxy for HTTP/3 (SOCKS5 with UDP ASSOCIATE)
    udp_proxy: Option<String>,
    headers: Headers,
    retry_config: RetryConfig,
    redirect_config: RedirectConfig,
    timeout_config: TimeoutConfig,
    /// Whether HTTP/3 is enabled (requires "http3" feature)
    http3_enabled: bool,
    /// Preferred HTTP version (None for auto-negotiation)
    preferred_http_version: Option<HttpVersion>,
}

impl Client {
    /// Create a new client with the given profile
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use spectreq::Client;
    /// use spectreq::Profile;
    ///
    /// let client = Client::new(Profile::chrome_143_windows()).await?;
    /// ```
    pub async fn new(profile: Profile) -> Result<Self> {
        Ok(Self {
            profile,
            cache: Cache::new(),
            cookie_jar: CookieJar::new(),
            proxy: None,
            tcp_proxy: None,
            udp_proxy: None,
            headers: Headers::new(),
            retry_config: RetryConfig::default(),
            redirect_config: RedirectConfig::default(),
            timeout_config: TimeoutConfig::default(),
            http3_enabled: false,
            preferred_http_version: None,
        })
    }

    /// Create a new client with the given profile and proxy
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use spectreq::Client;
    /// use spectreq::Profile;
    ///
    /// let client = Client::with_proxy(
    ///     Profile::chrome_143_windows(),
    ///     Some("socks5://127.0.0.1:1080".to_string())
    /// ).await?;
    /// ```
    pub async fn with_proxy(profile: Profile, proxy: Option<String>) -> Result<Self> {
        Ok(Self {
            profile,
            cache: Cache::new(),
            cookie_jar: CookieJar::new(),
            proxy: proxy.clone(),
            tcp_proxy: proxy.clone(),
            udp_proxy: None,
            headers: Headers::new(),
            retry_config: RetryConfig::default(),
            redirect_config: RedirectConfig::default(),
            timeout_config: TimeoutConfig::default(),
            http3_enabled: false,
            preferred_http_version: None,
        })
    }

    /// Create a new client with the given profile, proxy, and custom headers
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use spectreq::Client;
    /// use spectreq::Profile;
    /// use std::collections::HashMap;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert("X-Custom-Header".to_string(), "value".to_string());
    ///
    /// let client = Client::with_options(
    ///     Profile::chrome_143_windows(),
    ///     None,
    ///     headers
    /// ).await?;
    /// ```
    pub async fn with_options(
        profile: Profile,
        proxy: Option<String>,
        headers: Headers,
    ) -> Result<Self> {
        Ok(Self {
            profile,
            cache: Cache::new(),
            cookie_jar: CookieJar::new(),
            proxy: proxy.clone(),
            tcp_proxy: proxy,
            udp_proxy: None,
            headers,
            retry_config: RetryConfig::default(),
            redirect_config: RedirectConfig::default(),
            timeout_config: TimeoutConfig::default(),
            http3_enabled: false,
            preferred_http_version: None,
        })
    }

    /// Create a client builder for advanced configuration
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use spectreq::Client;
    /// use spectreq::Profile;
    ///
    /// let client = Client::builder()
    ///     .profile(Profile::chrome_143_windows())
    ///     .enable_cache(true)
    ///     .enable_cookies(true)
    ///     .build()
    /// .await?;
    /// ```
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Perform a GET request to the specified URL
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request
    ///
    /// # Returns
    ///
    /// Returns an [`HttpResponse`] containing the status, headers, and body.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid, connection fails, or response cannot be received.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let resp = client.get("https://httpbin.org/get").await?;
    /// println!("Status: {}", resp.status);
    /// println!("Body: {}", resp.text()?);
    /// ```
    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.request(Method::GET, url, None).await
    }

    /// Perform a POST request with a body
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request
    /// * `body` - The request body (implements `Into<Bytes>`)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let data = r#"{"key": "value"}"#;
    /// let resp = client.post("https://httpbin.org/post", data.as_bytes()).await?;
    /// ```
    pub async fn post(&self, url: &str, body: impl Into<Bytes>) -> Result<HttpResponse> {
        self.request(Method::POST, url, Some(body.into())).await
    }

    /// Perform a PUT request with a body
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request
    /// * `body` - The request body (implements `Into<Bytes>`)
    pub async fn put(&self, url: &str, body: impl Into<Bytes>) -> Result<HttpResponse> {
        self.request(Method::PUT, url, Some(body.into())).await
    }

    /// Perform a DELETE request to the specified URL
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request
    pub async fn delete(&self, url: &str) -> Result<HttpResponse> {
        self.request(Method::DELETE, url, None).await
    }

    /// Perform a PATCH request with a body
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request
    /// * `body` - The request body (implements `Into<Bytes>`)
    pub async fn patch(&self, url: &str, body: impl Into<Bytes>) -> Result<HttpResponse> {
        self.request(Method::PATCH, url, Some(body.into())).await
    }

    /// Perform a HEAD request to the specified URL
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to request
    pub async fn head(&self, url: &str) -> Result<HttpResponse> {
        self.request(Method::HEAD, url, None).await
    }

    /// Perform a generic HTTP request
    ///
    /// This is the underlying method used by all other HTTP methods.
    /// It handles caching, cookies, compression, and browser impersonation.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method to use
    /// * `url` - The URL to request
    /// * `body` - Optional request body
    pub async fn request(
        &self,
        method: Method,
        url: &str,
        body: Option<Bytes>,
    ) -> Result<HttpResponse> {
        let url_obj = Url::parse(url).map_err(|e| SpectreError::InvalidUrl(e.to_string()))?;

        // Check cache first for GET/HEAD requests
        let (etag, last_modified) = match method {
            Method::GET | Method::HEAD => (
                self.cache.get_etag(url, method.as_str()),
                self.cache.get_last_modified(url, method.as_str()),
            ),
            _ => (None, None),
        };

        // Build the request
        let mut req_builder = Request::builder().method(method.clone()).uri(url);

        // Apply ordered headers from profile (including Sec-Fetch-*, Client Hints, etc.)
        let ordered_headers = self.profile.get_ordered_headers();
        for (key, value) in ordered_headers.iter() {
            // Skip certain headers that we handle specially
            if key.eq_ignore_ascii_case("host") || key.eq_ignore_ascii_case("content-length") {
                continue;
            }
            req_builder = req_builder.header(key, value);
        }

        // Add custom headers (these override defaults)
        for (key, value) in &self.headers {
            req_builder = req_builder.header(key, value);
        }

        // Add conditional request headers
        if let Some(etag) = &etag {
            req_builder = req_builder.header("If-None-Match", etag);
        }
        if let Some(last_modified) = &last_modified {
            req_builder = req_builder.header("If-Modified-Since", last_modified);
        }

        // Add cookies
        if let Some(cookie_value) = self.cookie_jar.get_cookie_value(&url_obj) {
            req_builder = req_builder.header("Cookie", cookie_value);
        }

        // Build body - always use Full<Bytes> for simplicity
        let body_bytes = body.unwrap_or_default();
        let req = req_builder
            .header("Content-Length", body_bytes.len())
            .body(Full::new(body_bytes))
            .map_err(|e| SpectreError::Http(e.to_string()))?;

        // Send the request using hyper with impersonation and optional proxy
        let start_time = std::time::Instant::now();
        let connector = ImpersonateConnector::with_proxy(self.profile.clone(), self.proxy.clone())?;
        let hyper_client = HyperClient::builder(TokioExecutor::new()).build(connector);

        let resp = hyper_client
            .request(req)
            .await
            .map_err(convert_hyper_error)?;
        
        let ttfb = start_time.elapsed();

        // Extract headers before moving resp
        let status = resp.status().as_u16();
        let headers: Vec<(String, String)> = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let etag = resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let last_modified = resp
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let cache_control = resp
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let content_encoding = resp
            .headers()
            .get("content-encoding")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Handle 304 Not Modified
        if status == 304 {
            if let Some(cached_body) = self.cache.get_body(url, method.as_str()) {
                let timing = RequestTiming {
                    ttfb,
                    total: start_time.elapsed(),
                    ..RequestTiming::default()
                };
                
                return Ok(HttpResponse {
                    status,
                    headers,
                    body: cached_body,
                    wire_size: 0,
                    content_type,
                    etag,
                    last_modified,
                    from_cache: true,
                    timing,
                });
            }
        }

        // Collect body
        let body_bytes = BodyExt::collect(resp.into_body()).await?.to_bytes();

        // Decompress
        let decompressed = body_bytes
            .decompress_auto(content_encoding.as_deref())
            .map_err(|e| SpectreError::Compression(e.to_string()))?;

        // Cache the response
        if matches!(method.as_str(), "GET" | "HEAD") {
            self.cache.put(
                url,
                method.as_str(),
                etag.clone(),
                last_modified.clone(),
                Some(decompressed.data.clone()),
                cache_control.as_deref(),
                content_type.clone(),
            );
        }

        // Store cookies
        let set_cookies: Vec<&str> = headers
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
            .map(|(_, v)| v.as_str())
            .collect();
        if !set_cookies.is_empty() {
            self.cookie_jar.set_cookies(&set_cookies, &url_obj);
        }
        
        let total = start_time.elapsed();
        let timing = RequestTiming {
            ttfb,
            total,
            ..RequestTiming::default()
        };

        Ok(HttpResponse {
            status,
            headers,
            body: decompressed.data,
            wire_size: decompressed.wire_size,
            content_type,
            etag,
            last_modified,
            from_cache: false,
            timing,
        })
    }

    /// Get the cache
    ///
    /// Returns a reference to the internal cache for direct manipulation.
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Get the cookie jar
    ///
    /// Returns a reference to the internal cookie jar for direct manipulation.
    pub fn cookie_jar(&self) -> &CookieJar {
        &self.cookie_jar
    }

    /// Get the profile
    ///
    /// Returns a reference to the browser profile being used.
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// Get the proxy configuration
    ///
    /// Returns the proxy URL if one is configured.
    pub fn proxy(&self) -> Option<&str> {
        self.proxy.as_deref()
    }

    /// Get the TCP proxy configuration (for HTTP/1.1 and HTTP/2)
    ///
    /// Returns the TCP proxy URL if one is configured.
    pub fn tcp_proxy(&self) -> Option<&str> {
        self.tcp_proxy.as_deref()
    }

    /// Get the UDP proxy configuration (for HTTP/3)
    ///
    /// Returns the UDP proxy URL if one is configured.
    pub fn udp_proxy(&self) -> Option<&str> {
        self.udp_proxy.as_deref()
    }

    /// Set a new proxy URL for subsequent requests
    ///
    /// This changes the proxy for all subsequent requests without recreating the client.
    /// The connector will be rebuilt on the next request.
    ///
    /// # Arguments
    ///
    /// * `proxy` - The proxy URL (e.g., "socks5://127.0.0.1:1080" or "http://proxy.example.com:8080")
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut client = Client::new(Profile::chrome_143_windows()).await?;
    ///
    /// // Make request through proxy1
    /// client.set_proxy(Some("socks5://127.0.0.1:1080".to_string()));
    /// let resp = client.get("https://httpbin.org/ip").await?;
    ///
    /// // Make request through proxy2
    /// client.set_proxy(Some("socks5://127.0.0.1:1081".to_string()));
    /// let resp = client.get("https://httpbin.org/ip").await?;
    ///
    /// // Make direct request (no proxy)
    /// client.set_proxy(None);
    /// let resp = client.get("https://httpbin.org/ip").await?;
    /// ```
    pub fn set_proxy(&mut self, proxy: Option<String>) {
        self.proxy = proxy.clone();
        self.tcp_proxy = proxy;
        // Rebuild connector on next request
    }

    /// Set TCP proxy for HTTP/1.1 and HTTP/2
    ///
    /// This sets a different proxy for TCP-based protocols (HTTP/1.1 and HTTP/2)
    /// independent of the UDP proxy used for HTTP/3.
    ///
    /// # Arguments
    ///
    /// * `proxy` - The TCP proxy URL (e.g., "socks5://127.0.0.1:1080" or "http://proxy.example.com:8080")
    pub fn set_tcp_proxy(&mut self, proxy: Option<String>) {
        self.tcp_proxy = proxy.clone();
        self.proxy = proxy;
    }

    /// Set UDP proxy for HTTP/3 (SOCKS5 with UDP ASSOCIATE)
    ///
    /// This sets a different proxy for HTTP/3 (UDP-based) independent of the TCP proxy.
    /// HTTP/3 uses QUIC which is UDP-based, so it requires a proxy that supports
    /// SOCKS5 UDP ASSOCIATE.
    ///
    /// # Arguments
    ///
    /// * `proxy` - The UDP proxy URL (typically "socks5://127.0.0.1:1080")
    pub fn set_udp_proxy(&mut self, proxy: Option<String>) {
        self.udp_proxy = proxy;
    }

    /// Get the effective proxy for a given protocol version
    ///
    /// Returns the appropriate proxy based on the HTTP protocol version.
    pub fn proxy_for_version(&self, http_version: HttpVersion) -> Option<&str> {
        match http_version {
            HttpVersion::H3 => self.udp_proxy.as_deref(),
            _ => self.tcp_proxy.as_deref(),
        }
    }

    /// Get the custom headers
    ///
    /// Returns a reference to the custom headers that will be added to all requests.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Check if HTTP/3 is enabled
    ///
    /// Returns true if HTTP/3 (QUIC) is enabled for the client.
    /// Requires the "http3" feature flag.
    pub fn http3_enabled(&self) -> bool {
        self.http3_enabled
    }

    /// Get the preferred HTTP version
    ///
    /// Returns the preferred HTTP version if one is set.
    /// If None, the client will auto-negotiate the best available protocol.
    pub fn preferred_http_version(&self) -> Option<HttpVersion> {
        self.preferred_http_version
    }

    /// Enable or disable HTTP/3
    ///
    /// When enabled, the client will attempt to use HTTP/3 (QUIC) for HTTPS requests.
    /// Requires the "http3" feature flag.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut client = Client::new(Profile::chrome_143_windows()).await?;
    /// client.set_http3_enabled(true);
    /// ```
    pub fn set_http3_enabled(&mut self, enabled: bool) {
        self.http3_enabled = enabled;
    }

    /// Set the preferred HTTP version
    ///
    /// Force the client to use a specific HTTP version.
    /// If set, the client will only attempt to use this version.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut client = Client::new(Profile::chrome_143_windows()).await?;
    /// client.set_preferred_http_version(Some(HttpVersion::H2));
    /// ```
    pub fn set_preferred_http_version(&mut self, version: Option<HttpVersion>) {
        self.preferred_http_version = version;
    }

    /// Make an HTTP/3 request (if HTTP/3 is enabled)
    ///
    /// This method attempts to make an HTTP/3 request if HTTP/3 is enabled.
    /// If HTTP/3 is not enabled or the request fails, it falls back to HTTP/2/1.
    #[cfg(feature = "http3")]
    async fn request_http3(
        &self,
        method: hyper::Method,
        url: &str,
        body: Option<Bytes>,
    ) -> Result<HttpResponse> {
        // Only use HTTP/3 if enabled and this is an HTTPS request
        if !self.http3_enabled || !url.starts_with("https://") {
            return self.request_http2(method, url, body).await;
        }

        // Try HTTP/3 first
        match self.try_request_http3(&method, url, body.clone()).await {
            Ok(resp) => Ok(resp),
            Err(_) => {
                // Fall back to HTTP/2/1 on failure
                self.request_http2(method, url, body).await
            }
        }
    }

    /// Try to make an HTTP/3 request
    #[cfg(feature = "http3")]
    async fn try_request_http3(
        &self,
        _method: &hyper::Method,
        _url: &str,
        _body: Option<Bytes>,
    ) -> Result<HttpResponse> {
        // HTTP/3 is not yet fully implemented
        // The quinn/h3 API is evolving rapidly and needs updates
        Err(SpectreError::Http(
            "HTTP/3 is not yet implemented. Please use HTTP/2 or HTTP/1.1.".to_string(),
        ))
    }

    /// Make an HTTP/2 or HTTP/1.1 request
    #[allow(dead_code)]
    async fn request_http2(
        &self,
        method: hyper::Method,
        url: &str,
        body: Option<Bytes>,
    ) -> Result<HttpResponse> {
        self.request(method, url, body).await
    }
}

/// Builder for creating [`Client`] with custom configuration
///
/// Provides a fluent interface for configuring all aspects of the HTTP client.
///
/// # Examples
///
/// ```rust,ignore
/// use spectreq::Client;
/// use spectreq::Profile;
///
/// let client = Client::builder()
///     .profile(Profile::chrome_143_windows())
///     .enable_cache(true)
///     .enable_cookies(true)
///     .proxy("socks5://127.0.0.1:1080")
///     .header("X-Custom-Header", "value")
///     .build()
///     .await?;
/// ```
pub struct ClientBuilder {
    profile: Option<Profile>,
    enable_cache: bool,
    enable_cookies: bool,
    proxy: Option<String>,
    tcp_proxy: Option<String>,
    udp_proxy: Option<String>,
    headers: Headers,
    retry_config: RetryConfig,
    redirect_config: RedirectConfig,
    timeout_config: TimeoutConfig,
    http3_enabled: bool,
    preferred_http_version: Option<HttpVersion>,
}

impl ClientBuilder {
    /// Create a new client builder with default values
    ///
    /// By default, caching and cookies are enabled.
    pub fn new() -> Self {
        Self {
            profile: None,
            enable_cache: true,
            enable_cookies: true,
            proxy: None,
            tcp_proxy: None,
            udp_proxy: None,
            headers: Headers::new(),
            retry_config: RetryConfig::default(),
            redirect_config: RedirectConfig::default(),
            timeout_config: TimeoutConfig::default(),
            http3_enabled: false,
            preferred_http_version: None,
        }
    }

    /// Set the browser profile to use
    ///
    /// If not set, defaults to Chrome 120 on Windows.
    pub fn profile(mut self, profile: Profile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Enable or disable response caching
    pub fn enable_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }

    /// Enable or disable cookie handling
    pub fn enable_cookies(mut self, enable: bool) -> Self {
        self.enable_cookies = enable;
        self
    }

    /// Set proxy URL (supports http:// and socks5://)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // SOCKS5 proxy
    /// .proxy("socks5://127.0.0.1:1080")
    ///
    /// // HTTP proxy
    /// .proxy("http://proxy.example.com:8080")
    /// ```
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        let proxy_str = proxy.into();
        self.proxy = Some(proxy_str.clone());
        self.tcp_proxy = Some(proxy_str);
        self
    }

    /// Set TCP proxy for HTTP/1.1 and HTTP/2
    ///
    /// This sets a different proxy for TCP-based protocols (HTTP/1.1 and HTTP/2)
    /// independent of the UDP proxy used for HTTP/3.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // SOCKS5 proxy for H1/H2
    /// .tcp_proxy("socks5://127.0.0.1:1080")
    ///
    /// // HTTP proxy for H1/H2
    /// .tcp_proxy("http://proxy.example.com:8080")
    /// ```
    pub fn tcp_proxy(mut self, proxy: impl Into<String>) -> Self {
        let proxy_str = proxy.into();
        self.proxy = Some(proxy_str.clone());
        self.tcp_proxy = Some(proxy_str);
        self
    }

    /// Set UDP proxy for HTTP/3 (SOCKS5 with UDP ASSOCIATE)
    ///
    /// This sets a different proxy for HTTP/3 (UDP-based) independent of the TCP proxy.
    /// HTTP/3 uses QUIC which is UDP-based, so it requires a proxy that supports
    /// SOCKS5 UDP ASSOCIATE.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // SOCKS5 proxy for HTTP/3
    /// .udp_proxy("socks5://127.0.0.1:1080")
    /// ```
    pub fn udp_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.udp_proxy = Some(proxy.into());
        self
    }

    /// Set custom headers to include with all requests
    ///
    /// These headers will be added to the browser profile's headers.
    pub fn headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    /// Add a custom header to include with all requests
    ///
    /// These headers will be added to the browser profile's headers.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set retry configuration
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Set redirect configuration
    pub fn redirect_config(mut self, config: RedirectConfig) -> Self {
        self.redirect_config = config;
        self
    }

    /// Set timeout configuration
    pub fn timeout_config(mut self, config: TimeoutConfig) -> Self {
        self.timeout_config = config;
        self
    }

    /// Enable or disable HTTP/3
    ///
    /// When enabled, the client will attempt to use HTTP/3 (QUIC) for HTTPS requests.
    /// Requires the "http3" feature flag.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let client = Client::builder()
    ///     .profile(Profile::chrome_143_windows())
    ///     .enable_http3(true)
    ///     .build()
    ///     .await?;
    /// ```
    pub fn enable_http3(mut self, enabled: bool) -> Self {
        self.http3_enabled = enabled;
        self
    }

    /// Set the preferred HTTP version
    ///
    /// Force the client to use a specific HTTP version.
    /// If set, the client will only attempt to use this version.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let client = Client::builder()
    ///     .profile(Profile::chrome_143_windows())
    ///     .preferred_http_version(HttpVersion::H2)
    ///     .build()
    ///     .await?;
    /// ```
    pub fn preferred_http_version(mut self, version: HttpVersion) -> Self {
        self.preferred_http_version = Some(version);
        self
    }

    /// Build the client
    ///
    /// Consumes the builder and returns the configured [`Client`].
    ///
    /// # Errors
    ///
    /// Returns an error if the profile is invalid.
    pub async fn build(self) -> Result<Client> {
        let profile = self
            .profile
            .unwrap_or_else(|| Profile::chrome_120_windows());
        let mut client = Client::with_options(profile, self.proxy, self.headers).await?;
        // Set split proxy configuration
        client.tcp_proxy = self.tcp_proxy;
        client.udp_proxy = self.udp_proxy;
        client.retry_config = self.retry_config;
        client.redirect_config = self.redirect_config;
        client.timeout_config = self.timeout_config;
        client.http3_enabled = self.http3_enabled;
        client.preferred_http_version = self.preferred_http_version;
        Ok(client)
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_builder() {
        let profile = Profile::chrome_120_windows();
        let client = Client::builder()
            .profile(profile)
            .enable_cache(true)
            .enable_cookies(true)
            .build()
            .await;
        assert!(client.is_ok());
    }

    #[test]
    fn test_http_response() {
        let resp = HttpResponse {
            status: 200,
            headers: vec![
                ("content-type".to_string(), "text/html".to_string()),
                ("etag".to_string(), "\"12345\"".to_string()),
            ],
            body: b"hello world".to_vec(),
            wire_size: 11,
            content_type: Some("text/html".to_string()),
            etag: Some("\"12345\"".to_string()),
            last_modified: None,
            from_cache: false,
            timing: RequestTiming::new(),
        };

        assert_eq!(resp.status, 200);
        assert_eq!(resp.text().unwrap(), "hello world");
        assert_eq!(resp.header("content-type"), Some(&"text/html".to_string()));
        assert_eq!(resp.header("etag"), Some(&"\"12345\"".to_string()));
    }

    #[test]
    fn test_http_response_json() {
        let resp = HttpResponse {
            status: 200,
            headers: vec![],
            body: br#"{"key": "value"}"#.to_vec(),
            wire_size: 17,
            content_type: Some("application/json".to_string()),
            etag: None,
            last_modified: None,
            from_cache: false,
            timing: RequestTiming::new(),
        };

        let json = resp.json().unwrap();
        assert_eq!(json["key"], "value");
    }

    #[test]
    fn test_http_response_ok() {
        let success = HttpResponse {
            status: 200,
            headers: vec![],
            body: vec![],
            wire_size: 0,
            content_type: None,
            etag: None,
            last_modified: None,
            from_cache: false,
            timing: RequestTiming::new(),
        };

        let redirect = HttpResponse {
            status: 301,
            headers: vec![],
            body: vec![],
            wire_size: 0,
            content_type: None,
            etag: None,
            last_modified: None,
            from_cache: false,
            timing: RequestTiming::new(),
        };

        let error = HttpResponse {
            status: 404,
            headers: vec![],
            body: vec![],
            wire_size: 0,
            content_type: None,
            etag: None,
            last_modified: None,
            from_cache: false,
            timing: RequestTiming::new(),
        };

        assert!(success.ok());
        assert!(redirect.ok());
        assert!(!error.ok());
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_on_status, vec![408, 429, 500, 502, 503, 504]);
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new().max_retries(5).retry_on_status(vec![200]);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_on_status, vec![200]);
    }

    #[test]
    fn test_redirect_config_default() {
        let config = RedirectConfig::default();
        assert!(config.follow);
        assert_eq!(config.max_redirects, 20);
    }

    #[test]
    fn test_redirect_config_builder() {
        let config = RedirectConfig::new().follow(false).max_redirects(10);

        assert!(!config.follow);
        assert_eq!(config.max_redirects, 10);
    }

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.connect.as_secs(), 10);
        assert_eq!(config.read.as_secs(), 30);
        assert_eq!(config.total.as_secs(), 60);
    }

    #[test]
    fn test_timeout_config_custom() {
        let config = TimeoutConfig {
            connect: Duration::from_secs(5),
            read: Duration::from_secs(15),
            total: Duration::from_secs(30),
        };

        assert_eq!(config.connect.as_secs(), 5);
        assert_eq!(config.read.as_secs(), 15);
        assert_eq!(config.total.as_secs(), 30);
    }
}
