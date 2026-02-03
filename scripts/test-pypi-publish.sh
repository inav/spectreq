#!/usr/bin/env bash
#
# Test PyPI publish script for Spectre.
#
# This script builds the Python wheel and publishes to TestPyPI
# for validation before the actual release.
#
# Usage:
#   ./scripts/test-pypi-publish.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
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

# TestPyPI configuration
TEST_PYPI_URL="https://test.pypi.org/simple/"
TEST_PYPI_UPLOAD_URL="https://upload.pypi.org/legacy/"

# Extract version
VERSION=$(grep -m 1 '^version = ' Cargo.toml | head -n 1 | sed 's/version = "\(.*\)"/\1/')

info "Testing PyPI publish workflow for version $VERSION..."
echo ""

# ============================================================================
# 1. Check prerequisites
# ============================================================================
info "Checking prerequisites..."

# Check for required tools
if ! command -v python3 &> /dev/null; then
    fail "python3 not found. Please install Python 3.11 or later."
fi
success "python3 found: $(python3 --version)"

if ! command -v maturin &> /dev/null; then
    info "Installing maturin..."
    pip install maturin[patchelf]
fi
success "maturin found"

if ! command -v twine &> /dev/null; then
    info "Installing twine..."
    pip install twine
fi
success "twine found"

# Check for TEST_PYPI_API_TOKEN
if [[ -z "${TEST_PYPI_API_TOKEN:-}" ]]; then
    warn "TEST_PYPI_API_TOKEN not set"
    info "Get your token from: https://test.pypi.org/manage/account/token/"
    info "Set it with: export TEST_PYPI_API_TOKEN=your_token_here"
    echo ""
    read -p "Continue without TEST_PYPI_API_TOKEN? (build only) [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        fail "Aborted by user"
    fi
    BUILD_ONLY=true
else
    BUILD_ONLY=false
    success "TEST_PYPI_API_TOKEN is set"
fi

# ============================================================================
# 2. Clean previous builds
# ============================================================================
info "Cleaning previous builds..."

rm -rf spectre-py/dist/
rm -rf spectre-py/target/wheels/
success "Cleaned previous builds"

# ============================================================================
# 3. Build wheels
# ============================================================================
info "Building Python wheels..."

cd spectre-py
maturin build --release --strip --out dist --find-interpreter

if [[ ! -d dist ]] || [[ -z "$(ls -A dist)" ]]; then
    fail "Build failed: no wheels found in dist/"
fi

# List built files
echo ""
info "Built files:"
ls -lh dist/
echo ""

cd "$PROJECT_ROOT"
success "Wheels built successfully"

# ============================================================================
# 4. Run twine check
# ============================================================================
info "Running twine check..."

cd spectre-py
if ! twine check dist/*; then
    fail "Twine check failed. Please fix the issues."
fi
cd "$PROJECT_ROOT"
success "Twine check passed"

# ============================================================================
# 5. Publish to TestPyPI (if token is set)
# ============================================================================
if [[ "$BUILD_ONLY" == "true" ]]; then
    info "BUILD_ONLY mode: skipping publish"
    echo ""
    info "To publish manually:"
    echo "  cd spectre-py"
    echo "  TWINE_USERNAME=__token__ \\"
    echo "  TWINE_PASSWORD=\$TEST_PYPI_API_TOKEN \\"
    echo "  twine upload --repository testpypi dist/*"
    echo ""
    exit 0
fi

info "Publishing to TestPyPI..."

cd spectre-py
TWINE_USERNAME=__token__ \
TWINE_PASSWORD="$TEST_PYPI_API_TOKEN" \
twine upload --repository testpypi --verbose dist/*

if [[ $? -ne 0 ]]; then
    error "Failed to publish to TestPyPI"
    fail "Please check the error messages above"
fi

cd "$PROJECT_ROOT"
success "Published to TestPyPI"

# ============================================================================
# 6. Show test install instructions
# ============================================================================
echo ""
info "========================================="
info "TestPyPI Publish Successful!"
info "========================================="
echo ""
info "To test install from TestPyPI:"
echo ""
echo -e "${BLUE}# Create a temporary virtual environment${NC}"
echo "python3 -m venv test_env"
echo "source test_env/bin/activate  # On Windows: test_env\\Scripts\\activate"
echo ""
echo -e "${BLUE}# Install from TestPyPI${NC}"
echo "pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ spectre-py==$VERSION"
echo ""
echo -e "${BLUE}# Test the installation${NC}"
echo "python -c 'from spectre import Client, Profile; print(\"Import successful!\")'"
echo ""
echo -e "${BLUE}# Verify version${NC}"
echo "pip show spectre-py"
echo ""
echo -e "${BLUE}# Clean up${NC}"
echo "deactivate"
echo "rm -rf test_env"
echo ""
info "TestPyPI package URL:"
echo "https://test.pypi.org/project/spectre-py/"
echo ""

exit 0
