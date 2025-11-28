#!/bin/bash
# Run all JEB85 test scripts

set -euo pipefail
cd "$(dirname "$0")"

PASS=0
FAIL=0
SKIP=0

for script in [0-9][0-9]-*.sh; do
    if [ "$script" = "run-all.sh" ]; then
        continue
    fi

    echo "Running $script..."
    if OUTPUT=$(bash "$script" 2>&1); then
        echo "$OUTPUT"
        case "$OUTPUT" in
            *SKIP*)
                SKIP=$((SKIP + 1))
                ;;
            *)
                PASS=$((PASS + 1))
                ;;
        esac
    else
        echo "$OUTPUT"
        FAIL=$((FAIL + 1))
    fi
    echo ""
done

echo "================================"
echo "Results: $PASS passed, $FAIL failed, $SKIP skipped"
echo "================================"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
