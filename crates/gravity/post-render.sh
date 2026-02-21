#!/bin/bash
# post-render.sh — called automatically when a render completes
# Copies video to shared volume, runs exploration for next round, saves results.
cd "$(dirname "$0")"

SHARED_DIR="${GRAVITY_SHARED_DIR:-/Users/matte/.openclaw/workspace/shared/gravity}"
VOLUME="/Volumes/My Shared Files/shared"
ROUND_FILE="/tmp/gravity_cycle_round.txt"
RESULTS_FILE="/tmp/gravity_explore_results.txt"

# Read/increment round
ROUND=$(cat "$ROUND_FILE" 2>/dev/null || echo "1")
NEXT_ROUND=$(( ROUND + 1 ))
echo "$NEXT_ROUND" > "$ROUND_FILE"

echo "[post-render] Round $ROUND complete. Starting post-render tasks..."

# Copy latest video to shared volume
LATEST=$(ls -t "$SHARED_DIR"/gravity_*.mp4 2>/dev/null | head -1)
if [ -n "$LATEST" ] && [ -d "$VOLUME" ]; then
    cp "$LATEST" "$VOLUME/" && echo "[post-render] Copied $(basename $LATEST) → shared volume"
fi

# Run exploration for next round
echo "[post-render] Running explore round $NEXT_ROUND..."
bash explore.sh 120 "$NEXT_ROUND" 2>&1 | tee "$RESULTS_FILE"
echo "[post-render] Exploration done. Results at $RESULTS_FILE"
echo "EXPLORE_READY round=$NEXT_ROUND" > /tmp/gravity_explore_ready.txt
