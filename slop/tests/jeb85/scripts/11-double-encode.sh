#!/bin/bash
# Test: Double encoding - apply split-64k + encode-z85 + join-lines TWICE
# Input: Mixed binary and text pattern
# Pipeline: (split-64k encode-z85 join-lines) × 2
# Then: (split-lines decode-z85 join) × 2
# Expected: Should roundtrip perfectly through double encoding

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/mixed-text-binary.bin"

# First encoding: split into chunks, encode, join with newlines
ENCODED_ONCE=$(jeb split-64k encode-z85 join-lines < "$INPUT")

# Second encoding: do it again on the already-encoded data
DOUBLE_ENCODED=$(echo "$ENCODED_ONCE" | jeb split-64k encode-z85 join-lines)

# First decoding: split by lines, decode, join
DECODED_ONCE=$(echo "$DOUBLE_ENCODED" | jeb split-lines decode-z85 join)

# Second decoding: do it again to get back original
ACTUAL=$(echo "$DECODED_ONCE" | jeb split-lines decode-z85 join)
EXPECTED=$(cat "$INPUT")

if [ "$ACTUAL" = "$EXPECTED" ]; then
    echo "✓ PASS: 11-double-encode (double roundtrip)"
else
    echo "✗ FAIL: 11-double-encode (double roundtrip)"
    echo "Input size:  $(cat "$INPUT" | wc -c) bytes"
    echo "Output size: $(echo -n "$ACTUAL" | wc -c) bytes"
    exit 1
fi

# Verify double-encoded output is still text-safe
if echo "$DOUBLE_ENCODED" | LC_ALL=C grep -q '[^ -~]' | grep -v $'\n'; then
    echo "✗ FAIL: 11-double-encode (double-encoded output not text-safe)"
    exit 1
fi

echo "✓ PASS: 11-double-encode (output is text-safe after double encoding)"
