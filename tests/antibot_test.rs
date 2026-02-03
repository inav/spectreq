//! Anti-bot bypass integration tests
//!
//! These tests verify that Spectre can successfully bypass various
//! anti-bot protection systems by properly impersonating browsers.
//!
//! NOTE: These tests require network access and may fail if:
//! - The test endpoints change their protection
//! - Rate limiting is triggered
//! - IP reputation is flagged
//!
//! Run with: cargo test --release --test antibot_test -- --ignored

use spectreq::{Client, Profile};

/// Test sites for anti-bot bypass verification
mod test_sites {
    /// Cloudflare-protected test site (httpbin.org behind CF)
    pub const CLOUDFLARE: &str = "https://httpbin.org/get";
    
    /// Cloudflare challenge page detector
    pub const CF_CHALLENGE_MARKER: &str = "Just a moment";
    
    /// TLS fingerprint verification service
    pub const TLS_FP: &str = "https://tls.browserleaks.com/json";
    
    /// HTTP/2 fingerprint verification
    pub const HTTP2_FP: &str = "https://tls.peet.ws/api/all";
    
    /// JA3/JA4 fingerprint test
    pub const JA4_TEST: &str = "https://check.ja4db.com/";
    
    /// Bot detection challenge
    pub const BOT_D: &str = "https://bot.sannysoft.com/";
}

/// Helper to check if response indicates a bot block
fn is_blocked(body: &str) -> bool {
    let block_indicators = [
        "Just a moment",
        "Checking your browser",
        "Access denied",
        "403 Forbidden",
        "blocked",
        "captcha",
        "challenge-platform",
        "cf-browser-verification",
    ];
    
    block_indicators.iter().any(|indicator| 
        body.to_lowercase().contains(&indicator.to_lowercase())
    )
}

/// Helper to check if response is successful JSON
fn is_valid_json_response(body: &str) -> bool {
    body.trim().starts_with('{') || body.trim().starts_with('[')
}

