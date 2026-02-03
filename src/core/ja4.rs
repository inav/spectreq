//! JA4 and JA4H fingerprinting implementation
//!
//! JA4 is a modern TLS fingerprinting method that captures:
//! - TLS version
//! - Cipher suites (abbreviated)
//! - Extensions (with order)
//! - Signature algorithms
//! - ALPN protocols
//! - GREASE indicators
//!
//! JA4H captures HTTP/2 fingerprinting via:
//! - Header names (abbreviated, with order)
//! - HTTP/2 settings
//! - Cookie presence

use sha2::{Digest, Sha256};

use crate::core::profile::{Http2Settings, Profile};

/// JA4 and JA4H fingerprint results
#[derive(Debug, Clone)]
pub struct Ja4Fingerprint {
    /// JA4 TLS fingerprint (e.g., "t13d1516h2_8daaf2a23f6c")
    pub ja4: String,
    /// JA4H HTTP fingerprint (e.g., "chrome28_0_0_0_0_0_0_0_0_0")
    pub ja4h: String,
    /// Raw JA4 components before hashing
    pub ja4_components: Ja4RawComponents,
    /// Raw JA4H components before hashing
    pub ja4h_components: Ja4hRawComponents,
}

/// Raw JA4 components (before hashing)
#[derive(Debug, Clone)]
pub struct Ja4RawComponents {
    /// TLS version (e.g., "13" for TLS 1.3)
    pub tls_version: String,
    /// Cipher count and first two ciphers (e.g., "d1516")
    pub cipher_segment: String,
    /// Extension count and abbreviated extensions (e.g., "h2")
    pub extension_segment: String,
    /// Signature algorithms segment
    pub signature_segment: String,
    /// ALPN segment
    pub alpn_segment: String,
    /// GREASE indicators
    pub grease_segment: String,
}

/// Raw JA4H components (before hashing)
#[derive(Debug, Clone)]
pub struct Ja4hRawComponents {
    /// Browser identifier + number of headers (e.g., "chrome28")
    pub header_count_segment: String,
    /// HTTP/2 settings (e.g., "0_0_0_0_0_0_0_0_0")
    pub http2_settings_segment: String,
    /// Cookie presence (0 or 1)
    pub cookie_segment: String,
    /// Language header (0, 1, or 2)
    pub language_segment: String,
}

/// Abbreviate a cipher suite to first 2 uppercase letters
///
/// Examples:
/// - TLS_AES_128_GCM_SHA256 -> AE
/// - TLS_AES_256_GCM_SHA384 -> AE
/// - TLS_CHACHA20_POLY1305_SHA256 -> CH
fn abbreviate_cipher(cipher: &str) -> String {
    // Remove TLS_ prefix if present
    let cipher = cipher.strip_prefix("TLS_").unwrap_or(cipher);

    // Get first two alphabetic characters, uppercase
    let chars: Vec<char> = cipher.chars().filter(|c| c.is_alphabetic()).collect();

    if chars.len() >= 2 {
        format!("{}{}", chars[0], chars[1]).to_uppercase()
    } else {
        cipher[..2.min(cipher.len())].to_uppercase()
    }
}

/// Abbreviate an extension name
///
/// Extension abbreviations per JA4 spec:
/// - server_name -> sn
/// - status_request -> cr
/// - supported_groups -> sg
/// - ec_point_formats -> ep
/// - signature_algorithms -> sa
/// - application_layer_protocol_negotiation -> ad
/// - key_share -> ks
/// - etc.
fn abbreviate_extension(ext: &str) -> String {
    match ext {
        "server_name" => "sn".to_string(),
        "status_request" => "cr".to_string(),
        "supported_groups" => "sg".to_string(),
        "ec_point_formats" => "ep".to_string(),
        "signature_algorithms" => "sa".to_string(),
        "signature_algorithms_cert" => "sc".to_string(),
        "application_layer_protocol_negotiation" => "ad".to_string(),
        "key_share" => "ks".to_string(),
        "psk_key_exchange_modes" => "pm".to_string(),
        "supported_versions" => "sv".to_string(),
        "compress_certificate" => "cc".to_string(),
        "record_size_limit" => "rl".to_string(),
        "encrypted_client_hello" => "ec".to_string(),
        "extended_master_secret" => "em".to_string(),
        "padding" => "pd".to_string(),
        " renegotiation_info" => "ri".to_string(),
        "session_ticket" => "st".to_string(),
        "alpn" => "ad".to_string(),
        "next_protocol_negotiation" => "pn".to_string(),
        _ => {
            // For unknown extensions, take first 2 lowercase letters
            let name = ext.replace('_', "");
            if name.len() >= 2 {
                name[..2].to_lowercase()
            } else {
                name.to_lowercase()
            }
        }
    }
}

