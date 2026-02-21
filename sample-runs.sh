#!/bin/bash
# sample-runs.sh — run a matrix of 5-minute renders with varied parameters
# Usage: ./sample-runs.sh [output_dir]
# Sends each result to Discord automatically via openclaw

set -e
cd "$(dirname "$0")"

DURATION=300   # 5 minutes per sample
SEED=42
DISCORD_CHANNEL="1467063568712339561"
SHARED_DIR="/Users/matte/.openclaw/workspace/shared/gravity"
OUT_DIR="${1:-/tmp/gravity-samples}"
mkdir -p "$OUT_DIR"

BINARY="../../target/release/gravity"

run_sample() {
    local name="$1"
    local label="$2"
    shift 2
    local args=("$@")

    echo ""
    echo "════════════════════════════════════════"
    echo "  SAMPLE: $name"
    echo "  args: ${args[*]}"
    echo "════════════════════════════════════════"

    local out="$OUT_DIR/${name}.mp4"

    # Clean run state
    rm -f state/checkpoint.bin state/orig_state.bin segments.txt segments/seg_*.mp4 2>/dev/null || true

    # Run gravity for DURATION seconds
    "$BINARY" --seconds "$DURATION" --seed "$SEED" "${args[@]}" 2>&1

    # Find output (gravity writes to shared/gravity/)
    local latest
    latest=$(ls -t "$SHARED_DIR"/gravity_*.mp4 2>/dev/null | head -1)
    if [ -z "$latest" ]; then
        echo "[!] No output found for $name"
        return
    fi

    cp "$latest" "$out"
    echo "  → saved $out"

    # Make 30s preview (first 10s + middle 10s + last 10s at preview res)
    local preview="$SHARED_DIR/sample_${name}.mp4"
    local dur
    dur=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$out" 2>/dev/null | tr -d '[:space:]')
    local mid_start
    mid_start=$(echo "scale=1; $dur / 2 - 5" | bc)
    local last_start
    last_start=$(echo "scale=1; $dur - 10" | bc)
    local tmp
    tmp=$(mktemp -d)

    ffmpeg -y -i "$out" -t 10 -vf "scale=512:320:flags=neighbor" \
        -c:v libx264 -crf 22 -preset fast "$tmp/a.mp4" 2>/dev/null
    ffmpeg -y -i "$out" -ss "$mid_start" -t 10 -vf "scale=512:320:flags=neighbor" \
        -c:v libx264 -crf 22 -preset fast "$tmp/b.mp4" 2>/dev/null
    ffmpeg -y -i "$out" -ss "$last_start" -t 10 -vf "scale=512:320:flags=neighbor" \
        -c:v libx264 -crf 22 -preset fast "$tmp/c.mp4" 2>/dev/null
    ffmpeg -y -f lavfi -i "color=black:s=512x320:r=60" -t 0.25 \
        -c:v libx264 -crf 22 -preset fast "$tmp/gap.mp4" 2>/dev/null

    printf "file '%s'\nfile '%s'\nfile '%s'\nfile '%s'\nfile '%s'\n" \
        "$tmp/a.mp4" "$tmp/gap.mp4" "$tmp/b.mp4" "$tmp/gap.mp4" "$tmp/c.mp4" > "$tmp/list.txt"
    ffmpeg -y -f concat -safe 0 -i "$tmp/list.txt" -c copy "$preview" 2>/dev/null
    rm -rf "$tmp"

    # Send to Discord
    openclaw message send --channel discord \
        -t "$DISCORD_CHANNEL" \
        --media "$preview" \
        -m "🔬 **sample: $name** | $label"

    rm -f "$preview"
}

# ── Parameter matrix ────────────────────────────────────────────────────────
# Format: run_sample <name> <label> [gravity args...]

run_sample "baseline"      "G=0.5 soft=6 cap=1.125 rate=32 (current defaults)" \
    --wrap

run_sample "low-gravity"   "G=0.25 — diffuse, spread out, less clustering" \
    --wrap --g 0.25

run_sample "high-gravity"  "G=1.0 — tight clusters, fast orbital motion" \
    --wrap --g 1.0

run_sample "sharp"         "softening=3 — hard gravity wells, spiky dynamics" \
    --wrap --softening 3.0

run_sample "spread"        "softening=12 — gentle gradient, smoother flow" \
    --wrap --softening 12.0

run_sample "slow"          "speed_cap=0.5 — deliberate, meditative" \
    --wrap --speed-cap 0.5

run_sample "fast"          "speed_cap=2.25 — energetic, more chaotic" \
    --wrap --speed-cap 2.25

run_sample "high-conway"   "rate_limit=128 — max Conway churn" \
    --wrap --rate-limit 128

run_sample "low-conway"    "rate_limit=8 — minimal Life activity" \
    --wrap --rate-limit 8

run_sample "dampen"        "damping ON — momentum absorbed on collision" \
    --wrap --dampen

run_sample "dense"         "seed_density=1/32 — 4× more starting cells" \
    --wrap --seed-density 32

run_sample "sparse"        "seed_density=1/256 — very few starting cells" \
    --wrap --seed-density 256

echo ""
echo "All samples done → $OUT_DIR"
