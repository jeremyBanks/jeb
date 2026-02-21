#!/bin/bash
# segment-watcher.sh — sends a Discord preview clip whenever a new segment completes
cd "$(dirname "$0")"

DISCORD_CHANNEL="1467063568712339561"
SEEN_FILE="/tmp/gravity_segments_seen.txt"
PREVIEW_DIR="/Users/matte/.openclaw/workspace/shared/gravity"
touch "$SEEN_FILE"

echo "[watcher] started, watching segments/"

TOTAL=69

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
        chunk_num=$(( frame_offset / 3840 + 1 ))

        preview="${PREVIEW_DIR}/preview_chunk${chunk_num}.mp4"

        if ffmpeg -y -i "$seg" -t 15 -vf scale=960:600 -c:v libx264 -crf 22 -preset fast "$preview" 2>/dev/null; then
            if openclaw message send --channel discord \
                -t "$DISCORD_CHANNEL" \
                --media "$preview" \
                -m "chunk ${chunk_num}/${TOTAL} — \`$seg_name\`"; then
                echo "[watcher] sent chunk $chunk_num"
                echo "$seg" >> "$SEEN_FILE"  # only mark seen on success
            else
                echo "[watcher] send failed for chunk $chunk_num, will retry"
            fi
            rm -f "$preview"
        else
            echo "[watcher] ffmpeg failed for chunk $chunk_num, will retry"
        fi
    done

    sleep 5
done
