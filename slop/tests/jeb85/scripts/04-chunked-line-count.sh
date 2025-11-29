#!/bin/bash
# Test: Verify chunk count after split-64k
# Input: 100 KiB (should produce 2 chunks: 64k + 36k)
# Expected: 2 lines of encoded output

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/zeros-100k.bin"

ENCODED=$(jeb split-64k encode-z85 join-lines < "$INPUT")
LINE_COUNT=$(echo "$ENCODED" | wc -l)

if [ "$LINE_COUNT" -eq 2 ]; then
    echo "✓ PASS: 04-chunked-line-count (2 lines)"
else
    echo "✗ FAIL: 04-chunked-line-count"
    echo "Expected: 2 lines"
    echo "Actual:   $LINE_COUNT lines"
    exit 1
fi
