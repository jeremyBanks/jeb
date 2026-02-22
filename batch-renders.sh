#!/bin/bash
# batch-renders.sh — queue-driven sequential renderer.
# Reads configs one at a time from batch-queue.txt (gravity crate root).
# Each line: "label:extra args..."  — blank lines and # comments ignored.
# Append lines to batch-queue.txt at any time to add more to the queue.
# Empty queue = script exits cleanly.
#
# Duration: 64×64×16 = 65536 frames @ 60fps = 1092s per render (~18 min).
# Previews sent to Discord automatically via segment-watcher.sh.
# Videos auto-archive to shared storage after each concat.
set -euo pipefail
cd "$(dirname "$0")/.."

BIN="/Users/matte/jeb/target/release/gravity"
QUEUE_FILE="batch-queue.txt"
SECONDS_EACH=1092   # 64*64*16 / 60

if [ ! -f "$QUEUE_FILE" ]; then
    echo "[batch] no queue file at $QUEUE_FILE — nothing to do"
    exit 0
fi

COMMIT=$(git rev-parse --short=12 HEAD 2>/dev/null || echo "unknown")
ALWAYS_ARGS="--tile-2x2"  # applied to every render regardless of queue entry
echo "=== batch-renders.sh started | ${SECONDS_EACH}s per render | always: $ALWAYS_ARGS | commit=$COMMIT ==="
echo "=== queue: $QUEUE_FILE ==="
echo ""

pop_queue() {
    # Find first non-empty, non-comment line; delete it; print it.
    local lnum
    lnum=$(grep -n "^[^#[:space:]]" "$QUEUE_FILE" 2>/dev/null | head -1 | cut -d: -f1)
    [ -z "$lnum" ] && return 1
    sed -n "${lnum}p" "$QUEUE_FILE"
    # macOS sed needs '' after -i; GNU sed does not
    sed -i '' "${lnum}d" "$QUEUE_FILE" 2>/dev/null || sed -i "${lnum}d" "$QUEUE_FILE"
    return 0
}

while true; do
    entry=$(pop_queue) || { echo "[batch] queue empty — done."; exit 0; }

    label="${entry%%:*}"
    raw_args="${entry#*:}"
    # shellcheck disable=SC2206
    extra=($raw_args)

    SEED=$(od -An -N4 -tu4 /dev/urandom | tr -d ' ')
    RUN_ID="$(date +%Y%m%d_%H%M%S)_${label}"

    remaining=$(grep -c "^[^#[:space:]]" "$QUEUE_FILE" 2>/dev/null || echo 0)
    echo "──────────────────────────────────────────────────────"
    echo "[batch] $label  run_id=$RUN_ID  seed=$SEED  (${remaining} more in queue)"
    echo "──────────────────────────────────────────────────────"

    # ── Kill any existing non-headless render ──────────────────────────────
    EXISTING=$(pgrep -f "gravity.*--seconds" 2>/dev/null | while read -r pid; do
        ps -p "$pid" -o args= 2>/dev/null | grep -q "\-\-headless" || echo "$pid"
    done || true)
    if [ -n "$EXISTING" ]; then
        echo "[batch] killing existing render(s): $EXISTING"
        kill -9 $EXISTING 2>/dev/null || true
        sleep 3
    fi

    # ── Clean up leftover frames/segments from any killed renders ─────────
    # Frames and segments should never persist after a render finishes.
    # If a render was SIGKILL'd mid-chunk they'll be stranded here.
    # cleanup-old-runs.sh handles this properly after each render, but also
    # do a quick sweep here before starting so we never start full.
    LEFTOVER_FRAMES=$(find runs/ -path "*/frames/*.png" -type f 2>/dev/null | wc -l | tr -d ' ')
    if [ "$LEFTOVER_FRAMES" -gt 0 ]; then
        echo "[batch] cleaning $LEFTOVER_FRAMES leftover frame PNGs from previous run(s)..."
        find runs/ -path "*/frames/*.png" -type f -delete
    fi

    # ── Clean state ────────────────────────────────────────────────────────
    rm -f state/checkpoint.bin state/orig_state.bin state/run_info.txt state/last_stats.txt
    rm -f segments.txt 2>/dev/null || true

    # ── Launch render ──────────────────────────────────────────────────────
    export GRAVITY_SHARED_DIR="${GRAVITY_SHARED_DIR:-/Users/matte/.openclaw/workspace/shared/gravity}"
    mkdir -p "$GRAVITY_SHARED_DIR"

    "$BIN" \
        --seed "$SEED" --run-id "$RUN_ID" --commit "$COMMIT" \
        --seconds "$SECONDS_EACH" --epilogue \
        $ALWAYS_ARGS \
        "${extra[@]}" \
        > "/tmp/gravity_render_${label}.log" 2>&1 &
    RENDER_PID=$!
    echo "[batch] render PID=$RENDER_PID"

    # ── Start watcher once run_info is ready ──────────────────────────────
    pkill -f "segment-watcher" 2>/dev/null || true
    sleep 1
    for i in $(seq 25); do
        sleep 2
        [ -f "runs/${RUN_ID}/run_info.txt" ] && break
        [ "$i" -eq 25 ] && echo "[batch] WARNING: run_info.txt not found after 50s"
    done
    cp "runs/${RUN_ID}/run_info.txt" state/run_info.txt 2>/dev/null || true

    nohup bash segment-watcher.sh > /tmp/watcher.log 2>&1 &
    WATCHER_PID=$!
    echo "[batch] watcher PID=$WATCHER_PID"

    # ── Wait for render ────────────────────────────────────────────────────
    echo "[batch] waiting for render..."
    if wait "$RENDER_PID"; then
        echo "[batch] ✓ $label complete"
    else
        echo "[batch] ✗ $label exited non-zero — see /tmp/gravity_render_${label}.log"
    fi

    kill "$WATCHER_PID" 2>/dev/null || true
    sleep 5
    echo ""
done
