//! Ordered headers and client hints for browser fingerprinting
//!
//! This module provides ordered headers (critical for JA4H fingerprinting),
//! Sec-Fetch-* header generation, and Client Hints support.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ordered headers that preserve insertion order for JA4H fingerprinting
pub type OrderedHeaders = IndexMap<String, String>;

/// Fetch mode for Sec-Fetch-Mode header
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FetchMode {
    /// Navigate mode (top-level navigation)
    #[serde(rename = "navigate")]
    Navigate,
    /// CORS mode (cross-origin request with CORS)
    #[serde(rename = "cors")]
    Cors,
    /// No-CORS mode (opaque response)
    #[serde(rename = "no-cors")]
    NoCors,
    /// Same-origin mode
    #[serde(rename = "same-origin")]
    SameOrigin,
}

impl FetchMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            FetchMode::Navigate => "navigate",
            FetchMode::Cors => "cors",
            FetchMode::NoCors => "no-cors",
            FetchMode::SameOrigin => "same-origin",
        }
    }
}

/// Fetch destination for Sec-Fetch-Dest header
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FetchDest {
    /// Document (top-level page)
    #[serde(rename = "document")]
    Document,
    /// Empty (no specific destination)
    #[serde(rename = "empty")]
    Empty,
    /// Image resource
    #[serde(rename = "image")]
    Image,
    /// Script resource
    #[serde(rename = "script")]
    Script,
    /// Style resource
    #[serde(rename = "style")]
    Style,
    /// Font resource
    #[serde(rename = "font")]
    Font,
    /// Audio/video resource
    #[serde(rename = "audio")]
    Audio,
    /// Video resource
    #[serde(rename = "video")]
    Video,
    /// WebSocket connection
    #[serde(rename = "websocket")]
    WebSocket,
    /// Form submission
    #[serde(rename = "form")]
    Form,
    /// Frame/nested browsing context
    #[serde(rename = "frame")]
    Frame,
    /// IFrame
    #[serde(rename = "iframe")]
    IFrame,
    /// Nested navigation/worker
    #[serde(rename = "nested-navigation")]
    NestedNavigation,
    /// Object
    #[serde(rename = "object")]
    Object,
    /// Report
    #[serde(rename = "report")]
    Report,
    /// Manifest
    #[serde(rename = "manifest")]
    Manifest,
    /// XSLT transformation
    #[serde(rename = "xslt")]
    Xslt,
    /// Ping
    #[serde(rename = "ping")]
    Ping,
}

impl FetchDest {
    pub fn as_str(&self) -> &'static str {
        match self {
            FetchDest::Document => "document",
            FetchDest::Empty => "empty",
            FetchDest::Image => "image",
            FetchDest::Script => "script",
            FetchDest::Style => "style",
            FetchDest::Font => "font",
            FetchDest::Audio => "audio",
            FetchDest::Video => "video",
            FetchDest::WebSocket => "websocket",
            FetchDest::Form => "form",
            FetchDest::Frame => "frame",
            FetchDest::IFrame => "iframe",
            FetchDest::NestedNavigation => "nested-navigation",
            FetchDest::Object => "object",
            FetchDest::Report => "report",
            FetchDest::Manifest => "manifest",
            FetchDest::Xslt => "xslt",
            FetchDest::Ping => "ping",
        }
    }
}

/// Fetch site type for Sec-Fetch-Site header
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FetchSite {
    /// Same origin (same scheme, host, port)
    #[serde(rename = "same-origin")]
    SameOrigin,
    /// Same site (same eTLD+1)
    #[serde(rename = "same-site")]
    SameSite,
    /// Cross site (different eTLD+1)
    #[serde(rename = "cross-site")]
    CrossSite,
    /// None (for navigations triggered by the user)
    #[serde(rename = "none")]
    None,
}

