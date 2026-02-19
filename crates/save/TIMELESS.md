# Save: Timeless Timestamp Design

## Origin

The `commit` bash script (recovered from history-pit, circa 2021-2022) had this
logic:

```bash
declare -i tick=256
declare -i step=16384
declare -i drift=32768

declare -i parent_timestamp=0
for parent in $parents; do
    this_parent_timestamp="$(git log --format=%ct -1 "$parent")"
    if ((this_parent_timestamp > parent_timestamp)); then
        parent_timestamp=$this_parent_timestamp
    fi
done

current_timestamp="$(date +%s)"

if ((current_timestamp - parent_timestamp > drift)); then
    timestamp=$((current_timestamp - (current_timestamp % step)))
else
    timestamp=$((parent_timestamp + tick - (parent_timestamp % tick)))
fi
```

This is the behaviour we want to model `--timeless` on.

## Semantics

- **tick**: minimum increment from parent (snap parent timestamp up to next
  multiple of tick)
- **step**: large rounding unit for "current time" snapshots
- **drift**: maximum gap from parent before we snap to current time instead

### Decision logic

```
if (now - max_parent_timestamp) > drift:
    # We're far enough from parent that wall clock makes sense.
    # But still round down to nearest `step` boundary.
    timestamp = now - (now % step)
else:
    # We're close to parent. Don't use wall clock.
    # Snap parent timestamp up to next `tick` boundary.
    timestamp = parent_timestamp + tick - (parent_timestamp % tick)
```

This means timestamps are:
- Always a multiple of `tick`
- Either pinned close to parent (rounded up) or to current wall clock (rounded
  down to `step`)
- Never arbitrary/millisecond-precise
- Deterministic given same inputs

## Relationship to tree-hash target

**Orthogonal.** The timeless logic sets the *starting point* for brute-force
timestamp search. The tree-hash (or generation-index) *target* for the commit
hash prefix is a separate concern. Both can be enabled independently:

- `--timeless` only → deterministic timestamp baseline, normal hash target
- `--tree-target` only → wall-clock timestamp, brute-force toward tree hash
- Both → deterministic timestamp + tree hash target
- Neither → wall clock + generation-index target (current default)

## Suggested parameter values for jeb monorepo

From the old shell script (the "precise" variant):
- `tick = 256`  (~ 4 min 16 sec)
- `step = 16384` (~ 4 hours 33 min)
- `drift = 32768` (~ 9 hours 6 min)

From the "loose" variant in the same script:
- `tick = 32`, `step = 64`, `drift = 128`  (very tight, seconds-scale)

The 256/16384/32768 values seem like the meaningful ones for a real repo.

## Implementation plan

### 1. CLI flags / env vars

```
--timeless              SAVE_TIMELESS=1
--tick <N>              SAVE_TICK=256
--step <N>              SAVE_STEP=16384
--drift <N>             SAVE_DRIFT=32768
```

`--timeless` enables the behaviour; `--tick/--step/--drift` tune it (defaults
as above when `--timeless` is set).

### 2. `.env` / dotenv file support

All `save` options that support env vars should also be loadable from a
`.save.env` (or `.env`) file in the repo root. Standard dotenv format:

```
SAVE_TIMELESS=1
SAVE_TICK=256
SAVE_STEP=16384
SAVE_DRIFT=32768
```

Priority: CLI args > process env vars > `.save.env` file > defaults.

The dotenv file lets projects configure `save` behaviour without modifying shell
profiles or CI environment — just commit a `.save.env` to the repo root.

### 3. Enable in jeb monorepo (experimental)

Add a `.save.env` to `/Users/matte/jeb/` with the timeless values to experiment
with this behaviour in the actual repo.

### 4. Rust implementation sketch

```rust
fn compute_timeless_timestamp(
    parent_timestamp: i64,
    now: i64,
    tick: i64,
    step: i64,
    drift: i64,
) -> i64 {
    if now - parent_timestamp > drift {
        now - (now % step)
    } else {
        parent_timestamp + tick - (parent_timestamp % tick)
    }
}
```

This replaces `target_timestamp = seconds` in `cli.rs` when `--timeless` is
active. The `min_timestamp` constraint (must be ≥ parent) is already enforced
by `brute_force_timestamps`.

## Notes

- The `--timeless` flag already exists in `cli.rs` (`pub timeless: bool`) but
  is unimplemented — just needs the logic wired in.
- All existing `save` options already support env vars via clap's `env =
  "SAVE_..."` attribute — dotenv loading just needs a pre-pass to read the file
  and populate `std::env` before clap parses.
