#!/bin/bash
# migrate-to-shared.sh — move completed gravity videos to shared storage
# For each runs/*.mp4:
#   - if already in shared with matching content: delete local
#   - if not in shared: copy, verify (cmp -s), delete local
#   - if copy fails or mismatch: warn, keep local
#
# Logging deliberately avoids printing the destination path — it's an
# internal detail that doesn't need to appear in output.
set -euo pipefail
cd "$(dirname "$0")/.."

SHARED_DIR="${GRAVITY_SHARED_DIR:-/Users/matte/.openclaw/workspace/shared/gravity}"
mkdir -p "$SHARED_DIR"

RUNS_DIR="runs"
ok=0; skipped=0; failed=0

shopt -s nullglob
videos=("$RUNS_DIR"/*.mp4)

if [[ ${#videos[@]} -eq 0 ]]; then
    echo "No completed videos found in $RUNS_DIR/"
    exit 0
fi

echo "Found ${#videos[@]} video(s) to process"
echo ""

for local_path in "${videos[@]}"; do
    fname="$(basename "$local_path")"
    shared_path="$SHARED_DIR/$fname"
    size_mb=$(( $(stat -f%z "$local_path" 2>/dev/null || echo 0) / 1048576 ))

    echo "  $fname (${size_mb}MB)"

    if [[ -f "$shared_path" ]]; then
        # Already exists in shared — just verify
        if cmp -s "$local_path" "$shared_path"; then
            rm "$local_path"
            echo "    ✓ already in shared (verified) — local copy removed"
            (( ok++ ))
        else
            echo "    ✗ MISMATCH — files differ! Keeping local copy. Manual check needed."
            (( failed++ ))
        fi
    else
        # Not in shared — copy, verify, delete
        echo "    copying..."
        if cp "$local_path" "$shared_path"; then
            if cmp -s "$local_path" "$shared_path"; then
                rm "$local_path"
                echo "    ✓ copied, verified, local copy removed"
                (( ok++ ))
            else
                rm -f "$shared_path"
                echo "    ✗ copy verification FAILED — shared copy removed, local kept"
                (( failed++ ))
            fi
        else
            echo "    ✗ copy FAILED — local copy kept"
            (( failed++ ))
        fi
    fi
done

echo ""
echo "Done: $ok archived, $skipped skipped, $failed failed"
[[ $failed -eq 0 ]] || exit 1
