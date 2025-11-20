#!/bin/bash
# Test: JEB85 with mixed binary and text
# Input: Text with embedded null bytes
# Expected: Should produce deterministic encoding and roundtrip correctly

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/mixed-text-binary.bin"
EXPECTED_ENCODED="expected/mixed-text-binary.jeb85"

ORIGINAL=$(cat "$INPUT")
ENCODED=$(jeb encode-jeb85 < "$INPUT")

# Test 1: Check encoded output matches expected
if [ -f "$EXPECTED_ENCODED" ]; then
    EXPECTED_CONTENT=$(cat "$EXPECTED_ENCODED")
    if [ "$ENCODED" = "$EXPECTED_CONTENT" ]; then
        echo "✓ PASS: 14-jeb85-mixed (encoded output)"
    else
        echo "✗ FAIL: 14-jeb85-mixed (encoded output)"
        echo "Expected: $EXPECTED_CONTENT"
        echo "Actual:   $ENCODED"
        exit 1
    fi
fi

# Test 2: Should roundtrip correctly
DECODED=$(echo -n "$ENCODED" | jeb decode-jeb85)
if [ "$DECODED" = "$ORIGINAL" ]; then
    echo "✓ PASS: 14-jeb85-mixed (roundtrip)"
else
    echo "✗ FAIL: 14-jeb85-mixed (roundtrip)"
    exit 1
fi

# Test 3: Encoded output should be text-safe
if echo "$ENCODED" | LC_ALL=C grep -q '[^ -~]' | grep -v $'\n'; then
    echo "✗ FAIL: 14-jeb85-mixed (output not text-safe)"
    exit 1
fi

echo "✓ PASS: 14-jeb85-mixed (output is text-safe)"
