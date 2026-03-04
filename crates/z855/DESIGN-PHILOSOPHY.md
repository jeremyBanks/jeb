# Z855 Design Philosophy

## Core Values

### Liberal Decoder, Conservative Encoder
**The decoder accepts ANY bytes in passthrough sections** (after `,`, `~`, or `|` escapes), even if the encoder would never produce them. This follows the Robustness Principle (Postel's Law): be conservative in what you send, liberal in what you accept.

**Why this matters:**
- Real-world data is messy
- Future versions might use different canonicalization
- Interoperability > strict validation
- Decoder never rejects valid structure, even if bytes are "unsafe" or non-canonical

**Example:** Encoder only emits printable ASCII in passthrough sections, but decoder accepts control characters, high bytes, even NUL bytes. Test cases 300-303 verify this.

### Position Invariance
Z85-encoded blocks must appear at exactly the same character positions as standard Z85. Output never longer (except mid-block transition characters may differ).

This is a **hard requirement**, not an optimization. It enables:
- Drop-in compatibility with standard Z85 parsers
- Predictable output lengths
- Transparent handling of standard Z85 data

### Format Agnostic (Zipng)
The zipng decoder works with **any indexed image format** the `image` crate can load - PNG, GIF, BMP, whatever. Don't assume PNG-specific behavior.

**Implementation:** Use `image::DynamicImage` and extract RGB data generically. Build our own reverse color map (`HashMap<[u8;3], u8>`) rather than trusting format-specific palette extraction.

### Binary-Safe File Handling
Use `fs::read()` + `from_utf8_unchecked()` instead of `fs::read_to_string()` for files that may contain non-UTF-8 bytes in passthrough sections.

**Rationale:** UTF-8 validation would reject valid z855-encoded data with raw bytes. The decoder is binary-safe; file loading should be too.

## Testing Philosophy

### Comprehensive Coverage
- **92 explicit test cases** covering edge cases, boundaries, unsafe bytes
- **Property-based testing** (TODO) for invariants: roundtrip, length bounds, transparency
- **All three implementations** (Rust, TypeScript, min.mjs) validated against the same test suite

### Test What Matters
1. **Roundtrip**: `decode(encode(x)) == x` for all inputs
2. **Length bounds**: Never exceed theoretical maximum
3. **Transparency**: Standard Z85 blocks roundtrip identically
4. **Position invariance**: Encoded blocks appear at correct positions
5. **Decoder robustness**: Never panics, even with malformed input

### Liberal Decoder Tests (300-303)
Verify the decoder truly accepts ANY bytes in passthrough sections:
- Control characters (0x00-0x06)
- High bytes (0xFF, 0xFE)
- Mixed unsafe/non-canonical sequences
- Long passthrough sections with arbitrary data

## Design Constraints

From `DESIGN-CONSTRAINTS.md` (canonical reference):

### Priority Order
1. **Correctness & position invariant** - Non-negotiable
2. **Context compatibility** - CSV/JSON safety where practical
3. **Transparency** - Standard Z85 roundtrips identically

### Entry/Exit Symmetry
Both boundaries cost ~2 bits/byte disambiguation:
- **Entry** uses leading characters (big-endian)
- **Exit** uses trailing characters (little-endian)

Asymmetric defaults are natural - different stability characteristics for different use cases.

### Escape Character Budget
~1-2.6 free bits per escape character choice carry disambiguation cheaper than extra Z85 characters.

Character tiers:
- 2 free: `_`, `~`
- 1 near-free: `` ` ``
- 3 CSV-cost: `|`, `,`, `;`

## Lessons Learned

### Implementation Race (Feb 2026)
8 AI agents independently implemented z855 from the design document. Only Haiku passed mid-block boundaries. Key insight: **mid-block entry is partial encoding** (N bytes → N+1 chars directly), not zero-padded leading chars.

### Manual Implementation Success
10th attempt (first manual one) achieved 7/7 criteria, 18/18 tests. Breakthrough came from understanding partial encoding, not just following the spec literally.

### Bug Fixes via Testing
Comprehensive test suite caught 2 CRITICAL bugs:
1. **Length violation**: Mid-block could exceed standard Z85 length (fixed with budget check)
2. **Decoder crash**: Exit logic fragmented blocks incorrectly (fixed opportunistic exit)

Property testing revealed actual violations of hard requirements, not just "design questions."

---

*This document captures design values learned through implementation, testing, and debugging. It's not just aesthetic - these principles prevented real bugs and enabled real interoperability.*
