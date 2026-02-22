
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
