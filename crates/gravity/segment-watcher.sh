#!/bin/bash
# segment-watcher.sh — sends a 3-part preview per completed segment.
# Run-aware: reads run_id from state/run_info.txt at startup and validates
# every segment against it. Self-terminates if the run changes (new render started).
# Scopes seen-file to run_id so restarts never replay old segments.
cd "$(dirname "$0")"

DISCORD_CHANNEL="1467063568712339561"
PREVIEW_DIR="/Users/matte/.openclaw/workspace/shared/gravity"
PREVIEW_W=512
PREVIEW_H=-2   # auto-height preserves aspect ratio (use -2 for codec alignment)
CLIP_FULL=2      # seconds at full speed per clip section
CLIP_SLOW_SRC=1  # seconds of source for slow section (→ 3s output at 1/3 speed)
SLOW_FPS=20
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
LAST_TIME=$(date +%s)
chunk_count=0  # incremented per segment; no hardcoded total

# Estimate total frames from run_info (seconds * 60fps)
RUN_SECONDS=$(grep "^seconds:" state/run_info.txt 2>/dev/null | awk '{print $2}')
TOTAL_FRAMES=$(( ${RUN_SECONDS:-0} * 60 ))

echo "[watcher] started for run $RUN_ID"

# ── preview builder ─────────────────────────────────────────────────────────
make_preview() {
    local seg="$1" out="$2"
    local dur
    dur=$(ffprobe -v error -show_entries format=duration \
          -of csv=p=0 "$seg" 2>/dev/null | tr -d '[:space:]')
    dur=${dur:-0}

    local scale="scale=${PREVIEW_W}:${PREVIEW_H}:flags=neighbor"

    local clip_total; clip_total=$(echo "scale=3; $CLIP_FULL + $CLIP_SLOW_SRC" | bc)
    if (( $(echo "$dur < $(echo "scale=3; $clip_total * 2 + 1" | bc)" | bc -l) )); then
        ffmpeg -y -i "$seg" -vf "$scale" -r $SLOW_FPS -c:v libx264 -crf 22 -preset fast "$out" 2>/dev/null
        return
    fi

    # Compute start times for each of the 3 positions (beginning / middle / end)
    local a0=0
    local b0; b0=$(echo "scale=3; $dur / 2 - $clip_total / 2" | bc)
    local c0; c0=$(echo "scale=3; $dur - $clip_total"          | bc)
    # Slow section starts 2s into each clip
    local a1; a1=$(echo "scale=3; $a0 + $CLIP_FULL" | bc)
    local b1; b1=$(echo "scale=3; $b0 + $CLIP_FULL" | bc)
    local c1; c1=$(echo "scale=3; $c0 + $CLIP_FULL" | bc)

    echo "[watcher] preview seeks: a=${a0}+${a1} b=${b0}+${b1} c=${c0}+${c1} (dur=${dur})"

    # Single input decoded once, split 6 ways for video + 6 ways for audio.
    # trim+setpts/atrim+asetpts for accurate section extraction.
    # Slow sections: video pts ×3, audio also trimmed to match (not pitch-shifted).
    local slow_audio_dur; slow_audio_dur=$(echo "scale=3; $CLIP_SLOW_SRC * 3" | bc)
    ffmpeg -y -i "$seg" -filter_complex "
        [0:v]split=6[v0][v1][v2][v3][v4][v5];
        [0:a]asplit=6[a0][a1][a2][a3][a4][a5];
        [v0]trim=start=${a0}:duration=${CLIP_FULL},setpts=PTS-STARTPTS,${scale},fps=${SLOW_FPS}[vaf];
        [v1]trim=start=${a1}:duration=${CLIP_SLOW_SRC},setpts=3*(PTS-STARTPTS),${scale},fps=${SLOW_FPS}[vas];
        [v2]trim=start=${b0}:duration=${CLIP_FULL},setpts=PTS-STARTPTS,${scale},fps=${SLOW_FPS}[vbf];
        [v3]trim=start=${b1}:duration=${CLIP_SLOW_SRC},setpts=3*(PTS-STARTPTS),${scale},fps=${SLOW_FPS}[vbs];
        [v4]trim=start=${c0}:duration=${CLIP_FULL},setpts=PTS-STARTPTS,${scale},fps=${SLOW_FPS}[vcf];
        [v5]trim=start=${c1}:duration=${CLIP_SLOW_SRC},setpts=3*(PTS-STARTPTS),${scale},fps=${SLOW_FPS}[vcs];
        [a0]atrim=start=${a0}:duration=${CLIP_FULL},asetpts=PTS-STARTPTS[aaf];
        [a1]atrim=start=${a1}:duration=${slow_audio_dur},asetpts=PTS-STARTPTS[aas];
        [a2]atrim=start=${b0}:duration=${CLIP_FULL},asetpts=PTS-STARTPTS[abf];
        [a3]atrim=start=${b1}:duration=${slow_audio_dur},asetpts=PTS-STARTPTS[abs];
        [a4]atrim=start=${c0}:duration=${CLIP_FULL},asetpts=PTS-STARTPTS[acf];
        [a5]atrim=start=${c1}:duration=${slow_audio_dur},asetpts=PTS-STARTPTS[acs];
        [vaf][aaf][vas][aas][vbf][abf][vbs][abs][vcf][acf][vcs][acs]concat=n=6:v=1:a=1[vout][aout]
    " -map "[vout]" -map "[aout]" -c:v libx264 -crf 22 -preset fast -c:a aac -b:a 128k "$out" 2>/tmp/watcher_ffmpeg.log
}

