#!/bin/bash
# Test: JEB85 with chunked encoding
# Input: 100 KiB file
# Pipeline: split-64k encode-jeb85 join-lines | split-lines decode-jeb85 join
# Expected: Should roundtrip correctly

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/zeros-100k.bin"

# Chunked encode/decode roundtrip
ENCODED=$(jeb split-64k encode-jeb85 join-lines < "$INPUT")
ACTUAL=$(echo "$ENCODED" | jeb split-lines decode-jeb85 join)
EXPECTED=$(cat "$INPUT")

if [ "$ACTUAL" = "$EXPECTED" ]; then
    echo "✓ PASS: 15-jeb85-chunked (roundtrip)"
else
    echo "✗ FAIL: 15-jeb85-chunked (roundtrip)"
    echo "Input size:  $(cat "$INPUT" | wc -c) bytes"
    echo "Output size: $(echo -n "$ACTUAL" | wc -c) bytes"
    exit 1
fi

# Should produce 2 lines (64k + 36k chunks)
LINE_COUNT=$(echo "$ENCODED" | wc -l)
if [ "$LINE_COUNT" -eq 2 ]; then
    echo "✓ PASS: 15-jeb85-chunked (2 lines)"
else
    echo "✗ FAIL: 15-jeb85-chunked (expected 2 lines, got $LINE_COUNT)"
    exit 1
fi
