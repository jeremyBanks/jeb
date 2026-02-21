#!/bin/bash
# explore.sh — headless parameter sweep to find configs with continuous movement
# Samples stats every 60 frames (1s sim-time); tracks how metrics evolve over the run.
# Usage: ./explore.sh [sim_seconds]   (default 90)

BIN="/Users/matte/jeb/target/release/gravity"
SIM_SECONDS="${1:-90}"
SEED=42
WORKDIR="/tmp/gravity_explore_$$"
mkdir -p "$WORKDIR/state" "$WORKDIR/segments"

cleanup() { rm -rf "$WORKDIR"; }
trap cleanup EXIT

echo "=== Gravity parameter exploration ==="
echo "sim_seconds=${SIM_SECONDS} | seed=${SEED} | pop_target=5120 | wrap | speed_cap=6"
echo ""
echo "Columns:"
echo "  spd_end   final avg_spd  (want: high)"
echo "  spd_drop  end/start ratio  (want: ≥1 = not slowing down)"
echo "  p10_end   final p10 speed  (want: >0 = no stalled cells)"
echo "  blk_end   final block coverage /640  (want: spread across grid)"
echo "  blk_drop  end/start block ratio  (want: ~1 = not collapsing into clusters)"
echo "  dense_end max cells/8x8 block  (want: low = not piling up)"
echo "  COM_drift total COM movement in pixels"
echo ""
printf "%-18s %8s %9s %8s %8s %9s %9s %9s\n" \
    "CONFIG" "spd_end" "spd_drop" "p10_end" "blk_end" "blk_drop" "dense_end" "COM_drift"
printf "%-18s %8s %9s %8s %8s %9s %9s %9s\n" \
    "------------------" "-------" "--------" "-------" "-------" "--------" "---------" "---------"

run_config() {
    local label="$1"; shift
    local extra_args="$@"
    local outfile="$WORKDIR/${label}.txt"

    # Run headless from our temp workdir so state/segments don't collide
    ( cd "$WORKDIR" && GRAVITY_SHARED_DIR="$WORKDIR" \
        "$BIN" --seconds "$SIM_SECONDS" --headless --seed "$SEED" \
        --pop-target 5120 --wrap --speed-cap 6.0 \
        $extra_args ) > "$outfile" 2>&1

    local stat_lines
    stat_lines=$(grep "avg_spd" "$outfile")
    local n_samples
    n_samples=$(echo "$stat_lines" | grep -c "avg_spd" || echo 0)

    if [ "$n_samples" -lt 2 ]; then
        printf "%-18s  (no stats — %d samples)\n" "$label" "$n_samples"
        return
    fi

    local first_line last_line mid_line
    first_line=$(echo "$stat_lines" | head -1)
    last_line=$(echo  "$stat_lines" | tail -1)
    # Take ~50% mark sample
    local mid_idx=$(( n_samples / 2 ))
    mid_line=$(echo "$stat_lines" | sed -n "${mid_idx}p")

    parse_field() { echo "$1" | grep -oE "$2=[0-9.]+" | head -1 | cut -d= -f2; }

    local spd_start spd_end p10_end blk_raw_start blk_end dense_end
    spd_start=$(parse_field "$first_line" "avg_spd")
    spd_end=$(parse_field   "$last_line"  "avg_spd")
    p10_end=$(parse_field   "$last_line"  "p10")
    dense_end=$(parse_field "$last_line"  "dense")

    # blk=189/640 — extract just the numerator
    blk_raw_start=$(echo "$first_line" | grep -oE 'blk=[0-9]+' | cut -d= -f2)
    blk_end=$(echo       "$last_line"  | grep -oE 'blk=[0-9]+' | cut -d= -f2)

    # COM drift: parse com=(x,y) from first and last
    local com_start com_end
    com_start=$(echo "$first_line" | grep -oE 'com=\([0-9.]+,[0-9.]+\)' | grep -oE '[0-9.]+,[0-9.]+')
    com_end=$(echo   "$last_line"  | grep -oE 'com=\([0-9.]+,[0-9.]+\)' | grep -oE '[0-9.]+,[0-9.]+')
    local fx fy lx ly com_drift
    fx=$(echo "$com_start" | cut -d, -f1); fy=$(echo "$com_start" | cut -d, -f2)
    lx=$(echo "$com_end"   | cut -d, -f1); ly=$(echo "$com_end"   | cut -d, -f2)
    com_drift=$(echo "scale=1; sqrt(($lx-$fx)^2+($ly-$fy)^2)" | bc -l 2>/dev/null || echo "?")

    # Ratios (end/start): >1 = growing, <1 = dropping
    local spd_drop blk_drop
    spd_drop=$(echo "scale=2; ${spd_end:-0} / ${spd_start:-1}" | bc -l 2>/dev/null || echo "?")
    blk_drop=$(echo "scale=2; ${blk_end:-0} / ${blk_raw_start:-1}" | bc -l 2>/dev/null || echo "?")

    printf "%-18s %8s %9s %8s %7s/640 %9s %9s %9s\n" \
        "$label" "${spd_end:-?}" "${spd_drop:-?}" "${p10_end:-?}" \
        "${blk_end:-?}" "${blk_drop:-?}" "${dense_end:-?}" "${com_drift:-?}"

    # Clean up per-run segments/state for next run
    rm -f "$WORKDIR/segments"/*.mp4 "$WORKDIR/state"/* 2>/dev/null
}

# Baseline
run_config "baseline"       "--gravity 0.03125 --softening 6"

# Higher G — more orbital energy
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
echo "=== Done ==="
echo "Ideal: spd_end high, spd_drop ≥1, p10_end >0, blk_end high, blk_drop ~1, dense low"
