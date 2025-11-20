#!/bin/bash
# Test: Z85 encoding of "Hello, World!"
# Expected output should be deterministic Z85 text

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/hello.bin"
EXPECTED="expected/hello.z85"

ACTUAL=$(jeb encode-z85 < "$INPUT")

if [ -f "$EXPECTED" ]; then
    EXPECTED_CONTENT=$(cat "$EXPECTED")
    if [ "$ACTUAL" = "$EXPECTED_CONTENT" ]; then
        echo "✓ PASS: 02-encode-z85-hello"
    else
        echo "✗ FAIL: 02-encode-z85-hello"
        echo "Expected: $EXPECTED_CONTENT"
        echo "Actual:   $ACTUAL"
        exit 1
    fi
else
    echo "? SKIP: 02-encode-z85-hello (no expected output yet)"
    echo "Actual output:"
    echo "$ACTUAL"
    echo ""
    echo "To create expected output, run:"
    echo "  jeb encode-z85 < $INPUT > $EXPECTED"
fi
