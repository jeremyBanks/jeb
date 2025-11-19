#!/bin/bash
# Test: Basic Z85 encode/decode roundtrip
# Input: "Hello, World!" (13 bytes)
# Expected: Should roundtrip perfectly

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/hello.bin"
EXPECTED="$INPUT"  # Should get back exactly what we put in

# Encode then decode
ACTUAL=$(jeb encode-z85 < "$INPUT" | jeb decode-z85)
EXPECTED_CONTENT=$(cat "$EXPECTED")

if [ "$ACTUAL" = "$EXPECTED_CONTENT" ]; then
    echo "✓ PASS: 01-encode-decode-z85"
else
    echo "✗ FAIL: 01-encode-decode-z85"
    echo "Expected: $(echo -n "$EXPECTED_CONTENT" | xxd -p)"
    echo "Actual:   $(echo -n "$ACTUAL" | xxd -p)"
    exit 1
fi
