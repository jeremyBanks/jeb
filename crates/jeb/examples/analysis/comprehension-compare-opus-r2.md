

I'll analyze both documents systematically. Let me read Document A carefully first to establish the ground truth, then check Document B against it.

## Discrepancy Analysis: Document A vs Document B

### 1. Faithful Restatements (B gets right)

B correctly captures the core concepts: the position invariant, priority ordering, escape character tiering, the entry/exit disambiguation asymmetry, the budget concept, the mod-85 arithmetic property, resolved decisions, and open questions. The overall structure and reasoning are faithfully represented.

### 2. Discrepancies and Issues

**2a. B mischaracterizes the break-even point for size savings**

- **A (§7):** "At 4 raw bytes: standard Z85 = 5 characters, raw passthrough = 4 bytes + 1 escape = 5 characters. **Zero net savings.** The benefit is transparency (raw bytes are readable), not compactness. Net character savings begin at 5+ raw bytes"
- **B:** "The break-even point for *size savings* is 5 raw bytes — below that, there's no compaction benefit"

This is subtly wrong. A says savings **begin at** 5+ bytes (budget ≥ 2, with at least 1 character saved after escape overhead). At exactly 4 bytes there are zero savings. B says the break-even is 5 bytes, which implies 5 bytes also has zero savings. A's framing is that 4 bytes = zero savings, 5+ = positive savings. B's "break-even point... is 5 raw bytes — below that, there's no compaction benefit" is ambiguous about whether 5 itself saves or not.

**Verdict:** Minor ambiguity, not a clear contradiction — but B's phrasing leans toward the wrong reading.

**2b. B omits the raw byte order guarantee**

- **A (§0, "Raw byte order"):** "Bytes within raw sections appear in their original sequential order, matching the input stream. Endianness discussion (§6) applies only to how partial Z85 blocks encode boundary bytes, not to raw data ordering."
- **B:** No mention.

**Verdict:** Missing fact from A.

**2c. B omits the raw byte values policy distinction**

- **A (§0, "Raw byte values"):** "The decoder imposes no restriction on what bytes appear in a raw section — it knows the length from the prefix and passes bytes through without validation. The *encoder* decides which bytes to include based on the desired compatibility profile... This is an encoder policy decision, not a format constraint."
- **B:** No mention. B says the decoder "copies those bytes directly to the output" which implies no restriction, but never explicitly states the encoder/decoder asymmetry on raw byte validation, or that the encoder's choice is a policy decision.

**Verdict:** Missing nuance from A.

**2d. B omits the "multiple raw sections" explicit guarantee**

- **A (§0):** "A single encoded stream can contain multiple raw sections interleaved with standard Z85 blocks. Each raw section is independent (its own escape, length, boundary handling). There is no limit on the number of raw sections per stream."
- **B:** Implies this (talks about "regions" plural) but never states it explicitly, and doesn't mention the independence of each section or the no-limit guarantee.

**Verdict:** Missing explicit statement from A.

**2e. B slightly mischaracterizes the escape character position constraint**

- **A (§3):** "The escape character must appear within the character positions of the Z85 block(s) being replaced — practically, within ~5 characters of the transition point. (This is a design target, not yet a hard specification.)"
- **B:** "A raw section is introduced by an escape character... followed by some prefix information (at minimum a length indicator), followed by the literal input bytes."

B describes the layout but omits A's specific constraint that the escape must appear within the character positions of the replaced blocks, and that this is a design target rather than a hard spec.

**Verdict:** Missing constraint from A.

**2f. B's "unclear" flag #1 is partially addressed by A**

B flags: "how the decoder actually selects among [the ~3 exit candidates] isn't specified."

This is fair — A does say the mechanism is "free in terms of escape budget, costs decoder complexity" and that the decoder "substitutes the known raw bytes into the block's Z85 arithmetic and solves for the boundary bytes," but doesn't give the final selection algorithm for narrowing ~3 candidates to 1. A acknowledges this is decoder complexity it's willing to accept. So B's flag is legitimate — this is genuinely unspecified in A.

