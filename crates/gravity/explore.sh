#!/bin/bash
# explore.sh — headless parameter sweep tracking blob movement over time
# 5-minute sims, snapshots every 30s, tracks whether dense clusters drift
# Usage: ./explore.sh [sim_seconds]

BIN="/Users/matte/jeb/target/release/gravity"
SIM_SECONDS="${1:-300}"   # 5 minutes sim time per config
SEED=42
WORKDIR="/tmp/gravity_explore_$$"
mkdir -p "$WORKDIR/state" "$WORKDIR/segments"
trap "rm -rf $WORKDIR" EXIT

echo "=== Gravity parameter exploration ==="
echo "sim=${SIM_SECONDS}s ($(( SIM_SECONDS / 60 ))m) | seed=${SEED} | pop=5120 | wrap | cap=6"
echo "Snapshots every 30s of sim time. Hot = coordinates of 3 densest 8×8 blocks."
echo ""

run_config() {
    local label="$1"; shift
    local extra_args="$@"
    local outfile="$WORKDIR/${label}.txt"
    rm -f "$WORKDIR/state"/* "$WORKDIR/segments"/*.mp4 2>/dev/null

    echo "--- [$label] $extra_args"

    ( cd "$WORKDIR" && GRAVITY_SHARED_DIR="$WORKDIR" \
        "$BIN" --seconds "$SIM_SECONDS" --headless --seed "$SEED" \
        --pop-target 5120 --wrap --speed-cap 6.0 \
        $extra_args ) > "$outfile" 2>&1

    # Collect all stat lines (printed every 60 frames = 1s in headless)
    local stat_lines
    stat_lines=$(grep "avg_spd" "$outfile")
    local n_samples
    n_samples=$(echo "$stat_lines" | grep -c "." || echo 0)

    if [ "$n_samples" -lt 2 ]; then
        echo "  ERROR: only $n_samples samples found"
        echo "  (last lines:)"; tail -5 "$outfile"
        return
    fi

    # Print snapshot every 30s of sim time = every 30 lines
    echo "  time   avg_spd   p10    blk/640  dense  hot_blocks"
    local i=0
    while IFS= read -r line; do
        i=$(( i + 1 ))
        # Print at t=0s, t=30s, t=60s, ... (every 30 lines)
        if [ $(( (i - 1) % 30 )) -eq 0 ] || [ "$i" -eq "$n_samples" ]; then
            local t=$(( (i - 1) ))
            local spd p10 blk dense hot
            spd=$(echo   "$line" | grep -oE 'avg_spd=[0-9.]+' | cut -d= -f2)
            p10=$(echo   "$line" | grep -oE 'p10=[0-9.]+'     | cut -d= -f2)
            blk=$(echo   "$line" | grep -oE 'blk=[0-9]+'      | cut -d= -f2)
            dense=$(echo "$line" | grep -oE 'dense=[0-9]+'    | cut -d= -f2)
            hot=$(echo   "$line" | grep -oE 'hot=\[[^]]*\]'   | sed 's/hot=\[//;s/\]//')
            printf "  t=%3ds  %7s  %6s  %4s/640  %5s  %s\n" \
                "$t" "${spd:-?}" "${p10:-?}" "${blk:-?}" "${dense:-?}" "${hot:-?}"
        fi
    done <<< "$stat_lines"

    # Hot-block drift: compare first and last hot block coordinates
    local first_line last_line
    first_line=$(echo "$stat_lines" | head -1)
    last_line=$(echo  "$stat_lines" | tail -1)
    local hot_start hot_end
    hot_start=$(echo "$first_line" | grep -oE 'hot=\[[^]]*\]' | sed 's/hot=\[//;s/\]//')
    hot_end=$(echo   "$last_line"  | grep -oE 'hot=\[[^]]*\]' | sed 's/hot=\[//;s/\]//')
    if [ "$hot_start" = "$hot_end" ]; then
        echo "  ⚠️  hot blocks UNCHANGED start→end: static blobs"
    else
        echo "  ✅ hot blocks moved: start=[$hot_start] → end=[$hot_end]"
    fi

    # p10 trend: is it staying positive (good) or dropping to zero (static cells)
    local p10_start p10_end
    p10_start=$(echo "$first_line" | grep -oE 'p10=[0-9.]+' | cut -d= -f2)
    p10_end=$(echo   "$last_line"  | grep -oE 'p10=[0-9.]+' | cut -d= -f2)
    echo "  p10: start=${p10_start:-?} → end=${p10_end:-?}"
    echo ""
}

# Baseline
run_config "baseline"   "--gravity 0.03125 --softening 6"

# Higher G
run_config "G×4"        "--gravity 0.125   --softening 6"
run_config "G×8"        "--gravity 0.25    --softening 6"

# Sharper wells
run_config "soft=3"     "--gravity 0.03125 --softening 3"
run_config "soft=1.5"   "--gravity 0.03125 --softening 1.5"

# Combined
run_config "G×4+s3"     "--gravity 0.125   --softening 3"
run_config "G×4+s4"     "--gravity 0.125   --softening 4"
run_config "G×8+s4"     "--gravity 0.25    --softening 4"

echo "=== Done ==="