/// Abbreviate a header name for JA4H
///
/// Header abbreviations per JA4H spec:
/// - accept -> ac
/// - accept-encoding -> ae
/// - accept-language -> al
/// - user-agent -> ua
/// - sec-ch-ua -> cu
/// - sec-ch-ua-mobile -> cm
/// - sec-ch-ua-platform -> cp
/// - sec-fetch-site -> fs
/// - sec-fetch-mode -> fm
/// - sec-fetch-dest -> fd
/// - sec-fetch-user -> fu
/// - cookie -> co
/// - etc.
#[allow(dead_code)]
fn abbreviate_header(header: &str) -> String {
    match header.to_lowercase().as_str() {
        "accept" => "ac".to_string(),
        "accept-encoding" => "ae".to_string(),
        "accept-language" => "al".to_string(),
        "user-agent" => "ua".to_string(),
        "sec-ch-ua" => "cu".to_string(),
        "sec-ch-ua-mobile" => "cm".to_string(),
        "sec-ch-ua-platform" => "cp".to_string(),
        "sec-ch-ua-arch" => "ca".to_string(),
        "sec-ch-ua-bitness" => "cb".to_string(),
        "sec-ch-ua-full-version" => "cv".to_string(),
        "sec-ch-ua-model" => "cd".to_string(),
        "sec-fetch-site" => "fs".to_string(),
        "sec-fetch-mode" => "fm".to_string(),
        "sec-fetch-dest" => "fd".to_string(),
        "sec-fetch-user" => "fu".to_string(),
        "cookie" => "co".to_string(),
        "referer" => "rf".to_string(),
        "authorization" => "au".to_string(),
        "content-type" => "ct".to_string(),
        "content-length" => "cl".to_string(),
        "cache-control" => "cc".to_string(),
        "pragma" => "pg".to_string(),
        "upgrade-insecure-requests" => "ui".to_string(),
        _ => {
            // For unknown headers, take first 2 lowercase letters after removing hyphens
            let name = header.replace('-', "");
            if name.len() >= 2 {
                name[..2].to_lowercase()
            } else {
                name.to_lowercase()
            }
        }
    }
}

/// Get browser identifier for JA4H
fn get_browser_identifier(profile: &Profile) -> &'static str {
    match (profile.browser, profile.os) {
        (crate::core::BrowserName::Chrome, crate::core::OS::Windows) => "chrome",
        (crate::core::BrowserName::Chrome, _) => "chrome",
        (crate::core::BrowserName::Firefox, _) => "firefox",
        (crate::core::BrowserName::Safari, _) => "safari",
        (crate::core::BrowserName::Edge, _) => "edge",
    }
}

