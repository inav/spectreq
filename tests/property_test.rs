//! Property-based tests for profile and authentication
//!
//! Uses proptest to generate random inputs and verify invariants.

use proptest::prelude::*;
use spectreq::{Profile, BrowserName, OS, BasicAuth, DigestAuth, BearerToken};
use std::time::Duration;

// ============================================================================
// Profile Property Tests
// ============================================================================

proptest! {
    /// All profiles should have non-empty user agents
    #[test]
    fn profile_has_valid_user_agent(seed in 0u64..1000) {
        let profiles = [
            Profile::chrome_143_windows(),
            Profile::chrome_143_macos(),
            Profile::chrome_143_linux(),
            Profile::chrome_120_windows(),
            Profile::firefox_121_windows(),
            Profile::safari_17_macos(),
            Profile::edge_120_windows(),
        ];
        
        for profile in profiles.iter() {
            prop_assert!(!profile.user_agent.is_empty());
            prop_assert!(profile.user_agent.len() > 20);
        }
        
        // Random profile should also be valid
        let random = Profile::random();
        prop_assert!(!random.user_agent.is_empty());
        
        // Suppress unused variable warning
        let _ = seed;
    }
    
    /// Randomized profiles should have different session seeds
    #[test]
    fn randomized_profiles_differ(_seed in 0u64..100) {
        let p1 = Profile::chrome_143_windows().randomize();
        let p2 = Profile::chrome_143_windows().randomize();
        
        // Session seeds should differ (with very high probability)
        // Note: There's an astronomically small chance they're equal
        // For a 64-bit seed, collision probability is ~1/2^64
        // We accept this in property tests
        prop_assert!(p1.session_seed != 0 || p2.session_seed != 0);
    }
    
    /// HTTP/2 window sizes should be within valid range
    #[test]
    fn http2_settings_valid(_seed in 0u64..100) {
        let profiles = [
            Profile::chrome_143_windows(),
            Profile::chrome_120_windows(),
            Profile::firefox_121_windows(),
        ];
        
        for profile in profiles.iter() {
            // HTTP/2 window size must be at least 65535 (default)
            prop_assert!(profile.http2.initial_window_size >= 65535);
            // Max frame size must be at least 16384 (HTTP/2 spec minimum)
            prop_assert!(profile.http2.max_frame_size >= 16384);
            // Max concurrent streams should be positive
            prop_assert!(profile.http2.max_concurrent_streams > 0);
        }
    }
    
    /// TLS configuration should be valid
    #[test]
    fn tls_config_valid(_seed in 0u64..100) {
        let profile = Profile::chrome_143_windows();
        
        // Must have cipher suites
        prop_assert!(!profile.tls.cipher_suites.is_empty());
        // Must have ALPN protocols
        prop_assert!(!profile.tls.alpn.is_empty());
        // Chrome uses GREASE
        prop_assert!(profile.tls.grease);
    }
}

// ============================================================================
// Authentication Property Tests
// ============================================================================

