
## 2026-02-21 — threads + clumping (interesting landscape point)
```
--gravity 0.15 --softening 4 --speed-cap 4
--pop-target 1536 --pop-band 192 --rate-limit 16
--circles 15 --vel-scale 2.0 --init-vel spin
--wrap --dampen-y 0.0625
```
Produces threads + clumping. Some loose orbital strands with clusters
that jam/fracture. Not the final target but a solid point in the
parameter landscape — good cascade energy, multiple structures coexist.
Notable fixes in this session:
- pop_band now hard cutoff (not gradient) so population actually floats
- rate_limit defaults to pop_band
- spin-flat 8× flatten was too far (lines not orbits); reverted to spin

## 2026-02-21 — ⭐ GREAT — rate_limit=64 breakthrough
```
--gravity 0.15 --softening 4 --speed-cap 4
--pop-target 1536 --pop-band 192 --rate-limit 64
--circles 15 --vel-scale 2.0 --init-vel spin
--wrap --dampen-y 0.0625
```
Jeremy: "Whoa save that save that wow wow wow so good"
Key insight: rate_limit=64 (vs 16) breaks up crystalline jams —
enough Conway churn to eat into the core of jammed structures,
not just nibble the surface. Creates cascade-collapse dynamics
with active internal motion and threading. ⭐ KEEPER.
run_id: 20260221_235720 seed: 4065649357

## 2026-02-22 — 3am params (long-render baseline)
```
--gravity 0.03125 --softening 6 --speed-cap 4.5
--pop-target 5120 --pop-band 160 --rate-limit 4
--init-vel zero --wrap
```
The config used for the long 73-min production render. Very gentle gravity,
high softening, slow Conway rate. Population is large (5120) and band is
narrow (160) so Conway rarely fires. Creates slow, organic blob drift.
run_id: 20260222_052051

## 2026-02-22 — spin x1 (single circle, centered)
```
--width 256 --height 160
--gravity 0.165 --softening 3.5 --speed-cap 4.9
--pop-target 1536 --pop-band 192 --rate-limit 64
--circles 1 --vel-scale 1 --init-vel spin
--wrap --dampen-y 0.0625
--vel-nudge-rate 0.03125
```
Standard 64s test config. One circle, CCW spin, mild Y damping.
test runs: 20260222_114715, 20260222_121634

## 2026-02-22 — spin x1 + dampen-x
```
--width 256 --height 160
--gravity 0.165 --softening 3.5 --speed-cap 4.9
--pop-target 1536 --pop-band 192 --rate-limit 64
--circles 1 --vel-scale 1 --init-vel spin
--wrap --dampen-x 0.015625 --dampen-y 0.0625
--vel-nudge-rate 0.03125
```
Adds horizontal damping (¼ of Y value) to spin x1. Test to see if
X damping calms horizontal drift without killing the spin energy.
test run: 20260222_122143

## 2026-02-22 — radial-out x1
```
--width 256 --height 160
--gravity 0.165 --softening 3.5 --speed-cap 4.9
--pop-target 1536 --pop-band 192 --rate-limit 64
--circles 1 --vel-scale 1 --init-vel radial-out
--wrap --dampen-y 0.0625
--vel-nudge-rate 0.03125
```
Cells start moving outward from center at normal speed.
test run: 20260222_120853

## 2026-02-22 — radial-out x2
```
--width 256 --height 160
--gravity 0.165 --softening 3.5 --speed-cap 4.9
--pop-target 1536 --pop-band 192 --rate-limit 64
--circles 1 --vel-scale 2 --init-vel radial-out
--wrap --dampen-y 0.0625
--vel-nudge-rate 0.03125
```
Radial-out with 2× velocity scale — faster initial explosion.
test run: 20260222_121317_x2

## 2026-02-22 — radial-out x4
```
--width 256 --height 160
--gravity 0.165 --softening 3.5 --speed-cap 4.9
--pop-target 1536 --pop-band 192 --rate-limit 64
--circles 1 --vel-scale 4 --init-vel radial-out
--wrap --dampen-y 0.0625
--vel-nudge-rate 0.03125
```
Radial-out with 4× velocity scale — very fast initial explosion, cells
mostly capped immediately. Test: does shockwave create interesting patterns?
test run: 20260222_121317_x4
