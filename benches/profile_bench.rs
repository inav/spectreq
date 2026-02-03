//! Profile creation and serialization benchmarks
//!
//! Run with: cargo bench --bench profile_bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use spectreq::Profile;

/// Benchmark profile creation
fn profile_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_creation");

    group.bench_function("chrome_143_windows", |b| {
        b.iter(|| black_box(Profile::chrome_143_windows()))
    });

    group.bench_function("chrome_120_windows", |b| {
        b.iter(|| black_box(Profile::chrome_120_windows()))
    });

    group.bench_function("firefox_121_windows", |b| {
        b.iter(|| black_box(Profile::firefox_121_windows()))
    });

    group.bench_function("safari_17_macos", |b| {
        b.iter(|| black_box(Profile::safari_17_macos()))
    });

    group.bench_function("random", |b| b.iter(|| black_box(Profile::random())));

    group.bench_function("random_chrome", |b| {
        b.iter(|| black_box(Profile::random_chrome()))
    });

    group.finish();
}

/// Benchmark profile randomization
fn profile_randomization(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_randomization");

    group.bench_function("randomize", |b| {
        b.iter(|| {
            let profile = Profile::chrome_143_windows();
            black_box(profile.randomize())
        })
    });

    group.finish();
}

/// Benchmark profile serialization
fn profile_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("profile_serialization");

    let profile = Profile::chrome_143_windows();

    group.bench_function("to_json", |b| {
        b.iter(|| black_box(profile.to_json().unwrap()))
    });

    group.bench_function("to_yaml", |b| {
        b.iter(|| black_box(profile.to_yaml().unwrap()))
    });

    let json = profile.to_json().unwrap();
    let yaml = profile.to_yaml().unwrap();

    group.bench_function("from_json", |b| {
        b.iter(|| black_box(Profile::from_json(&json).unwrap()))
    });

    group.bench_function("from_yaml", |b| {
        b.iter(|| black_box(Profile::from_yaml(&yaml).unwrap()))
    });

    group.finish();
}

/// Benchmark header generation
fn header_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("header_generation");

    let profiles = [
        ("chrome_143", Profile::chrome_143_windows()),
        ("firefox_121", Profile::firefox_121_windows()),
        ("safari_17", Profile::safari_17_macos()),
    ];

    for (name, profile) in profiles.iter() {
        group.bench_with_input(
            BenchmarkId::new("get_ordered_headers", name),
            profile,
            |b, p| b.iter(|| black_box(p.get_ordered_headers())),
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    profile_creation,
    profile_randomization,
    profile_serialization,
    header_generation,
);
criterion_main!(benches);
