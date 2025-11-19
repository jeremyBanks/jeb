#!/bin/bash
# Test: Binary data (not ASCII) with Z85 encoding
# Input: 16 bytes of 0x00-0x0f
# Expected: Should roundtrip perfectly

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/binary16.bin"

ACTUAL=$(jeb encode-z85 < "$INPUT" | jeb decode-z85)
EXPECTED=$(cat "$INPUT")

if [ "$ACTUAL" = "$EXPECTED" ]; then
    echo "✓ PASS: 08-binary-data-z85"
else
    echo "✗ FAIL: 08-binary-data-z85"
    echo "Expected: $(cat "$INPUT" | xxd -p)"
    echo "Actual:   $(echo -n "$ACTUAL" | xxd -p)"
    exit 1
fi
