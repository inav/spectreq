//! Session persistence for Spectre client
//!
//! This module provides session persistence capabilities, allowing
//! TLS sessions, cookies, and other state to be saved and restored
//! across restarts.
//!
//! ## Features
//!
//! - **Plain JSON storage**: Simple, readable session files
//! - **Encrypted storage**: Password-protected with ChaCha20-Poly1305 + Argon2
//!
//! ## Example
//!
//! ```rust,ignore
//! use spectreq::client::session::SessionManager;
//!
//! let mut manager = SessionManager::new();
//! // ... add sessions ...
//!
//! // Save encrypted (recommended)
//! manager.save_encrypted("sessions.enc", "my_password")?;
//!
//! // Load encrypted
//! manager.load_encrypted("sessions.enc", "my_password")?;
//! ```

use crate::core::Profile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::SystemTime;
use thiserror::Error;

// Encryption imports
use argon2::Argon2;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;

/// Errors that can occur during session persistence
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: invalid password or corrupted file")]
    Decryption,

    #[error("Invalid file format")]
    InvalidFormat,
}

/// Serialized session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedSession {
    /// Browser profile used for this session
    #[serde(flatten)]
    pub profile_data: ProfileData,

    /// Cookies stored as (name, value, domain, path) tuples
    pub cookies: Vec<(String, String, String, String)>,

    /// TLS session tickets for session resumption
    pub tls_tickets: Vec<Vec<u8>>,

    /// When the session was created
    #[serde(with = "serde_system_time")]
    pub created_at: SystemTime,

    /// When the session was last accessed
    #[serde(with = "serde_system_time")]
    pub last_accessed: SystemTime,

    /// Custom session metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// Helper module for SystemTime serialization
mod serde_system_time {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::SystemTime;

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}

/// Simplified profile data for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    pub browser: String,
    pub os: String,
    pub version: String,
    pub user_agent: String,
    pub session_seed: u64,
}

impl From<&Profile> for ProfileData {
    fn from(profile: &Profile) -> Self {
        Self {
            browser: format!("{:?}", profile.browser),
            os: format!("{:?}", profile.os),
            version: profile.version.clone(),
            user_agent: profile.user_agent.clone(),
            session_seed: profile.session_seed,
        }
    }
}

impl SerializedSession {
    /// Create a new serialized session
    pub fn new(profile_data: ProfileData) -> Self {
        let now = SystemTime::now();
        Self {
            profile_data,
            cookies: Vec::new(),
            tls_tickets: Vec::new(),
            created_at: now,
            last_accessed: now,
            metadata: HashMap::new(),
        }
    }

    /// Add a cookie to the session
    pub fn add_cookie(&mut self, name: String, value: String, domain: String, path: String) {
        self.cookies.push((name, value, domain, path));
    }

    /// Add a TLS session ticket
    pub fn add_tls_ticket(&mut self, ticket: Vec<u8>) {
        self.tls_tickets.push(ticket);
    }

    /// Update the last accessed time
    pub fn update_access_time(&mut self) {
        self.last_accessed = SystemTime::now();
    }

    /// Check if the session is expired
    pub fn is_expired(&self, max_age: std::time::Duration) -> bool {
        match self.last_accessed.duration_since(self.created_at) {
            Ok(age) => age > max_age,
            Err(_) => true, // Clock went backwards, consider expired
        }
    }

