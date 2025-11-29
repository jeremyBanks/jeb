#!/bin/bash
# Test: Verify encoded output contains only printable ASCII
# All encoded outputs should be text-safe (no control chars, valid UTF-8)
# This ensures outputs are git-friendly and can be diffed

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/binary16.bin"

# Encode binary data - output should be text only
ENCODED=$(jeb encode-z85 < "$INPUT")

# Check if output is valid UTF-8
if ! echo "$ENCODED" | iconv -f UTF-8 -t UTF-8 >/dev/null 2>&1; then
    echo "✗ FAIL: 09-output-is-text (not valid UTF-8)"
    exit 1
fi

# Check for control characters (allow newline for multi-line outputs)
# Printable ASCII: 0x20-0x7E, plus newline (0x0A)
if echo -n "$ENCODED" | LC_ALL=C grep -q '[^ -~]' | grep -v $'\n'; then
    echo "✗ FAIL: 09-output-is-text (contains control characters)"
    echo "Output: $(echo -n "$ENCODED" | xxd -p | head -c 80)"
    exit 1
fi

echo "✓ PASS: 09-output-is-text (valid UTF-8, printable ASCII)"
