#!/bin/bash
# run-loop.sh — run gravity continuously, incrementing seed each time
# Usage: ./run-loop.sh [start_seed] [seconds]
set -e
cd "$(dirname "$0")"

SEED=${1:-44}
SECONDS_PER_RUN=${2:-4096}  # 64*64 = 4096s

while true; do
    echo "=== Starting run seed=$SEED, ${SECONDS_PER_RUN}s ==="

    # Patch seed in binary via env var (we'll add --seed flag instead)
    # Clean state
    rm -f segments.txt state/checkpoint.bin
    rm -f segments/*.mp4 2>/dev/null || true

    cargo run --release -- --seconds $SECONDS_PER_RUN --epilogue --seed $SEED

    # Rename output to include seed
    if [ -f "gravity_${SECONDS_PER_RUN}s.mp4" ]; then
        mv "gravity_${SECONDS_PER_RUN}s.mp4" "gravity_${SECONDS_PER_RUN}s_seed${SEED}.mp4"
        echo "=== Run seed=$SEED complete: gravity_${SECONDS_PER_RUN}s_seed${SEED}.mp4 ==="
    fi

    SEED=$((SEED + 1))
done
