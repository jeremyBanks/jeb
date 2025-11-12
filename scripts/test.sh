#!/bin/bash
# Run tests in both debug and release mode
set -euo pipefail

echo "Running tests (debug mode)..."
cargo test --verbose

echo ""
echo "Running tests (release mode)..."
cargo test --release --verbose
