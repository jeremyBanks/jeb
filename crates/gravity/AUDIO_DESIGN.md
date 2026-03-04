# Gravity Audio System — Design Draft

*Working notes from design discussion, Feb 21–22 2026.*

## Current system (directional buckets)

8 persistent voices indexed by velocity direction (N/NE/E/SE/S/SW/W/NW via `atan2`).
Each voice's amplitude/pan/reverb driven by aggregate stats of cells moving in that direction.

**Problem:** In near-steady-state renders — where most cells are moving at roughly the same
speed in similar directions — the directional voices lock into static levels and drone.
There's real variation (wave texture moving through, population oscillating ±10%) but the
directional bucketing doesn't capture it.

---

## Proposed system: 9 spatial regions (3×3)

### Core idea

Divide the canvas into a 3×3 grid. Each region gets a persistent voice driven by three
smoothed values: **population**, **CoG-X**, **CoG-Y** (centre of gravity of cells within
the region). All three run exponential moving averages (EMA) so per-tick noise is smoothed
but slow variation comes through.

### Smoothing

EMA time constant: roughly **1/16 second ≈ 4 ticks at 60fps** (α ≈ 0.25).

```rust
cog_x_smooth += (cog_x_raw - cog_x_smooth) * 0.25;
```

- CoG and population use this timescale (~4 ticks)
- Amplitude slewing is *lighter* (shorter time constant) so population oscillations
  produce real rhythm rather than being washed out
- Exact values TBD in implementation

### Parameter mapping

| Source | Controls | Notes |
|---|---|---|
| Population (smoothed) | **Amplitude** | Hard gate below threshold — no droning from near-silence |
| CoG-X within region | **Pitch bend** ±1 semitone | Gives "aliveness" as wave texture drifts through |
| CoG-Y within region | **Filter warmth** | CoG low (dense/settled) = warm; CoG high (sparse/active) = brighter |
| Column (0/1/2) | **Pan** | Mild: −0.3 / 0 / +0.3. Informational, never critical |
| Row (0/1/2) | **Reverb send** | Top row = airier; bottom row = dryer/grounded |

### Pitch layout — 3×3 grid

C major pentatonic across ~2 octaves, centered in the warm-bright register (~220–660 Hz).
Nothing harsh, nothing muddy.

```
Top row:    A4 (440)   C5 (523)   E5 (659)
Middle row: E4 (330)   G4 (392)   A4 (440)   ← A4 repeats, needs adjustment
Bottom row: A3 (220)   C4 (262)   E4 (330)   ← E4 repeats
```

Spatial logic: top = higher pitch (lighter), bottom = lower pitch (grounded).
Left/right symmetric — horizontal position affects pan not pitch.
Exact pitch grid to be finalised — need 9 unique frequencies in the target range.

**Candidate grid (no repeats, pentatonic-ish):**
```
Top:    B4 (494)   D5 (587)   G5 (784)   ← G5 may be too bright
Mid:    G4 (392)   A4 (440)   C5 (523)
Bot:    C4 (262)   E4 (330)   G4 (392)   ← G4 repeats
```
*Still working this out — the constraint is 9 unique warm-to-bright pitches.*

### Rhythm

Not engineered — emergent. Population in each region oscillates naturally (~1–2 Hz) as the
wave texture moves through. With lighter amplitude slewing, these oscillations come through
as real beats. No special rhythm synthesis needed.

### Droning fix

Two mechanisms:
1. **Hard amplitude gate** — voices below a population threshold go fully silent rather
   than sustaining at low level with reverb tails
2. **Lighter amplitude slewing** — voices respond faster to population changes, don't hold
   artificially after activity drops

---

## Open questions

- **Exact pitch grid** — 9 unique pitches, no repeats, ~220–660 Hz range, musically coherent
- **CoG-X → pitch bend range** — ±0.5 semitone? ±1 semitone? needs tuning
- **CoG-Y → filter** — what cutoff range? (e.g. 400–2000 Hz as CoG moves top to bottom)
- **Amplitude slew rate** — how much lighter than CoG slew? (maybe α=0.4 for amplitude vs 0.25 for CoG)
- **Pan range** — ±0.3 feels right but may want ±0.4
- **Reverb range** — top row at what send level vs bottom row?
- **Synthesis** — current oscillator + LP filter + reverb sufficient? or change timbre?

---

## What this captures that current system misses

- **Where** activity is on screen (current system only captures *how* cells are moving)
- **Wave texture moving through** — CoG drifts as pattern moves → pitch/filter breathe
- **Natural rhythm** — population oscillation gives organic beat without engineering it
- **Spatial grounding** — close eyes and roughly know where clusters are from stereo + pitch height
