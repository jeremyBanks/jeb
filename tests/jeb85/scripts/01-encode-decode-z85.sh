#!/bin/bash
# Test: Basic Z85 encode/decode roundtrip
# Input: "Hello, World!" (13 bytes)
# Expected: Should roundtrip perfectly and exit with code 0

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/hello.bin"
EXPECTED="$INPUT"  # Should get back exactly what we put in

# Test 1: Roundtrip correctness
ACTUAL=$(jeb encode-z85 < "$INPUT" | jeb decode-z85)
EXPECTED_CONTENT=$(cat "$EXPECTED")

if [ "$ACTUAL" = "$EXPECTED_CONTENT" ]; then
    echo "✓ PASS: 01-encode-decode-z85 (roundtrip)"
else
    echo "✗ FAIL: 01-encode-decode-z85 (roundtrip)"
    echo "Expected: $(echo -n "$EXPECTED_CONTENT" | xxd -p)"
    echo "Actual:   $(echo -n "$ACTUAL" | xxd -p)"
    exit 1
fi

# Test 2: Exit code should be 0 (success)
jeb encode-z85 < "$INPUT" | jeb decode-z85 > /dev/null
EXIT_CODE=$?
if [ "$EXIT_CODE" -eq 0 ]; then
    echo "✓ PASS: 01-encode-decode-z85 (exit code 0)"
else
    echo "✗ FAIL: 01-encode-decode-z85 (exit code)"
    echo "Expected: 0"
    echo "Actual:   $EXIT_CODE"
    exit 1
fi
