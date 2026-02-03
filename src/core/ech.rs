//! Encrypted Client Hello (ECH) support
//!
//! ECH (formerly ESNI) encrypts the ClientHello message, hiding the SNI
//! from network observers. This module provides ECH config fetching and
//! parsing from DNS HTTPS records.

use serde::{Deserialize, Serialize};

/// ECH configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchConfig {
    /// Raw ECH config bytes
    pub config: Vec<u8>,
    /// Retry configs for ECH retry
    pub retry_configs: Vec<u8>,
    /// ECH version
    pub version: u16,
}

impl EchConfig {
    /// Create a new ECH config
    pub fn new(config: Vec<u8>) -> Self {
        let version = if config.len() >= 2 {
            u16::from_be_bytes([config[0], config[1]])
        } else {
            0xfe0a // Default ECH version
        };
        Self {
            config,
            retry_configs: Vec::new(),
            version,
        }
    }

    /// Create a new ECH config with retry configs
    pub fn with_retry(config: Vec<u8>, retry_configs: Vec<u8>) -> Self {
        let version = if config.len() >= 2 {
            u16::from_be_bytes([config[0], config[1]])
        } else {
            0xfe0a
        };
        Self {
            config,
            retry_configs,
            version,
        }
    }

    /// Check if ECH config is empty
    pub fn is_empty(&self) -> bool {
        self.config.is_empty()
    }

    /// Get the ECH version
    pub fn version(&self) -> u16 {
        self.version
    }
}

impl Default for EchConfig {
    fn default() -> Self {
        Self {
            config: Vec::new(),
            retry_configs: Vec::new(),
            version: 0xfe0a,
        }
    }
}

/// ECH fetch result
#[derive(Debug, Clone)]
pub struct EchFetchResult {
    /// ECH config if found
    pub config: Option<EchConfig>,
    /// Whether the server supports ECH
    pub supports_ech: bool,
}

impl EchFetchResult {
    /// Create a new result with ECH config
    pub fn with_config(config: EchConfig) -> Self {
        Self {
            config: Some(config),
            supports_ech: true,
        }
    }

    /// Create a new result without ECH config
    pub fn without_config() -> Self {
        Self {
            config: None,
            supports_ech: false,
        }
    }

    /// Create a new result indicating ECH support but no config
    pub fn supported_only() -> Self {
        Self {
            config: None,
            supports_ech: true,
        }
    }
}

/// DNS over HTTPS response for ECH config fetching
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct DnsResponse {
    status: u32,
    answer: Option<Vec<DnsRecord>>,
}

/// DNS record from DoH response
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct DnsRecord {
    name: String,
    #[serde(rename = "type")]
    rr_type: u16,
    data: Option<String>,
}

/// Fetch ECH configuration for a domain
///
/// This queries DNS over HTTPS (RFC 8484) for HTTPS records (RFC 9460/9461)
/// which may contain ECH configs.
///
/// The implementation:
/// 1. Queries a DoH resolver for HTTPS records
/// 2. Parses SVCB/HTTPS resource records
/// 3. Extracts ech-config fields from the records
/// 4. Returns the parsed ECH configs
///
/// # Arguments
///
/// * `domain` - The domain to fetch ECH configs for
///
/// # Returns
///
/// Returns an EchFetchResult with the config if found.
///
/// # Examples
///
/// ```rust,ignore
/// let result = fetch_ech_configs("cloudflare.com").await;
/// if let Some(config) = result.config {
///     println!("Found ECH config, version: 0x{:04x}", config.version());
/// }
/// ```
pub async fn fetch_ech_configs(domain: &str) -> EchFetchResult {
    // First check if domain is known to support ECH
    if !domain_supports_ech(domain) {
        return EchFetchResult::without_config();
    }

    // Build DoH query for HTTPS records (type 65)
    // We use Cloudflare's DoH service as the resolver
    let doh_url = format!("https://1.1.1.1/dns-query?name={}&type=HTTPS", domain);

    // Perform DoH query
    #[cfg(feature = "ech")]
    {
        match fetch_doh_https(&doh_url).await {
            Ok(configs) => {
                if let Some(config) = configs.first() {
                    EchFetchResult::with_config(config.clone())
                } else {
                    EchFetchResult::supported_only()
                }
            }
            Err(_) => EchFetchResult::supported_only(),
        }
    }

    #[cfg(not(feature = "ech"))]
    {
        let _ = doh_url; // Suppress unused warning
        EchFetchResult::supported_only()
    }
}

/// Fetch and parse DNS over HTTPS response for HTTPS records
#[cfg(feature = "ech")]
async fn fetch_doh_https(url: &str) -> Result<Vec<EchConfig>, Box<dyn std::error::Error>> {
    use std::time::Duration;

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    // Make DoH request
    let response = client
        .get(url)
        .header("Accept", "application/dns-json")
        .send()
        .await?;

    // Parse JSON response
    let dns_response: DnsResponse = response.json().await?;

    // Extract ECH configs from HTTPS records
    let mut configs = Vec::new();

    if let Some(records) = dns_response.answer {
        for record in records {
            // HTTPS records have type 65
            if record.rr_type == 65 {
                // Parse SVCB/HTTPS record data for ECH config
                // The data is base64-encoded wire format
                if let Some(data) = record.data {
                    if let Some(ech_config) = parse_https_record_ech(&data) {
                        configs.push(ech_config);
                    }
                }
            }
        }
    }

    Ok(configs)
}

