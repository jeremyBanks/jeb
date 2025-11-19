#!/bin/bash
# Test: JEB85 binary mode encoding
# Input: Pure binary data
# Expected: Should encode (not passthrough) and roundtrip

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/binary16.bin"

# Binary should be encoded (not equal to original)
ENCODED=$(jeb encode-jeb85 < "$INPUT")
ORIGINAL=$(cat "$INPUT")

if [ "$ENCODED" != "$ORIGINAL" ]; then
    echo "✓ PASS: 13-jeb85-binary-mode (encoded, not passthrough)"
else
    echo "✗ FAIL: 13-jeb85-binary-mode (should be encoded)"
    exit 1
fi

# Should roundtrip correctly
DECODED=$(echo -n "$ENCODED" | jeb decode-jeb85)
if [ "$DECODED" = "$ORIGINAL" ]; then
    echo "✓ PASS: 13-jeb85-binary-mode (roundtrip)"
else
    echo "✗ FAIL: 13-jeb85-binary-mode (roundtrip)"
    exit 1
fi
