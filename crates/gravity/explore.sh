#!/bin/bash
# explore.sh — headless sweep, tracks blob drift across time snapshots
# Writes best_config.txt with winning args when done.
# Usage: ./explore.sh [sim_seconds] [round_number]

BIN="/Users/matte/jeb/target/release/gravity"
SIM_SECONDS="${1:-120}"
ROUND="${2:-1}"
SEED=42
WORKDIR="/tmp/gravity_explore_$$"
BEST_FILE="$(dirname "$0")/best_config.txt"
mkdir -p "$WORKDIR/state" "$WORKDIR/segments"
trap "rm -rf $WORKDIR" EXIT

echo "=== Explore round $ROUND | sim=${SIM_SECONDS}s | seed=$SEED ==="

# Parameter sets by round — each round tries fresh combinations
declare -a CONFIGS
case "$ROUND" in
1)
  CONFIGS=(
    "baseline:--gravity 0.03125 --softening 6"
    "G×4:--gravity 0.125 --softening 6"
    "G×8:--gravity 0.25 --softening 6"
    "soft=3:--gravity 0.03125 --softening 3"
    "G×4+s3:--gravity 0.125 --softening 3"
    "G×4+s4:--gravity 0.125 --softening 4"
    "G×8+s4:--gravity 0.25 --softening 4"
    "G×2+s4:--gravity 0.0625 --softening 4"
  )
  ;;
2)
  CONFIGS=(
    "G×8+s3:--gravity 0.25 --softening 3"
    "G×16+s6:--gravity 0.5 --softening 6"
    "G×16+s4:--gravity 0.5 --softening 4"
    "G×32:--gravity 1.0 --softening 6"
    "G×4+s1.5:--gravity 0.125 --softening 1.5"
    "G×2+s3:--gravity 0.0625 --softening 3"
    "G×2+s1.5:--gravity 0.0625 --softening 1.5"
    "G×8+s1.5:--gravity 0.25 --softening 1.5"
  )
  ;;
3)
  CONFIGS=(
    "pop2560+G4:--gravity 0.125 --softening 4 --pop-target 2560"
    "pop7680+G4:--gravity 0.125 --softening 4 --pop-target 7680"
    "pop5120+G4+nodamp:--gravity 0.125 --softening 4"
    "cap3+G4:--gravity 0.125 --softening 4 --speed-cap 3"
    "cap9+G4:--gravity 0.125 --softening 4 --speed-cap 9"
    "cap3+G8:--gravity 0.25 --softening 4 --speed-cap 3"
    "nodamp+G2:--gravity 0.0625 --softening 3"
    "G×4+s8:--gravity 0.125 --softening 8"
  )
  ;;
*)
  # Round 4+: randomise around best from prior round
  CONFIGS=(
    "r${ROUND}a:--gravity 0.125 --softening 3"
    "r${ROUND}b:--gravity 0.175 --softening 3"
    "r${ROUND}c:--gravity 0.125 --softening 2"
    "r${ROUND}d:--gravity 0.2 --softening 4"
    "r${ROUND}e:--gravity 0.08 --softening 3"
    "r${ROUND}f:--gravity 0.25 --softening 3"
    "r${ROUND}g:--gravity 0.15 --softening 3.5"
    "r${ROUND}h:--gravity 0.1 --softening 2.5"
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
        --pop-target 5120 --wrap --speed-cap 6.0 \
        $extra_args ) > "$local_out" 2>&1

    stat_lines=$(grep "avg_spd" "$local_out")
    n_samples=$(echo "$stat_lines" | grep -c "." 2>/dev/null || echo 0)
    if [ "$n_samples" -lt 2 ]; then
        echo "  ERROR: $n_samples samples. Last output:"
        tail -3 "$local_out"
        continue
    fi

    # Print snapshots every 10s so we can see full trajectory
    echo "  time   avg_spd   p10    blk/640  dense  hot_blocks"
    i=0
    while IFS= read -r line; do
        i=$(( i + 1 ))
        if [ $(( (i-1) % 10 )) -eq 0 ] || [ "$i" -eq "$n_samples" ]; then
            spd=$(echo   "$line" | grep -oE 'avg_spd=[0-9.]+' | cut -d= -f2)
            p10=$(echo   "$line" | grep -oE 'p10=[0-9.]+'     | cut -d= -f2)
            blk=$(echo   "$line" | grep -oE 'blk=[0-9]+'      | cut -d= -f2)
            dense=$(echo "$line" | grep -oE 'dense=[0-9]+'    | cut -d= -f2)
            hot=$(echo   "$line" | grep -oE 'hot=\[[^]]*\]'   | sed 's/hot=\[//;s/\]//')
            printf "  t=%3ds  %7s  %6s  %4s/640  %5s  %s\n" \
                "$((i-1))" "${spd:-?}" "${p10:-?}" "${blk:-?}" "${dense:-?}" "${hot:-?}"
        fi
    done <<< "$stat_lines"

    # Hot block drift verdict
    first_line=$(echo "$stat_lines" | head -1)
    last_line=$(echo  "$stat_lines" | tail -1)
    hs=$(echo "$first_line" | grep -oE 'hot=\[[^]]*\]' | sed 's/hot=\[//;s/\]//')
    he=$(echo "$last_line"  | grep -oE 'hot=\[[^]]*\]' | sed 's/hot=\[//;s/\]//')
    if [ "$hs" = "$he" ]; then
        echo "  ⚠️  hot blocks STATIC: [$hs]"
    else
        echo "  ✅ hot blocks MOVED: [$hs] → [$he]"
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

// [recovery] edit target not found, appending:
SIM_SECONDS="${1:-120}"
