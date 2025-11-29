#!/bin/bash
# Test: JEB85 binary mode encoding
# Input: Pure binary data
# Expected: Should produce deterministic encoding and roundtrip

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/binary16.bin"
EXPECTED_ENCODED="expected/binary16.jeb85"

# Test 1: Binary should be encoded (not passthrough)
ENCODED=$(jeb encode-jeb85 < "$INPUT")
ORIGINAL=$(cat "$INPUT")

if [ "$ENCODED" != "$ORIGINAL" ]; then
    echo "✓ PASS: 13-jeb85-binary-mode (encoded, not passthrough)"
else
    echo "✗ FAIL: 13-jeb85-binary-mode (should be encoded)"
    exit 1
fi

# Test 2: Check encoded output matches expected
if [ -f "$EXPECTED_ENCODED" ]; then
    EXPECTED_CONTENT=$(cat "$EXPECTED_ENCODED")
    if [ "$ENCODED" = "$EXPECTED_CONTENT" ]; then
        echo "✓ PASS: 13-jeb85-binary-mode (encoded output)"
    else
        echo "✗ FAIL: 13-jeb85-binary-mode (encoded output)"
        echo "Expected: $EXPECTED_CONTENT"
        echo "Actual:   $ENCODED"
        exit 1
    fi
fi

# Test 3: Should roundtrip correctly
DECODED=$(echo -n "$ENCODED" | jeb decode-jeb85)
if [ "$DECODED" = "$ORIGINAL" ]; then
    echo "✓ PASS: 13-jeb85-binary-mode (roundtrip)"
else
    echo "✗ FAIL: 13-jeb85-binary-mode (roundtrip)"
    exit 1
fi
