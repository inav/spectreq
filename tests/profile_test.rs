//! Integration tests for browser profiles
//!
//! Tests for profile creation, properties, and TLS fingerprinting.

use spectreq::{Profile, BrowserName, OS};

#[test]
fn test_chrome_120_windows_profile() {
    let profile = Profile::chrome_120_windows();
    assert_eq!(profile.browser, BrowserName::Chrome);
    assert_eq!(profile.os, OS::Windows);
    assert!(profile.version.starts_with("120"));
    assert!(profile.user_agent.contains("Chrome/120"));
    assert!(profile.user_agent.contains("Windows NT"));
}

#[test]
fn test_chrome_120_macos_profile() {
    let profile = Profile::chrome_120_macos();
    assert_eq!(profile.browser, BrowserName::Chrome);
    assert_eq!(profile.os, OS::MacOS);
    assert!(profile.user_agent.contains("Macintosh"));
}

#[test]
fn test_chrome_120_linux_profile() {
    let profile = Profile::chrome_120_linux();
    assert_eq!(profile.browser, BrowserName::Chrome);
    assert_eq!(profile.os, OS::Linux);
    assert!(profile.user_agent.contains("X11"));
    assert!(profile.user_agent.contains("Linux"));
}

#[test]
fn test_chrome_120_android_profile() {
    let profile = Profile::chrome_120_android();
    assert_eq!(profile.browser, BrowserName::Chrome);
    assert_eq!(profile.os, OS::Android);
    assert!(profile.user_agent.contains("Android"));
}

#[test]
fn test_chrome_131_windows_profile() {
    let profile = Profile::chrome_131_windows();
    assert_eq!(profile.browser, BrowserName::Chrome);
    assert!(profile.version.starts_with("131"));
    // Chrome 131+ has larger HTTP/2 window size (6MB)
    assert!(profile.http2.initial_window_size > 65536);
}

#[test]
fn test_chrome_143_windows_profile() {
    let profile = Profile::chrome_143_windows();
    assert_eq!(profile.browser, BrowserName::Chrome);
    assert!(profile.version.starts_with("143"));
    assert!(profile.user_agent.contains("Chrome/143"));
}

#[test]
fn test_chrome_143_macos_profile() {
    let profile = Profile::chrome_143_macos();
    assert_eq!(profile.os, OS::MacOS);
    assert!(profile.version.starts_with("143"));
}

#[test]
fn test_chrome_143_linux_profile() {
    let profile = Profile::chrome_143_linux();
    assert_eq!(profile.os, OS::Linux);
    assert!(profile.version.starts_with("143"));
}

#[test]
fn test_chrome_143_android_profile() {
    let profile = Profile::chrome_143_android();
    assert_eq!(profile.os, OS::Android);
    assert!(profile.version.starts_with("143"));
}

#[test]
fn test_firefox_121_windows_profile() {
    let profile = Profile::firefox_121_windows();
    assert_eq!(profile.browser, BrowserName::Firefox);
    assert_eq!(profile.os, OS::Windows);
    assert!(profile.version.starts_with("121"));
    assert!(profile.user_agent.contains("Firefox/121"));
    // Firefox has different HTTP/2 settings than Chrome
    assert!(profile.http2.max_concurrent_streams <= 100);
}

#[test]
fn test_safari_17_macos_profile() {
    let profile = Profile::safari_17_macos();
    assert_eq!(profile.browser, BrowserName::Safari);
    assert_eq!(profile.os, OS::MacOS);
    assert!(profile.version.starts_with("17"));
    assert!(profile.user_agent.contains("Safari"));
    assert!(profile.user_agent.contains("Version/17"));
}

#[test]
fn test_edge_120_windows_profile() {
    let profile = Profile::edge_120_windows();
    assert_eq!(profile.browser, BrowserName::Edge);
    assert_eq!(profile.os, OS::Windows);
    assert!(profile.version.starts_with("120"));
    assert!(profile.user_agent.contains("Edg/120"));
}

