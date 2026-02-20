#!/bin/bash
# run-loop.sh — run gravity continuously, incrementing seed each time
# Usage: ./run-loop.sh [start_seed] [seconds]
set -e
cd "$(dirname "$0")"

SEED=${1:-45}
SECONDS_PER_RUN=${2:-4096}  # 64*64 = 4096s

while true; do
    echo "=== Starting run seed=$SEED, ${SECONDS_PER_RUN}s ==="
    rm -f segments.txt state/checkpoint.bin
    rm -f segments/*.mp4 2>/dev/null || true

    ./target/release/gravity --seconds $SECONDS_PER_RUN --epilogue --seed $SEED

    OUTPUT="gravity_${SECONDS_PER_RUN}s.mp4"
    if [ -f "$OUTPUT" ]; then
        DEST="gravity_${SECONDS_PER_RUN}s_seed${SEED}.mp4"
        mv "$OUTPUT" "$DEST"
        SIZE=$(du -h "$DEST" | cut -f1)
        echo "=== Done: $DEST ($SIZE) ==="
    fi

    SEED=$((SEED + 1))
done
