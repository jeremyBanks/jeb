Looking at this document as a fresh reader, I'll provide feedback on clarity, contradictions, implicit assumptions, and missing analysis.

## Clarity Issues

**Section numbering is inconsistent.** You jump from §6 to §8, §10, §11 with gaps at §7 and §9. Then §7 and §9 appear later. This makes cross-references confusing and suggests the document wasn't written linearly.

**"Stability" terminology is overloaded.** In §5 you define stability as "fraction of byte values where leading Z85 characters are the same for ALL possible values of remaining unknown bytes." But later you use phrases like "naturally stable" and "self-stable" without clear definitions. A reader might confuse this with other meanings of "stable" (unchanging over time, robust to perturbation).

**Budget calculations are scattered and inconsistent.** The budget formula appears in §7 but examples using it appear throughout earlier sections. In §5's table, you show "Disambiguation bits when not [stable]" but don't clearly connect this to the budget analysis until much later.

**Raw section length constraints are unclear.** You state minimum 4 bytes in §10 but don't justify why shorter sections are impossible or undesirable. The relationship between raw section length and block alignment is explained in pieces across multiple sections.

## Contradictory or Confusing Sections

**§6 claims exit disambiguation is "free" but then acknowledges decoder complexity cost.** You can't have it both ways - either it's free or it costs decoder complexity. The correct framing is that it's free in terms of *escape character budget* but expensive in *decoder complexity*. This distinction matters for the design trade-offs.

**Position invariant (§2) vs. partial blocks allowance.** You state that complete Z85 blocks must appear "at the exact same positions" as standard Z85, but then say partial blocks "may differ from what standard Z85 would produce." A fresh reader needs clearer boundaries: which blocks are subject to position invariance and which aren't?

**Entry vs. exit boundary terminology.** You define entry as "where encoder cuts a Z85 block to begin a raw section" but the actual cutting happens between blocks or within blocks. The mental model of "cutting blocks" versus "choosing boundaries between regions" isn't consistently applied.

## Implicit Assumptions That Should Be Explicit

**The decoder can efficiently perform the "exit disambiguation from raw context" computation.** You assume in §6 that back-referencing recently decoded raw bytes and solving Z85 arithmetic is reasonable complexity, but you don't state performance requirements or implementation constraints.

**Standard Z85 knowledge.** You assume readers know Z85's big-endian encoding, base-85 arithmetic, and 4:5 block structure. For a design document, you should either include this background or clearly state the prerequisite knowledge.

**"Opportunistic" encoder behavior is consistently beneficial.** You repeatedly state the encoder decides when raw passthrough helps, but don't specify the decision criteria. What makes a region "beneficial" for raw encoding? Just character savings, or also transparency/readability?

**Raw byte order within sections.** You state this in assumptions but it's actually a design choice that affects complexity. Why not allow the encoder to reorder bytes within raw sections for better stability or compression?

**Single-pass decoding.** You require length-before-data (§10) but don't explicitly state that the decoder should work in a single forward pass. This seems to be an implicit design goal affecting multiple decisions.

## Missing Constraint Analysis

**Decoder state complexity.** You analyze escape character budgets and disambiguation costs but don't analyze decoder state requirements. How much lookahead/lookbehind? How much buffering for the "raw context" exit disambiguation?

**Error handling and malformed input.** What happens when the decoder encounters invalid escape sequences, length mismatches, or corruption? How does this interact with the self-signaling property?

**Alignment with existing Z85 implementations.** You mention that standard Z85 decoders will reject extended Z85, but don't analyze upgrade/migration paths or whether certain design choices make transitions easier.

**Performance analysis.** Character budget and context compatibility are analyzed, but not encoding/decoding speed, memory usage, or implementation complexity compared to standard Z85.

**Real-world data validation.** Your stability percentages assume uniform random bytes, and you mention real data will be "often higher" but provide no analysis. This is a critical gap - the design's practical value depends on how well it handles actual text-heavy inputs.

**Stream processing constraints.** Can the format be decoded incrementally? What's the minimum buffer size needed? How does this interact with the length-before-data requirement?

**Maximum raw section lengths.** You show calculations going to hundreds of bytes but don't analyze practical upper bounds or whether very long raw sections create different problems.

The document shows sophisticated analysis of the core encoding constraints but needs better organization and more explicit treatment of implementation and real-world usage constraints.