impl FetchSite {
    pub fn as_str(&self) -> &'static str {
        match self {
            FetchSite::SameOrigin => "same-origin",
            FetchSite::SameSite => "same-site",
            FetchSite::CrossSite => "cross-site",
            FetchSite::None => "none",
        }
    }
}

/// Request context for generating Sec-Fetch-* headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Fetch mode (navigate, cors, no-cors, same-origin)
    pub mode: FetchMode,
    /// Fetch destination (document, empty, image, script, etc.)
    pub dest: FetchDest,
    /// Fetch site type (same-origin, same-site, cross-site, none)
    pub site: FetchSite,
    /// Whether the request was user-triggered
    pub user_triggered: bool,
    /// The referer URL for this request
    pub referer: Option<String>,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            mode: FetchMode::Cors,
            dest: FetchDest::Empty,
            site: FetchSite::CrossSite,
            user_triggered: true,
            referer: None,
        }
    }
}

impl RequestContext {
    /// Create a context for a top-level navigation
    pub fn navigation() -> Self {
        Self {
            mode: FetchMode::Navigate,
            dest: FetchDest::Document,
            site: FetchSite::None,
            user_triggered: true,
            referer: None,
        }
    }

    /// Create a context for a same-origin request
    pub fn same_origin() -> Self {
        Self {
            mode: FetchMode::Cors,
            dest: FetchDest::Empty,
            site: FetchSite::SameOrigin,
            user_triggered: true,
            referer: None,
        }
    }

    /// Create a context for a cross-site request
    pub fn cross_site() -> Self {
        Self {
            mode: FetchMode::Cors,
            dest: FetchDest::Empty,
            site: FetchSite::CrossSite,
            user_triggered: true,
            referer: None,
        }
    }

    /// Set the referer URL
    pub fn with_referer(mut self, referer: String) -> Self {
        self.referer = Some(referer);
        self
    }

    /// Set the fetch mode
    pub fn with_mode(mut self, mode: FetchMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the fetch destination
    pub fn with_dest(mut self, dest: FetchDest) -> Self {
        self.dest = dest;
        self
    }
}

/// Client Hints for Sec-CH-UA-* headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHints {
    /// Sec-CH-UA: Brand and version list (e.g., `"Chromium";v="131", "Google Chrome";v="131", "Not:A-Brand";v="24"`)
    pub ua: String,
    /// Sec-CH-UA-Mobile: ?1 or ?0
    pub ua_mobile: String,
    /// Sec-CH-UA-Platform: Platform name
    pub ua_platform: String,
    /// Sec-CH-UA-Arch: CPU architecture (optional)
    pub ua_arch: Option<String>,
    /// Sec-CH-UA-Bitness: CPU bitness (optional)
    pub ua_bitness: Option<String>,
    /// Sec-CH-UA-Model: Device model (optional, mostly for mobile)
    pub ua_model: Option<String>,
    /// Sec-CH-UA-Full-Version: Full browser version (optional, high-entropy)
    pub ua_full_version: Option<String>,
}

impl Default for ClientHints {
    fn default() -> Self {
        Self {
            ua: String::new(),
            ua_mobile: "?0".to_string(),
            ua_platform: String::new(),
            ua_arch: None,
            ua_bitness: None,
            ua_model: None,
            ua_full_version: None,
        }
    }
}

