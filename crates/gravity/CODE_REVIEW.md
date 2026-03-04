# Code Review: Gravity Particle Simulation

**Reviewer:** Claude Opus 4.5
**Date:** 2026-02-22
**File:** `src/main.rs` (~2325 lines)

---

## Architecture Overview

This is a gravity-driven particle simulation combining three distinct systems:

1. **Barnes-Hut N-body Gravity** (lines 207-343): O(n log n) force calculation using a quadtree with θ=0.1 opening angle. Supports toroidal wrapping via `min_image` distance correction.

2. **Conway's Game of Life** (lines 729-878): B3/S23 rules with rate-limited births/deaths. Maintains population within a band around a target. Dead cells transfer momentum to neighbors.

3. **Spatial Audio Synthesis** (lines 1356-1439): 9-voice drone system (3×3 grid), with Schroeder reverb (4 comb + 2 allpass filters). Voices respond to regional cell density, speed, and center-of-gravity.

The simulation renders to PNG frames, encodes chunks via ffmpeg, and produces a final concatenated video with muxed audio.

---

## Notable Strengths

### 1. Principled Color Science (lines 1573-1831)
The Oklab colorspace implementation is exemplary:
- Correct sRGB → linear → Oklab transforms
- Hue blending via unit-vector mean in LCH (line 1698-1701) — avoids neutral desaturation
- Gamut mapping via binary search chroma reduction (lines 1817-1827) — preserves hue exactly
- Position-based hue rotation creates visual interest without breaking perceptual uniformity

### 2. Robust Checkpoint/Resume (lines 598-688)
The checkpoint system is well-designed:
- Binary format with explicit field layout
- Separate original-state persistence for epilogue convergence (lines 693-727)
- Graceful SIGINT handling discards partial chunks, keeps completed work (lines 2144-2151)

### 3. Barnes-Hut Implementation (lines 207-343)
Clean quadtree with proper recursion depth guard (line 241). The square root node (lines 893-898) fixes directional bias in the opening-angle criterion. The θ=0.1 choice prioritizes accuracy over speed, appropriate for visual fidelity.

### 4. Velocity Inheritance Physics (lines 855-866)
The distinction between wrap-mode (inherits avg neighbor velocity) and no-wrap mode (born at rest) is physically motivated and prevents wall-facing momentum injection.

---

## Design Concerns

### 1. Monolithic File Structure
2325 lines in a single file is unwieldy. Natural module boundaries exist:

| Candidate Module | Lines | Purpose |
|------------------|-------|---------|
| `color.rs` | 1573-1831 | Oklab, palette, gamut mapping |
| `audio.rs` | 17-166, 1356-1439 | Reverb, voices, synthesis |
| `quadtree.rs` | 207-343 | Barnes-Hut implementation |
| `conway.rs` | 729-878, 1226-1324 | Life rules, deaths, births |
| `checkpoint.rs` | 598-727 | Serialization/deserialization |

The current structure makes local reasoning difficult and increases merge conflict surface.

### 2. Static Grid Dimensions (lines 6-12)
```rust
static W_CELL: OnceLock<usize> = OnceLock::new();
static H_CELL: OnceLock<usize> = OnceLock::new();
```
Using `OnceLock` globals for dimensions works but creates implicit coupling. Every function that touches spatial data depends on these being initialized. A `GridConfig` struct passed through the call chain would be more explicit.

### 3. Argument Parsing Boilerplate (lines 1944-2043)
50+ lines of manual argument parsing when `clap` would provide:
- Type inference and validation
- Auto-generated `--help`
- Consistent error messages
- No off-by-one risks in positional access

### 4. God Struct (Sim, lines 168-197)
The `Sim` struct holds 25+ fields mixing physics state, audio state, Conway state, and configuration. Consider splitting:
- `PhysicsState` (cells, velocities, bounds)
- `ConwayState` (tick_count, prev_live, births/deaths)
- `AudioState` (region_stats, region_voices, reverb)
- `Config` (g, softening, speed_cap, wrap_*, etc.)

---

## Potential Bugs and Edge Cases

### 1. Integer Division Drift (line 19)
```rust
const SAMPLES_PER_FRAME: usize = 735; // 44100 / 60, truncated
```
At 60fps: `735 * 60 = 44100` — exact. But the comment says "truncated (acceptable drift)". This is actually exact; the comment is misleading. However, if FPS changes, this will desync audio and video.

**Fix:** Compute `SAMPLES_PER_FRAME` from `SAMPLE_RATE / FPS` with proper rounding, or document the FPS constraint.

### 2. Collision Ordering Bias (lines 990-993)
```rust
for i in (1..n).rev() {
    let j = (xoru64(&mut self.rng) as usize) % (i + 1);
    self.order.swap(i, j);
}
```
This Fisher-Yates shuffle is correct, but the same RNG feeds both simulation and shuffle. In deterministic replay scenarios, the shuffle order affects physics outcomes. Consider a separate RNG for non-physics randomness.

### 3. Wrap + Bounce Conflict (lines 1007-1022)
The logic allows both `wrap_x` and `bounce_x` to be set, but wrap is checked first:
```rust
if self.wrap_x {
    nx = nx.rem_euclid(W() as f32);
} else if self.bounce_x { ... }
```
If both are true, bounce is silently ignored. The CLI doesn't prevent this combination. Add validation or document mutual exclusivity.

### 4. Float-to-Index Truncation (lines 93-94)
```rust
fn gx(&self) -> usize { (self.px.floor() as i32).rem_euclid(W() as i32) as usize }
```
If `px` is negative (possible during bounce reflection before clamping), `floor()` gives a more negative value. The `rem_euclid` handles this, but the chain of casts `f32 → i32 → usize` is fragile. Consider:
```rust
fn gx(&self) -> usize { self.px.rem_euclid(W() as f32).floor() as usize }
```

