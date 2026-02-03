//! HTTP/3 (QUIC) support for Spectre client
//!
//! This module provides HTTP/3 over QUIC protocol support using
//! the Quinn QUIC implementation and h3 HTTP/3 library.
//!
//! This feature requires the "http3" feature flag to be enabled.
//!
//! **NOTE**: The HTTP/3 implementation is currently experimental and
//! may not work with all servers. The quinn/h3 API is evolving rapidly.

use crate::core::{Profile, Result};

/// Check if HTTP/3 is supported (ALPN h3 is available)
pub fn supports_http3(tls_config: &rustls::ClientConfig) -> bool {
    tls_config.alpn_protocols.iter().any(|p| p == b"h3")
}

/// Add h3 ALPN to TLS config for HTTP/3 support
pub fn enable_http3(profile: &Profile) -> Result<Profile> {
    let mut updated = profile.clone();
    updated.tls.alpn.push("h3".to_string());
    Ok(updated)
}

// HTTP/3 feature-gated types - stubs for API compatibility
// Full implementation requires updating to latest quinn/h3 APIs

#[cfg(feature = "http3")]
pub type Http3Response = HttpResponse;

#[cfg(feature = "http3")]
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_http3() {
        let profile = Profile::chrome_143_windows();
        let updated = enable_http3(&profile).unwrap();
        assert!(updated.tls.alpn.contains(&"h3".to_string()));
    }
}
