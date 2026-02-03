//! HTTP client benchmarks
//!
//! Run with: cargo bench --bench client_bench
//!
//! Note: Some benchmarks require network access.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use spectreq::{Profile, BasicAuth, DigestAuth, BearerToken};
use std::time::Duration;

/// Benchmark authentication header generation
fn auth_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("auth");
    
    // Basic auth
    let basic = BasicAuth::new("username", "password123");
    group.bench_function("basic_auth_header", |b| {
        b.iter(|| black_box(basic.authorization_header()))
    });
    
    // Bearer token
    let bearer = BearerToken::new("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test");
    group.bench_function("bearer_token_header", |b| {
        b.iter(|| black_box(bearer.authorization_header()))
    });
    
    // Bearer token expiry check
    let bearer_exp = BearerToken::with_expiration("token", Duration::from_secs(3600));
    group.bench_function("bearer_is_expired", |b| {
        b.iter(|| black_box(bearer_exp.is_expired()))
    });
    
    group.bench_function("bearer_expires_soon", |b| {
        b.iter(|| black_box(bearer_exp.expires_soon(Duration::from_secs(60))))
    });
    
    // Digest auth
    let digest = DigestAuth::new("admin", "secret");
    group.bench_function("digest_auth_header", |b| {
        b.iter(|| {
            black_box(digest.authorization_header(
                "GET",
                "/protected/resource",
                "example.com",
                "dcd98b7102dd2f0e8b11d0f600bfb0c093",
                Some("auth"),
                Some("5ccc069c403ebaf9f0171e9517f40e41"),
                1,
            ))
        })
    });
    
    group.finish();
}

/// Benchmark compression type parsing
fn compression_benchmarks(c: &mut Criterion) {
    use spectreq::CompressionType;
    
    let mut group = c.benchmark_group("compression");
    
    let encodings = ["gzip", "br", "deflate", "zstd", "identity"];
    
    for encoding in encodings.iter() {
        group.bench_function(format!("parse_{}", encoding), |b| {
            b.iter(|| black_box(CompressionType::from_encoding(encoding)))
        });
    }
    
    group.finish();
}

/// Benchmark TLS configuration building
fn tls_benchmarks(c: &mut Criterion) {
    use spectreq::build_tls_config;
    
    let mut group = c.benchmark_group("tls");
    
    let chrome = Profile::chrome_143_windows();
    group.bench_function("build_tls_config_chrome", |b| {
        b.iter(|| black_box(build_tls_config(&chrome).unwrap()))
    });
    
    let firefox = Profile::firefox_121_windows();
    group.bench_function("build_tls_config_firefox", |b| {
        b.iter(|| black_box(build_tls_config(&firefox).unwrap()))
    });
    
    group.finish();
}

/// Benchmark JA4 fingerprint generation
fn ja4_benchmarks(c: &mut Criterion) {
    use spectreq::{get_ja4_components, calculate_ja4};
    
    let mut group = c.benchmark_group("ja4");
    
    let chrome = Profile::chrome_143_windows();
    group.bench_function("get_ja4_components", |b| {
        b.iter(|| black_box(get_ja4_components(&chrome)))
    });
    
    group.bench_function("calculate_ja4", |b| {
        b.iter(|| black_box(calculate_ja4(&chrome)))
    });
    
    group.finish();
}

criterion_group!(
    benches,
    auth_benchmarks,
    compression_benchmarks,
    tls_benchmarks,
    ja4_benchmarks,
);
criterion_main!(benches);
