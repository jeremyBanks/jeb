# Z85 Entry vs Exit Boundary Analysis — Empirical Summary

## Entry Boundaries (Leading Characters)

Entry boundaries emit partial leading Z85 characters for known bytes before a raw section.

### Stability Rates (Empirically Verified)

| Known Bytes | Chars Emitted | Stability | Verified |
|------------|---------------|-----------|----------|
| 1 byte (b0) | 1 leading char | **68.0%** | ✓ 174/256 |
| 2 bytes (b0,b1) | 2 leading chars | **89.3%** | ✓ 58,543/65,536 |
| 3 bytes (b0,b1,b2) | 3 leading chars | **96.5%** | ✓ 16,185,079/16,777,216 |

### How It Works

- **Stable case:** The known bytes uniquely determine the leading characters, regardless of unknown bytes
- **Unstable case:** Different unknown byte values produce different leading characters
  - Requires ~2 bits/byte disambiguation from escape character info budget
  - Or fall back to different cut position

### Mathematical Basis

Stability depends on whether the byte range crosses an 85^4 (or 85^3, 85^2) boundary:

- 1-byte: b0's range [b0×2^24, (b0+1)×2^24 - 1] crosses boundary when floor differs
  - Range width: 2^24 = 16,777,216
  - Boundary spacing: 85^4 = 52,200,625
  - Critical ratio: 85^4 / 2^24 ≈ 3.111 → every ~3rd value crosses
  - Result: 174/256 stable (68.0%)

- 2-byte and 3-byte follow same principle with finer granularity
  - Smaller ranges → less likely to cross boundaries
  - Result: 89.3% and 96.5% stable

### Practical Implications

**Entry boundaries are highly viable for mid-block cuts:**
- Nearly 7 in 10 single-byte cuts work without disambiguation
- Nearly 9 in 10 two-byte cuts work  
- Over 96% of three-byte cuts work

When unstable, the encoder either:
- Spends ~2 bits/byte from escape budget to disambiguate, OR
- Falls back to different cut position

---

## Exit Boundaries (Trailing Characters)

Exit boundaries emit partial trailing Z85 characters for known bytes after a raw section.

### Opportunistic Zero-Padding Viability

The opportunistic strategy: only allow exit cuts when the actual boundary bytes encode the same trailing characters as if unknown bytes were zeros.

| Known Bytes | Strategy | Viability | Verified |
|------------|----------|-----------|----------|
| 1 byte (b3) | Zero-padding | **~0.02%** | 2/10,000 random blocks |
| 2 bytes (b2,b3) | Zero-padding | **~0.00%** | 0/10,000 random blocks |
| 3 bytes (b1,b2,b3) | Zero-padding | **~0.31%** | 31/10,000 random blocks |

### Why So Rare?

Because `V mod 85 = (b0 + b1 + b2 + b3) mod 85`, the trailing character depends on the **sum of all bytes**, not just the known ones.

For the zero-padding check to pass, encoding `[b0,b1,b2,b3]` must produce the same trailing chars as `[0,0,0,b3]`. This only works when the actual unknown bytes happen to contribute the same residue as zeros would — a very restrictive constraint.

### Practical Implications

**Exit boundaries are rarely viable with opportunistic zero-padding:**
- Only ~0.02% of 1-byte exit scenarios work
- Effectively 0% of 2-byte scenarios
- Slightly better at ~0.31% for 3-byte scenarios

In practice, the encoder will:
- Try zero-padding opportunistically for each potential exit cut
- Usually fail the check and fall back to **block-aligned exit**
- Or extend the raw section to avoid the problematic boundary

---

## Comparison & Design Conclusions

### Entry vs Exit Success Rates

| Boundary Type | 1-byte | 2-byte | 3-byte |
|--------------|--------|--------|--------|
| **Entry** (stable) | 68.0% | 89.3% | 96.5% |
| **Exit** (zero-padding viable) | ~0.02% | ~0.00% | ~0.31% |

**Entry boundaries are 3,400× to 300,000× more viable than exits.**

### Asymmetric Strategy

The empirical data strongly supports asymmetric treatment:

- **Entry boundaries:** Use natural leading-character stability
  - Works for most byte values
  - Disambiguation needed only for ~32% of 1-byte cases (fewer for multi-byte)
  - Practical and effective

- **Exit boundaries:** Opportunistic zero-padding rarely works
  - Only ~0.02-0.31% of cases viable
  - Most exits will be block-aligned or avoided
  - The strategy is theoretically sound but practically limited

### Recommended Encoding Strategy

1. **Prefer entry cuts** — they work for 68%+ of scenarios
2. **Try opportunistic exit cuts** — zero cost when they work, but don't expect them to work often
3. **Fall back to block-aligned** — when neither entry nor exit cuts are viable
4. **Extend raw sections** — if boundaries are problematic, include more bytes as raw

The asymmetry is real and significant. Design should prioritize entry boundary support.

---

## Analysis Scripts

- `entry-stability-empirical.py` — 1-byte entry verification
- `entry-stability-2and3.py` — 2-byte and 3-byte entry verification  
- `zero-padding-exits.py` — Exit boundary viability testing
- `exit-stability.py` — Mathematical exploration of exit instability

All scripts in `/Users/matte/jeb/crates/jeb/examples/analysis/`.
