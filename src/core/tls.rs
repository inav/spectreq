//! TLS configuration for browser impersonation
//!
//! This module provides TLS configuration building from browser profiles,
//! with optional support for post-quantum TLS key exchange.

use crate::core::profile::Profile;
use rustls::version::{TLS12, TLS13};
use rustls::{ClientConfig, RootCertStore, SupportedCipherSuite};

#[cfg(feature = "post-quantum")]
use std::sync::Arc;

/// Get default cipher suites for TLS 1.3
#[allow(dead_code)]
fn get_default_cipher_suites() -> Vec<SupportedCipherSuite> {
    // Use the default provider with ring crypto
    rustls::crypto::ring::default_provider()
        .cipher_suites
        .to_vec()
}

/// Build a rustls ClientConfig from a profile's TLS configuration
///
/// This function builds a TLS configuration that mimics the browser's
/// TLS fingerprint, including:
/// - Protocol versions (TLS 1.2, 1.3)
/// - ALPN protocols (h2, http/1.1)
/// - Platform certificates
///
/// When the "post-quantum" feature is enabled and the profile version
/// indicates Chrome 131+, this will use post-quantum hybrid key exchange.
#[allow(unused_variables)]
pub fn build_tls_config(profile: &Profile) -> Result<ClientConfig, crate::core::SpectreError> {
    let mut root_store = RootCertStore::empty();

    // Add platform certificates
    let cert_result = rustls_native_certs::load_native_certs();
    for cert in cert_result.certs {
        root_store
            .add(cert)
            .map_err(crate::core::SpectreError::Tls)?;
    }

    // Build TLS configuration - use post-quantum for Chrome 131+ if feature enabled
    #[cfg(feature = "post-quantum")]
    {
        if supports_post_quantum(profile) {
            // Use the post-quantum crypto provider with X25519Kyber768 hybrid key exchange
            let crypto_provider = rustls_post_quantum::provider();
            let config = ClientConfig::builder_with_provider(Arc::new(crypto_provider))
                .with_protocol_versions(&[&TLS13])
                .map_err(|e| crate::core::SpectreError::Tls(e))?
                .with_root_certificates(root_store)
                .with_no_client_auth();
            return Ok(config);
        }
    }

    // Standard TLS configuration (without post-quantum)
    let config =
        ClientConfig::builder_with_provider(rustls::crypto::ring::default_provider().into())
            .with_protocol_versions(&[&TLS12, &TLS13])
            .map_err(crate::core::SpectreError::Tls)?
            .with_root_certificates(root_store)
            .with_no_client_auth();

    Ok(config)
}

/// Get the JA4 (JA4) TLS fingerprint components for a profile
///
/// JA4 format: `<lengths>_<ciphers>_<extensions>_<versions>`
///
/// Where:
/// - `lengths` - counts of ciphers, extensions, and ALPN protocols
/// - `ciphers` - abbreviated cipher suite names
/// - `extensions` - TLS extension names
/// - `versions` - TLS protocol version (e.g., "13" for TLS 1.3)
///
/// Note: For full JA4 fingerprint calculation, use the ja4 module's
/// `calculate_ja4()` function instead.
pub fn get_ja4_components(profile: &Profile) -> Ja4Components {
    let cipher_count = profile.tls.cipher_suites.len() as u8;
    let extension_count = profile.tls.extensions.len() as u8;
    let alpn_count = profile.tls.alpn.len() as u8;

    // Create a simplified cipher string (first 2 chars of each cipher name, uppercase)
    let cipher_str: String = profile
        .tls
        .cipher_suites
        .iter()
        .map(|s| {
            s.chars()
                .filter(|c| c.is_ascii_uppercase())
                .take(2)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(",");

    // Create extension string
    let ext_str: String = profile.tls.extensions.join(",");

    // Versions
    let versions = if profile.tls.max_version.as_deref() == Some("1.3") {
        "13"
    } else {
        "12"
    };

    Ja4Components {
        cipher_count,
        extension_count,
        alpn_count,
        cipher_str,
        ext_str,
        versions: versions.to_string(),
        grease: profile.tls.grease,
    }
}

/// JA4 TLS fingerprint components
///
/// This is a simplified version of JA4 components. For full JA4
/// fingerprinting with proper hashing, use the ja4 module.
#[derive(Debug, Clone)]
pub struct Ja4Components {
    pub cipher_count: u8,
    pub extension_count: u8,
    pub alpn_count: u8,
    pub cipher_str: String,
    pub ext_str: String,
    pub versions: String,
    pub grease: bool,
}

impl std::fmt::Display for Ja4Components {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}_{}_{}_{}",
            self.cipher_count, self.extension_count, self.cipher_str, self.ext_str, self.versions
        )
    }
}

/// Check if the profile supports post-quantum TLS
///
/// This returns true if the profile represents a browser that supports
/// post-quantum hybrid key exchange (currently Chrome 131+).
#[allow(unused_variables)]
pub fn supports_post_quantum(profile: &Profile) -> bool {
    #[cfg(feature = "post-quantum")]
    {
        if profile.browser != crate::core::BrowserName::Chrome {
            return false;
        }

        let version_major = profile.version.split('.').next().unwrap_or("0");
        version_major.parse::<u32>().unwrap_or(0) >= 131
    }

    #[cfg(not(feature = "post-quantum"))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tls_config_default() {
        let profile = Profile::chrome_120_windows();
        let config = build_tls_config(&profile);
        assert!(config.is_ok());
    }

    #[test]
    fn test_build_tls_config_chrome_131() {
        let profile = Profile::chrome_131_windows();
        let config = build_tls_config(&profile);
        assert!(config.is_ok());
    }

    #[test]
    fn test_build_tls_config_chrome_143() {
        let profile = Profile::chrome_143_windows();
        let config = build_tls_config(&profile);
        assert!(config.is_ok());
    }

    #[test]
    fn test_ja4_components() {
        let profile = Profile::chrome_120_windows();
        let ja4 = get_ja4_components(&profile);
        assert!(ja4.cipher_count > 0);
        assert!(ja4.extension_count > 0);
        assert_eq!(ja4.versions, "13");
    }

    #[test]
    fn test_supports_post_quantum() {
        let chrome120 = Profile::chrome_120_windows();
        let _chrome131 = Profile::chrome_131_windows();
        let _chrome143 = Profile::chrome_143_windows();

        assert!(!supports_post_quantum(&chrome120));
        // Chrome 131+ should support post-quantum (if feature is enabled)
        #[cfg(feature = "post-quantum")]
        {
            assert!(supports_post_quantum(&_chrome131));
            assert!(supports_post_quantum(&_chrome143));
        }
        #[cfg(not(feature = "post-quantum"))]
        {
            // Without the feature, post-quantum is not supported
            assert!(!supports_post_quantum(&_chrome131));
            assert!(!supports_post_quantum(&_chrome143));
        }
    }
}
