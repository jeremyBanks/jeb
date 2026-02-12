This is a well-structured design constraints document that effectively narrows down the design space. Here's my review:

## 1. Clarity Issues for Fresh Readers

**The position invariant (§2.P1) needs a concrete example early.** The concept is crucial but abstract. Add a simple example showing how a 12-byte input might look in standard Z85 vs extended Z85 with a raw section in the middle.

**"Budget" is introduced informally in §5 before its formal definition in §7.** The first mention ("insufficient budget") is confusing without context. Either define it earlier or avoid using the term until §7.

**The entry/exit boundary terminology shifts meaning between sections.** In §5-6, "entry" means where you enter a raw section (cutting a Z85 block). But in layout discussions, it could be confused with stream entry. Consider "raw-entry" and "raw-exit" for clarity.

## 2. Confusing or Contradictory Sections

**§6's title "Entry vs Exit Boundary Symmetry" contradicts its content about asymmetry.** The section demonstrates they're asymmetric in cost. Consider "Entry vs Exit Boundary Analysis" or similar.

**The stability percentages in §5 vs §6 appear inconsistent.** §5 gives specific percentages (68%, 89%, 96%) for entry boundaries. §6 claims exit boundaries have the same stability profile but doesn't provide the analysis. Either show the exit stability calculation or clarify why it's omitted.

**§10's "raw to end of input" escape seems to contradict the length-before-data requirement.** If the decoder must know length before reading raw bytes, how does "to end" work? The decoder doesn't know the end until it gets there. This needs clarification.

## 3. Implicit Assumptions That Should Be Explicit

**The decoder's access pattern is assumed but not stated.** The document assumes streaming/forward-only decoding (can't look ahead to find raw section end). State this explicitly, as it drives the length-before-data requirement.

**The maximum raw section length is unbounded in theory but practically limited.** With finite escape characters and overhead budget, there's a maximum encodable length. State whether implementations should support arbitrary-length raw sections or can impose reasonable limits.

**Escape character reuse within a stream isn't addressed.** Can the same escape character appear multiple times in one stream? The document implies yes ("Multiple raw sections") but doesn't state how this affects information encoding.

**The cost of decoder back-referencing for exit boundaries isn't quantified.** §6 calls this "free" in terms of escape budget but acknowledges decoder complexity. How far back must the decoder look? Is there a maximum lookback distance?

## 4. Missing Constraint Analysis

**No analysis of error detection/recovery.** What happens if an encoded stream is corrupted? Can the decoder detect invalid encodings? Should it fail fast or attempt recovery? This affects whether certain bit patterns are reserved for error detection.

**The interaction between consecutive raw sections (§11.Q6) affects budget calculations.** If two raw sections can be adjacent, their combined overhead might be less than treating them separately. This optimization opportunity isn't explored.

**No analysis of worst-case expansion.** While the format guarantees output ≤ standard Z85 length, what's the worst-case overhead for toggling between raw and Z85? An adversarial input that forces frequent transitions could approach standard Z85 length despite containing printable text.

**The "raw byte values" assumption (§0) hand-waves encoder policy.** It says encoders decide which bytes to include based on "compatibility profiles" but doesn't explore how this affects format universality. Can decoders make assumptions about what bytes appear raw?

## 5. Specific Technical Issues

**§9's "4 disambiguation candidates" seems wrong.** For a 1-byte entry cut, §5 shows ~2 bits needed (3-4 candidates), not a fixed 4. The multiplication in §9 (4×4×4=64) may overestimate the required combinations.

**The modular arithmetic example in §6 needs careful review.** The claim that `2^8 ≡ 2^16 ≡ 2^24 ≡ 1 (mod 85)` should be verified. If correct, this is a beautiful insight that deserves more prominence.

**Length encoding efficiency isn't analyzed.** How many bits does length encoding consume from the available budget? Is there an optimal encoding scheme that maximizes the length range for a given bit budget?

## Overall Assessment

This is a thoughtful exploration that successfully narrows the design space. The key insights about entry/exit asymmetry and information multiplexing are valuable. The document would benefit from:

1. A concrete worked example early on
2. Consistent terminology 
3. Making streaming assumptions explicit
4. Addressing error handling constraints
5. Tightening the mathematical analysis in §9

The level of detail is appropriate for a constraints document — it identifies the key trade-offs without over-specifying implementation details.
