# Opus Review of DESIGN-CONSTRAINTS.md

*Fresh-eyes clarity review. Reviewer was given no code access, told the doc is in-progress design space exploration.*

## 1. Clarity to a Fresh Reader

- §1 Mental Model — needs one sentence of motivation (why raw passthrough? "save ~20% overhead for printable regions")
- §2 P1 — "slightly shorter" is vague. Either give formula or say "depends on raw section length and alignment"
- §5 — "72 out of 82" is unexplained. 85 possible c0 values exist, where did the other 3 go? Math doesn't check out as stated
- §6 "When No Budget to Signal Endianness" title — fresh reader hasn't internalized escape-signaling yet. Simplify title.
- §7 budget formula (`⌈N×5/4⌉ - N`) is never stated explicitly

## 2. Contradictions

- §3 lookahead: "should never need more than ~1 block" then "guideline, not hard limit" — contradicts itself
- §5 vs §6: §5 doesn't specify entry-only, reader could think it applies to both directions
- §6 "Rejected Alternative" table: compares chars and info-bits in same column (incommensurable units)
- §8 "1 escape char = 0 info bits": really means log2(num_escape_chars), should state formula

## 3. Implicit Assumptions

- Encoder is smart, decoder is dumb — never stated as principle
- "Block-aligned" means relative to stream start, not raw section — undefined
- Position invariant applies to Z85 blocks that remain Z85-encoded — could confuse fresh reader
- Byte order within raw sections: raw bytes are sequential? Never stated.
- Decoder must know it's reading extended vs standard Z85 — out-of-band or self-signaling?
- Stability percentages are about "what fraction of positions are usable" not "what fraction will be correct" — different things

## 4. Missing Analysis

- **Break-even analysis:** At 4 bytes, raw uses 4+1 escape = 5 chars, same as Z85. Zero savings. When does it actually help?
- **Stream boundaries:** What happens at start/end of input with no preceding/following Z85 block?
- **Consecutive raw sections:** Can they be adjacent? What separates them?
- **Escape char position within block:** Fixed position or floating? Affects decoder logic.
- **Exit stability for 2-3 bytes:** Only 1-byte exit analyzed, 2-3 byte cases have "—" 
- **Information flow:** Can escape char bits actually carry disambiguation? Depends on whether decoder sees escape before or after it needs the disambig. Sequencing constraint.

## Minor Issues

- §11 zipng URL points to nickel-org, probably wrong
- "chars" vs "characters" inconsistent
- §4 table has `\|` markdown artifact
- Budget=1 case packs a lot of meaning into 0 info bits — worth flagging
