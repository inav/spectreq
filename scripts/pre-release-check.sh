#!/usr/bin/env bash
#
# Pre-release validation checklist for Spectre.
#
# This script performs comprehensive validation before a release,
# checking version consistency, changelog completeness, test status,
# and ensuring we're on the correct branch.
#
# Usage:
#   ./scripts/pre-release-check.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Helper functions
info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }
fail() { error "$1"; exit 1; }
success() { echo -e "${GREEN}[PASS]${NC} $1"; }

# Get project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

info "Starting pre-release validation for Spectre..."

# ============================================================================
# 1. Check we're on main branch
# ============================================================================
info "Checking branch..."

BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$BRANCH" != "main" ]]; then
    fail "Not on main branch. Current branch: $BRANCH"
fi
success "On main branch"

# ============================================================================
# 2. Check for uncommitted changes
# ============================================================================
info "Checking for uncommitted changes..."

if ! git diff-index --quiet HEAD --; then
    fail "There are uncommitted changes. Please commit or stash them first."
fi
success "No uncommitted changes"

# ============================================================================
# 3. Check for untracked files
# ============================================================================
info "Checking for untracked files..."

UNTRACKED=$(git ls-files --others --exclude-standard | wc -l)
if [[ "$UNTRACKED" -gt 0 ]]; then
    warn "Found $UNTRACKED untracked file(s):"
    git ls-files --others --exclude-standard | head -5
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        fail "Aborted by user"
    fi
else
    success "No untracked files"
fi

# ============================================================================
# 4. Version consistency check
# ============================================================================
info "Checking version consistency..."

if ! python3 scripts/sync-version.py --verify; then
    fail "Version mismatch between Cargo.toml and pyproject.toml. Run: python scripts/sync-version.py"
fi
success "Versions are consistent"

# ============================================================================
# 5. Extract version for further checks
# ============================================================================
VERSION=$(grep -m 1 '^version = ' Cargo.toml | head -n 1 | sed 's/version = "\(.*\)"/\1/')
info "Current version: $VERSION"

# ============================================================================
# 6. Validate SemVer format
# ============================================================================
info "Validating SemVer format..."

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]; then
    fail "Invalid SemVer format: $VERSION"
fi
success "Valid SemVer format"

# ============================================================================
# 7. Check CHANGELOG.md has version section
# ============================================================================
info "Checking CHANGELOG.md..."

if [[ ! -f CHANGELOG.md ]]; then
    fail "CHANGELOG.md not found"
fi

if ! grep -q "\[$VERSION\]" CHANGELOG.md; then
    fail "CHANGELOG.md does not have section for version $VERSION"
fi
success "CHANGELOG.md has version section"

# ============================================================================
# 8. Check CHANGELOG.md is not empty for this version
# ============================================================================
info "Checking CHANGELOG.md entries..."

# Extract content between version sections
AWK_SCRIPT="/\[$VERSION\]/{flag=1; next} /\[.*\]/{if(flag) exit} flag"
CHANGELOG_CONTENT=$(awk "$AWK_SCRIPT" CHANGELOG.md)

if [[ -z "$CHANGELOG_CONTENT" ]] || [[ $(echo "$CHANGELOG_CONTENT" | wc -l) -lt 3 ]]; then
    warn "CHANGELOG.md section for $VERSION appears sparse"
    echo "Content:"
    echo "$CHANGELOG_CONTENT"
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        fail "Aborted by user"
    fi
else
    success "CHANGELOG.md has meaningful content"
fi

# ============================================================================
# 9. Check if version is already published to crates.io
# ============================================================================
info "Checking if version is already published to crates.io..."

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "https://crates.io/api/v1/crates/spectre-core/$VERSION" || echo "000")
if [[ "$HTTP_CODE" == "200" ]]; then
    fail "Version $VERSION is already published to crates.io. Please bump version first."
elif [[ "$HTTP_CODE" == "404" ]]; then
    success "Version not yet published to crates.io (as expected)"
else
    warn "Could not verify crates.io status (HTTP code: $HTTP_CODE)"
fi

# ============================================================================
# 10. Check if version is already published to PyPI
# ============================================================================
info "Checking if version is already published to PyPI..."

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "https://pypi.org/pypi/spectre/$VERSION/json" || echo "000")
if [[ "$HTTP_CODE" == "200" ]]; then
    fail "Version $VERSION is already published to PyPI. Please bump version first."
elif [[ "$HTTP_CODE" == "404" ]]; then
    success "Version not yet published to PyPI (as expected)"
else
    warn "Could not verify PyPI status (HTTP code: $HTTP_CODE)"
fi

# ============================================================================
# 11. Run Rust tests
# ============================================================================
info "Running Rust tests..."

if ! cargo test --workspace --quiet 2>&1; then
    fail "Rust tests failed. Please fix failing tests before releasing."
fi
success "Rust tests passed"

# ============================================================================
# 12. Check Python build (if maturin available)
# ============================================================================
info "Checking Python build..."

if command -v maturin &> /dev/null; then
    cd spectre-py
    if ! maturin build --release --strip --out dist --quiet 2>&1; then
        fail "Python build failed. Please fix build issues before releasing."
    fi
    cd "$PROJECT_ROOT"
    success "Python build successful"
else
    warn "maturin not found, skipping Python build check"
fi

# ============================================================================
# 13. Check Rust formatting
# ============================================================================
info "Checking Rust formatting..."

if ! cargo fmt --all -- --check 2>&1; then
    warn "Rust code is not formatted. Run: cargo fmt"
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        fail "Aborted by user"
    fi
else
    success "Rust code is formatted"
fi

# ============================================================================
# 14. Run clippy (if available)
# ============================================================================
info "Running clippy..."

if cargo clippy --version &> /dev/null; then
    if ! cargo clippy --workspace -- -D warnings 2>&1; then
        warn "Clippy found issues. Please review before releasing."
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            fail "Aborted by user"
        fi
    else
        success "Clippy checks passed"
    fi
else
    warn "clippy not found, skipping lint checks"
fi

# ============================================================================
# 15. Summary
# ============================================================================
echo ""
info "========================================="
info "Pre-release validation PASSED!"
info "========================================="
echo ""
info "Ready to release version $VERSION"
echo ""
info "Next steps:"
echo "  1. Review CHANGELOG.md one more time"
echo "  2. Commit changes with: git commit -m 'release: $VERSION'"
echo "  3. Create tag: git tag v$VERSION"
echo "  4. Push: git push && git push --tags"
echo ""
echo "Or use cargo-release:"
echo "  cargo release --patch --execute"
echo ""

exit 0
