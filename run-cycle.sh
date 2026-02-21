#!/bin/bash
# run-cycle.sh — infinite loop: explore → render → copy → repeat
# Each cycle: explore finds best params, kicks off full render,
# while render runs the next explore round is queued.
# Usage: ./run-cycle.sh [starting_seed] [seconds_per_render]

cd "$(dirname "$0")"

SEED="${1:-100}"
RENDER_SECONDS="${2:-4369}"
SHARED_DIR="${GRAVITY_SHARED_DIR:-/Users/matte/.openclaw/workspace/shared/gravity}"
EXPLORE_LOG="/tmp/gravity_explore_log.txt"
ROUND=1

echo "[cycle] Starting. seed=$SEED render=${RENDER_SECONDS}s"
echo "[cycle] Shared dir: $SHARED_DIR"

# Kick off watcher if not already running
if ! pgrep -f "segment-watcher.sh" > /dev/null; then
    bash segment-watcher.sh &>/tmp/watcher.log &
    echo "[cycle] Started segment watcher (PID $!)"
fi

while true; do
    echo ""
    echo "=========================================="
    echo "[cycle] Round $ROUND | seed=$SEED"
    echo "=========================================="

    # Phase 1: headless exploration to find best params for NEXT render
    echo "[cycle] Phase 1: exploring configs (round $ROUND)..."
    bash explore.sh 300 "$ROUND" 2>&1 | tee "$EXPLORE_LOG"

    # Read best config
    BEST_FILE="$(pwd)/best_config.txt"
    if [ ! -f "$BEST_FILE" ]; then
        echo "[cycle] WARNING: no best_config.txt found, using defaults"
        BEST_ARGS="--gravity 0.125 --softening 4"
        BEST_LABEL="fallback"
    else
        BEST_ARGS=$(head -1 "$BEST_FILE")
        BEST_LABEL=$(grep "^label=" "$BEST_FILE" | cut -d= -f2)
    fi
    echo "[cycle] Best config: [$BEST_LABEL] → $BEST_ARGS"

    # Phase 2: full render with best params
    echo "[cycle] Phase 2: starting full render (seed=$SEED, ${RENDER_SECONDS}s)..."
    echo "[cycle] Extra args: $BEST_ARGS"
    rm -f state/checkpoint.bin state/orig_state.bin state/run_info.txt \
          segments.txt /tmp/gravity_segments_seen.txt
    rm -f segments/seg_*.mp4 2>/dev/null

    # Run render — this blocks until done
    ./run-loop.sh "$SEED" "$RENDER_SECONDS" --pop-target 5120 --wrap --speed-cap 6.0 $BEST_ARGS
    RENDER_EXIT=$?

    echo "[cycle] Render finished (exit=$RENDER_EXIT)"

    # Phase 3: copy completed video to shared volume if it exists
    VOLUME="/Volumes/My Shared Files/shared"
    if [ -d "$VOLUME" ]; then
        LATEST=$(ls -t "$SHARED_DIR"/gravity_*.mp4 2>/dev/null | head -1)
        if [ -n "$LATEST" ]; then
            cp "$LATEST" "$VOLUME/" && echo "[cycle] Copied $(basename $LATEST) → shared volume"
        fi
    fi

    # Increment seed and round for next cycle
    SEED=$(( SEED + 1 ))
    ROUND=$(( ROUND + 1 ))

    echo "[cycle] Cycle complete. Next: round=$ROUND seed=$SEED"
    sleep 2
done
