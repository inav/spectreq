# Spectre HTTP Client

A highly efficient HTTP/HTTPS client in Rust with Python bindings that impersonates real browsers (Chrome, Firefox, Safari, Edge) at the network stack level while minimizing bandwidth usage through intelligent caching and compression.

## Features

- **Browser Impersonation**: Network-level impersonation of Chrome, Firefox, Safari, and Edge
  - TLS fingerprinting (JA4/JA4H compatible)
  - HTTP/2 settings per browser
  - TCP configuration per OS
  - User-Agent and header matching
  - Client Hints (Sec-CH-UA-*)
  - Sec-Fetch-* headers
- **Post-Quantum TLS**: Optional X25519MLKem768 hybrid key exchange (Chrome 131+)
- **Bandwidth Optimization**:
  - ETag/Last-Modified conditional requests
  - Automatic decompression (Brotli, Gzip, Deflate, Zstd)
  - Wire size tracking
- **Smart Caching**: In-memory cache with Cache-Control support
- **Cookie Management**: Automatic cookie storage and sending
- **Connection Pooling**: Efficient HTTP/2 and HTTP/1.1 connection reuse with configurable limits
- **Proxy Support**:
  - HTTP CONNECT and SOCKS5 proxy support
  - Runtime proxy switching (change proxy without recreating client)
  - Split proxy configuration (different proxy for HTTP/1-2 vs HTTP/3)
  - Smart proxy rotation with health checking
- **Advanced Routing**:
  - Domain fronting (SNI override for CDN routing)
  - Request timing metrics (DNS, TCP, TLS, TTFB)
  - HTTP/3 support with automatic fallback
  - Encrypted Client Hello (ECH) support
- **Middleware System**: Composable request/response processing
  - Rate limiting
  - Request logging
  - Circuit breaker
  - Custom middleware support
- **Authentication**: Built-in auth helpers
  - Bearer token with auto-refresh
  - Basic auth caching
  - Digest auth (RFC 2617)
  - NTLM support
- **Observability**:
  - Request/response metrics collection
  - Performance percentiles (p50, p90, p95, p99)
  - Per-request timing breakdown
- **Custom Headers**: Set custom headers per client that override defaults
- **Python Bindings**: Full async Python API via PyO3 (Python 3.8+)
- **Certificate Pinning**: SPKI hash-based certificate verification
- **Session Persistence**: Save/load sessions with cookies and TLS tickets

## Documentation

