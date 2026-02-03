#!/usr/bin/env python3
"""
Version sync script for Spectre.

This script extracts the version from Cargo.toml (workspace root)
and updates pyproject.toml to match. This ensures a single source
of truth for versioning across the Rust and Python ecosystems.

Usage:
    python scripts/sync-version.py          # Sync version
    python scripts/sync-version.py --verify # Verify only (don't update)
    python scripts/sync-version.py --get    # Print version and exit
"""

import argparse
import re
import sys
from pathlib import Path
from typing import Tuple, Optional


# Regex patterns for version extraction
CARGO_VERSION_RE = re.compile(r'^version\s*=\s*"([^"]+)"', re.MULTILINE)
PYPROJECT_VERSION_RE = re.compile(r'^version\s*=\s*"([^"]+)"', re.MULTILINE)

# SemVer pattern for validation
SEMVER_RE = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"  # Major.Minor.Patch
    r"(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)"  # Pre-release (optional)
    r"(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?"
    r"(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$"  # Build metadata (optional)
)


def find_project_root() -> Path:
    """Find the project root directory (contains Cargo.toml)."""
    current = Path(__file__).resolve().parent
    while current != current.parent:
        if (current / "Cargo.toml").exists():
            return current
        current = current.parent
    raise FileNotFoundError("Could not find project root (Cargo.toml not found)")


def extract_cargo_version(project_root: Path) -> Tuple[str, str]:
    """
    Extract version from Cargo.toml (single crate or workspace).

    Returns:
        Tuple of (version, file_path)
    """
    cargo_toml = project_root / "Cargo.toml"

    if not cargo_toml.exists():
        raise FileNotFoundError(f"Cargo.toml not found at {cargo_toml}")

    content = cargo_toml.read_text()

    # First, try to find version in [package] section (single crate)
    in_package = False
    match = None

    for line in content.splitlines():
        line_stripped = line.strip()

        # Track if we're in [package] section
        if line_stripped == "[package]":
            in_package = True
            continue
        elif line_stripped.startswith("[") and in_package:
            # Left the [package] section
            in_package = False
        elif line_stripped.startswith("[workspace]"):
            # This is a workspace - skip to next section
            continue

        if in_package:
            match = CARGO_VERSION_RE.match(line_stripped)
            if match:
                break

    if match:
        version = match.group(1)
        return version, str(cargo_toml)

    # If not found in [package], look for [workspace.package] (workspace mode)
    in_workspace_package = False

    for line in content.splitlines():
        line_stripped = line.strip()

        if line_stripped == "[workspace.package]":
            in_workspace_package = True
            continue
        elif line_stripped.startswith("[") and in_workspace_package:
            # Left the [workspace.package] section
            in_workspace_package = False
        elif line_stripped.startswith("[workspace]"):
            in_workspace_package = False
            # Check if [workspace.package] follows on next line
            continue

        if in_workspace_package:
            match = CARGO_VERSION_RE.match(line_stripped)
            if match:
                break

    if not match:
        raise ValueError(
            "Could not find version in [package] or [workspace.package] section of Cargo.toml"
        )

    version = match.group(1)
    return version, str(cargo_toml)


def extract_pyproject_version(project_root: Path) -> Tuple[Optional[str], str]:
    """
    Extract version from pyproject.toml.

    Returns:
        Tuple of (version or None, file_path)
    """
    pyproject = project_root / "pyproject.toml"

    if not pyproject.exists():
        return None, str(pyproject)

    content = pyproject.read_text()
    match = PYPROJECT_VERSION_RE.search(content)

    if match:
        return match.group(1), str(pyproject)

    return None, str(pyproject)


def validate_semver(version: str) -> bool:
    """Validate that version string follows SemVer."""
    return bool(SEMVER_RE.match(version))


def update_pyproject_version(project_root: Path, new_version: str) -> bool:
    """
    Update version in pyproject.toml.

    Returns:
        True if updated, False if already matches
    """
    pyproject = project_root / "pyproject.toml"

    if not pyproject.exists():
        raise FileNotFoundError(f"pyproject.toml not found at {pyproject}")

    content = pyproject.read_text()
    lines = content.splitlines(keepends=True)
    updated = False

    for i, line in enumerate(lines):
        match = CARGO_VERSION_RE.match(line.strip())
        if match and "version" in line.lower():
            # Replace the version while preserving the rest of the line
            new_line = re.sub(
                r'version\s*=\s*"([^"]+)"', f'version = "{new_version}"', line
            )
            if new_line != line:
                lines[i] = new_line
                updated = True
            break

    if updated:
        pyproject.write_text("".join(lines))

    return updated


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Sync version from Cargo.toml to pyproject.toml"
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Verify version consistency without updating",
    )
    parser.add_argument("--get", action="store_true", help="Print the version and exit")
    parser.add_argument(
        "--quiet", action="store_true", help="Suppress output (except for --get)"
    )

    args = parser.parse_args()

    try:
        # Find project root
        project_root = find_project_root()

        # Extract Cargo version
        cargo_version, cargo_path = extract_cargo_version(project_root)

        if args.get:
            print(cargo_version)
            return 0

        # Validate SemVer
        if not validate_semver(cargo_version):
            print(f"Error: Invalid SemVer version: {cargo_version}", file=sys.stderr)
            return 1

        # Extract pyproject version
        pyproject_version, pyproject_path = extract_pyproject_version(project_root)

        if args.verify:
            if pyproject_version is None:
                print(f"Error: No version found in {pyproject_path}", file=sys.stderr)
                return 1

            if cargo_version != pyproject_version:
                print(
                    f"Error: Version mismatch!\n"
                    f"  Cargo.toml:    {cargo_version}\n"
                    f"  pyproject.toml: {pyproject_version}",
                    file=sys.stderr,
                )
                return 1

            if not args.quiet:
                print(f"Version consistency verified: {cargo_version}")
            return 0

        # Update mode
        if pyproject_version is None:
            print(f"Warning: No version found in {pyproject_path}, adding...")
            update_pyproject_version(project_root, cargo_version)
            if not args.quiet:
                print(f"Added version {cargo_version} to {pyproject_path}")
            return 0

        if cargo_version == pyproject_version:
            if not args.quiet:
                print(f"Versions already in sync: {cargo_version}")
            return 0

        # Update pyproject.toml
        update_pyproject_version(project_root, cargo_version)
        if not args.quiet:
            print(f"Updated {pyproject_path}: {pyproject_version} -> {cargo_version}")

        return 0

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
