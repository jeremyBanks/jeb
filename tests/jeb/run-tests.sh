#!/bin/bash
# Test runner for jeb tests
# Runs all tests and checks if outputs changed using git diff

set -euo pipefail

cd "$(dirname "$0")"

# Run all tests
for test_dir in [0-9]*; do
    if [ -d "$test_dir" ] && [ -f "$test_dir/test.sh" ]; then
        echo "Running test $test_dir..."
        cd "$test_dir"
        bash test.sh 2>/dev/null || true  # Don't fail on expected errors
        cd ..
    fi
done

echo ""
echo "All tests executed. Checking for changes..."
echo ""

# Check if outputs changed
cd /home/user/json-entity-bucket
if git diff --quiet tests/jeb/; then
    echo "✓ PASS: All test outputs match expected (no changes detected)"
    exit 0
else
    echo "✗ FAIL: Test outputs changed:"
    echo ""
    git diff tests/jeb/
    exit 1
fi