- [Rust Documentation](https://docs.rs/spectreq) - API documentation for Rust
- [Python Examples](./spectreq-py/examples/) - Example scripts demonstrating Python usage
- [Rust Examples](./examples/) - Example programs for Rust

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Available Browser Profiles](#available-browser-profiles)
- [Python API Reference](#python-api-reference)
- [Rust API Reference](#rust-api-reference)
- [Advanced Features](#advanced-features)

## Installation

### From PyPI

```bash
pip install spectreq-py
```

### From Source

```bash
# Build the Rust library
cargo build --release

# Build the Python wheel
cd spectreq-py
maturin build --release --strip

# Install the wheel
pip install target/wheels/spectreq-*.whl
```

### Development Install

```bash
cd spectreq-py
maturin develop --release
```

## Requirements

- **Rust**: 1.93.0 or later
- **Python**: 3.13 or later (for Python bindings)

## Quick Start

### Usage

### Rust

Add `spectreq` to your `Cargo.toml`.

```rust
use spectreq::{Client, Profile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = Profile::chrome_143_windows();
    let client = Client::new(profile);
    
    let response = client.get("https://httpbin.org/get").await?;
    println!("Status: {}", response.status);
    
    Ok(())
}
```

See `examples/*.rs` for more examples:
- [Basic Request](examples/basic.rs)
- [Cookies](examples/cookies.rs)
- [Timing Metrics](examples/timing.rs)

### Python

Install via pip:
```bash
pip install spectreq-py
```

```python
import asyncio
from spectreq import Client, Profile

async def main():
    profile = Profile.chrome_143_windows()
    client = Client(profile)
    
    response = await client.get("https://httpbin.org/get")
    print(f"Status: {response.status_code}")
    print(f"Timing: {response.timing.total}s")

asyncio.run(main())
```

See `examples/python/*.py` for more examples:
- [Basic Request](examples/python/basic_request.py)
- [Cookies](examples/python/cookies_demo.py)
- [Timing Metrics](examples/python/timing_demo.py)



### With Proxy and Custom Headers

```python
import asyncio
from spectreq import Client, Profile

async def main():
    profile = Profile.chrome_120_windows()

    # Create client with proxy and custom headers
    client = Client(
        profile=profile,
        proxy="http://proxy.example.com:8080",
        headers={
            "Authorization": "Bearer token",
            "X-API-Key": "secret",
            "accept-language": "EN"
        }
    )

## Available Browser Profiles

### Chrome Profiles

| Method | Version | OS | User-Agent Example |
|--------|---------|-----|-------------------|
| `Profile.chrome_120_windows()` | 120.0.6099.109 | Windows 11 | Mozilla/5.0 (Windows NT 10.0; Win64; x64)... |
| `Profile.chrome_120_macos()` | 120.0.6099.109 | macOS Sonoma | Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)... |
| `Profile.chrome_120_linux()` | 120.0.6099.109 | Linux | Mozilla/5.0 (X11; Linux x86_64)... |
| `Profile.chrome_120_android()` | 120.0.6099.43 | Android 13 | Mozilla/5.0 (Linux; Android 13)... |
| `Profile.chrome_131_windows()` | 131.0.0.0 | Windows 11 | Mozilla/5.0 (Windows NT 10.0; Win64; x64)... Chrome/131... |
| `Profile.chrome_133_windows()` | 133.0.0.0 | Windows 11 | Mozilla/5.0 (Windows NT 10.0; Win64; x64)... Chrome/133... |
| `Profile.chrome_141_windows()` | 141.0.0.0 | Windows 11 | Mozilla/5.0 (Windows NT 10.0; Win64; x64)... Chrome/141... |
| `Profile.chrome_143_windows()` | 143.0.0.0 | Windows 11 | Mozilla/5.0 (Windows NT 10.0; Win64; x64)... Chrome/143... |
| `Profile.chrome_143_macos()` | 143.0.0.0 | macOS Sonoma | Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)... Chrome/143... |
| `Profile.chrome_143_linux()` | 143.0.0.0 | Linux | Mozilla/5.0 (X11; Linux x86_64)... Chrome/143... |
| `Profile.chrome_143_android()` | 143.0.6099.43 | Android 13 | Mozilla/5.0 (Linux; Android 13)... Chrome/143... |

**Note**: Chrome 131+ uses a larger HTTP/2 initial window size (~6MB) compared to earlier versions (64KB).

### Firefox Profiles

| Method | Version | OS | User-Agent Example |
|--------|---------|-----|-------------------|
| `Profile.firefox_121_windows()` | 121.0 | Windows 11 | Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0)... |

### Safari Profiles

| Method | Version | OS | User-Agent Example |
|--------|---------|-----|-------------------|
| `Profile.safari_17_macos()` | 17.0 | macOS Sonoma | Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)... |

### Edge Profiles

| Method | Version | OS | User-Agent Example |
|--------|---------|-----|-------------------|
| `Profile.edge_120_windows()` | 120.0.2210.61 | Windows 11 | Mozilla/5.0 (Windows NT 10.0; Win64; x64)... Edg/120.0.2210.61 |

## Python API Reference

### Profile Class

The `Profile` class provides pre-configured browser profiles for impersonation.

#### Static Methods

```python
from spectreq import Profile

# Chrome 120 Series
Profile.chrome_120_windows()
Profile.chrome_120_macos()
Profile.chrome_120_linux()
Profile.chrome_120_android()

# Chrome 131+ Series (with larger HTTP/2 window size)
Profile.chrome_131_windows()
Profile.chrome_133_windows()
Profile.chrome_141_windows()

# Chrome 143 Series (latest)
Profile.chrome_143_windows()
Profile.chrome_143_macos()
Profile.chrome_143_linux()
Profile.chrome_143_android()

# Firefox
Profile.firefox_121_windows()

# Safari
Profile.safari_17_macos()

# Edge
Profile.edge_120_windows()

# Random profile selection (anti-detection)
Profile.random()         # Random from all browsers
Profile.random_chrome()  # Random Chrome profile

# Load from file
Profile.from_json_file("profiles/custom.json")
Profile.from_yaml_file("profiles/custom.yaml")

# Load from string
Profile.from_json('{"browser": "Chrome", ...}')
Profile.from_yaml('browser: Chrome\nos: Windows...')
```

#### Anti-Detection Features

```python
# Get a random profile each request
profile = Profile.random()

# Or randomize session-specific values
profile = Profile.chrome_143_windows().randomize()

# Export profile for later use
yaml_str = profile.to_yaml()
json_str = profile.to_json()
```

#### Properties

```python
profile = Profile.chrome_143_windows()

# Get browser name
print(profile.browser)  # "Chrome"

# Get operating system
print(profile.os)  # "Windows"

# Get version string
print(profile.version)  # "143.0.0.0"

# Get user agent string
print(profile.user_agent)  # Full UA string

# Get profile as dict for debugging
info = profile.to_dict()
print(info)
# {'browser': 'Chrome', 'os': 'Windows', 'version': '143.0.0.0', ...}
```

### Client Class

#### Constructor

```python
# Create client with profile
client = Client(profile=profile)

# With proxy
client = Client(
    profile=profile,
    proxy="http://proxy.example.com:8080"
)

# With custom headers (overrides defaults)
client = Client(
    profile=profile,
    headers={
        "Authorization": "Bearer token",
        "X-API-Key": "secret"
    }
)

# With both proxy and headers
client = Client(
    profile=profile,
    proxy="http://user:pass@proxy.example.com:8080",
    headers={
        "Authorization": "Bearer token",
        "X-Custom-Header": "value"
    }
)
```

#### Properties

```python
# Get the proxy configuration
proxy = client.proxy  # None or "http://proxy.example.com:8080"

# Get custom headers
headers = client.headers  # Dict of custom headers
```

#### HTTP Methods

All HTTP methods are async and return a `Response` object.

```python
# GET request
resp = await client.get("https://example.com")

# POST request with bytes
resp = await client.post("https://example.com", body=b"data")

# POST request with string
resp = await client.post("https://example.com", body="data".encode())

# PUT request
resp = await client.put("https://example.com", body=b"data")

# DELETE request
resp = await client.delete("https://example.com")

# PATCH request
resp = await client.patch("https://example.com", body=b"data")

# HEAD request
resp = await client.head("https://example.com")
```

### Response Class

#### Properties

```python
resp = await client.get("https://example.com")

# HTTP status code
status = resp.status_code  # 200

# Wire size (compressed size from network)
wire_size = resp.wire_size  # 366

# Whether response was from cache (304 Not Modified)
from_cache = resp.from_cache  # False or True
```

#### Methods

```python
# Get response body as text
text = resp.text()  # "<!doctype html>..."

# Get response body as bytes
content = resp.content()  # b"<!doctype html>..."

# Parse response as JSON
data = resp.json()  # {"key": "value"}

# Get all headers as dict
headers = resp.headers_dict()
# {"content-type": "text/html", "content-length": "1256"}

# Get specific header value
content_type = resp.get_header("content-type")  # "text/html"
etag = resp.get_header("etag")  # '"33a64af5573fc"'

# Check if request was successful (2xx status)
if resp.ok():
    print("Success!")
```

## HTTP Methods Reference

### Default Headers

All requests include these headers automatically:

| Header | Default Value | Notes |
|--------|---------------|-------|
| `User-Agent` | From profile | Browser-specific |
| `Accept` | `*/*` | |
| `Accept-Encoding` | From profile | `gzip, deflate, br, zstd` |
| `Connection` | `keep-alive` | |
| `Cookie` | Auto-added | If cookies exist |

### Conditional Request Headers

For GET and HEAD requests, the client automatically adds:

| Header | When Added | Description |
|--------|------------|-------------|
| `If-None-Match` | Cached ETag exists | ETag-based validation |
| `If-Modified-Since` | Cached Last-Modified exists | Time-based validation |

### Supported HTTP Methods

| Method | Body Support | Cache Support |
|--------|--------------|---------------|
| GET | No | Yes (ETag, Last-Modified) |
| POST | Yes | No |
| PUT | Yes | No |
| DELETE | No | No |
| PATCH | Yes | No |
| HEAD | No | Yes (ETag, Last-Modified) |

## Caching

The client includes an in-memory cache that respects:

- **ETag**: Sends `If-None-Match` header
- **Last-Modified**: Sends `If-Modified-Since` header
- **Cache-Control**: Respects `max-age` directive

### Default Cache Behavior

| Setting | Default |
|---------|---------|
| Cache enabled | Yes |
| Default max-age (if not specified) | 5 minutes (300 seconds) |
| Cache key format | `METHOD:URL` |

### Cache Example

```python
# First request - fetches from server
resp1 = await client.get("https://example.com")
print(f"Wire size: {resp1.wire_size}")  # e.g., 366 bytes

# Second request - may return 304 with cached body
resp2 = await client.get("https://example.com")
print(f"From cache: {resp2.from_cache}")  # True if 304
print(f"Wire size: {resp2.wire_size}")  # 0 if 304
```

## Compression Support

The client automatically decompresses responses based on the `Content-Encoding` header.

### Supported Compression Types

| Encoding | Support | Compression Ratio |
|----------|---------|-------------------|
| Brotli (`br`) | Yes | ~15-25% |
| Gzip (`gzip`) | Yes | ~20-30% |
| Deflate (`deflate`) | Yes | ~20-30% |
| Zstd (`zstd`) | Yes | ~15-25% |
| Identity (`identity`) | Yes | 100% (no compression) |

### Compression Example

```python
# Accept-Encoding header is set automatically
resp = await client.get("https://example.com")

# Wire size is the compressed size
# Body is automatically decompressed
print(f"Wire (compressed): {resp.wire_size} bytes")
print(f"Body (decompressed): {len(resp.content())} bytes")
```

## Cookie Management

The client automatically stores and sends cookies.

### Cookie Methods (Rust only)

```rust
// Get cookie value for a URL
let cookie_value = client.cookie_jar().get_cookie_value(&url);

// Set cookies from Set-Cookie headers
client.cookie_jar().set_cookies(&["session=abc123"], &url);

// Clear all cookies
client.cookie_jar().clear();

// Get number of cookies
let count = client.cookie_jar().len();

// Check if empty
let empty = client.cookie_jar().is_empty();

// Remove cookies for a domain
client.cookie_jar().remove_for_domain("example.com");
```

## Proxy Support

Spectre supports HTTP/HTTPS proxies with automatic CONNECT method handling for HTTPS URLs.

### Proxy Configuration

```python
# HTTP proxy
client = Client(
    profile=profile,
    proxy="http://proxy.example.com:8080"
)

# HTTPS proxy with authentication
client = Client(
    profile=profile,
    proxy="http://user:password@proxy.example.com:8080"
)

# From environment variable
import os
proxy = os.getenv("HTTP_PROXY")
client = Client(profile=profile, proxy=proxy)
```

### How It Works

- **HTTP URLs**: Direct connection to proxy, hyper handles the request
- **HTTPS URLs**: Uses HTTP CONNECT method to establish a tunnel through the proxy
  1. Client connects to proxy
  2. Sends `CONNECT host:port HTTP/1.1` request
  3. Proxy returns `200 Connection established`
  4. TLS handshake is performed through the tunnel
  5. Encrypted data flows through the proxy

### Proxy URL Format

```
http://[user:password@]host[:port]
https://[user:password@]host[:port]
```

Examples:

- `http://proxy.example.com:8080`
- `http://user:pass@proxy.example.com:8080`
- `http://proxy.example.com` (defaults to port 8080)

## Custom Headers

Custom headers can be set per client and will override default headers.

### Setting Custom Headers

```python
# Set custom headers that override defaults
client = Client(
    profile=profile,
    headers={
        "Authorization": "Bearer token123",
        "X-API-Key": "secret-key",
        "accept-language": "ar",  # Overrides default Accept-Language
        "User-Agent": "CustomBot/1.0"  # Overrides profile UA
    }
)
```

### Default Headers

These headers are set by default (can be overridden):

| Header | Default Value |
|--------|---------------|
| `User-Agent` | From profile |
| `Accept` | `*/*` |
| `Accept-Encoding` | From profile (e.g., `gzip, deflate, br, zstd`) |
| `Connection` | `keep-alive` |

### Header Priority

Custom headers have higher priority and will override defaults:

```python
# This will override the default Accept-Encoding
client = Client(
    profile=profile,
    headers={"Accept-Encoding": "gzip"}  # Only gzip, no br/zstd
)
```

## TLS Fingerprinting

Each profile has unique TLS settings to match real browsers:

| Browser | Cipher Suites | Extensions | GREASE | ALPN |
|---------|--------------|------------|--------|------|
| Chrome 120 | 9 ciphers | 7 extensions | Yes | h2, http/1.1 |
| Firefox 121 | 7 ciphers | 7 extensions | No | h2, http/1.1 |
| Safari 17 | 5 ciphers | 7 extensions | No | h2, http/1.1 |
| Edge 120 | 3 ciphers | 7 extensions | Yes | h2, http/1.1 |

### TLS Configuration (Profile Builder)

```rust
use spectreq::Profile;

let custom_profile = Profile::builder()
    .browser(BrowserName::Chrome)
    .os(OS::Windows)
    .version("120.0")
    .user_agent("Custom User-Agent")
    .http2_initial_window_size(65536)
    .http2_max_concurrent_streams(256)
    .http2_header_table_size(65536)
    .tls_cipher_suites(vec![
        "TLS_AES_128_GCM_SHA256".to_string(),
        "TLS_AES_256_GCM_SHA384".to_string(),
    ])
    .tls_grease(true)
    .tcp_ttl(128)
    .build();
```

### HTTP/2 Settings

| Setting | Chrome | Firefox | Safari | Edge |
|---------|--------|---------|--------|------|
| Initial Window Size | 65536 | 65535 | 65536 | 65536 |
| Max Concurrent Streams | 256 | 100 | 100 | 256 |
| Header Table Size | 65536 | 4096 | 4096 | 65536 |

## Error Handling

### Python

```python
from spectreq import SpectreError

try:
    resp = await client.get("https://example.com")
except Exception as e:
    print(f"Request failed: {e}")
```

### Rust

```rust
use spectreq::SpectreError;

match client.get("https://example.com").await {
    Ok(resp) => println!("Success: {}", resp.status),
    Err(SpectreError::Http(e)) => eprintln!("HTTP error: {}", e),
    Err(SpectreError::Tls(e)) => eprintln!("TLS error: {}", e),
    Err(SpectreError::InvalidUrl(e)) => eprintln!("Invalid URL: {}", e),
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Error Types

| Error | Description |
|-------|-------------|
| `Tls` | TLS handshake or certificate errors |
| `Http` | HTTP protocol errors |
| `Connection` | Network connection errors |
| `InvalidProfile` | Profile configuration errors |
| `Compression` | Decompression errors |
| `InvalidUrl` | URL parsing errors |
| `Timeout` | Request timeout |
| `Io` | File/IO errors |
| `Hyper` | Hyper client errors |

## Advanced Examples

### Python Examples

The following example scripts are available in the `examples/python/` directory:

| Example | Description |
|---------|-------------|
| `basic_request.py` | Simple GET request demonstration |
| `cookies_demo.py` | CookieJar and session management |
| `timing_demo.py` | Response timing and metrics |
| `post_request.py` | POST request with JSON data |
| `profiles.py` | Display all available browser profiles |

Run examples with:

```bash
python examples/python/basic_request.py
python examples/python/cookies_demo.py
python examples/python/timing_demo.py
```

### Rust Examples

The following example programs are available in `examples/`:

| Example | Description |
|---------|-------------|
| `basic.rs` | Simple GET request |
| `cookies.rs` | Cookie management |
| `timing.rs` | Request timing metrics |

Run examples with:

```bash
cargo run --example basic
cargo run --example cookies
cargo run --example timing
```

### Multiple Requests with Different Profiles

```python
import asyncio
from spectreq import Client, Profile

async def fetch_with_all_profiles(url):
    profiles = [
        ("Chrome", Profile.chrome_120_windows()),
        ("Firefox", Profile.firefox_121_windows()),
        ("Safari", Profile.safari_17_macos()),
    ]

    for name, profile in profiles:
        client = Client(profile=profile)
        resp = await client.get(url)
        print(f"{name}: {resp.status_code}, Wire: {resp.wire_size}")

asyncio.run(fetch_with_all_profiles("https://example.com"))
```

### POST with JSON

```python
import json
from spectreq import Client, Profile

async def post_json():
    profile = Profile.chrome_120_windows()
    client = Client(profile=profile)

    data = json.dumps({"key": "value"}).encode()
    resp = await client.post("https://httpbin.org/post", body=data)
    print(resp.text())
```

### Download with Progress

```python
async def download(url, filename):
    profile = Profile.chrome_120_windows()
    client = Client(profile=profile)

    resp = await client.get(url)
    with open(filename, "wb") as f:
        f.write(resp.content())

    print(f"Downloaded {len(resp.content())} bytes (wire: {resp.wire_size})")
```

### Custom Profile in Rust

```rust
use spectreq::{Profile, BrowserName, OS};

let custom = Profile::builder()
    .browser(BrowserName::Chrome)
    .os(OS::Linux)
    .version("121.0")
    .user_agent("Mozilla/5.0 (X11; Linux x86_64)...")
    .build();

let client = Client::new(custom).await?;
```

## Performance Tips

1. **Reuse Clients**: Create one client and reuse it for multiple requests
2. **Enable Caching**: Default caching reduces bandwidth for repeated requests
3. **Use HTTP/2**: All profiles support HTTP/2 for multiplexing
4. **Compression**: Automatic compression reduces bandwidth by ~70-85%
5. **Connection Pooling**: Reuses connections automatically for better performance
6. **Proxy Rotation**: Smart rotation with health checking improves reliability

## Advanced Features (Rust API)

Spectre provides advanced features beyond basic HTTP requests. These are available in the Rust API:

### Runtime Proxy Switching

Change proxy configuration without recreating the client:

```rust
use spectreq::Client;
use spectreq::Profile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = Profile::chrome_143_windows();
    let mut client = Client::new(profile).await?;

    // Make request with initial proxy (or no proxy)
    let resp = client.get("https://example.com").await?;
    println!("Status: {}", resp.status);

    // Change proxy mid-session
    client.set_proxy(Some("socks5://127.0.0.1:1080".to_string()));

    // Set different proxies for HTTP/1-2 vs HTTP/3
    client.set_tcp_proxy(Some("http://proxy1.example.com:8080".to_string()));
    client.set_udp_proxy(Some("socks5://proxy2.example.com:1080".to_string()));

    // Subsequent requests use new proxy
    let resp = client.get("https://example.com").await?;
    println!("Status: {}", resp.status);

    Ok(())
}
```

### Domain Fronting

Override SNI and connect target for CDN routing:

```rust
use spectreq::Profile;

// Create profile with domain fronting
let profile = Profile::builder()
    .browser(Profile::chrome_143_windows().browser)
    .os(Profile::chrome_143_windows().os)
    .version("143.0")
    // Connect to CDN but SNI to origin
    .connect_to(Some("cdn.example.com".to_string()))
    .sni_override(Some("origin.example.com".to_string()))
    .build();
```

### Request Timing Metrics

Get detailed timing information for each request:

```rust
use spectreq::Client;

let resp = client.get("https://example.com").await?;

// Access timing metrics
println!("DNS lookup: {:?}", resp.timing.dns_lookup);
println!("TCP connect: {:?}", resp.timing.tcp_connect);
println!("TLS handshake: {:?}", resp.timing.tls_handshake);
println!("TTFB: {:?}", resp.timing.ttfb);
println!("Total time: {:?}", resp.timing.total);
```

### HTTP/3 with Automatic Fallback

HTTP/3 support with automatic fallback to HTTP/2 and HTTP/1.1:

```rust
use spectreq::{Client, HttpVersion};

let profile = Profile::chrome_143_windows();

// Client with HTTP/3 enabled (requires "http3" feature)
let mut client = Client::new(profile).await?;
client.set_http3_enabled(true);
client.set_preferred_http_version(HttpVersion::H3);

// Will try HTTP/3, fall back to HTTP/2, then HTTP/1.1
let resp = client.get("https://cloudflare.com").await?;
println!("Status: {}", resp.status);
```

### Encrypted Client Hello (ECH)

Enable ECH for better privacy:

```rust
use spectreq::Profile;

// Enable ECH with auto-discovery
let profile = Profile::chrome_143_windows()
    .with_ech_enabled(true)
    .with_ech_config(
        // ECH config can be fetched from DNS or hardcoded
        Some(spectreq::EchConfig::from_dns("example.com").await?)
    );

let client = Client::new(profile).await?;
let resp = client.get("https://example.com").await?;
```

### Connection Pooling

Configure connection pool behavior:

```rust
use spectreq::{Client, PoolConfig};

let pool_config = PoolConfig::new()
    .max_connections_per_host(100)
    .max_idle_connections(10)
    .idle_timeout(std::time::Duration::from_secs(90))
    .max_lifetime(Some(std::time::Duration::from_secs(300)))
    .enabled(true);

let profile = Profile::chrome_143_windows();
let mut client = Client::new(profile).await?;
client.set_pool_config(pool_config);

// Check pool statistics
let stats = client.pool_stats("example.com").await?;
println!("Idle connections: {}", stats.idle_connections);
println!("Active connections: {}", stats.active_connections);
```

### Smart Proxy Rotation

Automatic proxy rotation with health checking:

```rust
use spectreq::{ProxyRotator, RotationConfig};
use std::time::Duration;

// Configure rotation behavior
let rotation_config = RotationConfig::new()
    .failure_threshold(3)
    .backoff_duration(Duration::from_secs(60))
    .max_backoff_duration(Duration::from_secs(3600))
    .health_check_interval(Duration::from_secs(300))
    .health_check_timeout(Duration::from_secs(10))
    .enabled(true);

// Create rotator and add proxies
let rotator = ProxyRotator::new(rotation_config);
rotator.add_proxy("socks5://proxy1.example.com:1080".await;
rotator.add_proxy("socks5://proxy2.example.com:1080".await;
rotator.add_proxy("socks5://proxy3.example.com:1080".await;

// Use with client
let mut client = Client::new(profile).await?;
client.set_proxy_rotator(rotator);

// Get proxy statuses
let statuses = client.proxy_statuses().await;
for status in statuses {
    println!("Proxy: {} - Healthy: {} - Success Rate: {:.2}%",
        status.url, status.is_healthy, status.success_rate * 100.0);
}
```

### Middleware Chain

Compose middleware for request/response processing:

```rust
use spectreq::{MiddlewareChainBuilder, RateLimiter, RequestLogger, CircuitBreaker};
use std::time::Duration;

// Build middleware chain
let middleware = MiddlewareChainBuilder::new()
    .rate_limiter(10, Duration::from_secs(60))  // 10 requests per minute
    .logger(true, false)  // Log headers, not body
    .circuit_breaker(5, Duration::from_secs(30))  // Open after 5 failures
    .build();

let mut client = Client::new(profile).await?;
client.set_middleware(middleware);
```

### Authentication Helpers

Built-in support for common authentication methods:

```rust
use spectreq::{
    BearerToken, BearerTokenManager, BasicAuth,
    DigestAuth, AuthConfig
};
use std::time::Duration;

// Bearer token with auto-refresh
let token = BearerToken::with_refresh_token(
    "access_token_123",
    "refresh_token_456",
    Duration::from_secs(3600)
);
let token_manager = BearerTokenManager::new()
    .with_refresh_url("https://auth.example.com/refresh");
token_manager.set_token(token).await;

// Basic auth
let auth = BasicAuth::new("username", "password");
let header = auth.authorization_header();  // "Basic base64(username:password)"

// Digest auth
let digest = DigestAuth::new("username", "password");
let header = digest.authorization_header(
    "GET",
    "/protected",
    "testrealm",
    "dcd98b7102dd2f0e8b11d0f600bfb0c093",
    Some("auth"),
    Some("5ccc069c403ebaf9f0171e9517f40e41"),
    1
);
```

### Metrics Collection

Collect and analyze request/response metrics:

```rust
use spectreq::{MetricsCollector, RequestTimer, RequestMetrics};

// Create metrics collector
let collector = MetricsCollector::with_max_metrics(10000);

// Make request with timer
let mut timer = RequestTimer::new();
timer.start_dns();
// ... DNS lookup happens ...
timer.end_dns();

timer.start_tcp();
// ... TCP connection happens ...
timer.end_tcp();

// ... rest of request ...

let metrics = RequestMetrics {
    status: resp.status.as_u16(),
    method: "GET".to_string(),
    url: "https://example.com".to_string(),
    response_time_ms: timer.elapsed().as_millis() as u64,
    dns_time_us: dns_duration.as_micros() as u64,
    tcp_time_us: tcp_duration.as_micros() as u64,
    tls_time_us: tls_duration.as_micros() as u64,
    ttfb_us: ttfb_duration.as_micros() as u64,
    request_size: 0,
    response_size: resp.body.len(),
    from_cache: false,
    retries: 0,
    timestamp: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_micros() as u64,
};

// Record metrics
collector.record(metrics).await;

// Get statistics
let stats = collector.stats().await;
println!("Total requests: {}", stats.total_requests);
println!("Success rate: {:.1}%",
    stats.successful_requests as f64 / stats.total_requests as f64 * 100.0);
println!("Avg response time: {:.2}ms", stats.avg_response_time_ms);

// Get percentiles
let percentiles = collector.percentiles().await;
println!("p50: {}ms", percentiles.p50_response_time_ms);
println!("p95: {}ms", percentiles.p95_response_time_ms);
println!("p99: {}ms", percentiles.p99_response_time_ms);
```

## Advanced Examples (Rust)

### Example: Proxy Switching

```rust
use spectreq::Client;
use spectreq::Profile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = Profile::chrome_143_windows();
    let mut client = Client::new(profile).await?;

    // Start without proxy
    let resp = client.get("https://httpbin.org/ip").await?;
    println!("No proxy: {}", resp.text()?);

    // Switch to SOCKS5 proxy
    client.set_proxy(Some("socks5://127.0.0.1:1080".to_string()));
    let resp = client.get("https://httpbin.org/ip").await?;
    println!("With SOCKS5: {}", resp.text()?);

    // Switch to HTTP proxy
    client.set_proxy(Some("http://proxy.example.com:8080".to_string()));
    let resp = client.get("https://httpbin.org/ip").await?;
    println!("With HTTP proxy: {}", resp.text()?);

    Ok(())
}
```

### Example: Middleware Chain

```rust
use spectreq::{
    Client, MiddlewareChainBuilder,
    RateLimiter, RequestLogger, CircuitBreaker
};
use spectreq::Profile;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = Profile::chrome_143_windows();

    // Create middleware chain
    let middleware = MiddlewareChainBuilder::new()
        .rate_limiter(100, Duration::from_secs(60))  // 100 req/min
        .logger(true, false)  // Log headers
        .circuit_breaker(5, Duration::from_secs(30))  // Circuit breaker
        .build();

    let mut client = Client::new(profile).await?;
    client.set_middleware(middleware);

    // Requests will be logged, rate-limited, and circuit-breaker protected
    for i in 0..10 {
        let resp = client.get("https://example.com").await?;
        println!("Request {}: {}", i, resp.status);
    }

    Ok(())
}
```

### Example: Metrics Collection

```rust
use spectreq::{Client, MetricsCollector};
use spectreq::Profile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = Profile::chrome_143_windows();
    let client = Client::new(profile).await?;

    // Enable metrics collection
    let collector = MetricsCollector::new();
    client.set_metrics_collector(collector.clone());

    // Make requests
    for _ in 0..10 {
        let resp = client.get("https://example.com").await?;
        println!("Status: {}", resp.status);
    }

    // Analyze metrics
    let stats = collector.stats().await;
    println!("Total requests: {}", stats.total_requests);
    println!("Successful: {}", stats.successful_requests);
    println!("Failed: {}", stats.failed_requests);
    println!("Avg response time: {:.2}ms", stats.avg_response_time_ms);
    println!("Cache hit rate: {:.1}%", stats.cache_hit_rate * 100.0);

    // Get percentiles
    let p = collector.percentiles().await;
    println!("p50: {}ms", p.p50_response_time_ms);
    println!("p95: {}ms", p.p95_response_time_ms);
    println!("p99: {}ms", p.p99_response_time_ms);

    Ok(())
}
```

### Example: Domain Fronting

```rust
use spectreq::Profile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to CloudFront but SNI to origin
    let profile = Profile::chrome_143_windows()
        .with_connect_to("cloudfront.net")
        .with_sni_override("example.com");

    let client = Client::new(profile).await?;
    let resp = client.get("https://example.com").await?;
    println!("Status: {}", resp.status);

    Ok(())
}
```

### Example: Connection Pooling

```rust
use spectreq::{Client, PoolConfig};
use spectreq::Profile;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = Profile::chrome_143_windows();

    // Configure connection pool
    let pool_config = PoolConfig::new()
        .max_connections_per_host(50)
        .max_idle_connections(5)
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Some(Duration::from_secs(300)))
        .enabled(true);

    let mut client = Client::new(profile).await?;
    client.set_pool_config(pool_config);

    // Multiple requests will reuse connections
    for i in 0..10 {
        let resp = client.get("https://example.com").await?;
        println!("Request {}: {}", i, resp.status);
    }

    // Check pool stats
    let stats = client.pool_stats("example.com").await?;
    println!("Idle connections: {}", stats.idle_connections);
    println!("Active connections: {}", stats.active_connections);

    Ok(())
}
```

## Project Structure

```
spectreq/
├── Cargo.toml              # Single crate configuration
├── src/                    # Main source directory
│   ├── lib.rs             # Public API re-exports
│   ├── core/              # Core types and profiles
│   │   ├── profile.rs     # Browser profiles
│   │   ├── tls.rs         # TLS configuration
│   │   ├── tcp.rs         # TCP configuration
│   │   └── error.rs       # Error types
│   ├── client/            # HTTP client implementation
│   │   ├── client.rs      # Main client
│   │   ├── connector.rs   # HTTP connector
│   │   ├── cache.rs       # HTTP caching
│   │   ├── cookies.rs     # Cookie management
│   │   └── compression.rs # Decompression
│   └── py/                # Python bindings (feature-gated)
│       ├── mod.rs         # Module definition
│       ├── client.rs      # Python client wrapper
│       └── profile.rs     # Python profile wrapper
├── spectreq-py/             # Python wrapper crate
│   └── Cargo.toml
├── examples/               # Rust examples
└── target/wheels/          # Built Python wheels
```

## License

MIT OR Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines on:

- Pull Request Process
- Conventional Commits
- Development Setup
- Running Tests
- Code Style

## Release Process

Spectre uses automated releases via GitHub Actions. Releases are triggered by version tags and are published to both Cargo (crates.io) and PyPI.

### Version Management

- **Version Source**: `Cargo.toml` workspace.package.version is the single source of truth
- **Python Version**: `pyproject.toml` version is automatically synced via script
- **Versioning**: Semantic Versioning (SemVer) - <https://semver.org/>
- **Changelog**: [CHANGELOG.md](./CHANGELOG.md) follows [Keep a Changelog](https://keepachangelog.com/) format

### Creating a Release

#### Automated Release (Recommended)

Using `cargo-release`:

```bash
# 1. Ensure Conventional Commits in your PRs
#    (Used for automated changelog generation)

# 2. Update CHANGELOG.md with new entries
#    Add features, bug fixes, breaking changes under [Unreleased]

# 3. Run release prep (creates a commit with version bump)
cargo release --minor --execute

# 4. Push the commit
git push origin main

# 5. Create and push the tag (triggers CI)
git tag v0.2.0
git push origin v0.2.0
```

#### Manual Release

```bash
# 1. Update version in Cargo.toml
#    Edit workspace.package.version

# 2. Sync version to pyproject.toml
python scripts/sync-version.py

# 3. Update CHANGELOG.md
#    - Move entries from [Unreleased] to new version section
#    - Add release date
#    - Update comparison links at bottom

# 4. Run pre-release checks
./scripts/pre-release-check.sh

# 5. Commit and tag
git add Cargo.toml pyproject.toml CHANGELOG.md
git commit -m "release: 0.2.0"
git tag v0.2.0

# 6. Push to trigger CI
git push origin main
git push origin v0.2.0
```

### What Happens During Release

When you push a version tag (e.g., `v0.2.0`), the GitHub Actions workflow:

1. **Verification**: Checks version consistency (Cargo.toml, pyproject.toml, tag)
2. **Testing**: Runs full test suite on Linux, macOS, Windows
3. **Build**:
   - Builds Rust crates
   - Builds Python wheels via cibuildwheel (manylinux, musllinux, macOS, Windows)
   - Builds source distribution
4. **Publish**: Simultaneously publishes to:
   - **Cargo** (crates.io): spectreq, spectreq, spectreq-py
   - **PyPI**: All wheels + sdist for `spectreq-py` package
5. **Release**: Creates GitHub Release with changelog entries

### Release Artifacts

| Artifact | Location | Description |
|----------|----------|-------------|
| `spectreq` | <https://crates.io/crates/spectreq-core> | Core Rust crate |
| `spectreq` | <https://crates.io/crates/spectreq-client> | Client Rust crate |
| `spectreq-py` | <https://crates.io/crates/spectreq-py> | Python bindings Rust crate |
| `spectre` | <https://pypi.org/project/spectreq/> | Python package (wheels) |

### Pre-release Checklist

Before creating a release:

- [ ] All tests pass (`cargo test --workspace` and `pytest`)
- [ ] CHANGELOG.md is updated with all changes
- [ ] Version is incremented correctly (MAJOR.MINOR.PATCH)
- [ ] Breaking changes are documented in CHANGELOG.md
- [ ] `scripts/pre-release-check.sh` passes
- [ ] Dependencies are up to date

### Yanking Releases

If a critical issue is found after release:

**Cargo (crates.io):**

```bash
cargo yank --vers 0.2.0 spectreq
cargo yank --vers 0.2.0 spectreq
cargo yank --vers 0.2.0 spectreq-py

# To unyank (if issue is fixed):
cargo unyank --vers 0.2.0 spectreq
```

**PyPI:**

```bash
pip install twine
twine yank spectreq-py 0.2.0

# To unyank:
twine unyank spectreq-py 0.2.0
```

**Note:** Yanking should be done within 24 hours of release to minimize impact.

### Testing Before Release

To test the release process without actually publishing:

```bash
# Test PyPI publish workflow
./scripts/test-pypi-publish.sh

# This builds the wheel and publishes to TestPyPI
# You can then test install from TestPyPI before the real release
```

### Version Bumping Guide

| Change Type | Bump | Example |
|-------------|------|---------|
| Bug fix | PATCH | 0.1.0 → 0.1.1 |
| New feature (backwards compatible) | MINOR | 0.1.0 → 0.2.0 |
| Breaking change | MAJOR | 0.1.0 → 1.0.0 |

**For 0.x versions:**

- Breaking changes increment MINOR (signals stability intent)
- Non-breaking features/fixes increment PATCH

## See Also

- [Examples](./examples/) - Example scripts demonstrating Spectre usage
- [JA4 TLS Fingerprinting](https://github.com/FoxIO-LLC/ja4)
- [HTTP/2 Specification](https://httpwg.org/specs/rfc9113.html)
- [RFC 7234 - HTTP Caching](https://httpwg.org/specs/rfc7234.html)