# ── main loop ───────────────────────────────────────────────────────────────
while true; do
    # Self-terminate if run_id changed (new render started, new watcher launched)
    CURRENT_RUN=$(grep "^run_id=" state/run_info.txt 2>/dev/null | cut -d= -f2)
    if [ -n "$CURRENT_RUN" ] && [ "$CURRENT_RUN" != "$RUN_ID" ]; then
        echo "[watcher] run changed to $CURRENT_RUN — exiting stale watcher for $RUN_ID"
        exit 0
    fi

    for seg in $(ls "runs/${RUN_ID}/segments/seg_"*.mp4 2>/dev/null | sort); do
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
        (( chunk_count++ )) || true
        chunk_num=$chunk_count
        # Percentage based on actual frame offset (accurate regardless of chunk size)
        if [ "$TOTAL_FRAMES" -gt 0 ]; then
            PCT=$(( frame_offset * 100 / TOTAL_FRAMES ))
        else
            PCT="?"
        fi

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
                WRAP_X=$(grep "^wrap_x:"    state/run_info.txt | awk '{print $2}')
                WRAP_Y=$(grep "^wrap_y:"    state/run_info.txt | awk '{print $2}')
                INITV=$(   grep "^init_vel:"   state/run_info.txt | awk '{print $2}')
                VELSC=$(   grep "^vel_scale:"  state/run_info.txt | awk '{print $2}')
                CMT=$(     grep "^commit:"     state/run_info.txt | awk '{print $2}')
                DAMP_X=$(  grep "^dampen_x:"   state/run_info.txt | awk '{print $2}')
                DAMP_Y=$(  grep "^dampen_y:"   state/run_info.txt | awk '{print $2}')
                INITPOP=$( grep "^init_pop:"   state/run_info.txt | awk '{print $2}')
                RES=$(   grep "^resolution:"  state/run_info.txt | awk '{print $2}')
                EXTRA=" | \`${RUN_ID}\` ${CMT} G=${G} soft=${S} cap=${SC} pop=${POP}±${BAND} init_pop=${INITPOP} rate=${RATE} wx=${WRAP_X} wy=${WRAP_Y} dx=${DAMP_X} dy=${DAMP_Y} vel=${INITV}×${VELSC} grid=${RES}"
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
