#!/bin/bash
# Test: Binary data (not ASCII) with Z85 encoding
# Input: 16 bytes of 0x00-0x0f
# Expected: Should produce deterministic encoding and roundtrip perfectly

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/binary16.bin"
EXPECTED_ENCODED="expected/binary16.z85"

# Test 1: Check encoded output matches expected
ENCODED=$(jeb encode-z85 < "$INPUT")
if [ -f "$EXPECTED_ENCODED" ]; then
    EXPECTED_CONTENT=$(cat "$EXPECTED_ENCODED")
    if [ "$ENCODED" = "$EXPECTED_CONTENT" ]; then
        echo "✓ PASS: 08-binary-data-z85 (encoded output)"
    else
        echo "✗ FAIL: 08-binary-data-z85 (encoded output)"
        echo "Expected: $EXPECTED_CONTENT"
        echo "Actual:   $ENCODED"
        exit 1
    fi
fi

# Test 2: Check roundtrip
DECODED=$(echo -n "$ENCODED" | jeb decode-z85)
ORIGINAL=$(cat "$INPUT")
if [ "$DECODED" = "$ORIGINAL" ]; then
    echo "✓ PASS: 08-binary-data-z85 (roundtrip)"
else
    echo "✗ FAIL: 08-binary-data-z85 (roundtrip)"
    echo "Expected: $(cat "$INPUT" | xxd -p)"
    echo "Actual:   $(echo -n "$DECODED" | xxd -p)"
    exit 1
fi
