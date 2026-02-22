#!/bin/bash
# batch-renders.sh — run each interesting config sequentially, watcher active throughout.
# Each render: 64×64×16 = 65536 frames @ 60fps = 1092s (~18 min).
# Previews sent to Discord automatically via segment-watcher.sh.
# Videos auto-archive to shared storage after each concat.
set -euo pipefail
cd "$(dirname "$0")/.."

BIN="/Users/matte/jeb/target/release/gravity"
SECONDS_EACH=1092   # 64*64*16 / 60

COMMIT=$(git rev-parse --short=12 HEAD 2>/dev/null || echo "unknown")

# ── CONFIGS: "label:extra args..." ─────────────────────────────────────────
# Order: run one at a time, in sequence below.
declare -a CONFIGS=(

  # ── 1. threads+clumping ────────────────────────────────────────────────
  "threads_clumping:\
--gravity 0.15 --softening 4 --speed-cap 4 \
--pop-target 1536 --pop-band 192 --rate-limit 16 \
--circles 15 --vel-scale 2.0 --init-vel spin \
--wrap --dampen-y 0.0625 \
--width 256 --height 160"

  # ── 2. GREAT — rate_limit=64 breakthrough ──────────────────────────────
  "great_rate64:\
--gravity 0.15 --softening 4 --speed-cap 4 \
--pop-target 1536 --pop-band 192 --rate-limit 64 \
--circles 15 --vel-scale 2.0 --init-vel spin \
--wrap --dampen-y 0.0625 \
--width 256 --height 160"

  # ── 3. 3am params (long-render baseline) ───────────────────────────────
  "3am_baseline:\
--gravity 0.03125 --softening 6 --speed-cap 4.5 \
--pop-target 5120 --pop-band 160 --rate-limit 4 \
--init-vel zero --wrap \
--width 256 --height 160"

  # ── 4. spin x1 ─────────────────────────────────────────────────────────
  "spin_x1:\
--gravity 0.165 --softening 3.5 --speed-cap 4.9 \
--pop-target 1536 --pop-band 192 --rate-limit 64 \
--circles 1 --vel-scale 1 --init-vel spin \
--wrap --dampen-y 0.0625 \
--vel-nudge-rate 0.03125 \
--width 256 --height 160"

  # ── 5. spin x1 + dampen-x ──────────────────────────────────────────────
  "spin_x1_dmpx:\
--gravity 0.165 --softening 3.5 --speed-cap 4.9 \
--pop-target 1536 --pop-band 192 --rate-limit 64 \
--circles 1 --vel-scale 1 --init-vel spin \
--wrap --dampen-x 0.015625 --dampen-y 0.0625 \
--vel-nudge-rate 0.03125 \
--width 256 --height 160"

  # ── 6. radial-out x1 ───────────────────────────────────────────────────
  "radout_x1:\
--gravity 0.165 --softening 3.5 --speed-cap 4.9 \
--pop-target 1536 --pop-band 192 --rate-limit 64 \
--circles 1 --vel-scale 1 --init-vel radial-out \
--wrap --dampen-y 0.0625 \
--vel-nudge-rate 0.03125 \
--width 256 --height 160"

  # ── 7. radial-out x2 ───────────────────────────────────────────────────
  "radout_x2:\
--gravity 0.165 --softening 3.5 --speed-cap 4.9 \
--pop-target 1536 --pop-band 192 --rate-limit 64 \
--circles 1 --vel-scale 2 --init-vel radial-out \
--wrap --dampen-y 0.0625 \
--vel-nudge-rate 0.03125 \
--width 256 --height 160"

  # ── 8. radial-out x4 ───────────────────────────────────────────────────
  "radout_x4:\
--gravity 0.165 --softening 3.5 --speed-cap 4.9 \
--pop-target 1536 --pop-band 192 --rate-limit 64 \
--circles 1 --vel-scale 4 --init-vel radial-out \
--wrap --dampen-y 0.0625 \
--vel-nudge-rate 0.03125 \
--width 256 --height 160"
)

total=${#CONFIGS[@]}
echo "=== batch-renders.sh | $total configs | ${SECONDS_EACH}s each | commit=$COMMIT ==="
echo ""

run_num=0
for entry in "${CONFIGS[@]}"; do
  (( run_num++ )) || true
  label="${entry%%:*}"
  raw_args="${entry#*:}"
  # shellcheck disable=SC2206
  extra=($raw_args)

  SEED=$(od -An -N4 -tu4 /dev/urandom | tr -d ' ')
  RUN_ID="$(date +%Y%m%d_%H%M%S)_${label}"

  echo "──────────────────────────────────────────────────────"
  echo "[$run_num/$total] $label  run_id=$RUN_ID  seed=$SEED"
  echo "──────────────────────────────────────────────────────"

  # ── Kill any existing non-headless render ──────────────────────────────
  EXISTING=$(pgrep -f "gravity.*--seconds" 2>/dev/null | while read -r pid; do
    ps -p "$pid" -o args= 2>/dev/null | grep -q "\-\-headless" || echo "$pid"
  done || true)
  if [ -n "$EXISTING" ]; then
    echo "[batch] killing existing render(s): $EXISTING"
    kill -9 $EXISTING 2>/dev/null || true
    sleep 3
  fi

  # ── Clean up state ─────────────────────────────────────────────────────
  rm -f state/checkpoint.bin state/orig_state.bin state/run_info.txt state/last_stats.txt
  rm -f segments.txt 2>/dev/null || true

  # ── Launch render in background ────────────────────────────────────────
  export GRAVITY_SHARED_DIR="${GRAVITY_SHARED_DIR:-/Users/matte/.openclaw/workspace/shared/gravity}"
  mkdir -p "$GRAVITY_SHARED_DIR"

  "$BIN" \
    --seed "$SEED" --run-id "$RUN_ID" --commit "$COMMIT" \
    --seconds "$SECONDS_EACH" --epilogue \
    "${extra[@]}" \
    > "/tmp/gravity_render_${label}.log" 2>&1 &
  RENDER_PID=$!
  echo "[batch] render PID=$RENDER_PID"

  # ── Wait for run_info.txt then start watcher ───────────────────────────
  pkill -f "segment-watcher" 2>/dev/null || true
  sleep 1
  for i in $(seq 20); do
    sleep 2
    [ -f "runs/${RUN_ID}/run_info.txt" ] && break
    [ "$i" -eq 20 ] && echo "[batch] WARNING: run_info.txt not found after 40s"
  done
  # Write to state/run_info.txt so watcher can find it
  cp "runs/${RUN_ID}/run_info.txt" state/run_info.txt 2>/dev/null || true

  nohup bash segment-watcher.sh > /tmp/watcher.log 2>&1 &
  WATCHER_PID=$!
  echo "[batch] watcher PID=$WATCHER_PID"

  # ── Wait for render to finish ──────────────────────────────────────────
  echo "[batch] waiting for render to complete..."
  wait "$RENDER_PID" && echo "[batch] ✓ $label done" || echo "[batch] ✗ $label exited non-zero (check /tmp/gravity_render_${label}.log)"

  # ── Stop watcher, short pause before next run ──────────────────────────
  kill "$WATCHER_PID" 2>/dev/null || true
  sleep 5
  echo ""
done

echo "=== All $total renders complete ==="