/// Calculate JA4 fingerprint from a profile
pub fn calculate_ja4(profile: &Profile) -> Ja4Fingerprint {
    let tls = &profile.tls;

    // TLS version: 13 for TLS 1.3, 12 for TLS 1.2
    let tls_version = if tls.max_version.as_deref() == Some("1.3") {
        "13"
    } else if tls.max_version.as_deref() == Some("1.2") {
        "12"
    } else {
        "13" // Default to TLS 1.3
    };

    // Cipher segment: count (hex) + abbreviated ciphers
    let cipher_count = tls.cipher_suites.len();
    let cipher_abbrevs: Vec<String> = tls
        .cipher_suites
        .iter()
        .take(2) // Only first 2 ciphers for JA4
        .map(|c| abbreviate_cipher(c))
        .collect();

    let cipher_segment = format!("{:02x}{}", cipher_count, cipher_abbrevs.join(""));

    // Extension segment: count (hex) + abbreviated extensions (in order!)
    let ext_count = tls.extensions.len();
    let ext_abbrevs: Vec<String> = tls
        .extensions
        .iter()
        .map(|e| abbreviate_extension(e))
        .collect();

    let extension_segment = format!("{:02x}{}", ext_count, ext_abbrevs.join(""));

    // Check for GREASE
    let has_grease = if tls.grease { "i" } else { "" };

    // Signature algorithms (hardcoded for now based on browser defaults)
    let signature_segment = "s"; // Simplified - would need actual sig algs from TLS config

    // ALPN segment
    let alpn_count = tls.alpn.len();
    let alpn_segment = if alpn_count > 0 {
        format!("{:02x}{}", alpn_count, tls.alpn.join(","))
    } else {
        "00".to_string()
    };

    // Build JA4 string: t<version><cipher><ext><sig><alpn>_<grease><hash>
    let ja4_part = format!(
        "t{}{}{}{}{}",
        tls_version, cipher_segment, extension_segment, signature_segment, alpn_segment
    );

    // Hash of the full JA4 components
    let full_string = format!(
        "{}{}{}{}{}",
        tls_version, cipher_segment, extension_segment, signature_segment, alpn_segment
    );
    let hash = format!("{:x}", Sha256::digest(full_string.as_bytes()));
    let hash_short = &hash[..12]; // First 12 bytes of hash

    let ja4 = format!("{}_{}{}", ja4_part, has_grease, hash_short);

    // Calculate JA4H
    let headers = profile.get_ordered_headers();
    let header_count = headers.len();

    // Browser identifier + header count
    let browser_id = get_browser_identifier(profile);
    let header_count_segment = format!("{}{}", browser_id, header_count);

    // HTTP/2 settings segment
    let http2_settings_segment = format_http2_settings(&profile.http2);

    // Cookie segment
    let cookie_segment = if headers.contains_key("cookie") {
        "1"
    } else {
        "0"
    };

    // Language segment
    let language_segment = if headers.contains_key("accept-language") {
        "1"
    } else {
        "0"
    };

    let ja4h = format!(
        "{}_{}_{}_{}",
        header_count_segment, http2_settings_segment, cookie_segment, language_segment
    );

    let ja4_components = Ja4RawComponents {
        tls_version: tls_version.to_string(),
        cipher_segment,
        extension_segment,
        signature_segment: signature_segment.to_string(),
        alpn_segment,
        grease_segment: has_grease.to_string(),
    };

    let ja4h_components = Ja4hRawComponents {
        header_count_segment,
        http2_settings_segment,
        cookie_segment: cookie_segment.to_string(),
        language_segment: language_segment.to_string(),
    };

    Ja4Fingerprint {
        ja4,
        ja4h,
        ja4_components,
        ja4h_components,
    }
}

/// Format HTTP/2 settings for JA4H
///
/// JA4H captures HTTP/2 settings as underscore-separated values
fn format_http2_settings(settings: &Http2Settings) -> String {
    // Standard HTTP/2 settings for JA4H:
    // 0: HEADER_TABLE_SIZE
    // 1: ENABLE_PUSH
    // 2: MAX_CONCURRENT_STREAMS
    // 3: INITIAL_WINDOW_SIZE
    // 4: MAX_FRAME_SIZE
    // 5: MAX_HEADER_LIST_SIZE
    // 6: (reserved)
    // 7: (reserved)
    // 8: (reserved)

    format!(
        "{}_{}_{}_{}_{}_0_0_0",
        settings.header_table_size,
        if settings.enable_push { 1 } else { 0 },
        if settings.max_concurrent_streams == u32::MAX {
            0xFFFFFFFFu32
        } else {
            settings.max_concurrent_streams
        },
        settings.initial_window_size,
        settings.max_frame_size,
    )
}

