# Contributing to Spectre

Thank you for your interest in contributing to Spectre! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Pull Request Process](#pull-request-process)
- [Conventional Commits](#conventional-commits)
- [Development Setup](#development-setup)
- [Running Tests](#running-tests)
- [Code Style](#code-style)
- [Release Process](#release-process)

## Code of Conduct

Be respectful, inclusive, and constructive. We aim to maintain a welcoming environment for all contributors.

## Pull Request Process

1. **Fork and Branch**

   ```bash
   git clone https://github.com/inav/spectre.git
   cd spectreq
   git checkout -b feat/your-feature-name
   ```

2. **Make Changes**
   - Write code following the style guidelines
   - Add tests for new functionality
   - Update documentation as needed

3. **Commit with Conventional Commits**

   ```bash
   git commit -m "feat: add HTTP/3 support for QUIC protocol"
   ```

4. **Push and Create PR**

   ```bash
   git push origin feat/your-feature-name
   ```

   Then create a pull request on GitHub.

5. **PR Labels**
   - Apply appropriate labels: `feat`, `fix`, `docs`, `test`, `chore`, `perf`, `breaking`
   - These labels are used by Release Drafter to categorize changelog entries

## Conventional Commits

Spectre uses [Conventional Commits](https://www.conventionalcommits.org/) for commit messages. This format enables automated changelog generation.

### Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat: add SOCKS5 proxy support` |
| `fix` | Bug fix | `fix: resolve cookie parsing issue for quoted values` |
| `docs` | Documentation changes | `docs: update installation instructions` |
| `style` | Code style changes (formatting, etc.) | `style: run rustfmt on all files` |
| `refactor` | Code refactoring | `refactor: simplify connection pool logic` |
| `perf` | Performance improvements | `perf: reduce memory allocation in header parsing` |
| `test` | Test additions or changes | `test: add integration tests for proxy rotation` |
| `chore` | Maintenance tasks | `chore: update dependencies to latest versions` |
| `ci` | CI/CD changes | `ci: add Python 3.13 to test matrix` |
| `build` | Build system changes | `build: upgrade to Rust 1.93` |

### Scopes

Scopes are optional but recommended for clarity:

| Scope | Description |
|-------|-------------|
| `core` | Changes to `spectreq` |
| `client` | Changes to `spectreq` |
| `py` | Changes to `spectreq-py` (Python bindings) |
| `tls` | TLS-related changes |
| `http2` | HTTP/2-related changes |
| `http3` | HTTP/3-related changes |
| `cache` | Caching-related changes |
| `proxy` | Proxy-related changes |
| `auth` | Authentication-related changes |

### Examples

**Simple feature:**

```bash
feat: add support for Brotli compression
```

**Feature with scope:**

```bash
feat(proxy): implement SOCKS5 proxy authentication
```

**Bug fix:**

```bash
fix: resolve memory leak in connection pool
```

**Bug fix with body:**

```bash
fix(auth): prevent infinite loop in digest auth nonce retry

The digest auth implementation could retry indefinitely
when the server returned the same nonce value. This adds
a maximum retry limit of 3 attempts.
```

**Breaking change:**

```bash
feat!: redesign middleware system API

BREAKING CHANGE: The middleware trait now requires async
methods. Existing middleware implementations must be
updated to use async fn.
```

**Documentation:**

```bash
docs: add examples for proxy rotation usage
```

**Tests:**

```bash
test: add integration tests for HTTP/3 fallback
```

**Chore:**

```bash
chore(deps): update hyper to 1.8
```

### Breaking Changes Format

For breaking changes, append `!` to the type and add a `BREAKING CHANGE` footer:

```bash
feat!: change Client::new signature to accept Builder

BREAKING CHANGE: Client::new now takes a ClientBuilder
instead of individual parameters. Use:
  Client::builder().profile(profile).build()
```

## Development Setup

### Prerequisites

- **Rust**: 1.93.0 or later
- **Python**: 3.11 or later (for Python bindings)
- **Maturin**: For building Python wheels

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Install Python Development Tools

```bash
# Install Maturin for Python bindings
pip install maturin[patchelf]

# Install pre-commit hooks
pip install pre-commit
pre-commit install
```

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/inav/spectre.git
cd spectreq

# Build the workspace
cargo build --release

# Build Python wheel
cd spectreq-py
maturin develop --release
```

## Running Tests

### Rust Tests

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test --package spectreq test_name

# Run tests in release mode
cargo test --workspace --release
```

### Python Tests

```bash
# Build and install the Python package
cd spectreq-py
maturin develop --release

# Run Python tests
pytest

# Run specific test
pytest tests/test_specific.py
```

### Integration Tests

```bash
# Run Rust integration tests
cargo test --workspace --test '*'

# Run Python examples
python examples/basic_request.py
python examples/profiles.py
```

### Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench profile_bench
cargo bench --bench client_bench

# View HTML reports
open target/criterion/report/index.html
```

### Property-Based Tests

```bash
# Run property tests (proptest)
cargo test --release --test property_test

# Run with more iterations (slower but more thorough)
PROPTEST_CASES=1000 cargo test --release --test property_test
```

## Code Style

### Rust

- Use `rustfmt` for formatting:

  ```bash
  cargo fmt
  ```

- Use `clippy` for linting:

  ```bash
  cargo clippy --workspace -- -D warnings
  ```

### Python

- Use `ruff` for formatting and linting:

  ```bash
  ruff check .
  ruff format .
  ```

- Or use pre-commit hooks:

  ```bash
  pre-commit run --all-files
  ```

### General Guidelines

- Write clear, self-documenting code
- Add comments for complex logic
- Keep functions focused and small
- Write tests for new functionality
- Update documentation for API changes

## Release Process

Spectre uses automated releases via GitHub Actions. Releases are triggered by version tags.

### Version Management

- Version is stored in `Cargo.toml` workspace (single source of truth)
- `pyproject.toml` version is synced automatically via script
- Releases follow Semantic Versioning (SemVer)

### Creating a Release

#### Option 1: Automated Release (Recommended)

```bash
# 1. Ensure Conventional Commits in your PRs
# 2. Update CHANGELOG.md with new entries
# 3. Run release prep
cargo release --minor --execute

# 4. This creates a PR with version bump
# 5. Merge PR, push tag to trigger CI
git push origin v0.2.0
```

#### Option 2: Manual Release

```bash
# 1. Update version in Cargo.toml
# 2. Sync version to pyproject.toml
python scripts/sync-version.py

# 3. Update CHANGELOG.md
# 4. Run pre-release checks
./scripts/pre-release-check.sh

# 5. Commit and tag
git add Cargo.toml pyproject.toml CHANGELOG.md
git commit -m "release: 0.2.0"
git tag v0.2.0
git push && git push --tags
```

### What Happens During Release

When you push a version tag (e.g., `v0.2.0`):

1. **Verification**: Version consistency is checked
2. **Testing**: Full test suite runs on Linux, macOS, Windows
3. **Build**: Wheels are built for all platforms
4. **Publish**: Published to Cargo and PyPI simultaneously
5. **Release**: GitHub Release is created with changelog

### Yanking Releases

If a critical issue is found after release:

```bash
# Yank from Cargo (within 24h window)
cargo yank --vers 0.2.0 spectreq
cargo yank --vers 0.2.0 spectreq
cargo yank --vers 0.2.0 spectreq-py

# Yank from PyPI
pip install twine
twine yank spectreq-py 0.2.0
```

### Pre-release Checklist

Before releasing:

- [ ] All tests pass
- [ ] CHANGELOG.md is updated
- [ ] Version is incremented correctly
- [ ] Breaking changes are documented
- [ ] Dependencies are up to date
- [ ] Documentation is updated

## Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [PyO3 Documentation](https://pyo3.rs/)
- [Maturin Documentation](https://www.maturin.rs/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)
