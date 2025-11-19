#!/bin/bash
# Test: JEB85 with chunked encoding
# Input: 100 KiB file
# Pipeline: split-64k encode-jeb85 join-lines | split-lines decode-jeb85 join
# Expected: Should produce deterministic encoding and roundtrip correctly

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/zeros-100k.bin"
EXPECTED_ENCODED="expected/zeros-100k-chunked.jeb85"

# Chunked encode
ENCODED=$(jeb split-64k encode-jeb85 join-lines < "$INPUT")

# Test 1: Check encoded output matches expected
if [ -f "$EXPECTED_ENCODED" ]; then
    EXPECTED_CONTENT=$(cat "$EXPECTED_ENCODED")
    if [ "$ENCODED" = "$EXPECTED_CONTENT" ]; then
        echo "✓ PASS: 15-jeb85-chunked (encoded output)"
    else
        echo "✗ FAIL: 15-jeb85-chunked (encoded output)"
        echo "Expected lines: $(echo -n "$EXPECTED_CONTENT" | wc -l)"
        echo "Actual lines:   $(echo -n "$ENCODED" | wc -l)"
        exit 1
    fi
fi

# Test 2: Should produce 2 lines (64k + 36k chunks)
LINE_COUNT=$(echo "$ENCODED" | wc -l)
if [ "$LINE_COUNT" -eq 2 ]; then
    echo "✓ PASS: 15-jeb85-chunked (2 lines)"
else
    echo "✗ FAIL: 15-jeb85-chunked (expected 2 lines, got $LINE_COUNT)"
    exit 1
fi

# Test 3: Decode and check roundtrip
DECODED=$(echo "$ENCODED" | jeb split-lines decode-jeb85 join)
ORIGINAL=$(cat "$INPUT")

if [ "$DECODED" = "$ORIGINAL" ]; then
    echo "✓ PASS: 15-jeb85-chunked (roundtrip)"
else
    echo "✗ FAIL: 15-jeb85-chunked (roundtrip)"
    echo "Input size:  $(cat "$INPUT" | wc -c) bytes"
    echo "Output size: $(echo -n "$DECODED" | wc -c) bytes"
    exit 1
fi
