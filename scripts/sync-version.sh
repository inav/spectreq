#!/usr/bin/env bash
#
# Version sync script for Spectre.
#
# This is a bash wrapper around the Python sync script for convenience.
# The Python script is the primary implementation for cross-platform support.
#
# Usage:
#   ./scripts/sync-version.sh          # Sync version
#   ./scripts/sync-version.sh --verify # Verify only
#   ./scripts/sync-version.sh --get    # Print version

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "Error: python3 not found" >&2
    exit 1
fi

# Run the Python script
cd "$PROJECT_ROOT"
exec python3 scripts/sync-version.py "$@"
