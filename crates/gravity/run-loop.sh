#!/bin/bash
# run-loop.sh — run gravity continuously with fresh random seeds each run
# Output goes directly to $GRAVITY_SHARED_DIR (default: workspace/shared/gravity)
# Handles SIGTERM/SIGINT gracefully — the binary will finish the current chunk
# and concatenate whatever is done before exiting.
# Usage: ./run-loop.sh [seconds] [extra flags...]
#   seconds: sim length (default: 4369 = 64³ frames at 60fps ≈ 73min)
#   extra flags: passed directly to gravity binary (override any default)
#
# Seed is randomised each run — seed is NOT a tuning parameter,
# long-term systemic behaviour is determined by physics params, not seed.
set -e
cd "$(dirname "$0")"

SECONDS_PER_RUN=${1:-4369}  # 64^3 frames at 60fps ≈ 4369s
shift 1 2>/dev/null || true
EXTRA_ARGS=("$@")  # remaining args passed through to binary

export GRAVITY_SHARED_DIR="${GRAVITY_SHARED_DIR:-/Users/matte/.openclaw/workspace/shared/gravity}"
mkdir -p "$GRAVITY_SHARED_DIR"

while true; do
    # Fresh random seed each run — seed is not a tuning param
    SEED=$(od -An -N4 -tu4 /dev/urandom | tr -d ' ')
    RUN_ID="$(date +%Y%m%d_%H%M%S)"
    COMMIT=$(git -C "$(dirname "$0")" rev-parse --short=12 HEAD 2>/dev/null || echo "unknown")
    echo "=== Starting run $RUN_ID commit=$COMMIT seed=$SEED, ${SECONDS_PER_RUN}s → $GRAVITY_SHARED_DIR ==="
    rm -f segments.txt state/checkpoint.bin state/orig_state.bin
    rm -f segments/*.mp4 2>/dev/null || true

    cargo run --release -- --seconds "$SECONDS_PER_RUN" --epilogue --seed "$SEED" \
        --run-id "$RUN_ID" --commit "$COMMIT" "${EXTRA_ARGS[@]}"

    echo "=== Done $RUN_ID ==="
done