**2g. B's "unclear" flag #6 is actually addressed by A**

B flags: "no analysis is given for the *adversarial* case"

- **A (§0, "Data distribution"):** "The uniform-random case represents a conservative baseline, not a worst case — adversarial input could maximize unstable boundaries, but the encoder can always avoid unstable cuts."

A explicitly addresses this. B's flag repeats A's own statement almost verbatim ("The encoder can always avoid unstable cuts, so correctness isn't at risk, but it could force the encoder to waste opportunities") — so this isn't really "unclear," B just didn't notice that A already said essentially the same thing.

**Verdict:** B's flag is redundant with A's own text.

**2h. B omits the block alignment reference frame**

- **A (§0, "Block alignment"):** "'Block-aligned' always means aligned to Z85's 4-byte / 5-character block boundaries relative to the **start of the Z85 stream**, not relative to the raw section or any other reference point."
- **B:** No mention.

**Verdict:** Missing definition from A.

**2i. B omits the forward-only decoding guarantee**

- **A (§0):** "The decoder processes the stream left to right in a single pass. It does not need to look ahead past the current raw section's prefix to determine length or boundaries. (It *may* reference recently decoded bytes — see §6 exit disambiguation — but never needs to scan forward.)"
- **B:** Says "The decoder reads left-to-right in one pass" but omits the no-look-ahead guarantee and the distinction between backward reference (allowed) and forward scanning (not needed).

**Verdict:** Partial capture — the key nuance about backward-but-not-forward is lost.

**2j. B omits A's "alignment preference for tiebreaking" detail**

- **A (§2, P3):** "**Alignment preference for tiebreaking:** When choosing between escape placement options, prefer the one that produces more structurally aligned raw data (more trailing zero bits → more likely to be at a meaningful boundary)"
- **B:** Says "with preference for structurally meaningful alignment (e.g., data at word boundaries is more useful to expose than data mid-structure)" — captures the spirit but omits the specific "more trailing zero bits" heuristic.

**Verdict:** Minor omission of the concrete heuristic.

**2k. B omits the "Rejected Alternative: Direct Byte Encoding" nuance about natural approach winning for tight budgets**

- **A (§6):** Gives a concrete comparison table and concludes "Natural wins for tight budgets" with a specific worked example showing budget=2 breakdown.
- **B's flag #4:** Says the comparison "assumes very tight budgets" and suggests revisiting for longer sections. This is a fair observation not addressed in A, but B doesn't mention A's explicit conclusion or the worked comparison.

**Verdict:** B's flag is a legitimate observation that goes beyond A; the omission of A's specific analysis is minor.

**2l. B slightly misstates the Z85 input length restriction**

- **A (§1):** "The original ZeroMQ spec requires input length to be a multiple of 4 bytes; we lift that restriction — arbitrary input lengths are supported, with partial final blocks handled the same way as mid-block boundary cuts"
- **B (resolved decisions):** "Arbitrary input lengths are supported (not limited to multiples of 4)"

B is correct but places this under "resolved decisions" as if it were a design choice, while A frames it in §1 (Mental Model) as a baseline assumption of the format. This is a minor structural issue, not a factual error.

### 3. Direct Contradictions

**None found.** B does not contradict A on any factual point. All discrepancies are omissions or ambiguities, not conflicts.

### 4. Summary

B is a **faithful rewrite** with some **omissions of precision**. The main gaps are:
- Several §0 assumptions not carried over (raw byte order, raw byte values policy, block alignment reference, forward-only decoding details, multiple raw sections guarantee)
- The escape-character position constraint (§3) is simplified
- The trailing-zero-bits tiebreaking heuristic is softened
- The break-even phrasing is slightly ambiguous

B's "unclear" flags are mostly legitimate, with the exception of #6 (adversarial case) which A already addresses. No direct contradictions exist.
