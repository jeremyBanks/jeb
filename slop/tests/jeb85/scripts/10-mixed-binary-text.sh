#!/bin/bash
# Test: Mixed binary and text input
# Input: "Hello\x00\x01\x02World\x00\xff"
# Expected: Should encode and roundtrip correctly

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/mixed-text-binary.bin"

# Roundtrip test
ACTUAL=$(jeb encode-z85 < "$INPUT" | jeb decode-z85)
EXPECTED=$(cat "$INPUT")

if [ "$ACTUAL" = "$EXPECTED" ]; then
    echo "✓ PASS: 10-mixed-binary-text (roundtrip)"
else
    echo "✗ FAIL: 10-mixed-binary-text (roundtrip)"
    echo "Expected: $(cat "$INPUT" | xxd -p)"
    echo "Actual:   $(echo -n "$ACTUAL" | xxd -p)"
    exit 1
fi

# Verify encoded output is text
ENCODED=$(jeb encode-z85 < "$INPUT")
if echo "$ENCODED" | LC_ALL=C grep -q '[^ -~]' | grep -v $'\n'; then
    echo "✗ FAIL: 10-mixed-binary-text (output not text-safe)"
    exit 1
fi

echo "✓ PASS: 10-mixed-binary-text (output is text-safe)"
