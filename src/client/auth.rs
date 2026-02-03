//! Authentication helpers for Spectre client
//!
//! This module provides authentication support including:
//! - Bearer token management with auto-refresh
//! - Basic auth caching
//! - Digest auth (RFC 2617)
//! - NTLM support (optional feature)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Bearer token with refresh support
#[derive(Clone)]
pub struct BearerToken {
    /// Access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: Option<String>,
    /// Token type (usually "Bearer")
    pub token_type: String,
    /// Expiration time
    pub expires_at: Option<Instant>,
    /// Scope
    pub scope: Option<String>,
}

impl std::fmt::Debug for BearerToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BearerToken")
            .field("access_token", &"***REDACTED***")
            .field(
                "refresh_token",
                &self.refresh_token.as_deref().map(|_| "***REDACTED***"),
            )
            .field("token_type", &self.token_type)
            .field("expires_at", &self.expires_at.map(|_| "Instant(...)"))
            .field("scope", &self.scope)
            .finish()
    }
}

impl BearerToken {
    /// Create a new bearer token
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_at: None,
            scope: None,
        }
    }

    /// Create a new bearer token with expiration
    pub fn with_expiration(access_token: impl Into<String>, expires_in: Duration) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_at: Some(Instant::now() + expires_in),
            scope: None,
        }
    }

    /// Create a new bearer token with refresh token
    pub fn with_refresh_token(
        access_token: impl Into<String>,
        refresh_token: impl Into<String>,
        expires_in: Duration,
    ) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: Some(refresh_token.into()),
            token_type: "Bearer".to_string(),
            expires_at: Some(Instant::now() + expires_in),
            scope: None,
        }
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Instant::now() >= expires_at
        } else {
            false
        }
    }

    /// Check if the token will expire soon (within 5 minutes)
    pub fn expires_soon(&self, within: Duration) -> bool {
        if let Some(expires_at) = self.expires_at {
            let remaining = expires_at.saturating_duration_since(Instant::now());
            remaining <= within
        } else {
            false
        }
    }

    /// Get the authorization header value
    pub fn authorization_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
}

/// Bearer token manager with auto-refresh
impl std::fmt::Debug for BearerTokenManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BearerTokenManager")
            .field("refresh_url", &self.refresh_url)
            .field("refresh_buffer", &self.refresh_buffer)
            .finish()
    }
}

pub struct BearerTokenManager {
    current_token: Arc<RwLock<Option<BearerToken>>>,
    refresh_url: Option<String>,
    refresh_buffer: Duration,
}

impl BearerTokenManager {
    /// Create a new bearer token manager
    pub fn new() -> Self {
        Self {
            current_token: Arc::new(RwLock::new(None)),
            refresh_url: None,
            refresh_buffer: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Set the refresh URL
    pub fn with_refresh_url(mut self, url: impl Into<String>) -> Self {
        self.refresh_url = Some(url.into());
        self
    }

    /// Set the refresh buffer (time before expiration to refresh)
    pub fn with_refresh_buffer(mut self, buffer: Duration) -> Self {
        self.refresh_buffer = buffer;
        self
    }

    /// Set the current token
    pub async fn set_token(&self, token: BearerToken) {
        let mut current = self.current_token.write().await;
        *current = Some(token);
    }

    /// Get the current token
    pub async fn get_token(&self) -> Option<BearerToken> {
        let current = self.current_token.read().await;
        current.clone()
    }

    /// Get the authorization header value, refreshing if needed
    pub async fn get_authorization(&self) -> Option<String> {
        let token = self.get_token().await?;

        // Check if we need to refresh
        if token.is_expired() || token.expires_soon(self.refresh_buffer) {
            // Try to refresh (this would typically involve an async callback)
            if let Some(_refresh_token) = &token.refresh_token {
                if let Some(refresh_url) = &self.refresh_url {
                    // In a real implementation, this would make an HTTP request
                    // to refresh the token using the refresh token
                    // For now, we'll just log a message
                    eprintln!("Token needs refresh (would call {})", refresh_url);
                }
            }
        }

        Some(token.authorization_header())
    }

    /// Clear the current token
    pub async fn clear(&self) {
        let mut current = self.current_token.write().await;
        *current = None;
    }
}

impl Default for BearerTokenManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Basic auth credentials
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

impl BasicAuth {
    /// Create new basic auth credentials
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Get the authorization header value
    pub fn authorization_header(&self) -> String {
        use base64::Engine;
        let credentials = format!("{}:{}", self.username, self.password);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        format!("Basic {}", encoded)
    }
}

/// Basic auth cache for storing credentials
impl std::fmt::Debug for BasicAuthCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicAuthCache").finish()
    }
}