impl ClientHints {
    /// Create a new ClientHints for Chrome 143 on Windows
    pub fn chrome_143_windows() -> Self {
        Self {
            ua: r#""Chromium";v="143", "Google Chrome";v="143", "Not:A-Brand";v="24""#.to_string(),
            ua_mobile: "?0".to_string(),
            ua_platform: r#""Windows""#.to_string(),
            ua_arch: Some(r#""x86""#.to_string()),
            ua_bitness: Some(r#""64""#.to_string()),
            ua_model: None,
            ua_full_version: Some(r#""143.0.0.0""#.to_string()),
        }
    }

    /// Create a new ClientHints for Chrome 143 on macOS
    pub fn chrome_143_macos() -> Self {
        Self {
            ua: r#""Chromium";v="143", "Google Chrome";v="143", "Not:A-Brand";v="24""#.to_string(),
            ua_mobile: "?0".to_string(),
            ua_platform: r#""macOS""#.to_string(),
            ua_arch: None,
            ua_bitness: None,
            ua_model: None,
            ua_full_version: Some(r#""143.0.0.0""#.to_string()),
        }
    }

    /// Create a new ClientHints for Chrome 143 on Linux
    pub fn chrome_143_linux() -> Self {
        Self {
            ua: r#""Chromium";v="143", "Google Chrome";v="143", "Not:A-Brand";v="24""#.to_string(),
            ua_mobile: "?0".to_string(),
            ua_platform: r#""Linux""#.to_string(),
            ua_arch: Some(r#""x86""#.to_string()),
            ua_bitness: Some(r#""64""#.to_string()),
            ua_model: None,
            ua_full_version: Some(r#""143.0.0.0""#.to_string()),
        }
    }

    /// Create a new ClientHints for Chrome 143 on Android
    pub fn chrome_143_android() -> Self {
        Self {
            ua: r#""Chromium";v="143", "Google Chrome";v="143", "Not:A-Brand";v="24""#.to_string(),
            ua_mobile: "?1".to_string(),
            ua_platform: r#""Android""#.to_string(),
            ua_arch: None,
            ua_bitness: None,
            ua_model: None,
            ua_full_version: Some(r#""143.0.6099.43""#.to_string()),
        }
    }

    /// Create a new ClientHints for Chrome 120 (legacy)
    pub fn chrome_120_windows() -> Self {
        Self {
            ua: r#""Chromium";v="120", "Not:A-Brand";v="99""#.to_string(),
            ua_mobile: "?0".to_string(),
            ua_platform: r#""Windows""#.to_string(),
            ua_arch: Some(r#""x86""#.to_string()),
            ua_bitness: Some(r#""64""#.to_string()),
            ua_model: None,
            ua_full_version: None,
        }
    }

    /// Create a new ClientHints for Chrome 131
    pub fn chrome_131_windows() -> Self {
        Self {
            ua: r#""Chromium";v="131", "Google Chrome";v="131", "Not:A-Brand";v="24""#.to_string(),
            ua_mobile: "?0".to_string(),
            ua_platform: r#""Windows""#.to_string(),
            ua_arch: Some(r#""x86""#.to_string()),
            ua_bitness: Some(r#""64""#.to_string()),
            ua_model: None,
            ua_full_version: Some(r#""131.0.0.0""#.to_string()),
        }
    }

    /// Create a new ClientHints for Chrome 133
    pub fn chrome_133_windows() -> Self {
        Self {
            ua: r#""Chromium";v="133", "Google Chrome";v="133", "Not:A-Brand";v="24""#.to_string(),
            ua_mobile: "?0".to_string(),
            ua_platform: r#""Windows""#.to_string(),
            ua_arch: Some(r#""x86""#.to_string()),
            ua_bitness: Some(r#""64""#.to_string()),
            ua_model: None,
            ua_full_version: Some(r#""133.0.0.0""#.to_string()),
        }
    }

    /// Create a new ClientHints for Chrome 141
    pub fn chrome_141_windows() -> Self {
        Self {
            ua: r#""Chromium";v="141", "Google Chrome";v="141", "Not:A-Brand";v="24""#.to_string(),
            ua_mobile: "?0".to_string(),
            ua_platform: r#""Windows""#.to_string(),
            ua_arch: Some(r#""x86""#.to_string()),
            ua_bitness: Some(r#""64""#.to_string()),
            ua_model: None,
            ua_full_version: Some(r#""141.0.0.0""#.to_string()),
        }
    }

    /// Convert ClientHints to an ordered headers map
    pub fn to_headers(&self) -> OrderedHeaders {
        let mut headers = OrderedHeaders::new();

        if !self.ua.is_empty() {
            headers.insert("sec-ch-ua".to_string(), self.ua.clone());
        }
        if !self.ua_mobile.is_empty() {
            headers.insert("sec-ch-ua-mobile".to_string(), self.ua_mobile.clone());
        }
        if !self.ua_platform.is_empty() {
            headers.insert("sec-ch-ua-platform".to_string(), self.ua_platform.clone());
        }
        if let Some(ref arch) = self.ua_arch {
            headers.insert("sec-ch-ua-arch".to_string(), arch.clone());
        }
        if let Some(ref bitness) = self.ua_bitness {
            headers.insert("sec-ch-ua-bitness".to_string(), bitness.clone());
        }
        if let Some(ref model) = self.ua_model {
            headers.insert("sec-ch-ua-model".to_string(), model.clone());
        }
        if let Some(ref full_version) = self.ua_full_version {
            headers.insert("sec-ch-ua-full-version".to_string(), full_version.clone());
        }

        headers
    }
}

/// Generate Sec-Fetch-* headers based on the request context
///
/// These headers provide information about the origin and context of a request,
/// which helps servers prevent CSRF attacks and unauthorized API access.
///
/// Returns an OrderedHeaders map with the appropriate Sec-Fetch-* headers.
pub fn generate_sec_fetch_headers(ctx: &RequestContext) -> OrderedHeaders {
    let mut headers = OrderedHeaders::new();

    // Sec-Fetch-Site: Indicates the relationship between the request initiator's origin
    // and the target origin
    headers.insert("sec-fetch-site".to_string(), ctx.site.as_str().to_string());

    // Sec-Fetch-Mode: Indicates the request's mode
    headers.insert("sec-fetch-mode".to_string(), ctx.mode.as_str().to_string());

    // Sec-Fetch-Dest: Indicates the request's destination
    headers.insert("sec-fetch-dest".to_string(), ctx.dest.as_str().to_string());

    // Sec-Fetch-User: Only present for user-triggered navigations
    if ctx.user_triggered && matches!(ctx.mode, FetchMode::Navigate) {
        headers.insert("sec-fetch-user".to_string(), "?1".to_string());
    }

    headers
}

/// Generate Client Hints for a given profile
///
/// This is a convenience function that generates ClientHints based on the
/// browser name, OS, and version. For more precise control, use the
/// ClientHints constructors directly.
pub fn generate_client_hints(
    browser: crate::core::BrowserName,
    os: crate::core::OS,
    version: &str,
) -> ClientHints {
    match (browser, os) {
        (crate::core::BrowserName::Chrome, crate::core::OS::Windows) => {
            if version.starts_with("143") {
                ClientHints::chrome_143_windows()
            } else if version.starts_with("141") {
                ClientHints::chrome_141_windows()
            } else if version.starts_with("133") {
                ClientHints::chrome_133_windows()
            } else if version.starts_with("131") {
                ClientHints::chrome_131_windows()
            } else if version.starts_with("120") {
                ClientHints::chrome_120_windows()
            } else {
                // Default to 143 for newer versions
                ClientHints::chrome_143_windows()
            }
        }
        (crate::core::BrowserName::Chrome, crate::core::OS::MacOS) => {
            if version.starts_with("143") {
                ClientHints::chrome_143_macos()
            } else {
                // Default to 143 for newer versions
                ClientHints::chrome_143_macos()
            }
        }
        (crate::core::BrowserName::Chrome, crate::core::OS::Linux) => {
            if version.starts_with("143") {
                ClientHints::chrome_143_linux()
            } else {
                // Default to 143 for newer versions
                ClientHints::chrome_143_linux()
            }
        }
        (crate::core::BrowserName::Chrome, crate::core::OS::Android) => {
            if version.starts_with("143") {
                ClientHints::chrome_143_android()
            } else {
                // Default to 143 for newer versions
                ClientHints::chrome_143_android()
            }
        }
        _ => ClientHints::default(), // Other browsers don't use Client Hints extensively
    }
}

/// Convert a HashMap to OrderedHeaders
///
/// This helper function allows converting existing HashMap-based headers
/// to the ordered IndexMap-based format.
pub fn hashmap_to_ordered(hash_map: &HashMap<String, String>) -> OrderedHeaders {
    let mut ordered = OrderedHeaders::new();
    for (key, value) in hash_map.iter() {
        ordered.insert(key.clone(), value.clone());
    }
    ordered
}

/// Merge multiple OrderedHeaders, with later headers taking precedence
///
/// This is useful for combining default headers, profile-specific headers,
/// and per-request custom headers.
pub fn merge_ordered_headers(headers: &[OrderedHeaders]) -> OrderedHeaders {
    let mut result = OrderedHeaders::new();
    for header_map in headers {
        for (key, value) in header_map.iter() {
            result.insert(key.clone(), value.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordered_headers_preserve_order() {
        let mut headers = OrderedHeaders::new();
        headers.insert("z".to_string(), "1".to_string());
        headers.insert("a".to_string(), "2".to_string());
        headers.insert("m".to_string(), "3".to_string());

        let keys: Vec<&String> = headers.keys().collect();
        assert_eq!(keys, vec!["z", "a", "m"]);
    }

    #[test]
    fn test_sec_fetch_headers_navigation() {
        let ctx = RequestContext::navigation();
        let headers = generate_sec_fetch_headers(&ctx);

        assert_eq!(headers.get("sec-fetch-site"), Some(&"none".to_string()));
        assert_eq!(headers.get("sec-fetch-mode"), Some(&"navigate".to_string()));
        assert_eq!(headers.get("sec-fetch-dest"), Some(&"document".to_string()));
        assert_eq!(headers.get("sec-fetch-user"), Some(&"?1".to_string()));
    }

    #[test]
    fn test_sec_fetch_headers_cross_site() {
        let ctx = RequestContext::cross_site();
        let headers = generate_sec_fetch_headers(&ctx);

        assert_eq!(
            headers.get("sec-fetch-site"),
            Some(&"cross-site".to_string())
        );
        assert_eq!(headers.get("sec-fetch-mode"), Some(&"cors".to_string()));
        assert_eq!(headers.get("sec-fetch-dest"), Some(&"empty".to_string()));
        assert!(headers.get("sec-fetch-user").is_none());
    }

    #[test]
    fn test_client_hints_chrome_143() {
        let hints = ClientHints::chrome_143_windows();
        let headers = hints.to_headers();

        assert!(headers.contains_key("sec-ch-ua"));
        assert!(headers.contains_key("sec-ch-ua-mobile"));
        assert!(headers.contains_key("sec-ch-ua-platform"));
        assert!(headers.contains_key("sec-ch-ua-arch"));
        assert!(headers.contains_key("sec-ch-ua-bitness"));
    }

    #[test]
    fn test_client_hints_android_mobile() {
        let hints = ClientHints::chrome_143_android();
        assert_eq!(hints.ua_mobile, "?1");
    }

    #[test]
    fn test_merge_ordered_headers() {
        let mut headers1 = OrderedHeaders::new();
        headers1.insert("a".to_string(), "1".to_string());
        headers1.insert("b".to_string(), "2".to_string());

        let mut headers2 = OrderedHeaders::new();
        headers2.insert("b".to_string(), "3".to_string()); // Override
        headers2.insert("c".to_string(), "4".to_string());

        let merged = merge_ordered_headers(&[headers1, headers2]);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged.get("a"), Some(&"1".to_string()));
        assert_eq!(merged.get("b"), Some(&"3".to_string()));
        assert_eq!(merged.get("c"), Some(&"4".to_string()));
    }
}