    /// Get the age of the session
    pub fn age(&self) -> std::time::Duration {
        self.created_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Session manager for persisting and loading sessions
#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: HashMap<String, SerializedSession>,
    max_age: std::time::Duration,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            max_age: std::time::Duration::from_secs(24 * 60 * 60), // 24 hours
        }
    }

    /// Set the maximum age for sessions
    pub fn with_max_age(mut self, max_age: std::time::Duration) -> Self {
        self.max_age = max_age;
        self
    }

    /// Add or update a session
    pub fn set_session(&mut self, key: String, session: SerializedSession) {
        self.sessions.insert(key, session);
    }

    /// Get a session by key
    pub fn get_session(&self, key: &str) -> Option<&SerializedSession> {
        self.sessions.get(key)
    }

    /// Get a mutable session by key
    pub fn get_session_mut(&mut self, key: &str) -> Option<&mut SerializedSession> {
        self.sessions.get_mut(key)
    }

    /// Remove a session
    pub fn remove_session(&mut self, key: &str) -> Option<SerializedSession> {
        self.sessions.remove(key)
    }

    /// Clear all sessions
    pub fn clear(&mut self) {
        self.sessions.clear();
    }

    /// Remove expired sessions
    pub fn cleanup_expired(&mut self) {
        self.sessions
            .retain(|_, session| !session.is_expired(self.max_age));
    }

    /// Get all session keys
    pub fn keys(&self) -> Vec<&String> {
        self.sessions.keys().collect()
    }

    /// Save sessions to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), SessionError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.sessions)?;
        Ok(())
    }

    /// Load sessions from a file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SessionError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(SessionError::FileNotFound(path.display().to_string()));
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.sessions = serde_json::from_reader(reader)?;
        self.cleanup_expired();
        Ok(())
    }

    /// Get the number of sessions
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Check if there are no sessions
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    // ========================================================================
    // Encrypted session persistence
    // ========================================================================

    /// Derive an encryption key from a password using Argon2
    fn derive_key(password: &str, salt: &[u8; 16]) -> [u8; 32] {
        let argon2 = Argon2::default();
        let mut key = [0u8; 32];

        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .expect("Argon2 key derivation failed");

        key
    }

    /// Save sessions to an encrypted file
    ///
    /// Uses ChaCha20-Poly1305 for authenticated encryption with Argon2
    /// for password-based key derivation.
    ///
    /// File format: [magic:4][version:1][salt:16][nonce:12][ciphertext:*]
    ///
    /// # Arguments
    ///
    /// * `path` - Path to save the encrypted sessions
    /// * `password` - Password for encryption
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// manager.save_encrypted("sessions.enc", "my_secret_password")?;
    /// ```
    pub fn save_encrypted<P: AsRef<Path>>(
        &self,
        path: P,
        password: &str,
    ) -> Result<(), SessionError> {
        // Serialize sessions to JSON
        let json = serde_json::to_string(&self.sessions)?;

        // Generate random salt and nonce
        let mut salt = [0u8; 16];
        let mut nonce_bytes = [0u8; 12];
        let mut rng = rand::rng();
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce_bytes);

        // Derive key from password
        let key = Self::derive_key(password, &salt);

        // Encrypt the JSON
        let cipher = ChaCha20Poly1305::new_from_slice(&key)
            .map_err(|e| SessionError::Encryption(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, json.as_bytes())
            .map_err(|_| SessionError::Encryption("Encryption failed".to_string()))?;

        // Write to file
        let mut file = File::create(path)?;

        // Magic bytes: "SPEC"
        file.write_all(b"SPEC")?;
        // Version: 1
        file.write_all(&[1u8])?;
        // Salt
        file.write_all(&salt)?;
        // Nonce
        file.write_all(&nonce_bytes)?;
        // Ciphertext
        file.write_all(&ciphertext)?;

        Ok(())
    }

    /// Load sessions from an encrypted file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the encrypted sessions file
    /// * `password` - Password for decryption
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// manager.load_encrypted("sessions.enc", "my_secret_password")?;
    /// ```
    pub fn load_encrypted<P: AsRef<Path>>(
        &mut self,
        path: P,
        password: &str,
    ) -> Result<(), SessionError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(SessionError::FileNotFound(path.display().to_string()));
        }

        // Read file
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // Verify minimum length: magic(4) + version(1) + salt(16) + nonce(12) + tag(16)
        if data.len() < 49 {
            return Err(SessionError::InvalidFormat);
        }

        // Check magic bytes
        if &data[0..4] != b"SPEC" {
            return Err(SessionError::InvalidFormat);
        }

        // Check version
        if data[4] != 1 {
            return Err(SessionError::InvalidFormat);
        }

        // Extract salt, nonce, and ciphertext
        let salt: [u8; 16] = data[5..21]
            .try_into()
            .map_err(|_| SessionError::InvalidFormat)?;
        let nonce_bytes: [u8; 12] = data[21..33]
            .try_into()
            .map_err(|_| SessionError::InvalidFormat)?;
        let ciphertext = &data[33..];

        // Derive key from password
        let key = Self::derive_key(password, &salt);

        // Decrypt
        let cipher = ChaCha20Poly1305::new_from_slice(&key)
            .map_err(|e| SessionError::Encryption(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| SessionError::Decryption)?;

        // Parse JSON
        let json = String::from_utf8(plaintext).map_err(|_| SessionError::Decryption)?;
        self.sessions = serde_json::from_str(&json)?;

        self.cleanup_expired();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_new() {
        let manager = SessionManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_session_manager_set_get() {
        let mut manager = SessionManager::new();
        let profile = ProfileData {
            browser: "Chrome".to_string(),
            os: "Windows".to_string(),
            version: "143.0.0.0".to_string(),
            user_agent: "test".to_string(),
            session_seed: 42,
        };
        let session = SerializedSession::new(profile);

        manager.set_session("test".to_string(), session);
        assert_eq!(manager.len(), 1);
        assert!(manager.get_session("test").is_some());
    }

    #[test]
    fn test_session_manager_remove() {
        let mut manager = SessionManager::new();
        let profile = ProfileData {
            browser: "Chrome".to_string(),
            os: "Windows".to_string(),
            version: "143.0.0.0".to_string(),
            user_agent: "test".to_string(),
            session_seed: 42,
        };
        let session = SerializedSession::new(profile);

        manager.set_session("test".to_string(), session);
        manager.remove_session("test");
        assert!(manager.is_empty());
    }

    #[test]
    fn test_serialized_session_add_cookie() {
        let profile = ProfileData {
            browser: "Chrome".to_string(),
            os: "Windows".to_string(),
            version: "143.0.0.0".to_string(),
            user_agent: "test".to_string(),
            session_seed: 42,
        };
        let mut session = SerializedSession::new(profile);

        session.add_cookie(
            "session_id".to_string(),
            "abc123".to_string(),
            "example.com".to_string(),
            "/".to_string(),
        );

        assert_eq!(session.cookies.len(), 1);
        assert_eq!(session.cookies[0].0, "session_id");
    }

    #[test]
    fn test_serialized_session_metadata() {
        let profile = ProfileData {
            browser: "Chrome".to_string(),
            os: "Windows".to_string(),
            version: "143.0.0.0".to_string(),
            user_agent: "test".to_string(),
            session_seed: 42,
        };
        let mut session = SerializedSession::new(profile);

        session.set_metadata("user".to_string(), "test_user".to_string());
        assert_eq!(session.get_metadata("user"), Some(&"test_user".to_string()));
    }

    #[test]
    fn test_encrypted_save_load() {
        use tempfile::NamedTempFile;

        let mut manager = SessionManager::new();
        let profile = ProfileData {
            browser: "Chrome".to_string(),
            os: "Windows".to_string(),
            version: "143.0.0.0".to_string(),
            user_agent: "test".to_string(),
            session_seed: 42,
        };
        let session = SerializedSession::new(profile);
        manager.set_session("test".to_string(), session);

        // Create a temporary file
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        // Save encrypted
        let password = "super_secret_password";
        manager
            .save_encrypted(path, password)
            .expect("Failed to save encrypted");

        // Load into new manager
        let mut new_manager = SessionManager::new();
        new_manager
            .load_encrypted(path, password)
            .expect("Failed to load encrypted");

        assert_eq!(new_manager.len(), 1);
        assert!(new_manager.get_session("test").is_some());

        // Try load with wrong password
        let mut bad_manager = SessionManager::new();
        let err = bad_manager.load_encrypted(path, "wrong_password");
        assert!(err.is_err());
        match err {
            Err(SessionError::Decryption) | Err(SessionError::Encryption(_)) => {}
            _ => panic!("Expected Decryption error, got {:?}", err),
        }
    }
}