/// Parse ECH config from HTTPS record data (base64 encoded wire format)
#[cfg(feature = "ech")]
fn parse_https_record_ech(data: &str) -> Option<EchConfig> {
    use base64::Engine;

    // Decode base64
    let wire_data = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .ok()?;

    // Parse SvcParamKey in wire format (RFC 9460)
    // SvcParamKey ech_config = 5
    parse_ech_config_from_wire(&wire_data)
}

/// Parse ECH config from wire format (binary)
///
/// This parses the ECHConfigList format from draft-ietf-tls-esni
/// The wire format includes:
/// - SvcParamKey (2 bytes)
/// - SvcParamValue length (2 bytes)
/// - ECHConfigList (variable)
#[allow(dead_code)]
fn parse_ech_config_from_wire(data: &[u8]) -> Option<EchConfig> {
    if data.len() < 4 {
        return None;
    }

    // Skip SvcParamKey and length, find the actual ECH config
    let mut pos = 4;
    if pos >= data.len() {
        return None;
    }

    // Try to parse as ECHConfigList
    // Format: ech_config_list(0), ech_config_list(1), ...
    // Each ech_config_list has: len(2), ech_config(len)
    let list_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;

    if pos + list_len > data.len() {
        return None;
    }

    // Get the first ECH config
    let config_len = if pos + 2 <= data.len() {
        u16::from_be_bytes([data[pos], data[pos + 1]]) as usize
    } else {
        return None;
    };
    pos += 2;

    if pos + config_len > data.len() {
        return None;
    }

    let config_bytes = data[pos..pos + config_len].to_vec();
    Some(EchConfig::new(config_bytes))
}

/// Parse ECH config from binary data
///
/// This parses the ECHConfig format from draft-ietf-tls-esni-13
/// The format is:
/// - version (2 bytes) - ECH version (e.g., 0xfe0a for draft-13)
/// - length (2 bytes) - length of config contents
/// - config contents (variable) including:
///   - cipher_suite (2 bytes)
///   - key_exchange (2 bytes)
///   - public_key (variable)
///   - extensions (variable)
///
/// Returns the parsed ECH config with version information.
pub fn parse_ech_config(data: &[u8]) -> Option<EchConfig> {
    if data.len() < 4 {
        return None;
    }

    // Parse version (currently unused but reserved for future use)
    let _version = u16::from_be_bytes([data[0], data[1]]);

    // Parse length
    let _length = u16::from_be_bytes([data[2], data[3]]);

    // For now, return the full config with version
    // A full implementation would parse:
    // - HpkeKeyExchange (2 bytes)
    // - HpkeSymmetricCipherSuites (list)
    // - maximum_name_length (1 byte)
    // - public_key (variable)
    // - extensions (variable)
    Some(EchConfig::with_retry(data.to_vec(), Vec::new()))
}

/// Check if a domain is known to support ECH
///
/// Returns true for domains known to support ECH (Cloudflare, Fastly, etc.)
pub fn domain_supports_ech(domain: &str) -> bool {
    let domain = domain.to_lowercase();

    // Cloudflare domains support ECH
    if domain.ends_with(".cloudflare.com")
        || domain.ends_with(".cloudflareinsights.com")
        || domain == "cloudflare.com"
    {
        return true;
    }

    // Fastly domains support ECH
    if domain.ends_with(".fastly.com") || domain.ends_with(".fastly.net") {
        return true;
    }

    // Google domains support ECH
    if domain.ends_with(".google.com")
        || domain.ends_with(".googlevideo.com")
        || domain.ends_with(".googleapis.com")
        || domain == "google.com"
    {
        return true;
    }

    // Cloudfront domains support ECH
    if domain.ends_with(".cloudfront.net") || domain.ends_with(".awscloudfront.com") {
        return true;
    }

    // Firefox telemetry endpoints
    if domain.contains("firefox.settings.services.mozilla.com")
        || domain.contains("shavar.services.mozilla.com")
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ech_config_empty() {
        let config = EchConfig::default();
        assert!(config.is_empty());
    }

    #[test]
    fn test_ech_config_new() {
        let config = EchConfig::new(vec![0xfe, 0x0a, 0x00, 0x01, 0x02]);
        assert!(!config.is_empty());
        assert_eq!(config.version(), 0xfe0a);
    }

    #[test]
    fn test_domain_supports_ech() {
        // Cloudflare
        assert!(domain_supports_ech("cloudflare.com"));
        assert!(domain_supports_ech("example.cloudflare.com"));

        // Fastly
        assert!(domain_supports_ech("example.fastly.com"));

        // Google
        assert!(domain_supports_ech("google.com"));
        assert!(domain_supports_ech("www.google.com"));

        // Not known to support ECH
        assert!(!domain_supports_ech("example.com"));
    }

    #[test]
    fn test_parse_ech_config_empty() {
        assert!(parse_ech_config(&[]).is_none());
        assert!(parse_ech_config(&[0x01, 0x02]).is_none());
    }

    #[test]
    fn test_parse_ech_config_valid() {
        // Valid ECH config (minimal)
        // version: 0xfe0a (draft-13)
        // length: 0x0003
        // data: 0x01 0x02 0x03
        let data = vec![0xfe, 0x0a, 0x00, 0x03, 0x01, 0x02, 0x03];
        let config = parse_ech_config(&data).unwrap();
        assert_eq!(config.version, 0xfe0a);
        assert_eq!(config.config, data);
    }
}