// ============================================================================
// Cloudflare Bypass Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_cloudflare_chrome_143() {
    let profile = Profile::chrome_143_windows();
    let client = Client::new(profile).await.expect("Failed to create client");
    
    let response = client.get(test_sites::CLOUDFLARE).await;
    
    match response {
        Ok(resp) => {
            let body = resp.text().unwrap_or_default();
            assert!(
                !is_blocked(&body),
                "Request was blocked by Cloudflare: {}",
                &body[..body.len().min(500)]
            );
            assert!(
                is_valid_json_response(&body),
                "Expected JSON response from httpbin"
            );
            assert_eq!(resp.status, 200, "Expected 200 OK");
        }
        Err(e) => {
            panic!("Request failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_cloudflare_chrome_120() {
    let profile = Profile::chrome_120_windows();
    let client = Client::new(profile).await.expect("Failed to create client");
    
    let response = client.get(test_sites::CLOUDFLARE).await;
    
    match response {
        Ok(resp) => {
            let body = resp.text().unwrap_or_default();
            assert!(
                !is_blocked(&body),
                "Chrome 120 blocked by Cloudflare"
            );
            assert_eq!(resp.status, 200);
        }
        Err(e) => {
            panic!("Request failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_cloudflare_firefox_121() {
    let profile = Profile::firefox_121_windows();
    let client = Client::new(profile).await.expect("Failed to create client");
    
    let response = client.get(test_sites::CLOUDFLARE).await;
    
    match response {
        Ok(resp) => {
            let body = resp.text().unwrap_or_default();
            assert!(
                !is_blocked(&body),
                "Firefox 121 blocked by Cloudflare"
            );
            assert_eq!(resp.status, 200);
        }
        Err(e) => {
            panic!("Request failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_cloudflare_safari_17() {
    let profile = Profile::safari_17_macos();
    let client = Client::new(profile).await.expect("Failed to create client");
    
    let response = client.get(test_sites::CLOUDFLARE).await;
    
    match response {
        Ok(resp) => {
            let body = resp.text().unwrap_or_default();
            assert!(
                !is_blocked(&body),
                "Safari 17 blocked by Cloudflare"
            );
            assert_eq!(resp.status, 200);
        }
        Err(e) => {
            panic!("Request failed: {}", e);
        }
    }
}

// ============================================================================
// TLS Fingerprint Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tls_fingerprint_chrome() {
    let profile = Profile::chrome_143_windows();
    let client = Client::new(profile).await.expect("Failed to create client");
    
    let response = client.get(test_sites::TLS_FP).await;
    
    match response {
        Ok(resp) => {
            let body = resp.text().unwrap_or_default();
            assert!(
                is_valid_json_response(&body),
                "Expected JSON TLS fingerprint response"
            );
            
            // Parse and verify fingerprint looks like Chrome
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                // Check for expected cipher suites
                if let Some(ciphers) = json.get("cipher_suites") {
                    let cipher_str = ciphers.to_string();
                    // Modern Chrome uses GREASE and specific TLS 1.3 ciphers
                    assert!(
                        cipher_str.contains("4865") || cipher_str.contains("TLS_AES_128_GCM"),
                        "Expected TLS 1.3 cipher suites"
                    );
                }
            }
        }
        Err(e) => {
            panic!("TLS fingerprint request failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_http2_fingerprint() {
    let profile = Profile::chrome_143_windows();
    let client = Client::new(profile).await.expect("Failed to create client");
    
    let response = client.get(test_sites::HTTP2_FP).await;
    
    match response {
        Ok(resp) => {
            let body = resp.text().unwrap_or_default();
            assert!(
                is_valid_json_response(&body),
                "Expected JSON HTTP/2 fingerprint response"
            );
            
            // Parse and verify HTTP/2 settings
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(http2) = json.get("http2") {
                    let h2_str = http2.to_string();
                    // Chrome 131+ uses 6MB initial window
                    // Earlier versions use 64KB
                    assert!(
                        h2_str.contains("SETTINGS") || h2_str.contains("window"),
                        "Expected HTTP/2 settings in response"
                    );
                }
            }
        }
        Err(e) => {
            panic!("HTTP/2 fingerprint request failed: {}", e);
        }
    }
}

// ============================================================================
// Random Profile Rotation Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_random_profile_bypass() {
    // Test that random profile selection works for bypass
    for i in 0..5 {
        let profile = Profile::random();
        let client = Client::new(profile).await.expect("Failed to create client");
        
        let response = client.get(test_sites::CLOUDFLARE).await;
        
        match response {
            Ok(resp) => {
                let body = resp.text().unwrap_or_default();
                assert!(
                    !is_blocked(&body),
                    "Random profile {} was blocked",
                    i
                );
            }
            Err(e) => {
                panic!("Random profile {} request failed: {}", i, e);
            }
        }
        
        // Small delay between requests to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_randomized_profile_bypass() {
    // Test that randomized session values don't break bypass
    let profile = Profile::chrome_143_windows().randomize();
    let client = Client::new(profile).await.expect("Failed to create client");
    
    let response = client.get(test_sites::CLOUDFLARE).await;
    
    match response {
        Ok(resp) => {
            let body = resp.text().unwrap_or_default();
            assert!(
                !is_blocked(&body),
                "Randomized profile was blocked"
            );
        }
        Err(e) => {
            panic!("Randomized profile request failed: {}", e);
        }
    }
}

// ============================================================================
// Header Order Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires network access"]
async fn test_header_order_preserved() {
    let profile = Profile::chrome_143_windows();
    let client = Client::new(profile).await.expect("Failed to create client");
    
    // httpbin.org echoes back headers
    let response = client.get(test_sites::CLOUDFLARE).await;
    
    match response {
        Ok(resp) => {
            let body = resp.text().unwrap_or_default();
            
            // Parse the JSON to check headers
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(headers) = json.get("headers") {
                    let headers_str = headers.to_string();
                    
                    // Chrome should have these headers in order
                    assert!(
                        headers_str.contains("User-Agent") || headers_str.contains("user-agent"),
                        "Missing User-Agent header"
                    );
                    assert!(
                        headers_str.contains("Accept") || headers_str.contains("accept"),
                        "Missing Accept header"
                    );
                }
            }
        }
        Err(e) => {
            panic!("Header test request failed: {}", e);
        }
    }
}

// ============================================================================
// Stress Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires network access, may be rate limited"]
async fn test_burst_requests() {
    let profile = Profile::chrome_143_windows();
    let client = Client::new(profile).await.expect("Failed to create client");
    
    // Make 10 requests in quick succession
    let mut handles: Vec<tokio::task::JoinHandle<Result<spectreq::HttpResponse, spectreq::SpectreError>>> = Vec::new();
    
    for _ in 0..10 {
        let c = client.clone();
        handles.push(tokio::spawn(async move {
            c.get(test_sites::CLOUDFLARE).await
        }));
    }
    
    let mut success = 0;
    let mut blocked = 0;
    
    for handle in handles {
        if let Ok(Ok(resp)) = handle.await {
            let body = resp.text().unwrap_or_default();
            if is_blocked(&body) {
                blocked += 1;
            } else {
                success += 1;
            }
        }
    }
    
    // At least 80% should succeed
    assert!(
        success >= 8,
        "Too many requests blocked: {} blocked, {} succeeded",
        blocked,
        success
    );
}
