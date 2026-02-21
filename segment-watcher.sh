#!/bin/bash
# segment-watcher.sh — sends a Discord preview clip whenever a new segment completes
cd "$(dirname "$0")"

DISCORD_CHANNEL="1467063568712339561"
SEEN_FILE="/tmp/gravity_segments_seen.txt"
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

        # Extract chunk number from frame offset in filename
        seg_name=$(basename "$seg" .mp4)
        frame_offset=$(echo "$seg_name" | sed 's/seg_0*//')
        frame_offset=${frame_offset:-0}
        chunk_num=$(( frame_offset / 3840 + 1 ))

        preview="/tmp/preview_chunk${chunk_num}.mp4"
        ffmpeg -y -i "$seg" -t 15 -vf scale=960:600 -c:v libx264 -crf 22 -preset fast "$preview" 2>/dev/null \
            && openclaw message send --channel discord \
                -t "$DISCORD_CHANNEL" \
                --media "$preview" \
                -m "chunk ${chunk_num}/${TOTAL} — \`$seg_name\`" \
            && echo "[watcher] sent chunk $chunk_num"

        echo "$seg" >> "$SEEN_FILE"
        rm -f "$preview"
    done

    sleep 5
done
