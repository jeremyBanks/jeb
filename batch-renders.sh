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
FRAMES="${GRAVITY_FRAMES:-32768}"   # 64*64*8 = 32768 default

if [ ! -f "$QUEUE_FILE" ]; then
    echo "[batch] no queue file at $QUEUE_FILE — nothing to do"
    exit 0
fi

COMMIT=$(git rev-parse --short=12 HEAD 2>/dev/null || echo "unknown")
ALWAYS_ARGS=( --tile-2x2 )  # applied to every render regardless of queue entry
echo "=== batch-renders.sh started | ${FRAMES} frames per render | always: ${ALWAYS_ARGS[*]} | commit=$COMMIT ==="
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

FREE_MIN_GB=4  # pause batch if free space drops below this

check_disk() {
    local free_kb
    free_kb=$(df / | awk 'NR==2 {print $4}')
    local free_gb=$(( free_kb / 1024 / 1024 ))
    if [ "$free_gb" -lt "$FREE_MIN_GB" ]; then
        echo "[batch] ⚠️  disk low: ${free_gb}GB free (threshold ${FREE_MIN_GB}GB) — pausing"
        openclaw message send --channel discord --target 1467063568712339561 \
            --message "⚠️ **Batch paused — disk low** ${free_gb}GB free, need >${FREE_MIN_GB}GB to continue." 2>/dev/null || true
        exit 1
    fi
}

while true; do
    check_disk
    entry=$(pop_queue) || { echo "[batch] queue empty — done."; exit 0; }

    label="${entry%%:*}"
    raw_args="${entry#*:}"
    # Split raw_args into array — these are simple CLI flags with no spaces in values
    IFS=' ' read -r -a extra <<< "$raw_args"

    SEED=$(od -An -N4 -tu4 /dev/urandom | tr -d ' ')
    RUN_ID="$(date +%Y%m%d_%H%M%S)_${label}"

    remaining=$(grep -c "^[^#[:space:]]" "$QUEUE_FILE" 2>/dev/null || echo 0)
    echo "──────────────────────────────────────────────────────"
    echo "[batch] $label  run_id=$RUN_ID  seed=$SEED  (${remaining} more in queue)"
    echo "──────────────────────────────────────────────────────"

    # ── Kill any existing non-headless render ──────────────────────────────
    while IFS= read -r pid; do
        if ! ps -p "$pid" -o args= 2>/dev/null | grep -q "\-\-headless"; then
            echo "[batch] killing existing render PID=$pid"
            kill -9 "$pid" 2>/dev/null || true
        fi
    done < <(pgrep -f "gravity.*--seconds" 2>/dev/null || true)
    sleep 3

    # ── Clean up leftover frames/segments from any killed renders ─────────
    LEFTOVER_FRAMES=$(find runs/ -path "*/frames/*.png" -type f 2>/dev/null | wc -l | tr -d ' ')
    if [ "$LEFTOVER_FRAMES" -gt 0 ]; then
        echo "[batch] cleaning $LEFTOVER_FRAMES leftover frame PNGs from previous run(s)..."
        find runs/ -path "*/frames/*.png" -type f -delete
    fi

    # ── Clean state ────────────────────────────────────────────────────────
    rm -f state/checkpoint.bin state/orig_state.bin state/run_info.txt state/last_stats.txt
    rm -f segments.txt 2>/dev/null || true

    # ── Launch render ──────────────────────────────────────────────────────
    export GRAVITY_SHARED_DIR="${GRAVITY_SHARED_DIR:-/Volumes/My Shared Files/shared/}"
    mkdir -p "$GRAVITY_SHARED_DIR"

    "$BIN" \
        --seed "$SEED" --run-id "$RUN_ID" --commit "$COMMIT" \
        --frames "$FRAMES" --epilogue \
        "${ALWAYS_ARGS[@]}" \
        "${extra[@]}" \
        > "/tmp/gravity_render_${label}.log" 2>&1 &
    RENDER_PID=$!
    echo "[batch] render PID=$RENDER_PID"

    # ── Start watcher once run_info is ready ──────────────────────────────
    pkill -f "segment-watcher" 2>/dev/null || true
    sleep 1
    for (( i=0; i<25; i++ )); do
        sleep 2
        [ -f "runs/${RUN_ID}/run_info.txt" ] && break
        [ "$i" -eq 24 ] && echo "[batch] WARNING: run_info.txt not found after 50s"
    done
    cp "runs/${RUN_ID}/run_info.txt" state/run_info.txt 2>/dev/null || true

    # Only launch segment watcher for long renders (>60s); short breadth runs
    # finish before the watcher can catch any segments.
    WATCHER_PID=""
    if [ "$SECONDS_EACH" -gt 60 ]; then
        nohup bash segment-watcher.sh > /tmp/watcher.log 2>&1 &
        WATCHER_PID=$!
        echo "[batch] watcher PID=$WATCHER_PID"
    fi

    # ── Wait for render ────────────────────────────────────────────────────
    echo "[batch] waiting for render..."
    if wait "$RENDER_PID"; then
        echo "[batch] ✓ $label complete"
    else
        echo "[batch] ✗ $label exited non-zero — see /tmp/gravity_render_${label}.log"
    fi

    [ -n "$WATCHER_PID" ] && kill "$WATCHER_PID" 2>/dev/null || true

    # ── Post-render cleanup: delete frames/segments/audio from older runs ──
    bash scripts/cleanup-old-runs.sh "$RUN_ID" 2>/dev/null || true

    sleep 5
    echo ""
done
