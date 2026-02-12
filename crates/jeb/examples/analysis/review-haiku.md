# Haiku Review of DESIGN-CONSTRAINTS.md

*Fresh-eyes clarity review. Reviewer was given no code access, told the doc is in-progress design space exploration.*

## 1. Clarity Issues

- §2 P1: "slightly shorter" is vague and quantitatively unbounded
- §3: "~1 block of lookahead" then "guideline, not hard limit" — too loose for constraints doc
- §3: "unambiguous" needs a referent (to the decoder? encoder?)
- §5 vs §6: Inconsistent column headers across three different stability tables. Fresh reader has to map between them. "Bits captured" (~6.4) relationship to stability % unclear.

## 2. Contradictions & Tensions

- **Minimum 4 bytes (§9) is unjustified and circular.** "No reason it would be higher" doesn't explain why it's not 1-3. Is it budget=1 being too tight? Overhead not paying off? Explain or move back to open questions.
- **"Roughly the same" entry/exit cost (§6) contradicts the numbers.** Entry: 68%→89%→96%. Exit: 100% at 1 byte. These aren't "roughly the same" — the asymmetry is the whole point. Reframe as "same cost structure, different stability profiles."
- **Escape char uses "mutually exclusive" (§8) is overstated.** Bits can be multiplexed (high=endianness, mid=length, low=disambig). Not one-capability-per-char.

## 3. Implicit Assumptions

- Data distribution assumptions buried in §5/§6 — move to front
- Budget formula (`⌈N×5/4⌉ - N`) not stated in §7
- "Opportunistic" encoding: "beneficial" is never defined
- Position invariant enforcement mechanism unclear (encoder constraint? design consequence?)

## 4. Missing Analysis

- **Escape char count vs layout complexity:** Do 6 escape chars require 6 different block layouts? Combinatorial explosion?
- **Length encoding constraints:** What about consecutive raw sections? How do implicit length classes interact with opportunistic heuristic?
- **Disambiguation bit allocation:** Where do they physically go? How encoded? Trade-off vs dedicated escape chars?
- **Mid-block boundary crossover point:** At what boundary byte count does direct encoding beat natural approach given a fixed escape budget?

## 5. Structural Suggestions

- Add §0 (Assumptions) collecting all hidden assumptions upfront
- Reorder §10 by dependency: design logic → layout/encoding → decodability
- Add example layout diagrams in §10 Q4

## Summary

Strengths: Excellent constraint analysis, clear priority order, solid math.
Gaps: Assumptions buried, minimum length unjustified, escape char capacity glosses over layout complexity, exit stability incomplete.
