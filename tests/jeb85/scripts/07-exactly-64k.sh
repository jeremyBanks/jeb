#!/bin/bash
# Test: Exactly 64 KiB input (edge case - single chunk)
# Expected: Should produce 1 line when chunked

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/zeros-64k.bin"

# Should produce exactly 1 chunk
ENCODED=$(jeb split-64k encode-z85 join-lines < "$INPUT")
LINE_COUNT=$(echo "$ENCODED" | wc -l)

if [ "$LINE_COUNT" -eq 1 ]; then
    echo "✓ PASS: 07-exactly-64k (1 line)"
else
    echo "✗ FAIL: 07-exactly-64k"
    echo "Expected: 1 line"
    echo "Actual:   $LINE_COUNT lines"
    exit 1
fi

# Should also roundtrip
ACTUAL=$(echo "$ENCODED" | jeb split-lines decode-z85 join)
EXPECTED=$(cat "$INPUT")

if [ "$ACTUAL" = "$EXPECTED" ]; then
    echo "✓ PASS: 07-exactly-64k (roundtrip)"
else
    echo "✗ FAIL: 07-exactly-64k (roundtrip)"
    exit 1
fi
