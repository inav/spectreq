//! Integration tests for authentication module
//!
//! Tests for Bearer, Basic, Digest, and NTLM authentication.

use spectreq::{BasicAuth, BearerToken, DigestAuth};
use std::time::Duration;

#[test]
fn test_bearer_token_creation() {
    let token = BearerToken::new("test_access_token");
    assert_eq!(token.authorization_header(), "Bearer test_access_token");
    assert!(!token.is_expired());
}

#[test]
fn test_bearer_token_with_expiration() {
    let token = BearerToken::with_expiration("test_token", Duration::from_secs(3600));
    assert!(!token.is_expired());
    assert!(!token.expires_soon(Duration::from_secs(60)));
    assert!(token.expires_soon(Duration::from_secs(7200)));
}

#[test]
fn test_bearer_token_with_refresh() {
    let token =
        BearerToken::with_refresh_token("access_token", "refresh_token", Duration::from_secs(3600));
    assert!(!token.is_expired());
    assert!(token.refresh_token.is_some());
}

#[test]
fn test_basic_auth_header() {
    let auth = BasicAuth::new("user", "pass");
    let header = auth.authorization_header();
    assert!(header.starts_with("Basic "));
    // "user:pass" base64 encoded is "dXNlcjpwYXNz"
    assert_eq!(header, "Basic dXNlcjpwYXNz");
}

#[test]
fn test_basic_auth_special_characters() {
    let auth = BasicAuth::new("user@domain.com", "p@ss:word!");
    let header = auth.authorization_header();
    assert!(header.starts_with("Basic "));
}

#[test]
fn test_digest_auth_md5_hash() {
    let auth = DigestAuth::new("admin", "secret");

    // Test that digest auth produces consistent output
    let header1 = auth.authorization_header(
        "GET",
        "/protected",
        "test_realm",
        "abc123",
        Some("auth"),
        Some("xyz789"),
        1,
    );

    let header2 = auth.authorization_header(
        "GET",
        "/protected",
        "test_realm",
        "abc123",
        Some("auth"),
        Some("xyz789"),
        1,
    );

    // Same inputs should produce same output
    assert_eq!(header1, header2);

    // Should contain digest auth components
    assert!(header1.starts_with("Digest "));
    assert!(header1.contains("username=\"admin\""));
    assert!(header1.contains("realm=\"test_realm\""));
    assert!(header1.contains("nonce=\"abc123\""));
    assert!(header1.contains("qop=auth"));
}

#[test]
fn test_digest_auth_without_qop() {
    let auth = DigestAuth::new("user", "pass");

    let header = auth.authorization_header("GET", "/resource", "realm", "nonce123", None, None, 1);

    assert!(header.starts_with("Digest "));
    assert!(!header.contains("qop="));
}

#[test]
fn test_digest_auth_different_inputs() {
    let auth = DigestAuth::new("user", "pass");

    let header1 =
        auth.authorization_header("GET", "/path1", "realm", "nonce1", Some("auth"), None, 1);

    let header2 =
        auth.authorization_header("GET", "/path2", "realm", "nonce1", Some("auth"), None, 1);

    // Different URIs should produce different responses
    assert_ne!(header1, header2);
}

#[tokio::test]
async fn test_bearer_token_manager() {
    use spectreq::BearerTokenManager;

    let manager = BearerTokenManager::new();

    // Initially no token
    assert!(manager.get_token().await.is_none());

    // Set a token
    let token = BearerToken::new("my_token");
    manager.set_token(token).await;

    // Should have token now
    let retrieved = manager.get_token().await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().access_token, "my_token");

    // Clear token
    manager.clear().await;
    assert!(manager.get_token().await.is_none());
}

#[tokio::test]
async fn test_basic_auth_cache() {
    use spectreq::BasicAuthCache;

    let cache = BasicAuthCache::new();

    // Initially empty
    assert!(cache.get("realm1").await.is_none());

    // Add credentials
    cache.add("realm1", BasicAuth::new("user1", "pass1")).await;
    cache.add("realm2", BasicAuth::new("user2", "pass2")).await;

    // Retrieve
    let auth1 = cache.get("realm1").await.unwrap();
    assert_eq!(auth1.username, "user1");

    let auth2 = cache.get("realm2").await.unwrap();
    assert_eq!(auth2.username, "user2");

    // Remove
    assert!(cache.remove("realm1").await);
    assert!(cache.get("realm1").await.is_none());
    assert!(cache.get("realm2").await.is_some());

    // Clear
    cache.clear().await;
    assert!(cache.get("realm2").await.is_none());
}

#[test]
fn test_extract_realm() {
    use spectreq::extract_realm;

    let header = r#"Digest realm="test_realm", nonce="abc123""#;
    assert_eq!(extract_realm(header), Some("test_realm".to_string()));

    let header_no_realm = "Digest nonce=\"abc123\"";
    assert_eq!(extract_realm(header_no_realm), None);
}

#[test]
fn test_extract_nonce() {
    use spectreq::extract_nonce;

    let header = r#"Digest realm="test", nonce="my_nonce_value""#;
    assert_eq!(extract_nonce(header), Some("my_nonce_value".to_string()));
}

#[test]
fn test_extract_opaque() {
    use spectreq::extract_opaque;

    let header = r#"Digest realm="test", opaque="opaque_value", qop="auth""#;
    assert_eq!(extract_opaque(header), Some("opaque_value".to_string()));
}

#[test]
fn test_extract_qop() {
    use spectreq::extract_qop;

    let header = r#"Digest realm="test", qop="auth""#;
    assert_eq!(extract_qop(header), Some("auth".to_string()));

    let header_auth_int = r#"Digest qop="auth-int""#;
    assert_eq!(extract_qop(header_auth_int), Some("auth-int".to_string()));
}
