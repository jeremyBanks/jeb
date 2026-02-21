#!/bin/bash
# segment-watcher.sh — sends a Discord preview clip whenever a new segment completes
# Usage: ./segment-watcher.sh
set -e
cd "$(dirname "$0")"

DISCORD_CHANNEL="1467063568712339561"
SEEN_FILE="/tmp/gravity_segments_seen.txt"
touch "$SEEN_FILE"

echo "[watcher] started, watching segments/"

while true; do
    # Find all completed segment files, sorted
    for seg in $(ls segments/seg_*.mp4 2>/dev/null | sort); do
        if grep -qF "$seg" "$SEEN_FILE" 2>/dev/null; then
            continue
        fi

        # Wait a moment to ensure file is fully written
        sleep 2

        # Double-check it still exists
        [ -f "$seg" ] || continue

        # Extract segment number from filename
        seg_name=$(basename "$seg" .mp4)
        seg_num=$(echo "$seg_name" | sed 's/seg_//' | sed 's/^0*//')
        seg_num=${seg_num:-0}
        chunk_num=$((seg_num / 3840))

        # Make a short preview (first 15s, half res)
        preview="/tmp/preview_chunk${chunk_num}.mp4"
        ffmpeg -y -i "$seg" -t 15 -vf scale=960:600 -c:v libx264 -crf 22 -preset fast "$preview" 2>/dev/null

        # Send to Discord
        openclaw message send --channel discord \
            -t "$DISCORD_CHANNEL" \
            --media "$preview" \
            -m "chunk $((chunk_num + 1))/69 done — seg \`$seg_name\`"

        echo "[watcher] sent preview for $seg_name (chunk $((chunk_num + 1))/69)"
        echo "$seg" >> "$SEEN_FILE"
        rm -f "$preview"
    done

    sleep 5
done