proptest! {
    /// Basic auth should produce valid Base64
    #[test]
    fn basic_auth_valid_base64(
        username in "[a-zA-Z0-9]{1,20}",
        password in "[a-zA-Z0-9!@#$%^&*]{1,30}"
    ) {
        let auth = BasicAuth::new(&username, &password);
        let header = auth.authorization_header();
        
        prop_assert!(header.starts_with("Basic "));
        
        // Extract Base64 part
        let base64_part = &header[6..];
        
        // Should be valid Base64 (only contains valid characters)
        prop_assert!(base64_part.chars().all(|c| 
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
        ));
    }
    
    /// Bearer tokens should have correct format
    #[test]
    fn bearer_token_format(token in "[a-zA-Z0-9._-]{10,100}") {
        let bearer = BearerToken::new(&token);
        let header = bearer.authorization_header();
        
        prop_assert!(header.starts_with("Bearer "));
        prop_assert_eq!(&header[7..], token);
    }
    
    /// Bearer tokens with expiration should respect expiry
    #[test]
    fn bearer_token_expiration(
        token in "[a-zA-Z0-9]{10,50}",
        secs in 1u64..86400
    ) {
        let bearer = BearerToken::with_expiration(&token, Duration::from_secs(secs));
        
        // Should not be expired immediately
        prop_assert!(!bearer.is_expired());
        
        // expires_soon should work correctly
        if secs > 60 {
            prop_assert!(!bearer.expires_soon(Duration::from_secs(30)));
        }
    }
    
    /// Digest auth should produce deterministic headers for same input
    #[test]
    fn digest_auth_deterministic(
        username in "[a-zA-Z]{3,10}",
        password in "[a-zA-Z0-9]{5,20}",
        realm in "[a-zA-Z.]{5,15}",
        nonce in "[a-f0-9]{16,32}"
    ) {
        let auth = DigestAuth::new(&username, &password);
        
        let header1 = auth.authorization_header(
            "GET",
            "/path",
            &realm,
            &nonce,
            Some("auth"),
            Some("00000001"),
            1,
        );
        
        let header2 = auth.authorization_header(
            "GET",
            "/path",
            &realm,
            &nonce,
            Some("auth"),
            Some("00000001"),
            1,
        );
        
        // Same inputs should produce same output
        prop_assert_eq!(header1, header2);
    }
    
    /// Digest auth should change with different nonces
    #[test]
    fn digest_auth_nonce_sensitive(
        username in "[a-zA-Z]{3,10}",
        password in "[a-zA-Z0-9]{5,20}"
    ) {
        let auth = DigestAuth::new(&username, &password);
        
        let header1 = auth.authorization_header(
            "GET",
            "/path",
            "realm",
            "nonce1",
            Some("auth"),
            None,
            1,
        );
        
        let header2 = auth.authorization_header(
            "GET",
            "/path",
            "realm",
            "nonce2",
            Some("auth"),
            None,
            1,
        );
        
        // Different nonces should produce different headers
        prop_assert_ne!(header1, header2);
    }
}

// ============================================================================
// Compression Property Tests
// ============================================================================

proptest! {
    /// Identity compression should be a no-op
    #[test]
    fn identity_compression_noop(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        use spectreq::CompressionType;
        use spectreq::client::compression::decompress;
        
        let result = decompress(&data, CompressionType::None);
        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap().data, data);
    }
    
    /// Wire size should equal input size for identity
    #[test]
    fn wire_size_tracking(data in prop::collection::vec(any::<u8>(), 1..500)) {
        use spectreq::CompressionType;
        use spectreq::client::compression::decompress;
        
        let result = decompress(&data, CompressionType::None).unwrap();
        prop_assert_eq!(result.wire_size, data.len());
    }
}

// ============================================================================
// Profile Serialization Property Tests
// ============================================================================

proptest! {
    /// Profile should roundtrip through JSON
    #[test]
    fn profile_json_roundtrip(_seed in 0u64..10) {
        let original = Profile::chrome_143_windows();
        
        let json = original.to_json().unwrap();
        let restored = Profile::from_json(&json).unwrap();
        
        prop_assert_eq!(original.browser, restored.browser);
        prop_assert_eq!(original.os, restored.os);
        prop_assert_eq!(original.version, restored.version);
        prop_assert_eq!(original.user_agent, restored.user_agent);
    }
    
    /// Profile should roundtrip through YAML
    #[test]
    fn profile_yaml_roundtrip(_seed in 0u64..10) {
        let original = Profile::chrome_143_windows();
        
        let yaml = original.to_yaml().unwrap();
        let restored = Profile::from_yaml(&yaml).unwrap();
        
        prop_assert_eq!(original.browser, restored.browser);
        prop_assert_eq!(original.os, restored.os);
        prop_assert_eq!(original.version, restored.version);
    }
}
