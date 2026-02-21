#!/bin/bash
# start-render.sh — launch a new render, killing any existing non-headless render first.
# Headless runs (--headless flag) are left alone.
# Usage: ./start-render.sh [gravity binary args...]
set -e
cd "$(dirname "$0")"

BIN="/Users/matte/jeb/target/release/gravity"

# Kill ALL non-headless gravity renders (force-kill, no SIGTERM)
EXISTING=$(pgrep -f "gravity.*--seconds" 2>/dev/null | while read pid; do
    ps -p "$pid" -o args= 2>/dev/null | grep -q "\-\-headless" || echo "$pid"
done)
if [ -n "$EXISTING" ]; then
    echo "[start-render] force-killing existing render(s): $EXISTING"
    kill -9 $EXISTING 2>/dev/null
    sleep 3  # give time for file handles to close
fi
# Verify they're dead
STILL=$(pgrep -f "gravity.*--seconds" 2>/dev/null | while read pid; do
    ps -p "$pid" -o args= 2>/dev/null | grep -q "\-\-headless" || echo "$pid"
done)
[ -n "$STILL" ] && { echo "[start-render] ERROR: processes still alive: $STILL"; exit 1; }

# Clean up state from previous run
rm -f state/checkpoint.bin state/orig_state.bin state/run_info.txt segments.txt
rm -f /tmp/gravity_segments_seen.txt
rm -rf segments/ && mkdir -p segments frames/chunk

# Restart watcher cleanly
pkill -f "segment-watcher" 2>/dev/null; sleep 1
nohup bash segment-watcher.sh > /tmp/watcher.log 2>&1 &
echo "[start-render] watcher PID=$!"

# Launch the render
COMMIT=$(git -C "$(dirname "$0")" rev-parse --short=12 HEAD 2>/dev/null || echo "unknown")
SEED=$(od -An -N4 -tu4 /dev/urandom | tr -d ' ')
RUN_ID="$(date +%Y%m%d_%H%M%S)"

echo "[start-render] starting: seed=$SEED run_id=$RUN_ID"
nohup "$BIN" \
    --seed "$SEED" --run-id "$RUN_ID" --commit "$COMMIT" \
    "$@" \
    > /tmp/gravity_render.log 2>&1 &

echo "[start-render] render PID=$!"
sleep 5
grep "gravity:\|softening:\|speed_cap:\|pop_band:\|rate_limit:\|init_vel:\|wrap:\|dampen:" state/run_info.txt 2>/dev/null || echo "run_info not yet written"
