#!/bin/bash
# batch-renders.sh — queue-driven sequential renderer.
# Reads configs one at a time from batch-queue.txt (in the gravity crate root).
# Each line: "label:extra args..."
# Append lines to batch-queue.txt while running to add more to the queue.
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
echo "=== batch-renders.sh started | ${SECONDS_EACH}s per render | commit=$COMMIT ==="
echo "=== queue: $QUEUE_FILE ==="
echo ""

while true; do
    # ── Pop first non-empty, non-comment line from queue ──────────────────
    entry=""
    while IFS= read -r line || [ -n "$line" ]; do
        [[ -z "$line" || "$line" == \#* ]] && continue
        entry="$line"
        break
    done < "$QUEUE_FILE"

    if [ -z "$entry" ]; then
        echo "[batch] queue empty — done."
        exit 0
    fi

    # Remove that line from queue (first occurrence)
    sed -i '' "0,/$(echo "$entry" | sed 's/[\/&]/\\&/g')/{/$(echo "$entry" | sed 's/[\/&]/\\&/g')/d;}" "$QUEUE_FILE" 2>/dev/null || \
    sed -i "0,/$(echo "$entry" | sed 's/[\/&]/\\&/g')/{/$(echo "$entry" | sed 's/[\/&]/\\&/g')/d;}" "$QUEUE_FILE" 2>/dev/null || true

    label="${entry%%:*}"
    raw_args="${entry#*:}"
    # shellcheck disable=SC2206
    extra=($raw_args)

    SEED=$(od -An -N4 -tu4 /dev/urandom | tr -d ' ')
    RUN_ID="$(date +%Y%m%d_%H%M%S)_${label}"

    echo "──────────────────────────────────────────────────────"
    echo "[batch] $label  run_id=$RUN_ID  seed=$SEED"
    remaining=$(grep -c "^[^#]" "$QUEUE_FILE" 2>/dev/null || echo 0)
    echo "[batch] $remaining more in queue after this one"
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

    # ── Clean up state from previous run ──────────────────────────────────
    rm -f state/checkpoint.bin state/orig_state.bin state/run_info.txt state/last_stats.txt
    rm -f segments.txt 2>/dev/null || true

    # ── Launch render ──────────────────────────────────────────────────────
    export GRAVITY_SHARED_DIR="${GRAVITY_SHARED_DIR:-/Users/matte/.openclaw/workspace/shared/gravity}"
    mkdir -p "$GRAVITY_SHARED_DIR"

    "$BIN" \
        --seed "$SEED" --run-id "$RUN_ID" --commit "$COMMIT" \
        --seconds "$SECONDS_EACH" --epilogue \
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
        echo "[batch] ✗ $label exited non-zero — check /tmp/gravity_render_${label}.log"
    fi

    kill "$WATCHER_PID" 2>/dev/null || true
    sleep 5
    echo ""
done
