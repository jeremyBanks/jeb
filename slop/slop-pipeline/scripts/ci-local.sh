#!/bin/bash
# Run all CI checks locally before pushing
# GitHub Actions calls these same scripts, ensuring identical behavior

set -euo pipefail

echo "========================================="
echo "Running local CI checks..."
echo "========================================="
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print step headers
print_step() {
    echo ""
    echo "========================================="
    echo "$1"
    echo "========================================="
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# 1. Run tests (debug mode)
print_step "1. Running tests (debug mode)"
if cargo test --verbose; then
    print_success "Tests passed (debug mode)"
else
    print_error "Tests failed (debug mode)"
    exit 1
fi

# 2. Check formatting
print_step "2. Checking code formatting (cargo fmt)"
if cargo fmt -- --check; then
    print_success "Formatting check passed"
else
    print_error "Formatting check failed"
    echo ""
    echo "Run 'cargo fmt' to fix formatting issues"
    exit 1
fi

# 3. Run clippy (linter)
print_step "3. Running linter (cargo clippy)"
if cargo clippy -- -D warnings; then
    print_success "Clippy passed"
else
    print_error "Clippy failed"
    exit 1
fi

# 4. Run tests (release mode)
print_step "4. Running tests (release mode)"
if cargo test --release --verbose; then
    print_success "Tests passed (release mode)"
else
    print_error "Tests failed (release mode)"
    exit 1
fi

# 5. Build release binary (optional, can be slow)
if [ "${SKIP_BUILD:-}" != "1" ]; then
    print_step "5. Building release binary"
    if cargo build --release --verbose; then
        print_success "Release build succeeded"
    else
        print_error "Release build failed"
        exit 1
    fi
else
    echo ""
    echo "Skipping release build (SKIP_BUILD=1)"
fi

# All checks passed!
echo ""
echo "========================================="
echo -e "${GREEN}All CI checks passed! ✓${NC}"
echo "========================================="
echo ""
echo "You can now commit and push with confidence."
