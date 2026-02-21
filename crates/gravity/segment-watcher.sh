#!/bin/bash
# segment-watcher.sh — sends a 3-part Discord preview clip whenever a new segment completes
# Preview: first 6s + middle 6s + last 6s @ 512×320 NN, with 1/8s black gap between parts
cd "$(dirname "$0")"

DISCORD_CHANNEL="1467063568712339561"
SEEN_FILE="/tmp/gravity_segments_seen.txt"
PREVIEW_DIR="/Users/matte/.openclaw/workspace/shared/gravity"
touch "$SEEN_FILE"

echo "[watcher] started, watching segments/"

TOTAL=137
LAST_TIME=$(date +%s)  # track time between segments
PREVIEW_W=512
PREVIEW_H=320
CLIP_DUR=4
GAP_DUR="0.125"  # 1/8 second

make_preview() {
    local seg="$1"
    local out="$2"

    local dur
    dur=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$seg" 2>/dev/null | tr -d '[:space:]')
    dur=${dur:-0}

    local scale="scale=${PREVIEW_W}:${PREVIEW_H}:flags=neighbor"

    if (( $(echo "$dur < $((CLIP_DUR * 2 + 1))" | bc -l) )); then
        ffmpeg -y -i "$seg" -vf "$scale" -c:v libx264 -crf 22 -preset fast "$out" 2>/dev/null
        return
    fi

    local mid_start last_start
    mid_start=$(echo "scale=3; $dur / 2 - $CLIP_DUR / 2" | bc)
    last_start=$(echo "scale=3; $dur - $CLIP_DUR" | bc)

    local tmp
    tmp=$(mktemp -d)

    ffmpeg -y -i "$seg" -t $CLIP_DUR -vf "$scale" -c:v libx264 -crf 22 -preset fast "$tmp/part_a.mp4" 2>/dev/null
    ffmpeg -y -i "$seg" -ss "$mid_start" -t $CLIP_DUR -vf "$scale" -c:v libx264 -crf 22 -preset fast "$tmp/part_b.mp4" 2>/dev/null
    ffmpeg -y -i "$seg" -ss "$last_start" -t $CLIP_DUR -vf "$scale" -c:v libx264 -crf 22 -preset fast "$tmp/part_c.mp4" 2>/dev/null

    ffmpeg -y -f lavfi -i "color=black:s=${PREVIEW_W}x${PREVIEW_H}:r=60" \
        -t $GAP_DUR -c:v libx264 -crf 22 -preset fast "$tmp/gap.mp4" 2>/dev/null

    printf "file '%s'\nfile '%s'\nfile '%s'\nfile '%s'\nfile '%s'\n" \
        "$tmp/part_a.mp4" "$tmp/gap.mp4" \
        "$tmp/part_b.mp4" "$tmp/gap.mp4" \
        "$tmp/part_c.mp4" > "$tmp/list.txt"

    ffmpeg -y -f concat -safe 0 -i "$tmp/list.txt" -c copy "$out" 2>/dev/null
    rm -rf "$tmp"
}

while true; do
    for seg in $(ls segments/seg_*.mp4 2>/dev/null | sort); do
        if grep -qF "$seg" "$SEEN_FILE"; then
            continue
        fi

        sleep 2
        [ -f "$seg" ] || continue

        seg_name=$(basename "$seg" .mp4)
        frame_offset=$(echo "$seg_name" | sed 's/seg_0*//')
        frame_offset=${frame_offset:-0}
        chunk_num=$(( frame_offset / 1920 + 1 ))

        preview="${PREVIEW_DIR}/preview_chunk${chunk_num}.mp4"

        if make_preview "$seg" "$preview"; then
            NOW=$(date +%s)
            ELAPSED=$(( NOW - LAST_TIME ))
            MINS=$(( ELAPSED / 60 ))
            SECS=$(( ELAPSED % 60 ))
            SIZE_MB=$(du -m "$seg" | cut -f1)
            META="${MINS}m${SECS}s | ${SIZE_MB}MB"

            # Include run_id, params, pop, sim/enc timing
            EXTRA=""
            RUN_ID=""
            PARAMS=""
            if [ -f "state/run_info.txt" ]; then
                RUN_ID=$(grep "^run_id=" state/run_info.txt | cut -d= -f2)
                COMMIT=$(grep "^commit:" state/run_info.txt | awk '{print $2}')
                G=$(grep "^gravity:" state/run_info.txt | awk '{print $2}')
                S=$(grep "^softening:" state/run_info.txt | awk '{print $2}')
                SC=$(grep "^speed_cap:" state/run_info.txt | awk '{print $2}')
                POP_T=$(grep "^pop_target:" state/run_info.txt | awk '{print $2}')
                BAND=$(grep "^pop_band:" state/run_info.txt | awk '{print $2}')
                RATE=$(grep "^rate_limit:" state/run_info.txt | awk '{print $2}')
                WRAP=$(grep "^wrap:" state/run_info.txt | awk '{print $2}')
                DAMPEN=$(grep "^dampen:" state/run_info.txt | awk '{print $2}')
                INITV=$(grep "^init_vel:" state/run_info.txt | awk '{print $2}')
                PARAMS="G=${G} soft=${S} cap=${SC} pop=${POP_T} band=${BAND} rate=${RATE} wrap=${WRAP} dampen=${DAMPEN} vel=${INITV}"
                [ -n "$COMMIT" ] && PARAMS="commit=${COMMIT} ${PARAMS}"
            fi
            if [ -f "state/last_stats.txt" ]; then
                POP=$(grep "^pop=" state/last_stats.txt | cut -d= -f2)
                SIM_MS=$(grep "^sim_ms=" state/last_stats.txt | cut -d= -f2)
                ENC_MS=$(grep "^enc_ms=" state/last_stats.txt | cut -d= -f2)
                [ -n "$POP" ] && EXTRA=" | pop=${POP}"
                [ -n "$SIM_MS" ] && EXTRA="${EXTRA} | sim=${SIM_MS}ms enc=${ENC_MS}ms"
            fi
            [ -n "$RUN_ID" ] && EXTRA="\`${RUN_ID}\` ${PARAMS}${EXTRA}"

            if openclaw message send --channel discord \
                -t "$DISCORD_CHANNEL" \
                --media "$preview" \
                -m "chunk ${chunk_num}/${TOTAL} | ${META}${EXTRA}"; then
                echo "[watcher] sent chunk $chunk_num (${META})"
                echo "$seg" >> "$SEEN_FILE"
                LAST_TIME=$NOW
            else
                echo "[watcher] send failed for chunk $chunk_num, will retry"
            fi
        else
            echo "[watcher] ffmpeg failed for chunk $chunk_num, will retry"
        fi
        rm -f "$preview"
    done

    sleep 5
done