/// Calculate JA4 from raw TLS handshake data
///
/// This function is useful when you have actual TLS handshake data
/// and want to calculate the JA4 fingerprint from it.
pub fn calculate_ja4_from_handshake(
    tls_version: u16,
    cipher_suites: &[u16],
    extensions: &[u16],
    _signature_algorithms: &[u16],
    _alpn_protocols: &[&[u8]],
    has_grease: bool,
) -> String {
    // TLS version
    let version_str = match tls_version {
        0x0304 => "13", // TLS 1.3
        0x0303 => "12", // TLS 1.2
        0x0302 => "11", // TLS 1.1
        0x0301 => "10", // TLS 1.0
        _ => "13",      // Default to TLS 1.3
    };

    // Cipher segment
    let cipher_count = cipher_suites.len();
    let cipher_abbrevs: Vec<String> = cipher_suites
        .iter()
        .take(2)
        .map(|c| format!("{:02x}", c))
        .collect();
    let cipher_segment = format!("{:02x}{}", cipher_count, cipher_abbrevs.join(""));

    // Extension segment
    let ext_count = extensions.len();
    let ext_abbrevs: Vec<String> = extensions.iter().map(|e| format!("{:02x}", e)).collect();
    let extension_segment = format!("{:02x}{}", ext_count, ext_abbrevs.join(""));

    // GREASE indicator
    let grease = if has_grease { "i" } else { "" };

    // Signature and ALPN (simplified)
    let signature_segment = "s";
    let alpn_segment = "02h2,http/1.1";

    let ja4_part = format!(
        "t{}{}{}{}{}",
        version_str, cipher_segment, extension_segment, signature_segment, alpn_segment
    );

    let full_string = format!(
        "{}{}{}{}{}",
        version_str, cipher_segment, extension_segment, signature_segment, alpn_segment
    );
    let hash = format!("{:x}", Sha256::digest(full_string.as_bytes()));
    let hash_short = &hash[..12];

    format!("{}_{}{}", ja4_part, grease, hash_short)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abbreviate_cipher() {
        assert_eq!(abbreviate_cipher("TLS_AES_128_GCM_SHA256"), "AE");
        assert_eq!(abbreviate_cipher("TLS_AES_256_GCM_SHA384"), "AE");
        assert_eq!(abbreviate_cipher("TLS_CHACHA20_POLY1305_SHA256"), "CH");
    }

    #[test]
    fn test_abbreviate_extension() {
        assert_eq!(abbreviate_extension("server_name"), "sn");
        assert_eq!(abbreviate_extension("key_share"), "ks");
        assert_eq!(
            abbreviate_extension("application_layer_protocol_negotiation"),
            "ad"
        );
    }

    #[test]
    fn test_abbreviate_header() {
        assert_eq!(abbreviate_header("accept"), "ac");
        assert_eq!(abbreviate_header("user-agent"), "ua");
        assert_eq!(abbreviate_header("sec-ch-ua"), "cu");
        assert_eq!(abbreviate_header("cookie"), "co");
    }

    #[test]
    fn test_format_http2_settings() {
        let settings = Http2Settings {
            initial_window_size: 6291456,
            max_concurrent_streams: 256,
            max_frame_size: 16384,
            header_table_size: 65536,
            enable_push: false,
        };

        let result = format_http2_settings(&settings);
        assert_eq!(result, "65536_0_256_6291456_16384_0_0_0");
    }

    #[test]
    fn test_calculate_ja4() {
        let profile = Profile::chrome_143_windows();
        let fingerprint = calculate_ja4(&profile);

        // JA4 should start with "t13" (TLS 1.3)
        assert!(fingerprint.ja4.starts_with("t13"));

        // JA4H should start with "chrome"
        assert!(fingerprint.ja4h.starts_with("chrome"));
    }
}
