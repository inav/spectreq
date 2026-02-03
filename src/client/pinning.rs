//! Certificate pinning for TLS connections
//!
//! Certificate pinning allows you to specify which SPKI (Subject Public Key Info)
//! hashes are trusted for a given host, providing an additional layer of security
//! beyond the standard certificate validation.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Errors that can occur during certificate pinning
#[derive(Debug, Error)]
pub enum PinError {
    #[error("Certificate pinning failed for host: {host}")]
    PinFailed { host: String },

    #[error("No pins configured for host: {host}")]
    NoPins { host: String },

    #[error("Invalid certificate format")]
    InvalidCertificate,

    #[error("Hash computation error: {0}")]
    HashError(String),
}

/// Certificate pinner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertPinner {
    /// Map of hostname to list of SPKI hashes
    pins: std::collections::HashMap<String, Vec<String>>,
}

impl Default for CertPinner {
    fn default() -> Self {
        Self::new()
    }
}

impl CertPinner {
    /// Create a new certificate pinner
    pub fn new() -> Self {
        Self {
            pins: std::collections::HashMap::new(),
        }
    }

    /// Add a pin for a host
    ///
    /// The `spki_hash` should be a hex-encoded SHA-256 hash of the
    /// DER-encoded Subject Public Key Info (SPKI) from the certificate.
    pub fn add_pin(&mut self, host: String, spki_hash: String) {
        self.pins.entry(host).or_default().push(spki_hash);
    }

    /// Add multiple pins for a host
    pub fn add_pins(&mut self, host: String, spki_hashes: Vec<String>) {
        self.pins.entry(host).or_default().extend(spki_hashes);
    }

    /// Remove all pins for a host
    pub fn remove_host(&mut self, host: &str) {
        self.pins.remove(host);
    }

    /// Get pins for a host
    pub fn get_pins(&self, host: &str) -> Option<&Vec<String>> {
        self.pins.get(host)
    }

    /// Check if a host has any pins configured
    pub fn has_pins(&self, host: &str) -> bool {
        self.pins.contains_key(host)
    }

    /// Verify a certificate's SPKI hash against pinned values
    ///
    /// The `cert_der` should be the DER-encoded certificate.
    /// The `spki_hash` should be a hex-encoded SHA-256 hash of the SPKI.
    pub fn verify_hash(&self, host: &str, spki_hash: &str) -> Result<(), PinError> {
        if let Some(pins) = self.get_pins(host) {
            if pins.iter().any(|pin| pin.eq_ignore_ascii_case(spki_hash)) {
                Ok(())
            } else {
                Err(PinError::PinFailed {
                    host: host.to_string(),
                })
            }
        } else {
            // If no pins are configured, verification succeeds
            Ok(())
        }
    }

    /// Compute SPKI hash from a DER-encoded certificate
    ///
    /// This extracts the SPKI from the certificate and computes its SHA-256 hash.
    pub fn compute_spki_hash(cert_der: &[u8]) -> Result<String, PinError> {
        // Parse the DER-encoded certificate to extract SPKI
        // X.509 certificate structure:
        // SEQUENCE {
        //   tbsCertificate TBSCertificate,
        //   signatureAlgorithm AlgorithmIdentifier,
        //   signatureValue BIT STRING
        // }
        //
        // The SPKI is in the subjectPublicKeyInfo field within tbsCertificate

        use der::Decode;
        use der::Encode;
        use x509_cert::Certificate;

        let cert = Certificate::from_der(cert_der).map_err(|_| PinError::InvalidCertificate)?;

        let spki = cert
            .tbs_certificate
            .subject_public_key_info
            .to_der()
            .map_err(|_| PinError::InvalidCertificate)?;

        // Compute SHA-256 hash
        let hash = Sha256::digest(&spki);
        Ok(hex::encode(hash))
    }

    /// Verify a DER-encoded certificate against pinned values
    pub fn verify_cert(&self, host: &str, cert_der: &[u8]) -> Result<(), PinError> {
        let hash = Self::compute_spki_hash(cert_der)?;
        self.verify_hash(host, &hash)
    }

    /// Clear all pins
    pub fn clear(&mut self) {
        self.pins.clear();
    }

    /// Get the number of hosts with pins
    pub fn len(&self) -> usize {
        self.pins.len()
    }

    /// Check if there are no pins
    pub fn is_empty(&self) -> bool {
        self.pins.is_empty()
    }
}

/// Helper function to compute a pin from a certificate
///
/// This is useful for generating pins to add to your code.
pub fn generate_pin_from_cert(cert_der: &[u8]) -> Result<String, PinError> {
    CertPinner::compute_spki_hash(cert_der)
}

/// Load pins from a JSON file
///
/// The JSON format should be:
/// ```json
/// {
///   "example.com": ["abc123...", "def456..."],
///   "api.example.com": ["ghi789..."]
/// }
/// ```
pub fn load_pins_from_json(json: &str) -> Result<CertPinner, serde_json::Error> {
    let pins: std::collections::HashMap<String, Vec<String>> = serde_json::from_str(json)?;
    Ok(CertPinner { pins })
}

/// Export pins to JSON format
pub fn export_pins_to_json(pinner: &CertPinner) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&pinner.pins)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cert_pinner_new() {
        let pinner = CertPinner::new();
        assert!(pinner.is_empty());
        assert_eq!(pinner.len(), 0);
    }

    #[test]
    fn test_cert_pinner_add_pin() {
        let mut pinner = CertPinner::new();
        pinner.add_pin("example.com".to_string(), "abc123".to_string());
        assert_eq!(pinner.len(), 1);
        assert!(pinner.has_pins("example.com"));
        assert!(!pinner.has_pins("other.com"));
    }

    #[test]
    fn test_cert_pinner_verify_hash() {
        let mut pinner = CertPinner::new();
        pinner.add_pin("example.com".to_string(), "abc123".to_string());

        // Should succeed with matching hash
        assert!(pinner.verify_hash("example.com", "abc123").is_ok());

        // Should fail with non-matching hash
        assert!(pinner.verify_hash("example.com", "wrong").is_err());

        // Should succeed when no pins are configured
        assert!(pinner.verify_hash("other.com", "any_hash").is_ok());
    }

    #[test]
    fn test_cert_pinner_remove_host() {
        let mut pinner = CertPinner::new();
        pinner.add_pin("example.com".to_string(), "abc123".to_string());
        pinner.remove_host("example.com");
        assert!(pinner.is_empty());
    }

    #[test]
    fn test_cert_pinner_clear() {
        let mut pinner = CertPinner::new();
        pinner.add_pin("example.com".to_string(), "abc123".to_string());
        pinner.add_pin("other.com".to_string(), "def456".to_string());
        pinner.clear();
        assert!(pinner.is_empty());
    }

    #[test]
    fn test_load_pins_from_json() {
        let json = r#"{
            "example.com": ["abc123", "def456"],
            "api.example.com": ["ghi789"]
        }"#;

        let pinner = load_pins_from_json(json).unwrap();
        assert_eq!(pinner.len(), 2);
        assert_eq!(pinner.get_pins("example.com").unwrap().len(), 2);
        assert_eq!(pinner.get_pins("api.example.com").unwrap().len(), 1);
    }

    #[test]
    fn test_export_pins_to_json() {
        let mut pinner = CertPinner::new();
        pinner.add_pin("example.com".to_string(), "abc123".to_string());

        let json = export_pins_to_json(&pinner).unwrap();
        assert!(json.contains("example.com"));
        assert!(json.contains("abc123"));
    }
}
