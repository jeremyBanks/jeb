#!/bin/bash
# run-cycle.sh — pipeline loop: render runs while next params are being explored
# Cycle: [explore N] overlaps with [render from N-1 params] → copy → repeat
# Usage: ./run-cycle.sh [starting_seed] [seconds_per_render]

cd "$(dirname "$0")"

SEED="${1:-100}"
RENDER_SECONDS="${2:-4369}"
SHARED_DIR="${GRAVITY_SHARED_DIR:-/Users/matte/.openclaw/workspace/shared/gravity}"
ROUND=1

echo "[cycle] Starting pipeline. seed=$SEED render=${RENDER_SECONDS}s"

# Kick off watcher if not already running
if ! pgrep -f "segment-watcher.sh" > /dev/null; then
    bash segment-watcher.sh &>/tmp/watcher.log &
    echo "[cycle] Started watcher PID $!"
fi

# Phase 1: explore round 1 before any render (nothing to pipeline against yet)
echo ""
echo "=== Round 1: exploring to find first render params ==="
bash explore.sh 300 1 2>&1 | tee /tmp/explore_round1.txt
BEST_ARGS=$(head -1 best_config.txt 2>/dev/null || echo "--gravity 0.125 --softening 4")
BEST_LABEL=$(grep "^label=" best_config.txt 2>/dev/null | cut -d= -f2 || echo "fallback")
echo "[cycle] Round 1 best: [$BEST_LABEL] $BEST_ARGS"

while true; do
    echo ""
    echo "=== Render seed=$SEED with [$BEST_LABEL]: $BEST_ARGS ==="

    # Start render in background
    rm -f state/checkpoint.bin state/orig_state.bin state/run_info.txt \
          segments.txt /tmp/gravity_segments_seen.txt
    rm -f segments/seg_*.mp4 2>/dev/null

    ./run-loop.sh "$SEED" "$RENDER_SECONDS" --pop-target 5120 --wrap --speed-cap 6.0 $BEST_ARGS &
    RENDER_PID=$!
    echo "[cycle] Render started (PID=$RENDER_PID)"

    # While render runs, explore next round
    NEXT_ROUND=$(( ROUND + 1 ))
    echo "[cycle] Exploring round $NEXT_ROUND in parallel..."
    bash explore.sh 300 "$NEXT_ROUND" 2>&1 | tee "/tmp/explore_round${NEXT_ROUND}.txt"
    NEXT_ARGS=$(head -1 best_config.txt 2>/dev/null || echo "--gravity 0.125 --softening 4")
    NEXT_LABEL=$(grep "^label=" best_config.txt 2>/dev/null | cut -d= -f2 || echo "fallback")
    echo "[cycle] Round $NEXT_ROUND best: [$NEXT_LABEL] $NEXT_ARGS"

    # Wait for render to finish
    echo "[cycle] Exploration done. Waiting for render (PID=$RENDER_PID) to finish..."
    wait "$RENDER_PID"
    echo "[cycle] Render complete."

    # Copy to shared volume if mounted
    VOLUME="/Volumes/My Shared Files/shared"
    if [ -d "$VOLUME" ]; then
        LATEST=$(ls -t "$SHARED_DIR"/gravity_*.mp4 2>/dev/null | head -1)
        if [ -n "$LATEST" ]; then
            cp "$LATEST" "$VOLUME/" && echo "[cycle] Copied $(basename $LATEST) → shared volume"
        fi
    fi

    # Advance: use next explore's best params for next render
    BEST_ARGS="$NEXT_ARGS"
    BEST_LABEL="$NEXT_LABEL"
    SEED=$(( SEED + 1 ))
    ROUND="$NEXT_ROUND"
    echo "[cycle] Next: seed=$SEED round=$ROUND label=$BEST_LABEL"
    sleep 2
done
