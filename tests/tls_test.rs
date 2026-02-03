//! TLS configuration and fingerprint tests
//!
//! Tests for TLS configuration building and JA4 fingerprint generation.

use spectreq::{
    Profile, build_tls_config, get_ja4_components, supports_post_quantum,
    calculate_ja4,
};

#[test]
fn test_build_tls_config_chrome_120() {
    let profile = Profile::chrome_120_windows();
    let config = build_tls_config(&profile);
    assert!(config.is_ok(), "Should build TLS config for Chrome 120");
}

#[test]
fn test_build_tls_config_chrome_143() {
    let profile = Profile::chrome_143_windows();
    let config = build_tls_config(&profile);
    assert!(config.is_ok(), "Should build TLS config for Chrome 143");
}

#[test]
fn test_build_tls_config_firefox() {
    let profile = Profile::firefox_121_windows();
    let config = build_tls_config(&profile);
    assert!(config.is_ok(), "Should build TLS config for Firefox");
}

#[test]
fn test_build_tls_config_safari() {
    let profile = Profile::safari_17_macos();
    let config = build_tls_config(&profile);
    assert!(config.is_ok(), "Should build TLS config for Safari");
}

#[test]
fn test_build_tls_config_edge() {
    let profile = Profile::edge_120_windows();
    let config = build_tls_config(&profile);
    assert!(config.is_ok(), "Should build TLS config for Edge");
}

#[test]
fn test_ja4_components_chrome() {
    let profile = Profile::chrome_143_windows();
    let ja4 = get_ja4_components(&profile);
    
    // Chrome should have reasonable cipher and extension counts
    assert!(ja4.cipher_count > 0, "Should have cipher suites");
    assert!(ja4.extension_count > 0, "Should have extensions");
    assert!(ja4.alpn_count > 0, "Should have ALPN protocols");
    assert!(ja4.grease, "Chrome should use GREASE");
    assert_eq!(ja4.versions, "13", "Chrome should prefer TLS 1.3");
}

#[test]
fn test_ja4_components_firefox() {
    let profile = Profile::firefox_121_windows();
    let ja4 = get_ja4_components(&profile);
    
    assert!(ja4.cipher_count > 0);
    // Firefox doesn't use GREASE
    assert!(!ja4.grease, "Firefox should not use GREASE");
}

#[test]
fn test_ja4_components_display() {
    let profile = Profile::chrome_120_windows();
    let ja4 = get_ja4_components(&profile);
    
    let display = format!("{}", ja4);
    assert!(!display.is_empty(), "JA4 display should not be empty");
    assert!(display.contains('_'), "JA4 should contain separator");
}

#[test]
fn test_supports_post_quantum_chrome_120() {
    let profile = Profile::chrome_120_windows();
    // Chrome 120 doesn't support post-quantum
    #[cfg(not(feature = "post-quantum"))]
    {
        assert!(!supports_post_quantum(&profile));
    }
}

#[test]
fn test_supports_post_quantum_chrome_131() {
    let profile = Profile::chrome_131_windows();
    
    #[cfg(feature = "post-quantum")]
    {
        assert!(supports_post_quantum(&profile), "Chrome 131+ should support PQ");
    }
    
    #[cfg(not(feature = "post-quantum"))]
    {
        assert!(!supports_post_quantum(&profile));
    }
}

#[test]
fn test_supports_post_quantum_chrome_143() {
    let _profile = Profile::chrome_143_windows();
    
    #[cfg(feature = "post-quantum")]
    {
        assert!(supports_post_quantum(&_profile), "Chrome 143 should support PQ");
    }
}

#[test]
fn test_supports_post_quantum_firefox() {
    let profile = Profile::firefox_121_windows();
    // Firefox doesn't support post-quantum yet
    assert!(!supports_post_quantum(&profile));
}

#[test]
fn test_ja4_fingerprint_structure() {
    let profile = Profile::chrome_143_windows();
    let ja4 = calculate_ja4(&profile);
    
    // JA4 fingerprint should not be empty
    assert!(!ja4.ja4.is_empty());
    // JA4 format: t13d1516h2_hash_hash
    // Should contain underscore separators
    let parts: Vec<&str> = ja4.ja4.split('_').collect();
    assert!(parts.len() >= 2, "JA4 should have multiple parts");
}

#[test]
fn test_different_profiles_different_fingerprints() {
    let chrome = Profile::chrome_143_windows();
    let firefox = Profile::firefox_121_windows();
    
    let ja4_chrome = calculate_ja4(&chrome);
    let ja4_firefox = calculate_ja4(&firefox);
    
    // Different browsers should have different fingerprints
    assert_ne!(ja4_chrome.ja4, ja4_firefox.ja4,
        "Chrome and Firefox should have different JA4 fingerprints");
}

#[test]
fn test_same_profile_consistent_fingerprint() {
    let profile1 = Profile::chrome_143_windows();
    let profile2 = Profile::chrome_143_windows();
    
    let ja4_1 = calculate_ja4(&profile1);
    let ja4_2 = calculate_ja4(&profile2);
    
    // Same profile should produce same fingerprint
    assert_eq!(ja4_1.ja4, ja4_2.ja4,
        "Same profile should produce consistent JA4 fingerprint");
}

#[test]
fn test_browser_versions_fingerprint_stability() {
    // Same browser family should have similar structure
    let chrome_120 = Profile::chrome_120_windows();
    let chrome_143 = Profile::chrome_143_windows();
    
    let ja4_120 = get_ja4_components(&chrome_120);
    let ja4_143 = get_ja4_components(&chrome_143);
    
    // Both should use GREASE and TLS 1.3
    assert_eq!(ja4_120.grease, ja4_143.grease);
    assert_eq!(ja4_120.versions, ja4_143.versions);
}
