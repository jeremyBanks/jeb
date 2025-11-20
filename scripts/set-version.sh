#!/bin/bash
# Set version based on current date and time
# Format: 0.0.0-vibes-YYYY-MM-DD.xxxx
# where xxxx = (yyyy XOR (dd * 100 + MM)) + (hh * 100 + mm) + 2000

set -euo pipefail

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

# Calculate xxxx = (yyyy XOR (dd * 100 + MM)) + (hh * 100 + mm) + 2000
first_part=$((yyyy ^ (dd * 100 + MM)))
second_part=$((hh * 100 + mm))
xxxx=$((first_part + second_part + 2000))

# Format the version (with zero-padded date components)
version="0.0.0-vibes-$(date +%Y-%m-%d).$xxxx"

echo "Calculated version: $version"
echo "  Date: $(date +%Y-%m-%d)"
echo "  Time: $(date +%H:%M)"
echo "  Formula: ($yyyy XOR ($dd * 100 + $MM)) + ($hh * 100 + $mm) + 2000"
echo "  = ($yyyy XOR $((dd * 100 + MM))) + $((hh * 100 + mm)) + 2000"
echo "  = $first_part + $second_part + 2000"
echo "  = $xxxx"

# Update Cargo.toml workspace version
if grep -q '^\[workspace\.package\]' Cargo.toml; then
    # Use sed to update the version line after [workspace.package]
    sed -i "/^\[workspace\.package\]/,/^\[/ s/^version = \".*\"/version = \"$version\"/" Cargo.toml
    echo "Updated workspace version in Cargo.toml to: $version"
else
    echo "Error: No [workspace.package] section found in Cargo.toml"
    exit 1
fi
