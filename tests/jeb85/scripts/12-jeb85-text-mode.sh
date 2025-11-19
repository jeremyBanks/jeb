#!/bin/bash
# Test: JEB85 text mode passthrough
# Input: Plain ASCII text
# Expected: Should pass through unchanged (text mode)

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/hello.bin"

# Text should pass through unchanged
ENCODED=$(jeb encode-jeb85 < "$INPUT")
ORIGINAL=$(cat "$INPUT")

if [ "$ENCODED" = "$ORIGINAL" ]; then
    echo "✓ PASS: 12-jeb85-text-mode (passthrough)"
else
    echo "✗ FAIL: 12-jeb85-text-mode (passthrough)"
    exit 1
fi

# Should also roundtrip
DECODED=$(echo -n "$ENCODED" | jeb decode-jeb85)
if [ "$DECODED" = "$ORIGINAL" ]; then
    echo "✓ PASS: 12-jeb85-text-mode (roundtrip)"
else
    echo "✗ FAIL: 12-jeb85-text-mode (roundtrip)"
    exit 1
fi