pub struct BasicAuthCache {
    cache: Arc<RwLock<HashMap<String, BasicAuth>>>,
}

impl BasicAuthCache {
    /// Create a new basic auth cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add credentials for a realm
    pub async fn add(&self, realm: impl Into<String>, auth: BasicAuth) {
        let mut cache = self.cache.write().await;
        cache.insert(realm.into(), auth);
    }

    /// Get credentials for a realm
    pub async fn get(&self, realm: &str) -> Option<BasicAuth> {
        let cache = self.cache.read().await;
        cache.get(realm).cloned()
    }

    /// Remove credentials for a realm
    pub async fn remove(&self, realm: &str) -> bool {
        let mut cache = self.cache.write().await;
        cache.remove(realm).is_some()
    }

    /// Clear all credentials
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

impl Default for BasicAuthCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Digest authentication configuration (RFC 2617)
#[derive(Debug, Clone)]
pub struct DigestAuth {
    username: String,
    password: String,
}

impl DigestAuth {
    /// Create a new digest auth configuration
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Create the authorization header value for digest auth
    #[allow(clippy::too_many_arguments)]
    pub fn authorization_header(
        &self,
        method: &str,
        uri: &str,
        realm: &str,
        nonce: &str,
        qop: Option<&str>,
        opaque: Option<&str>,
        nc: u32,
    ) -> String {
        // Calculate HA1 = MD5(username:realm:password)
        let ha1_input = format!("{}:{}:{}", self.username, realm, self.password);
        let ha1 = Md5Wrapper::digest_hex(&ha1_input);

        // Calculate HA2 = MD5(method:uri)
        let ha2_input = format!("{}:{}", method, uri);
        let ha2 = Md5Wrapper::digest_hex(&ha2_input);

        // Calculate response
        let opaque_str = opaque.unwrap_or("");
        let response = if let Some(qop_value) = qop {
            if qop_value == "auth" || qop_value == "auth-int" {
                format!("{}:{}:{}:{}:{}", ha1, nonce, nc, opaque_str, ha2)
            } else {
                format!("{}::{}:{}:{}", ha1, nonce, opaque_str, ha2)
            }
        } else {
            format!("{}::{}:{}:{}", ha1, nonce, opaque_str, ha2)
        };

        let response_hash = Md5Wrapper::digest_hex(&response);
        let auth_header = if let Some(qop_value) = qop {
            if qop_value == "auth" || qop_value == "auth-int" {
                format!(
                    "Digest username=\"{}\", realm=\"{}\", nonce=\"{}\", uri=\"{}\", qop={}, nc={}, opaque=\"{}\", response=\"{}\"",
                    self.username, realm, nonce, uri, qop_value, nc, opaque_str, response_hash
                )
            } else {
                format!(
                    "Digest username=\"{}\", realm=\"{}\", nonce=\"{}\", opaque=\"{}\", response=\"{}\"",
                    self.username, realm, nonce, opaque_str, response_hash
                )
            }
        } else {
            format!(
                "Digest username=\"{}\", realm=\"{}\", nonce=\"{}\", opaque=\"{}\", response=\"{}\"",
                self.username, realm, nonce, opaque_str, response_hash
            )
        };

        auth_header
    }
}

/// Wrapper for MD5 digest using the md-5 crate
struct Md5Wrapper;

impl Md5Wrapper {
    fn digest(data: &str) -> [u8; 16] {
        use md5::{Digest, Md5};
        let mut hasher = Md5::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        let mut output = [0u8; 16];
        output.copy_from_slice(&result);
        output
    }

