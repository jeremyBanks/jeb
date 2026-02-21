#!/bin/bash
# segment-watcher.sh — sends a 3-part preview per completed segment.
# Run-aware: reads run_id from state/run_info.txt at startup and validates
# every segment against it. Self-terminates if the run changes (new render started).
# Scopes seen-file to run_id so restarts never replay old segments.
cd "$(dirname "$0")"

DISCORD_CHANNEL="1467063568712339561"
PREVIEW_DIR="/Users/matte/.openclaw/workspace/shared/gravity"
PREVIEW_W=512
PREVIEW_H=320
CLIP_DUR=4
GAP_DUR="0.125"

# ── read current run_id ─────────────────────────────────────────────────────
if [ ! -f state/run_info.txt ]; then
    echo "[watcher] waiting for state/run_info.txt..."
    for i in $(seq 30); do sleep 2; [ -f state/run_info.txt ] && break; done
fi
RUN_ID=$(grep "^run_id=" state/run_info.txt 2>/dev/null | cut -d= -f2)
if [ -z "$RUN_ID" ]; then echo "[watcher] no run_id, exiting"; exit 1; fi

SEEN_FILE="/tmp/gravity_seen_${RUN_ID}.txt"
touch "$SEEN_FILE"
TOTAL=137
LAST_TIME=$(date +%s)

echo "[watcher] started for run $RUN_ID"

# ── preview builder ─────────────────────────────────────────────────────────
make_preview() {
    local seg="$1" out="$2"
    local dur
    dur=$(ffprobe -v error -show_entries format=duration \
          -of csv=p=0 "$seg" 2>/dev/null | tr -d '[:space:]')
    dur=${dur:-0}

    local scale="scale=${PREVIEW_W}:${PREVIEW_H}:flags=neighbor"

    if (( $(echo "$dur < $((CLIP_DUR * 2 + 1))" | bc -l) )); then
        ffmpeg -y -i "$seg" -vf "$scale" -c:v libx264 -crf 22 -preset fast "$out" 2>/dev/null
        return
    fi

    local mid_start last_start
    mid_start=$(echo  "scale=3; $dur / 2 - $CLIP_DUR / 2" | bc)
    last_start=$(echo "scale=3; $dur - $CLIP_DUR"          | bc)

    local tmp; tmp=$(mktemp -d)
    ffmpeg -y -i "$seg" -t          $CLIP_DUR -vf "$scale" \
        -c:v libx264 -crf 22 -preset fast "$tmp/a.mp4" 2>/dev/null
    ffmpeg -y -i "$seg" -ss "$mid_start"  -t $CLIP_DUR -vf "$scale" \
        -c:v libx264 -crf 22 -preset fast "$tmp/b.mp4" 2>/dev/null
    ffmpeg -y -i "$seg" -ss "$last_start" -t $CLIP_DUR -vf "$scale" \
        -c:v libx264 -crf 22 -preset fast "$tmp/c.mp4" 2>/dev/null
    ffmpeg -y -f lavfi -i "color=black:s=${PREVIEW_W}x${PREVIEW_H}:r=60" \
        -t $GAP_DUR -c:v libx264 -crf 22 -preset fast "$tmp/gap.mp4" 2>/dev/null

    printf "file '%s'\nfile '%s'\nfile '%s'\nfile '%s'\nfile '%s'\n" \
        "$tmp/a.mp4" "$tmp/gap.mp4" \
        "$tmp/b.mp4" "$tmp/gap.mp4" \
        "$tmp/c.mp4" > "$tmp/list.txt"
    ffmpeg -y -f concat -safe 0 -i "$tmp/list.txt" -c copy "$out" 2>/dev/null
    rm -rf "$tmp"
}

