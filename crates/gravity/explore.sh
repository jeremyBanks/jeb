#!/bin/bash
# explore.sh — headless parameter sweep, tracks blob drift across time snapshots.
# Writes best_config.txt with winning args when done.
# Usage: ./explore.sh [sim_seconds] [round_number]
#
# ── ALL AVAILABLE PARAMETERS ─────────────────────────────────────────────────
# --gravity N        Gravitational constant (default: 0.03125)
#                    Higher = stronger attraction, faster cluster collapse
# --softening N      Softening radius for gravity wells (default: 6)
#                    Higher = softer wells, cells deflect rather than capture
# --speed-cap N      Maximum cell speed (default: 1.125)
#                    MUST match prod value; too high → all cells pinned at cap
# --pop-target N     Target population (default: 2560 = W*H/16)
#                    Conway births/deaths steer toward this
# --pop-band N       Population band: allowed range is [target-band, target+band]
#                    (default: 1280) — wider = more population variance allowed
# --rate-limit N     Max Conway births+deaths per tick (default: 32)
#                    Lower = slower Conway evolution; higher = faster churn
# --seed-density N   Initial density = 1/N cells per pixel (default: 128)
#                    Lower N = denser start; higher N = sparser start
# --wrap             Toroidal boundary (default: off = hard walls)
#                    Cells that exit one side reappear on the other
# --dampen           Zero out system COM velocity each tick (default: off)
#                    Prevents whole system from drifting off-screen
# --steer            Steer cells back toward center (default: off)
# ─────────────────────────────────────────────────────────────────────────────
# Known-good production config (the original 73-min render):
#   G=0.03125  soft=6  cap=6  pop=5120  band=1280  rate=32  density=1/128
#   wrap=true  dampen=false  steer=false
# ─────────────────────────────────────────────────────────────────────────────

BIN="/Users/matte/jeb/target/release/gravity"
SIM_SECONDS="${1:-120}"
ROUND="${2:-1}"
SEED=42
WORKDIR="/tmp/gravity_explore_$$"
BEST_FILE="$(dirname "$0")/best_config.txt"
COMMIT=$(git -C "$(dirname "$0")" rev-parse --short=12 HEAD 2>/dev/null || echo "unknown")
mkdir -p "$WORKDIR/state" "$WORKDIR/segments"
trap "rm -rf $WORKDIR" EXIT

# ── BASE PARAMS (explicitly set; configs override individual flags) ────────────
# These match the known-good production config. Override any in CONFIGS entries.
BASE_ARGS="--pop-target 5120 --pop-band 160 --rate-limit 4 --seed-density 128 --speed-cap 4.5 --gravity 0.03125 --softening 6 --init-vel zero"
# Baseline = 3am known-good params. Note: --wrap is NOT in BASE_ARGS so we can
# test both modes. Add --wrap explicitly in any config entry that needs it.

echo "=== Explore round $ROUND | sim=${SIM_SECONDS}s | seed=$SEED ==="
echo "=== Base: $BASE_ARGS ==="

# Parameter sets by round — each round tries fresh combinations
declare -a CONFIGS
case "$ROUND" in
1)
  # Round 1: vary around 3am known-good baseline (BASE_ARGS).
  # Baseline: G=0.03125 soft=6 cap=4.5 pop_band=160 rate=4 init_vel=zero
  # Test wrap/nowrap, rate_limit, pop_band, and init_vel modes.
  CONFIGS=(
    "wrap+base:--wrap"
    "nowrap+base:"
    "wrap+rate=1:--wrap --rate-limit 1"
    "wrap+rate=8:--wrap --rate-limit 8"
    "wrap+band=80:--wrap --pop-band 80"
    "wrap+band=320:--wrap --pop-band 320"
    "wrap+swirl:--wrap --init-vel swirl"
    "wrap+spin:--wrap --init-vel spin"
  )
  ;;
2)
  # Round 2: vary gravity + softening with wrap, holding Conway params at baseline.
  CONFIGS=(
    "wrap+base:--gravity 0.03125 --softening 6 --wrap"
    "wrap+Ghalf+s6:--gravity 0.015 --softening 6 --wrap"
    "wrap+G×2+s6:--gravity 0.0625 --softening 6 --wrap"
    "wrap+G×4+s6:--gravity 0.125 --softening 6 --wrap"
    "wrap+G×1+s3:--gravity 0.03125 --softening 3 --wrap"
    "wrap+G×1+s12:--gravity 0.03125 --softening 12 --wrap"
    "wrap+G×2+s3:--gravity 0.0625 --softening 3 --wrap"
    "wrap+G×2+s12:--gravity 0.0625 --softening 12 --wrap"
  )
  ;;