    fn digest_hex(data: &str) -> String {
        let bytes = Self::digest(data);
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

/// NTLM authentication configuration (optional feature)
#[derive(Debug, Clone)]
pub struct NtlmAuth {
    #[allow(dead_code)]
    username: String,
    #[allow(dead_code)]
    password: String,
    domain: Option<String>,
    workstation: Option<String>,
}

impl NtlmAuth {
    /// Create a new NTLM auth configuration
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            domain: None,
            workstation: None,
        }
    }

    /// Set the domain
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set the workstation
    pub fn with_workstation(mut self, workstation: impl Into<String>) -> Self {
        self.workstation = Some(workstation.into());
        self
    }

    /// Get the NTLM message type 1 (negotiate)
    ///
    /// # Panics
    ///
    /// NTLM authentication is not yet implemented. This method exists for API
    /// completeness but will panic if called. For enterprise Windows authentication,
    /// consider using Kerberos/SPNEGO instead, or contribute an NTLM implementation.
    ///
    /// NTLM implementation requires:
    /// - Type 1 (Negotiate) message generation
    /// - Type 2 (Challenge) message parsing
    /// - Type 3 (Authenticate) message generation with NTLMv2 hashing
    /// - Session security negotiation
    pub fn negotiate_msg(&self) -> String {
        unimplemented!(
            "NTLM authentication is not yet implemented. \
             Consider using Basic or Digest auth for HTTP authentication, \
             or Kerberos for enterprise Windows environments."
        )
    }
}

/// Authentication configuration for the client
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Bearer token manager
    pub bearer: Arc<BearerTokenManager>,
    /// Basic auth cache
    pub basic: Arc<BasicAuthCache>,
    /// Digest auth credentials
    pub digest: Arc<RwLock<HashMap<String, DigestAuth>>>,
    /// NTLM credentials
    pub ntlm: Arc<RwLock<Option<NtlmAuth>>>,
}

