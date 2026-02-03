# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Encrypted Session Persistence**: Authenticated encryption for session files (ChaCha20-Poly1305 + Argon2)
- **Anti-Bot Testing**: Integration tests for Cloudflare, TLS/HTTP2 fingerprinting, and random profiles
- **Python Bindings**: Exposed `HttpResponse`, `BearerToken`, `BasicAuth`, `DigestAuth`
- **HTTP/3**: Feature flag and stub implementation (experimental)
- **Dynamic Profile Loading**: Load profiles from JSON/YAML files
  - `Profile::from_json_file()` / `Profile::from_yaml_file()`
  - `Profile::from_json()` / `Profile::from_yaml()`
  - `Profile::to_json()` / `Profile::to_yaml()`
- **Profile Randomization**: Anti-detection features
  - `Profile::random()` - Random profile from all browsers
  - `Profile::random_chrome()` - Random Chrome profile
  - `profile.randomize()` - Randomize session-specific values
- **Property-Based Tests**: Added proptest for comprehensive testing
- **Benchmarks**: Added criterion benchmarks for performance tracking
- **Sample Profiles**: Added `profiles/chrome_143_windows.yaml`
- **Documentation**: Added ARCHITECTURE.md and SECURITY.md

### Changed

- Authentication: Replace placeholder MD5 implementation with proper `md-5` crate
- Authentication: NTLM now explicitly marked as unimplemented with documentation
- TLS: Post-quantum support now properly uses `rustls-post-quantum` provider
- Error types: Added `Config` error variant for profile loading errors

### Fixed

- TLS fingerprint validation tests now properly check JA4 format

## [0.1.0] - 2026-02-02

### Added

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
- **Python Bindings**: Full async Python API via PyO3 (Python 3.13+)
- **Certificate Pinning**: SPKI hash-based certificate verification
- **Session Persistence**: Save/load sessions with cookies and TLS tickets

### Browser Profiles

#### Chrome Profiles

- `Profile.chrome_120_windows()` - Chrome 120 on Windows 11
- `Profile.chrome_120_macos()` - Chrome 120 on macOS Sonoma
- `Profile.chrome_120_linux()` - Chrome 120 on Linux
- `Profile.chrome_120_android()` - Chrome 120 on Android 13
- `Profile.chrome_131_windows()` - Chrome 131 on Windows 11 (with larger HTTP/2 window)
- `Profile.chrome_133_windows()` - Chrome 133 on Windows 11
- `Profile.chrome_141_windows()` - Chrome 141 on Windows 11
- `Profile.chrome_143_windows()` - Chrome 143 on Windows 11
- `Profile.chrome_143_macos()` - Chrome 143 on macOS Sonoma
- `Profile.chrome_143_linux()` - Chrome 143 on Linux
- `Profile.chrome_143_android()` - Chrome 143 on Android 13

#### Firefox Profiles

- `Profile.firefox_121_windows()` - Firefox 121 on Windows 11

#### Safari Profiles

- `Profile.safari_17_macos()` - Safari 17 on macOS Sonoma

#### Edge Profiles

- `Profile.edge_120_windows()` - Edge 120 on Windows 11

[Unreleased]: https://github.com/inav/spectre/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/inav/spectre/releases/tag/v0.1.0
