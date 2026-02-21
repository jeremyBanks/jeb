#!/bin/bash
# run-loop.sh — run gravity continuously, incrementing seed each time
# Output goes directly to $GRAVITY_SHARED_DIR (default: workspace/shared/gravity)
# Handles SIGTERM/SIGINT gracefully — the binary will finish the current chunk
# and concatenate whatever is done before exiting.
# Usage: ./run-loop.sh [start_seed] [seconds] [extra flags...]
set -e
cd "$(dirname "$0")"

SEED=${1:-50}
SECONDS_PER_RUN=${2:-4369}  # 64^3 frames at 60fps ≈ 4369s
shift 2 2>/dev/null || true
EXTRA_ARGS=("$@")  # remaining args passed through to binary

export GRAVITY_SHARED_DIR="${GRAVITY_SHARED_DIR:-/Users/matte/.openclaw/workspace/shared/gravity}"
mkdir -p "$GRAVITY_SHARED_DIR"

while true; do
    RUN_ID="$(date +%Y%m%d_%H%M%S)_seed${SEED}"
    echo "=== Starting run $RUN_ID, ${SECONDS_PER_RUN}s → $GRAVITY_SHARED_DIR ==="
    rm -f segments.txt state/checkpoint.bin state/orig_state.bin
    rm -f segments/*.mp4 2>/dev/null || true

    cargo run --release -- --seconds "$SECONDS_PER_RUN" --epilogue --seed "$SEED" \
        --run-id "$RUN_ID" "${EXTRA_ARGS[@]}"

    echo "=== Done $RUN_ID ==="
    SEED=$((SEED + 1))
done
