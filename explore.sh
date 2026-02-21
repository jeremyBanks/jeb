#!/bin/bash
# explore.sh — headless parameter sweep tracking blob movement over time
# 5-minute sim per config, 30-second snapshots, checks whether dense clusters drift.
# Usage: ./explore.sh [sim_seconds]   (default 300)

BIN="/Users/matte/jeb/target/release/gravity"
SIM_SECONDS="${1:-300}"
SNAPSHOT_INTERVAL=30   # seconds of sim time between snapshots
SEED=42
WORKDIR="/tmp/gravity_explore_$$"
mkdir -p "$WORKDIR/state" "$WORKDIR/segments"
cleanup() { rm -rf "$WORKDIR"; }
trap cleanup EXIT

echo "=== Gravity parameter exploration ==="
echo "sim=${SIM_SECONDS}s | snapshot every ${SNAPSHOT_INTERVAL}s | seed=${SEED}"
echo "Goal: blobs that move. Good = hot blocks drift between snapshots."
echo ""

run_config() {
    local label="$1"; shift
    local extra_args="$@"
    local outfile="$WORKDIR/${label}.txt"

    echo "--- [$label] $extra_args"

    ( cd "$WORKDIR" && GRAVITY_SHARED_DIR="$WORKDIR" \
        "$BIN" --seconds "$SIM_SECONDS" --headless --seed "$SEED" \
        --pop-target 5120 --wrap --speed-cap 6.0 \
        $extra_args ) > "$outfile" 2>&1

    # Grab one stat line per 30-second interval (every SNAPSHOT_INTERVAL*60 log lines since log_every=60 frames=1s)
    local stat_lines
    stat_lines=$(grep "avg_spd" "$outfile")
    local total_samples
    total_samples=$(echo "$stat_lines" | grep -c "avg_spd" || echo 0)

    if [ "$total_samples" -lt 2 ]; then
        echo "  ERROR: only $total_samples stat samples found"
        cat "$outfile" | tail -5
        rm -f "$WORKDIR/segments"/*.mp4 "$WORKDIR/state"/* 2>/dev/null
        return
    fi

    # Pick one