3)
  # Round 3: vary initial conditions (seed_density, pop_target) and dampen.
  # Different starting densities → different initial cluster topologies.
  CONFIGS=(
    "wrap+base:--gravity 0.03125 --softening 6 --wrap"
    "wrap+dense=1/64:--gravity 0.03125 --softening 6 --wrap --seed-density 64"
    "wrap+dense=1/256:--gravity 0.03125 --softening 6 --wrap --seed-density 256"
    "wrap+pop=1280:--gravity 0.03125 --softening 6 --wrap --pop-target 1280"
    "wrap+pop=2560:--gravity 0.03125 --softening 6 --wrap --pop-target 2560"
    "wrap+pop=10240:--gravity 0.03125 --softening 6 --wrap --pop-target 10240"
    "wrap+dampen:--gravity 0.03125 --softening 6 --wrap --dampen"
    "wrap+rate=8+band=512:--gravity 0.03125 --softening 6 --wrap --rate-limit 8 --pop-band 512"
  )
  ;;
4)
  # Round 4: explore init_vel modes — does initial angular momentum affect long-term?
  CONFIGS=(
    "swirl+wrap:--init-vel swirl --gravity 0.03125 --softening 6 --wrap"
    "random+wrap:--init-vel random --gravity 0.03125 --softening 6 --wrap"
    "spin+wrap:--init-vel spin --gravity 0.03125 --softening 6 --wrap"
    "spin-ccw+wrap:--init-vel spin-ccw --gravity 0.03125 --softening 6 --wrap"
    "radial-out+wrap:--init-vel radial-out --gravity 0.03125 --softening 6 --wrap"
    "zero+wrap:--init-vel zero --gravity 0.03125 --softening 6 --wrap"
    "spin+nowrap:--init-vel spin --gravity 0.03125 --softening 6"
    "spin+G×2:--init-vel spin --gravity 0.0625 --softening 6 --wrap"
  )
  ;;
*)
  # Round 5+: cross-product of best init_vel + best G/soft/rate from R1-R4
  CONFIGS=(
    "r${ROUND}a:--init-vel swirl --gravity 0.0625 --softening 6 --rate-limit 8 --wrap"
    "r${ROUND}b:--init-vel spin --gravity 0.0625 --softening 6 --rate-limit 8 --wrap"
    "r${ROUND}c:--init-vel swirl --gravity 0.03125 --softening 12 --rate-limit 8 --wrap"
    "r${ROUND}d:--init-vel spin --gravity 0.03125 --softening 12 --rate-limit 8 --wrap"
    "r${ROUND}e:--init-vel swirl --gravity 0.125 --softening 6 --rate-limit 8 --wrap"
    "r${ROUND}f:--init-vel spin --gravity 0.125 --softening 12 --rate-limit 8 --wrap"
    "r${ROUND}g:--init-vel random --gravity 0.0625 --softening 12 --pop-band 512 --wrap"
    "r${ROUND}h:--init-vel spin --gravity 0.03125 --softening 6 --speed-cap 2 --rate-limit 8 --wrap"
  )
  ;;
esac

BEST_SCORE="-1"
BEST_LABEL=""
BEST_ARGS=""

score_config() {
    local label="$1"
    local outfile="$WORKDIR/${label}.txt"
    local stat_lines
    stat_lines=$(grep "avg_spd" "$outfile")
    local n
    n=$(echo "$stat_lines" | grep -c "." 2>/dev/null || echo 0)
    [ "$n" -lt 2 ] && echo "0" && return

    local first last
    first=$(echo "$stat_lines" | head -1)
    last=$(echo  "$stat_lines" | tail -1)

    local hot_start hot_end
    hot_start=$(echo "$first" | grep -oE 'hot=\[[^]]*\]' | sed 's/hot=\[//;s/\]//')
    hot_end=$(echo   "$last"  | grep -oE 'hot=\[[^]]*\]' | sed 's/hot=\[//;s/\]//')
    local blob_moved=0
    [ "$hot_start" != "$hot_end" ] && blob_moved=1

    local p10 spd blk
    p10=$(echo "$last" | grep -oE 'p10=[0-9.]+' | cut -d= -f2 | awk '{printf "%d", $1*10}')
    spd=$(echo "$last" | grep -oE 'avg_spd=[0-9.]+' | cut -d= -f2 | awk '{printf "%d", $1*10}')
    blk=$(echo "$last" | grep -oE 'blk=[0-9]+'  | cut -d= -f2)

    # Score: blob movement is primary (weight 1000), then p10 and blk
    echo $(( blob_moved * 1000 + p10 + blk + spd ))
}

