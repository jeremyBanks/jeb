#!/bin/bash
# run-loop.sh — run gravity continuously, incrementing seed each time
# Output goes directly to $GRAVITY_SHARED_DIR (default: workspace/shared/gravity)
# Handles SIGTERM/SIGINT gracefully — the binary will finish the current chunk
# and concatenate whatever is done before exiting.
# Usage: ./run-loop.sh [start_seed] [seconds]
set -e
cd "$(dirname "$0")"

SEED=${1:-50}
SECONDS_PER_RUN=${2:-4369}  # 64^3 frames at 60fps ≈ 4369s

export GRAVITY_SHARED_DIR="${GRAVITY_SHARED_DIR:-/Users/matte/.openclaw/workspace/shared/gravity}"
mkdir -p "$GRAVITY_SHARED_DIR"

while true; do
    echo "=== Starting run seed=$SEED, ${SECONDS_PER_RUN}s → $GRAVITY_SHARED_DIR ==="
    rm -f segments.txt state/checkpoint.bin state/orig_state.bin
    rm -f segments/*.mp4 2>/dev/null || true

    cargo run --release -- --seconds "$SECONDS_PER_RUN" --epilogue --seed "$SEED"

    echo "=== Done seed=$SEED ==="
    SEED=$((SEED + 1))
done
