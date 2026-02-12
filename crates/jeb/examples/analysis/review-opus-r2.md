# Opus Review Round 2 of DESIGN-CONSTRAINTS.md

## Key Issues

### Clarity
- §1 Mental Model too thin — needs the mechanical picture (4 bytes → 5 chars, big-endian u32, base-85 MSB-first)
- §5 "Stable (%)" column header is ambiguous — rename to "% of known-byte values where leading chars fully determined"
- §9 ÷64 table is opaque — needs one sentence explaining where 64 comes from
- §6 title says "symmetry" but conclusion is asymmetry — misleading

### Contradictions
- §6 exit example: `V mod 85 = (b0+b1+b2+b3) mod 85` is WRONG for general Z85. Only true because 2^8 ≡ 1 (mod 85). Must flag this as a derived identity, not how Z85 works.
- §6 exit stability table missing stability percentages (entry has them prominently)
- §6 "Recommended Defaults" mixes tight-budget-specific and general rationale
- §2 P1 "exact same positions" then immediately "exception" reads as retraction

### Missing Assumptions
- Never states whether multiple raw sections per stream are supported
- Z85's encoding direction (big-endian, MSB-first) should be in §0
- Raw bytes assumed to be literal 1:1 in output — never stated
- **What bytes are valid in raw sections?** If any byte 0x00-0xFF is valid, output isn't printable ASCII. Fundamental gap.
- §9 "2-block case" doesn't precisely define what "straddling 2 blocks" means

### Missing Analysis
- No analysis of length encoding schemes (direct byte count? block count? logarithmic?)
- No fallback analysis: when disambiguation fails and encoder can't afford it
- Length × cut position interaction: some combos may be impossible
- Overhead scaling for longer sections (fixed or grows with length?)
- Error behavior not even acknowledged as deferred

### Minor
- §4 double-negative logic ("YES" = already broken = free) takes a beat to parse
- §12 feels orphaned — connect it or cut it
- "info bits" / "disambiguation bits" terminology inconsistent
- §3 "~5 characters" should be derivable from position invariant, not a target
