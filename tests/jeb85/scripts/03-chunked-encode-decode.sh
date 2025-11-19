#!/bin/bash
# Test: Chunked encode/decode roundtrip
# Input: 100 KiB of zeros
# Pipeline: split-64k → encode-z85 → join-lines → split-lines → decode-z85 → join
# Expected: Should roundtrip perfectly

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/zeros-100k.bin"
EXPECTED="$INPUT"

# Encode: split into 64k chunks, encode each, join with newlines
ENCODED=$(jeb split-64k encode-z85 join-lines < "$INPUT")

# Decode: split by lines, decode each, join back
ACTUAL=$(echo "$ENCODED" | jeb split-lines decode-z85 join)
EXPECTED_CONTENT=$(cat "$EXPECTED")

if [ "$ACTUAL" = "$EXPECTED_CONTENT" ]; then
    echo "✓ PASS: 03-chunked-encode-decode"
else
    echo "✗ FAIL: 03-chunked-encode-decode"
    echo "Input size:    $(stat -c%s "$INPUT") bytes"
    echo "Output size:   $(echo -n "$ACTUAL" | wc -c) bytes"
    exit 1
fi