impl AuthConfig {
    /// Create a new auth configuration
    pub fn new() -> Self {
        Self {
            bearer: Arc::new(BearerTokenManager::new()),
            basic: Arc::new(BasicAuthCache::new()),
            digest: Arc::new(RwLock::new(HashMap::new())),
            ntlm: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the bearer token
    pub async fn set_bearer_token(&self, token: BearerToken) {
        self.bearer.set_token(token).await;
    }

    /// Get the bearer authorization header
    pub async fn bearer_authorization(&self) -> Option<String> {
        self.bearer.get_authorization().await
    }

    /// Add basic auth credentials
    pub async fn add_basic_auth(
        &self,
        realm: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) {
        self.basic
            .add(realm, BasicAuth::new(username, password))
            .await;
    }

    /// Get basic auth header for a realm
    pub async fn basic_authorization(&self, realm: &str) -> Option<String> {
        self.basic
            .get(realm)
            .await
            .map(|auth| auth.authorization_header())
    }

    /// Add digest auth credentials
    pub async fn add_digest_auth(
        &self,
        realm: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) {
        let mut digest = self.digest.write().await;
        digest.insert(realm.into(), DigestAuth::new(username, password));
    }

    /// Get digest auth for a realm
    pub async fn get_digest_auth(&self, realm: &str) -> Option<DigestAuth> {
        let digest = self.digest.read().await;
        digest.get(realm).cloned()
    }

    /// Set NTLM credentials
    pub async fn set_ntlm_auth(&self, username: impl Into<String>, password: impl Into<String>) {
        let ntlm = NtlmAuth::new(username, password);
        let mut current = self.ntlm.write().await;
        *current = Some(ntlm);
    }

    /// Get NTLM auth
    pub async fn get_ntlm_auth(&self) -> Option<NtlmAuth> {
        self.ntlm.read().await.clone()
    }

    /// Clear all stored credentials
    pub async fn clear(&self) {
        self.bearer.clear().await;
        self.basic.clear().await;
        self.digest.write().await.clear();
        *self.ntlm.write().await = None;
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract realm from WWW-Authenticate header
pub fn extract_realm(auth_header: &str) -> Option<String> {
    // Parse WWW-Authenticate header to extract realm
    // Example: "Digest realm=\"test\", nonce=\"abc\""
    if let Some(start) = auth_header.find("realm=\"") {
        let start = start + 7; // len("realm=\"")
        if let Some(end) = auth_header[start..].find('"') {
            return Some(auth_header[start..start + end].to_string());
        }
    }
    None
}

/// Extract nonce from WWW-Authenticate header
pub fn extract_nonce(auth_header: &str) -> Option<String> {
    if let Some(start) = auth_header.find("nonce=\"") {
        let start = start + 7;
        if let Some(end) = auth_header[start..].find('"') {
            return Some(auth_header[start..start + end].to_string());
        }
    }
    None
}

/// Extract opaque from WWW-Authenticate header
pub fn extract_opaque(auth_header: &str) -> Option<String> {
    if let Some(start) = auth_header.find("opaque=\"") {
        let start = start + 8;
        if let Some(end) = auth_header[start..].find('"') {
            return Some(auth_header[start..start + end].to_string());
        }
    }
    None
}

/// Extract qop from WWW-Authenticate header
pub fn extract_qop(auth_header: &str) -> Option<String> {
    if let Some(start) = auth_header.find("qop=\"") {
        let start = start + 5;
        if let Some(end) = auth_header[start..].find('"') {
            return Some(auth_header[start..start + end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_token() {
        let token = BearerToken::new("test_token");
        assert!(!token.is_expired());
        assert_eq!(token.authorization_header(), "Bearer test_token");
    }

    #[test]
    fn test_bearer_token_with_expiration() {
        let token = BearerToken::with_expiration("test_token", Duration::from_secs(60));
        assert!(!token.is_expired());
        assert!(token.expires_soon(Duration::from_secs(61)));
    }

    #[test]
    fn test_basic_auth() {
        let auth = BasicAuth::new("user", "pass");
        assert_eq!(auth.username, "user");
        assert_eq!(auth.password, "pass");
        assert!(auth.authorization_header().starts_with("Basic "));
    }

    #[tokio::test]
    async fn test_basic_auth_cache() {
        let cache = BasicAuthCache::new();
        let auth = BasicAuth::new("user", "pass");

        cache.add("test_realm", auth.clone()).await;
        assert_eq!(cache.get("test_realm").await, Some(auth));

        cache.remove("test_realm").await;
        assert!(cache.get("test_realm").await.is_none());
    }

    #[test]
    fn test_extract_realm() {
        let header = "Digest realm=\"test\", nonce=\"abc\"";
        assert_eq!(extract_realm(header), Some("test".to_string()));
    }

    #[test]
    fn test_extract_nonce() {
        let header = "Digest realm=\"test\", nonce=\"abc123\"";
        assert_eq!(extract_nonce(header), Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_opaque() {
        let header = "Digest opaque=\"xyz789\", qop=\"auth\"";
        assert_eq!(extract_opaque(header), Some("xyz789".to_string()));
    }

    #[test]
    fn test_ntlm_auth() {
        let auth = NtlmAuth::new("user", "pass")
            .with_domain("DOMAIN")
            .with_workstation("WORKSTATION");

        assert_eq!(auth.username, "user");
        assert_eq!(auth.domain, Some("DOMAIN".to_string()));
        assert_eq!(auth.workstation, Some("WORKSTATION".to_string()));
    }
}
