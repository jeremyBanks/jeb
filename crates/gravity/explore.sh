#!/bin/bash
# explore.sh — headless parameter sweep to find configs with continuous movement
# Runs each config for SIM_SECONDS of sim time, reports avg_spd at end
# Usage: ./explore.sh

BIN="/Users/matte/jeb/target/release/gravity"
SIM_SECONDS=90  # sim-time seconds per config (headless runs faster than realtime)
SEED=42

echo "=== Gravity parameter exploration ==="
echo "sim time: ${SIM_SECONDS}s per config"
echo ""

run_config() {
    local label="$1"; shift
    local extra_args="$@"
    echo -n "[$label] $extra_args ... "

    # Run headless, capture last few stat lines
    local out
    out=$(timeout 300s "$BIN" --seconds "$SIM_SECONDS" --headless --seed "$SEED" \
        --pop-target 5120 --wrap --speed-cap 6.0 \
        $extra_args 2>/dev/null | grep "avg_spd" | tail -5)

    if [ -z "$out" ]; then
        echo "FAILED/TIMEOUT"
        return
    fi

    # Extract avg_spd values from last 5 lines
    local speeds
    speeds=$(echo "$out" | grep -oP 'avg_spd=\K[0-9.]+')
    local final_speed
    final_speed=$(echo "$speeds" | tail -1)
    local min_speed
    min_speed=$(echo "$speeds" | sort -n | head -1)

    # Also extract spread and pop
    local final_line
    final_line=$(echo "$out" | tail -1)
    local spread pop
    spread=$(echo "$final_line" | grep -oP 'spread=\K[0-9.]+')
    pop=$(echo "$final_line" | grep -oP 'pop=\K[0-9]+')

    echo "final_avg_spd=$final_speed  min_spd=$min_speed  spread=$spread  pop=$pop"
}

# Baseline
run_config "baseline" "--gravity 0.03125 --softening 6"

# Higher G — more orbital energy
run_config "G×4"      "--gravity 0.125   --softening 6"
run_config "G×8"      "--gravity 0.25    --softening 6"
run_config "G×16"     "--gravity 0.5     --softening 6"

# Lower softening — sharper wells, more close-range force
run_config "soft/2"   "--gravity 0.03125 --softening 3"
run_config "soft/4"   "--gravity 0.03125 --softening 1.5"

# Combined changes
run_config "G×4+s3"   "--gravity 0.125   --softening 3"
run_config "G×4+s4"   "--gravity 0.125   --softening 4"
run_config "G×8+s4"   "--gravity 0.25    --softening 4"
run_config "G×2+s4"   "--gravity 0.0625  --softening 4"

echo ""
echo "=== Done ==="