#[test]
fn test_tls_cipher_suites_populated() {
    let chrome = Profile::chrome_143_windows();
    let firefox = Profile::firefox_121_windows();
    let safari = Profile::safari_17_macos();
    
    // All profiles should have cipher suites
    assert!(!chrome.tls.cipher_suites.is_empty());
    assert!(!firefox.tls.cipher_suites.is_empty());
    assert!(!safari.tls.cipher_suites.is_empty());
}

#[test]
fn test_tls_extensions_populated() {
    let profile = Profile::chrome_143_windows();
    assert!(!profile.tls.extensions.is_empty());
}

#[test]
fn test_http2_settings() {
    let chrome = Profile::chrome_120_windows();
    let firefox = Profile::firefox_121_windows();
    
    // Chrome and Firefox have different HTTP/2 settings
    assert!(chrome.http2.initial_window_size > 0);
    assert!(firefox.http2.initial_window_size > 0);
    assert!(chrome.http2.max_concurrent_streams > 0);
    assert!(firefox.http2.max_concurrent_streams > 0);
}

#[test]
fn test_tcp_settings() {
    let windows_profile = Profile::chrome_143_windows();
    let linux_profile = Profile::chrome_143_linux();
    
    // TCP TTL should be set appropriately for OS
    // Windows typically uses TTL 128, Linux uses 64
    assert!(windows_profile.tcp.ttl.is_some());
    assert!(linux_profile.tcp.ttl.is_some());
    assert!(windows_profile.tcp.ttl.unwrap() > 0);
    assert!(linux_profile.tcp.ttl.unwrap() > 0);
}

#[test]
fn test_alpn_populated() {
    let profile = Profile::chrome_143_windows();
    assert!(!profile.tls.alpn.is_empty());
    // Should contain at least h2 and http/1.1
    assert!(profile.tls.alpn.contains(&"h2".to_string()) || 
            profile.tls.alpn.contains(&"http/1.1".to_string()));
}

#[test]
fn test_grease_settings() {
    let chrome = Profile::chrome_143_windows();
    let firefox = Profile::firefox_121_windows();
    
    // Chrome uses GREASE, Firefox doesn't
    assert!(chrome.tls.grease);
    assert!(!firefox.tls.grease);
}

#[test]
fn test_accept_encoding() {
    let profile = Profile::chrome_143_windows();
    
    // Should support multiple compression types (it's a string like "gzip, deflate, br, zstd")
    assert!(!profile.accept_encoding.is_empty());
    // Modern browsers support br (Brotli)
    assert!(profile.accept_encoding.contains("br") ||
            profile.accept_encoding.contains("gzip"));
}

#[test]
fn test_all_profiles_have_required_fields() {
    let profiles = vec![
        Profile::chrome_120_windows(),
        Profile::chrome_120_macos(),
        Profile::chrome_120_linux(),
        Profile::chrome_120_android(),
        Profile::chrome_131_windows(),
        Profile::chrome_133_windows(),
        Profile::chrome_141_windows(),
        Profile::chrome_143_windows(),
        Profile::chrome_143_macos(),
        Profile::chrome_143_linux(),
        Profile::chrome_143_android(),
        Profile::firefox_121_windows(),
        Profile::safari_17_macos(),
        Profile::edge_120_windows(),
    ];
    
    for profile in profiles {
        assert!(!profile.version.is_empty(), "Version should not be empty");
        assert!(!profile.user_agent.is_empty(), "User-Agent should not be empty");
        assert!(!profile.tls.cipher_suites.is_empty(), "Cipher suites should not be empty");
        assert!(profile.http2.initial_window_size > 0, "Initial window size should be > 0");
        assert!(profile.tcp.ttl.is_some() && profile.tcp.ttl.unwrap() > 0, "TCP TTL should be > 0");
    }
}
