# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Spectreq, please report it responsibly:

1. **Do NOT** open a public GitHub issue
2. Email security concerns to the maintainers privately
3. Include details about the vulnerability and steps to reproduce
4. Allow up to 90 days for a fix before public disclosure

## Security Practices

### Cryptography

- **TLS**: Uses rustls for memory-safe TLS implementation
- **Hashing**: MD5 only for RFC 2617 Digest auth (not for security)
- **Post-quantum**: Optional X25519Kyber768 hybrid key exchange

### Dependencies

- Regular dependency audits with `cargo-audit`
- Automated Dependabot updates
- License compatibility checked with `cargo-deny`

### Code Quality

- Memory-safe Rust implementation
- No `unsafe` code in core library
- Clippy linting with `-D warnings`
- Regular fuzzing of parsing code

## Authentication Security

### Supported Methods

| Method | Security Level | Notes |
|--------|---------------|-------|
| Bearer Token | High | Use with HTTPS only |
| Basic Auth | Medium | Base64 encoded, not encrypted |
| Digest Auth | Medium | MD5-based, legacy protocol |
| NTLM | Not Implemented | Deprecated, use Kerberos instead |

### Best Practices

1. **Always use HTTPS** when sending credentials
2. **Rotate tokens** regularly
3. **Use certificate pinning** for sensitive applications
4. **Enable ECH** when possible for privacy

## TLS Security

### Certificate Verification

By default, Spectreq:
- Verifies server certificates
- Uses platform certificate store
- Rejects expired/invalid certificates

### Certificate Pinning

For high-security applications, use SPKI pinning:

```rust
use spectreq::{Client, Profile, CertificatePin};

let client = Client::builder()
    .profile(Profile::chrome_143_windows())
    .pin_certificate(CertificatePin::spki_sha256(
        "example.com",
        "sha256/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
    ))
    .build()?;
```

## Proxy Security

### Recommendations

1. **SOCKS5**: Preferred for better security
2. **HTTP CONNECT**: Use over HTTPS when possible
3. **Authentication**: Store proxy credentials securely
4. **Rotation**: Use proxy rotation for high-volume requests

## Session Persistence

When saving sessions:
- Encrypt session files at rest
- Use appropriate file permissions
- Clear sessions after use
- Never commit session files to version control

## Anti-Bot Considerations

Spectreq is designed for legitimate use cases such as:
- Web scraping with proper authorization
- API testing and development
- Research and security analysis

**Do not use this library to:**
- Bypass security controls without authorization
- Conduct DDoS attacks
- Scrape data in violation of terms of service
- Engage in any illegal activity

## Changelog

Security-related changes are documented in [CHANGELOG.md](./CHANGELOG.md) under the "Security" heading.