### 5. Empty Palette Anchor (line 1596)
```rust
(0xF6, 0xF9, 0xFC), // #F6F9FC — near white (C < 0.02, skipped from wheel)
```
This color is listed in `PALETTE_SRGB` but documented as "skipped from wheel". It's never actually used — `DirectionalPalette::build` hardcodes the 4 directional colors plus dark anchor. Dead constant.

### 6. Unbounded Quadtree Growth (line 241)
```rust
if depth > 64 { return; } // safety: coincident particles
```
For coincident particles, this silently drops the duplicate from the tree. The force calculation will then miss the gravitational contribution. Consider instead treating coincident particles as a single point mass with combined mass, or jittering positions by epsilon.

---

## Performance Considerations

### 1. Repeated Grid Allocation (lines 737, 831, 985, 1093, etc.)
```rust
let mut grid = vec![usize::MAX; W() * H()];
```
This allocates a new 40KB+ vector (256×160 × 8 bytes) multiple times per tick. Use a persistent scratch buffer in `Sim` and `fill()` it instead.

### 2. Quadtree Rebuilds Every Frame (lines 892-903)
The tree is built from scratch each tick. For mostly-static configurations, incremental updates (insert/remove changed particles) would save work. Likely premature optimization given current scale, but worth noting for larger populations.

### 3. Audio Vec Growth (line 2167)
```rust
let mut chunk_audio: Vec<f32> = Vec::with_capacity(SAMPLES_PER_FRAME * this_chunk_frames * 2);
```
Good — pre-allocated. But `generate_audio` pushes samples individually (lines 1433-1434). Consider writing to a mutable slice instead of pushing.

### 4. PNG Encoding Overhead
Each frame writes a PNG via the `png` crate with default compression. For intermediate frames (deleted after ffmpeg encoding), consider:
- Raw RGB files (ffmpeg accepts them)
- Lower compression level (`CompressionType::Fast`)
- Or pipe directly to ffmpeg stdin

### 5. Cell Vec Reshuffling (line 586)
```rust
shuffle_vec(&mut cells, &mut rng);
```
Shuffling cells on init is fine, but `conway_step` also shuffles at line 731. If the order doesn't matter after Conway, this work is wasted.

---

## Specific Improvement Suggestions

### High Priority

**[H1]** Extract modules (see Design Concerns §1). Start with `color.rs` — it's self-contained.

**[H2]** Replace manual arg parsing with `clap`. Example:
```rust
#[derive(Parser)]
struct Args {
    #[arg(long)]
    seconds: usize,
    #[arg(long, default_value_t = 256)]
    width: usize,
    // ...
}
```

**[H3]** Add `#[must_use]` to pure functions like `rgb_to_oklab`, `oklab_to_srgb`, `velocity_color_oklab` to catch ignored returns.

### Medium Priority

**[M1]** (line 1719) Remove file-based palette hot-reload or document it properly. Magic files in `/tmp/` are surprising.

**[M2]** (lines 921-926) The speed cap logic is convoluted:
```rust
let effective_cap = c.prev_speed.max(self.speed_cap);
if spd > effective_cap { /* clamp */ }
let hard_ceil = self.speed_cap * 2.0;
c.prev_speed = c.prev_speed.min(spd).max(self.speed_cap).min(hard_ceil);
```
The interaction between `prev_speed`, `effective_cap`, and `hard_ceil` is hard to follow. Add a comment block explaining the intended behavior (hysteresis? runaway prevention?).

**[M3]** (lines 928-931) Per-axis speed clamping after isotropic clamping is redundant if `gx_scale` and `gy_scale` are both 1.0. Guard the block:
```rust
if gx_scale < 1.0 || gy_scale < 1.0 {
    c.vx = c.vx.clamp(-self.speed_cap * gx_scale, self.speed_cap * gx_scale);
    // ...
}
```

**[M4]** (line 1494-1497) The empty-sim stats string is manually constructed. Consider returning an `Option<Stats>` struct and formatting only when populated.

### Low Priority / Style

**[L1]** (line 105) `CombFilter`, `AllPass` could derive `Clone` for potential future parallelism in reverb.

**[L2]** (lines 968-970) The angle wrapping loop:
```rust
while dh >  std::f32::consts::PI { dh -= std::f32::consts::TAU; }
while dh < -std::f32::consts::PI { dh += std::f32::consts::TAU; }
```
Replace with `dh = dh.rem_euclid(TAU) - PI` or a helper function. The `while` loops technically handle arbitrarily large inputs but suggest uncertainty about input bounds.

**[L3]** Consistent naming: `prev_speed` vs `prev_live` — both are "previous frame" state but naming doesn't convey this relationship.

**[L4]** (line 2074-2082) The settings string uses inconsistent field widths (`pop_target:    ` vs `vel_nudge_rate:`). Minor cosmetic issue.

---

## Summary

This is a well-engineered creative coding project with solid physics, excellent color science, and thoughtful checkpoint handling. The main weaknesses are structural: a monolithic file and a god struct make the code harder to navigate and extend than necessary. The physics and audio systems are tightly coupled in ways that would complicate testing them in isolation.

**Risk assessment:**
- **Correctness:** Low risk. Edge cases are handled, numerics are stable.
- **Performance:** Adequate for current scale (~1000 cells). Would need optimization for 10k+.
- **Maintainability:** Medium risk. New contributors will struggle with the flat structure.

**Recommended next steps:**
1. Extract `color.rs` module (1 hour)
2. Switch to `clap` for args (30 min)
3. Document the speed cap / prev_speed interaction (15 min)
4. Add integration test: run 10 frames, assert pop within band, no NaN positions