for entry in "${CONFIGS[@]}"; do
    label="${entry%%:*}"
    extra_args="${entry#*:}"
    rm -f "$WORKDIR/state"/* "$WORKDIR/segments"/*.mp4 2>/dev/null
    echo ""
    echo "--- [$label] $extra_args"

    local_out="$WORKDIR/${label}.txt"
    ( cd "$WORKDIR" && GRAVITY_SHARED_DIR="$WORKDIR" \
        "$BIN" --seconds "$SIM_SECONDS" --headless --seed "$SEED" \
        --commit "$COMMIT" \
        $BASE_ARGS \
        $extra_args ) > "$local_out" 2>&1

    stat_lines=$(grep "avg_spd" "$local_out")
    n_samples=$(echo "$stat_lines" | grep -c "." 2>/dev/null || echo 0)
    if [ "$n_samples" -lt 2 ]; then
        echo "  ERROR: $n_samples samples. Last output:"
        tail -3 "$local_out"
        continue
    fi

    # Print snapshots every 30s
    echo "  time   avg_spd   p10    blk/640  dense  hot_blocks"
    i=0
    while IFS= read -r line; do
        i=$(( i + 1 ))
        if [ $(( (i-1) % 30 )) -eq 0 ] || [ "$i" -eq "$n_samples" ]; then
            spd=$(echo   "$line" | grep -oE 'avg_spd=[0-9.]+' | cut -d= -f2)
            p10=$(echo   "$line" | grep -oE 'p10=[0-9.]+'     | cut -d= -f2)
            blk=$(echo   "$line" | grep -oE 'blk=[0-9]+'      | cut -d= -f2)
            dense=$(echo "$line" | grep -oE 'dense=[0-9]+'    | cut -d= -f2)
            hot=$(echo   "$line" | grep -oE 'hot=\[[^]]*\]'   | sed 's/hot=\[//;s/\]//')
            printf "  t=%3ds  %7s  %6s  %4s/640  %5s  %s\n" \
                "$((i-1))" "${spd:-?}" "${p10:-?}" "${blk:-?}" "${dense:-?}" "${hot:-?}"
        fi
    done <<< "$stat_lines"

    # Drift verdict: compare SETTLED snapshots (skip first ~15s of ramp-up)
    # Use line 15+ as "settled" — by then cells have formed initial clusters
    settled_lines=$(echo "$stat_lines" | tail -n +16)
    n_settled=$(echo "$settled_lines" | grep -c "." 2>/dev/null || echo 0)

    first_line=$(echo "$stat_lines" | head -1)
    last_line=$(echo  "$stat_lines" | tail -1)

    if [ "$n_settled" -ge 2 ]; then
        settled_first=$(echo "$settled_lines" | head -1)
        settled_last=$(echo  "$settled_lines" | tail -1)
        hs=$(echo "$settled_first" | grep -oE 'hot=\[[^]]*\]' | sed 's/hot=\[//;s/\]//')
        he=$(echo "$settled_last"  | grep -oE 'hot=\[[^]]*\]' | sed 's/hot=\[//;s/\]//')
        # Count unique hot block positions across settled snapshots
        unique_hots=$(echo "$settled_lines" | grep -oE 'hot=\[[^]]*\]' | sort -u | wc -l | tr -d ' ')
        if [ "$hs" = "$he" ]; then
            echo "  ⚠️  STATIC after settling: hot always [$hs] ($unique_hots unique patterns)"
        else
            echo "  ✅ DRIFTING after settling: [$hs] → [$he] ($unique_hots unique patterns)"
        fi
    else
        settled_first="$first_line"
        settled_last="$last_line"
        hs=$(echo "$first_line" | grep -oE 'hot=\[[^]]*\]' | sed 's/hot=\[//;s/\]//')
        he=$(echo "$last_line"  | grep -oE 'hot=\[[^]]*\]' | sed 's/hot=\[//;s/\]//')
        echo "  (not enough settled samples)"
    fi

    p10s=$(echo "$first_line" | grep -oE 'p10=[0-9.]+' | cut -d= -f2)
    p10e=$(echo "$last_line"  | grep -oE 'p10=[0-9.]+' | cut -d= -f2)
    echo "  p10: $p10s → $p10e"

    sc=$(score_config "$label")
    echo "  score: $sc"

    if [ "$(echo "$sc > $BEST_SCORE" | bc -l)" = "1" ]; then
        BEST_SCORE="$sc"
        BEST_LABEL="$label"
        BEST_ARGS="$extra_args"
    fi
done

echo ""
echo "=== Best: [$BEST_LABEL] score=$BEST_SCORE ==="
echo "  args: $BEST_ARGS"
echo "$BEST_ARGS" > "$BEST_FILE"
echo "round=$ROUND" >> "$BEST_FILE"
echo "label=$BEST_LABEL" >> "$BEST_FILE"
echo "score=$BEST_SCORE" >> "$BEST_FILE"
echo "Written → $BEST_FILE"

# Write flag file so heartbeat can detect completion and act
echo "EXPLORE_READY round=$ROUND label=$BEST_LABEL score=$BEST_SCORE" > /tmp/gravity_explore_ready.txt
echo "results=/tmp/gravity_explore_round${ROUND}.txt" >> /tmp/gravity_explore_ready.txt
