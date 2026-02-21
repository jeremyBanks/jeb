#!/bin/bash
# explore.sh — headless parameter sweep to find configs with continuous movement
# Samples stats every 60 frames; reports clustering (blk), p10 speed, COM drift.
# Usage: ./explore.sh [--seconds N]

BIN="/Users/matte/jeb/target/release/gravity"
SIM_SECONDS="${1:-120}"  # sim-time seconds per config
SEED=42
TMPDIR_BASE="/tmp/gravity_explore_$$"
mkdir -p "$TMPDIR_BASE"

echo "=== Gravity parameter exploration ==="
echo "sim time: ${SIM_SECONDS}s per config | seed=$SEED"
echo "Metrics: avg_spd (higher=more movement), p10 (10th %ile speed; >0=no stalled cells),"
echo "         blk/640 (block coverage; higher=spread out), dense (max cells in 8x8 block),"
echo "         COM_drift (pixels the center-of-mass moved over the run)"
echo ""
printf "%-18s %8s %8s %8s %8s %8s %8s\n" "CONFIG" "avg_spd" "p10_spd" "blk/640" "dense" "COM_drift" "score"
printf "%-18s %8s %8s %8s %8s %8s %8s\n" "------------------" "--------" "--------" "--------" "--------" "---------" "-------"

SCORES=()
LABELS=()

run_config() {
    local label="$1"; shift
    local extra_args="$@"
    local outfile="$TMPDIR_BASE/${label}.txt"

    GRAVITY_SHARED_DIR="$TMPDIR_BASE" \
        "$BIN" --seconds "$SIM_SECONDS" --headless --seed "$SEED" \
        --pop-target 5120 --wrap --speed-cap 6.0 \
        $extra_args > "$outfile" 2>/dev/null

    if [ ! -s "$outfile" ]; then
        printf "%-18s  FAILED\n" "$label"
        return
    fi

    # Parse all stat lines (headless prints every 60 frames = 1s of sim)
    local stat_lines
    stat_lines=$(grep "avg_spd" "$outfile")
    local n_samples
    n_samples=$(echo "$stat_lines" | wc -l | tr -d ' ')

    if [ "$n_samples" -eq 0 ]; then
        printf "%-18s  NO_STATS\n" "$label"
        return
    fi

    # Final sample values
    local last
    last=$(echo "$stat_lines" | tail -1)
    local avg_spd p10 blk dense
    avg_spd=$(echo "$last" | grep -oE 'avg_spd=[0-9.]+' | cut -d= -f2)
    p10=$(echo "$last"     | grep -oE 'p10=[0-9.]+' | cut -d= -f2)
    blk=$(echo "$last"     | grep -oE 'blk=[0-9]+' | cut -d= -f2)
    dense=$(echo "$last"   | grep -oE 'dense=[0-9]+' | cut -d= -f2)

    # COM drift: distance COM moved from first to last sample
    local first_com last_com
    first_com=$(echo "$stat_lines" | head -1 | grep -oE 'com=\([0-9.]+,[0-9.]+\)' | tr -d 'com=()')
    last_com=$(echo "$last"        | grep -oE 'com=\([0-9.]+,[0-9.]+\)' | tr -d 'com=()')
    local fx fy lx ly com_drift
    fx=$(echo "$first_com" | cut -d, -f1)
    fy=$(echo "$first_com" | cut -d, -f2)
    lx=$(echo "$last_com"  | cut -d, -f1)
    ly=$(echo "$last_com"  | cut -d, -f2)
    com_drift=$(echo "scale=1; sqrt(($lx-$fx)^2+($ly-$fy)^2)" | bc -l 2>/dev/null || echo "?")

    # Score: higher is better for "continuous non-clustering movement"
    # Components: avg_spd * p10 boost * spread_ratio
    # p10>0 means no totally-stationary cells (good)
    # blk>100 means not all in a tiny cluster (good)
    local spread_ratio blk_ratio score
    blk_ratio=$(echo "scale=3; ${blk:-0} / 640" | bc -l 2>/dev/null || echo "0")
    score=$(echo "scale=2; ${avg_spd:-0} * (1 + ${p10:-0}) * (1 + $blk_ratio)" | bc -l 2>/dev/null || echo "0")

    printf "%-18s %8s %8s %7s/640 %8s %9s %7s\n" \
        "$label" "${avg_spd:-?}" "${p10:-?}" "${blk:-?}" "${dense:-?}" "${com_drift}" "${score}"

    SCORES+=("$score $label")
    LABELS+=("$label")
}

# Baseline
run_config "baseline"       "--gravity 0.03125 --softening 6"

# Higher G — more gravitational energy → faster orbits
run_config "G×4"            "--gravity 0.125   --softening 6"
run_config "G×8"            "--gravity 0.25    --softening 6"
run_config "G×16"           "--gravity 0.5     --softening 6"

# Lower softening — sharper wells, stronger close-range interaction
run_config "soft=3"         "--gravity 0.03125 --softening 3"
run_config "soft=1.5"       "--gravity 0.03125 --softening 1.5"

# Combined
run_config "G×4+soft=4"     "--gravity 0.125   --softening 4"
run_config "G×4+soft=3"     "--gravity 0.125   --softening 3"
run_config "G×8+soft=4"     "--gravity 0.25    --softening 4"
run_config "G×2+soft=4"     "--gravity 0.0625  --softening 4"
run_config "G×2+soft=3"     "--gravity 0.0625  --softening 3"

echo ""
echo "=== Top configs by score ==="
printf '%s\n' "${SCORES[@]}" | sort -rn | head -5 | while read score label; do
    echo "  $label  (score=$score)"
done

echo ""
echo "Note: score = avg_spd × (1+p10) × (1+blk/640)"
echo "Favors: fast cells, none stationary, spread across grid"

rm -rf "$TMPDIR_BASE"
