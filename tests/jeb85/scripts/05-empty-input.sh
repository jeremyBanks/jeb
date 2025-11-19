#!/bin/bash
# Test: Empty input handling
# Input: 0 bytes
# Expected: Empty output (or valid empty encoding)

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/empty.bin"

# Should handle empty input gracefully
ACTUAL=$(jeb encode-z85 < "$INPUT" | jeb decode-z85)

if [ -z "$ACTUAL" ]; then
    echo "✓ PASS: 05-empty-input"
else
    echo "✗ FAIL: 05-empty-input"
    echo "Expected: (empty)"
    echo "Actual:   $(echo -n "$ACTUAL" | xxd -p)"
    exit 1
fi
