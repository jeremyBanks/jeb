#!/bin/bash
# Set version based on current date and time
# Format: MAJOR.MINOR.PATCH-dev-YYYY-MM-DD.xxxx
# where xxxx = (yyyy XOR (dd * 100 + MM)) + (hh * 100 + mm) + 2048

set -euo pipefail

# Get current version from Cargo.toml
if grep -q '^\[workspace\.package\]' Cargo.toml; then
    current_version=$(grep '^\[workspace\.package\]' -A 10 Cargo.toml | grep '^version = ' | head -1 | cut -d'"' -f2)
else
    current_version=$(grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2)
fi

# Extract major.minor.patch from current version
# Handles both X.Y.Z-something and X.Y.Z formats
if [[ "$current_version" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+) ]]; then
    major="${BASH_REMATCH[1]}"
    minor="${BASH_REMATCH[2]}"
    patch="${BASH_REMATCH[3]}"
else
    echo "Error: Could not parse version from: $current_version"
    exit 1
fi

# Get current date and time components
yyyy=$(date +%Y)
MM=$(date +%m)
dd=$(date +%d)
hh=$(date +%H)
mm=$(date +%M)

# Remove leading zeros for arithmetic
yyyy=$((10#$yyyy))
MM=$((10#$MM))
dd=$((10#$dd))
hh=$((10#$hh))
mm=$((10#$mm))

# Calculate xxxx = (yyyy XOR (dd * 100 + MM)) + (hh * 100 + mm) + 2048
first_part=$((yyyy ^ (dd * 100 + MM)))
second_part=$((hh * 100 + mm))
xxxx=$((first_part + second_part + 2048))

# Format the version (with zero-padded date components)
version="$major.$minor.$patch-dev-$(date +%Y-%m-%d).$xxxx"

echo "Calculated version: $version"
echo "  Date: $(date +%Y-%m-%d)"
echo "  Time: $(date +%H:%M)"
echo "  Formula: ($yyyy XOR ($dd * 100 + $MM)) + ($hh * 100 + $mm) + 2048"
echo "  = ($yyyy XOR $((dd * 100 + MM))) + $((hh * 100 + mm)) + 2048"
echo "  = $first_part + $second_part + 2048"
echo "  = $xxxx"

# Update Cargo.toml workspace version
if grep -q '^\[workspace\.package\]' Cargo.toml; then
    # Use perl to update the version line after [workspace.package]
    perl -i -pe "s/^version = \".*\"/version = \"$version\"/ if /^\[workspace\.package\]/ .. (/^\[/ && !/^\[workspace\.package\]/);" Cargo.toml
    echo "Updated workspace version in Cargo.toml to: $version"
else
    echo "Error: No [workspace.package] section found in Cargo.toml"
    exit 1
fi
