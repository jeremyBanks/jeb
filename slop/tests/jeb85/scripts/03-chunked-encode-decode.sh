#!/bin/bash
# Test: Chunked encode/decode roundtrip
# Input: 100 KiB of zeros
# Pipeline: split-64k → encode-z85 → join-lines → split-lines → decode-z85 → join
# Expected: Should produce deterministic encoding and roundtrip perfectly

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/zeros-100k.bin"
EXPECTED_ENCODED="expected/zeros-100k-chunked.z85"

# Encode: split into 64k chunks, encode each, join with newlines
ENCODED=$(jeb split-64k encode-z85 join-lines < "$INPUT")

# Test 1: Check encoded output matches expected
if [ -f "$EXPECTED_ENCODED" ]; then
    EXPECTED_CONTENT=$(cat "$EXPECTED_ENCODED")
    if [ "$ENCODED" = "$EXPECTED_CONTENT" ]; then
        echo "✓ PASS: 03-chunked-encode-decode (encoded output)"
    else
        echo "✗ FAIL: 03-chunked-encode-decode (encoded output)"
        echo "Expected lines: $(echo -n "$EXPECTED_CONTENT" | wc -l)"
        echo "Actual lines:   $(echo -n "$ENCODED" | wc -l)"
        exit 1
    fi
fi

# Test 2: Decode and check roundtrip
DECODED=$(echo "$ENCODED" | jeb split-lines decode-z85 join)
ORIGINAL=$(cat "$INPUT")

if [ "$DECODED" = "$ORIGINAL" ]; then
    echo "✓ PASS: 03-chunked-encode-decode (roundtrip)"
else
    echo "✗ FAIL: 03-chunked-encode-decode (roundtrip)"
    echo "Input size:    $(stat -c%s "$INPUT") bytes"
    echo "Output size:   $(echo -n "$DECODED" | wc -c) bytes"
    exit 1
fi
