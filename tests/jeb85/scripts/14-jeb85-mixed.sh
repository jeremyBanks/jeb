#!/bin/bash
# Test: JEB85 with mixed binary and text
# Input: Text with embedded null bytes
# Expected: Should encode and roundtrip correctly

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/mixed-text-binary.bin"

# Should roundtrip correctly
ORIGINAL=$(cat "$INPUT")
DECODED=$(jeb encode-jeb85 < "$INPUT" | jeb decode-jeb85)

if [ "$DECODED" = "$ORIGINAL" ]; then
    echo "✓ PASS: 14-jeb85-mixed (roundtrip)"
else
    echo "✗ FAIL: 14-jeb85-mixed (roundtrip)"
    exit 1
fi

# Encoded output should be text-safe
ENCODED=$(jeb encode-jeb85 < "$INPUT")
if echo "$ENCODED" | LC_ALL=C grep -q '[^ -~]' | grep -v $'\n'; then
    echo "✗ FAIL: 14-jeb85-mixed (output not text-safe)"
    exit 1
fi

echo "✓ PASS: 14-jeb85-mixed (output is text-safe)"
