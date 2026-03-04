#!/bin/bash
# sample-runs.sh — 5-minute parameter exploration matrix
# Sends each result preview to Discord for comparison.
# Usage: ./sample-runs.sh
# Run from crates/gravity/ directory.

set -e
cd "$(dirname "$0")"

DURATION=300   # 5 minutes per sample
DISCORD_CHANNEL="1467063568712339561"
SHARED_DIR="/Users/matte/.openclaw/workspace/shared/gravity"
SAMPLES_DIR="$SHARED_DIR/samples"
mkdir -p "$SAMPLES_DIR"

BINARY="../../target/release/gravity"

SAMPLE_N=0

run_sample() {
    local name="$1"
    local label="$2"
    shift 2
    local args=("$@")

    SAMPLE_N=$(( SAMPLE_N + 1 ))
    local seed=$SAMPLE_N   # unique seed per run → unique output filename

    echo ""
    echo "══════════════════════════════════════════════"
    echo "  [$SAMPLE_N] $name"
    echo "  args: ${args[*]}"
    echo "══════════════════════════════════════════════"

    # Clean run state
    rm -f state/checkpoint.bin state/orig_state.bin segments.txt /tmp/gravity_segments_seen.txt
    rm -f segments/seg_*.mp4 2>/dev/null || true

    # Run
    local t0=$SECONDS
    "$BINARY" --seconds "$DURATION" --seed "$seed" "${args[@]}" 2>&1
    local elapsed=$(( SECONDS - t0 ))

    # Locate output
    local out="$SHARED_DIR/gravity_${DURATION}s_seed${seed}.mp4"
    if [ ! -f "$out" ]; then
        echo "[!] output not found: $out"
        return
    fi

    # Copy to samples dir with descriptive name
    local saved="$SAMPLES_DIR/${SAMPLE_N}_${name}.mp4"
    cp "$out" "$saved"
    local size_mb=$(du -m "$saved" | cut -f1)
    echo "  → $saved (${size_mb}MB, ${elapsed}s render time)"

    # 30s preview: first 10s + middle 10s + last 10s
    local preview="$SHARED_DIR/preview_sample_${SAMPLE_N}.mp4"
    local dur
    dur=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$out" 2>/dev/null | tr -d '[:space:]')
    local mid_start last_start tmp
    mid_start=$(echo "scale=1; $dur / 2 - 5" | bc)
    last_start=$(echo "scale=1; $dur - 10" | bc)
    tmp=$(mktemp -d)

    ffmpeg -y -i "$out" -t 10 -vf "scale=512:320:flags=neighbor" \
        -c:v libx264 -crf 22 -preset fast -c:a aac "$tmp/a.mp4" 2>/dev/null
    ffmpeg -y -i "$out" -ss "$mid_start" -t 10 -vf "scale=512:320:flags=neighbor" \
        -c:v libx264 -crf 22 -preset fast -c:a aac "$tmp/b.mp4" 2>/dev/null
    ffmpeg -y -i "$out" -ss "$last_start" -t 10 -vf "scale=512:320:flags=neighbor" \
        -c:v libx264 -crf 22 -preset fast -c:a aac "$tmp/c.mp4" 2>/dev/null
    ffmpeg -y -f lavfi -i "color=black:s=512x320:r=60" -t 0.25 -an \
        -c:v libx264 -crf 22 -preset fast "$tmp/gap.mp4" 2>/dev/null
    printf "file '%s'\nfile '%s'\nfile '%s'\nfile '%s'\nfile '%s'\n" \
        "$tmp/a.mp4" "$tmp/gap.mp4" "$tmp/b.mp4" "$tmp/gap.mp4" "$tmp/c.mp4" > "$tmp/list.txt"
    ffmpeg -y -f concat -safe 0 -i "$tmp/list.txt" -c copy "$preview" 2>/dev/null
    rm -rf "$tmp"

    openclaw message send --channel discord \
        -t "$DISCORD_CHANNEL" \
        --media "$preview" \
        -m "🔬 **[$SAMPLE_N] $name** — $label | ${elapsed}s render | ${size_mb}MB"

    rm -f "$preview"
}

# ── Sample matrix ─────────────────────────────────────────────────────────────
# Baseline
run_sample "baseline" \
    "G=0.5 soft=6 cap=1.125 rate=32 (all defaults)" \
    --wrap

# Gravity variants
run_sample "low-gravity" \
    "G=0.25 — diffuse, gentler pull, larger orbits" \
    --wrap --gravity 0.25

run_sample "high-gravity" \
    "G=1.0 — tight clusters, faster orbital collapse" \
    --wrap --gravity 1.0

# Softening variants
run_sample "sharp-wells" \
    "softening=3 — harder singularities, spiky dynamics" \
    --wrap --softening 3.0

run_sample "smooth-wells" \
    "softening=12 — very gentle gradient, laminar flow" \
    --wrap --softening 12.0

# Speed cap
run_sample "slow-cap" \
    "speed_cap=0.5 — meditative, deliberate movement" \
    --wrap --speed-cap 0.5

run_sample "fast-cap" \
    "speed_cap=2.25 — energetic, approaching chaotic" \
    --wrap --speed-cap 2.25

# Conway rate
run_sample "high-conway" \
    "rate_limit=128 — max Life churn when slow/sparse" \
    --wrap --rate-limit 128

run_sample "low-conway" \
    "rate_limit=8 — minimal Life, mostly gravity" \
    --wrap --rate-limit 8

# Damping
run_sample "dampen" \
    "--dampen ON — energy absorbed on collision, slower decay" \
    --wrap --dampen

# Density
run_sample "dense-start" \
    "seed_density=32 — 4× starting cells, denser initial state" \
    --wrap --seed-density 32

# Combo: high G + low speed (tight/slow — potentially most organic)
run_sample "tight-slow" \
    "G=1.0 soft=3 cap=0.75 — tight gravity, slow cap" \
    --wrap --gravity 1.0 --softening 3.0 --speed-cap 0.75

echo ""
echo "All ${SAMPLE_N} samples complete → $SAMPLES_DIR"
