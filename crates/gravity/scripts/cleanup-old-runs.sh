#!/bin/bash
# cleanup-old-runs.sh — delete leftover frames/segments/audio from old runs.
#
# Called after a render completes (or is killed) with the current RUN_ID.
# Finds all run directories whose timestamp is OLDER than the current run
# and removes their intermediate files: frames (PNGs), segments (MP4s),
# audio (PCM), and temp files. Never deletes run_info.txt or final MP4s.
#
# Usage: cleanup-old-runs.sh <RUN_ID>
#   e.g. cleanup-old-runs.sh 20260222_165137_nudge_up
#
# The timestamp prefix (YYYYMMDD_HHMMSS) is used for comparison, so any
# run whose ID sorts lexicographically before the current one is considered
# older. This means a brand-new run that just started won't be touched
# as long as its timestamp is >= the current run's timestamp.

set -euo pipefail
cd "$(dirname "$0")/.."

CURRENT_RUN_ID="${1:?Usage: $0 <RUN_ID>}"
RUNS_DIR="runs"

# Extract the timestamp prefix (first 15 chars: YYYYMMDD_HHMMSS)
CURRENT_TS="${CURRENT_RUN_ID:0:15}"

echo "[cleanup] starting post-run cleanup (current=$CURRENT_RUN_ID, ts=$CURRENT_TS)"

cleaned_runs=0
cleaned_files=0

for run_dir in "$RUNS_DIR"/*/; do
    [ -d "$run_dir" ] || continue
    dir_name=$(basename "$run_dir")

    # Skip if not a timestamped run dir
    [[ "$dir_name" =~ ^[0-9]{8}_[0-9]{6} ]] || continue

    dir_ts="${dir_name:0:15}"

    # Skip current run and any newer runs (protect concurrent/future runs)
    [[ "$dir_ts" < "$CURRENT_TS" ]] || continue

    found_something=0

    # Delete frame PNGs
    if [ -d "$run_dir/frames" ]; then
        png_count=$(find "$run_dir/frames" -name "*.png" -type f 2>/dev/null | wc -l | tr -d ' ')
        if [ "$png_count" -gt 0 ]; then
            find "$run_dir/frames" -name "*.png" -type f -delete
            echo "[cleanup]   $dir_name: deleted $png_count frame PNGs"
            cleaned_files=$(( cleaned_files + png_count ))
            found_something=1
        fi
    fi

    # Delete segment MP4s
    if [ -d "$run_dir/segments" ]; then
        seg_count=$(find "$run_dir/segments" -name "*.mp4" -type f 2>/dev/null | wc -l | tr -d ' ')
        if [ "$seg_count" -gt 0 ]; then
            find "$run_dir/segments" -name "*.mp4" -type f -delete
            echo "[cleanup]   $dir_name: deleted $seg_count segment MP4s"
            cleaned_files=$(( cleaned_files + seg_count ))
            found_something=1
        fi
    fi

    # Delete audio PCM and temp files
    misc_count=$(find "$run_dir" -maxdepth 2 \( -name "*.pcm" -o -name "*.mp4.tmp" -o -name "*.muxed.mp4" \) -type f 2>/dev/null | wc -l | tr -d ' ')
    if [ "$misc_count" -gt 0 ]; then
        find "$run_dir" -maxdepth 2 \( -name "*.pcm" -o -name "*.mp4.tmp" -o -name "*.muxed.mp4" \) -type f -delete
        echo "[cleanup]   $dir_name: deleted $misc_count audio/temp files"
        cleaned_files=$(( cleaned_files + misc_count ))
        found_something=1
    fi

    [ "$found_something" -eq 1 ] && cleaned_runs=$(( cleaned_runs + 1 ))
done

if [ "$cleaned_runs" -eq 0 ]; then
    echo "[cleanup] nothing to clean — all older runs already tidy"
else
    echo "[cleanup] done: cleaned $cleaned_runs old run(s), $cleaned_files files total"
fi
