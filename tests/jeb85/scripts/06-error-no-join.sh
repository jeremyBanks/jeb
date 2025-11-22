#!/bin/bash
# Test: Error when stream-of-streams reaches end without join
# Expected: Should exit with error, not produce binary output

set -euo pipefail
cd "$(dirname "$0")/.."

INPUT="inputs/hello.bin"

# This should error because we have stream-of-streams but no join
if jeb split-64k encode-z85 < "$INPUT" 2>/dev/null; then
    echo "✗ FAIL: 06-error-no-join (should have errored)"
    exit 1
else
    echo "✓ PASS: 06-error-no-join (correctly errored)"
fi