# ── main loop ───────────────────────────────────────────────────────────────
while true; do
    # Self-terminate if run_id changed (new render started, new watcher launched)
    CURRENT_RUN=$(grep "^run_id=" state/run_info.txt 2>/dev/null | cut -d= -f2)
    if [ -n "$CURRENT_RUN" ] && [ "$CURRENT_RUN" != "$RUN_ID" ]; then
        echo "[watcher] run changed to $CURRENT_RUN — exiting stale watcher for $RUN_ID"
        exit 0
    fi

    for seg in $(ls segments/seg_*.mp4 2>/dev/null | sort); do
        grep -qF "$seg" "$SEEN_FILE" && continue

        sleep 2
        [ -f "$seg" ] || continue

        # Validate segment belongs to THIS run (check mtime vs run start time)
        # run_id is YYYYMMDD_HHMMSS — convert to epoch for comparison
        RUN_EPOCH=$(date -j -f "%Y%m%d_%H%M%S" "$RUN_ID" "+%s" 2>/dev/null || echo 0)
        SEG_MTIME=$(stat -f %m "$seg" 2>/dev/null || echo 0)
        if [ "$SEG_MTIME" -lt "$RUN_EPOCH" ]; then
            echo "[watcher] skipping stale segment $seg (predates run $RUN_ID)"
            echo "$seg" >> "$SEEN_FILE"
            continue
        fi

        seg_name=$(basename "$seg" .mp4)
        frame_offset=$(echo "$seg_name" | sed 's/seg_0*//')
        frame_offset=${frame_offset:-0}
        chunk_num=$(( frame_offset / 1920 + 1 ))

        preview="${PREVIEW_DIR}/preview_${RUN_ID}_chunk${chunk_num}.mp4"

        if make_preview "$seg" "$preview"; then
            NOW=$(date +%s)
            ELAPSED=$(( NOW - LAST_TIME ))
            META="${ELAPSED}s | $(du -m "$seg" | cut -f1)MB"

            # Build params string from run_info
            EXTRA=""
            if [ -f "state/run_info.txt" ]; then
                G=$(    grep "^gravity:"    state/run_info.txt | awk '{print $2}')
                S=$(    grep "^softening:"  state/run_info.txt | awk '{print $2}')
                SC=$(   grep "^speed_cap:"  state/run_info.txt | awk '{print $2}')
                POP=$(  grep "^pop_target:" state/run_info.txt | awk '{print $2}')
                BAND=$( grep "^pop_band:"   state/run_info.txt | awk '{print $2}')
                RATE=$( grep "^rate_limit:" state/run_info.txt | awk '{print $2}')
                WRAP=$( grep "^wrap:"       state/run_info.txt | awk '{print $2}')
                INITV=$(grep "^init_vel:"   state/run_info.txt | awk '{print $2}')
                CMT=$(  grep "^commit:"     state/run_info.txt | awk '{print $2}')
                DAMP=$(grep "^dampen:"    state/run_info.txt | awk '{print $2}')
                EXTRA=" | \`${RUN_ID}\` ${CMT} G=${G} soft=${S} cap=${SC} pop=${POP}±${BAND} rate=${RATE} wrap=${WRAP} dampen=${DAMP} vel=${INITV}"
            fi

            # Current state from render log (last stats line before chunk boundary)
            STATE_LINE=$(grep "avg_spd" /tmp/gravity_render.log 2>/dev/null | tail -1)
            CUR_POP=$(  echo "$STATE_LINE" | grep -oE 'pop=[0-9]+'     | cut -d= -f2)
            CUR_SPD=$(  echo "$STATE_LINE" | grep -oE 'avg_spd=[0-9.]+' | cut -d= -f2)
            CUR_P10=$(  echo "$STATE_LINE" | grep -oE 'p10=[0-9.]+'     | cut -d= -f2)
            STATE_MSG=""
            [ -n "$CUR_POP" ] && STATE_MSG="pop=${CUR_POP} avg_spd=${CUR_SPD} p10=${CUR_P10}"

            MSG="chunk ${chunk_num}/${TOTAL} | ${META}${EXTRA}"
            [ -n "$STATE_MSG" ] && MSG="${MSG}
${STATE_MSG}"

            if openclaw message send --channel discord \
                -t "$DISCORD_CHANNEL" \
                --media "$preview" \
                -m "$MSG"; then
                echo "[watcher] sent chunk $chunk_num ($META)"
                echo "$seg" >> "$SEEN_FILE"
                LAST_TIME=$NOW
            else
                echo "[watcher] send failed chunk $chunk_num, will retry"
            fi
        fi
        rm -f "$preview"
    done

    sleep 5
done
