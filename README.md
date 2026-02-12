# Extended Z85 Encoding

A Rust implementation of Extended Z85 - a binary-to-text encoding that combines the Z85 standard with raw ASCII passthrough to reduce encoding overhead for text-heavy data.

## Overview

Z85 is a standard binary-to-text encoding (RFC 32) that encodes 4 bytes into 5 printable ASCII characters (~25% overhead). Extended Z85 adds "raw sections" that pass through eligible bytes unencoded, eliminating overhead for regions that are already printable ASCII.

For example:
- Input: `[0xDE, 0xAD, 0xBE, 0xEF, 'H', 'e', 'l', 'l', 'o', 0x01, 0x02, 0x03, 0x04]`
- Standard Z85: `rZUgH4erBi0sjjE` (15 characters)
- Extended Z85: `rZUgH_Hello0sjjE` (18 chars: the escape `_`, length, and literal "Hello" replace the middle 5 Z85 characters)

The format is a superset of standard Z85: data encoded with no raw sections is identical to standard Z85 output.

## Key Features

- ✅ **Position Invariant**: Complete Z85 blocks remain byte-identical to standard Z85 at the same positions
- ✅ **Non-Aligned Lengths** (R1): Raw sections support any length, not just multiples of 4 bytes
- ✅ **Mid-Block Boundaries** (R2): Raw sections can start/end mid-block, with proper Z85 encoding for boundary bytes
- ✅ **Raw Byte Eligibility** (R3): Configurable policy for which bytes to pass raw (default: printable ASCII 0x20-0x7E)
- ✅ **Consecutive Raw Sections** (R4): Decoder supports adjacent raw sections (encoder prefers merging for efficiency)
- ✅ **Roundtrip Correctness**: `decode(encode(data)) == data` for all valid inputs

## Design Choices (Answers to §12 Open Questions)

### Q1: How many escape characters?

**Decision: 2 escape characters** (`_` underscore and `~` tilde)

Both are Tier 1 (completely free per compatibility analysis - no additional escaping needed for JSON, shell, etc.). Provides 2 bits of information via choice of character.

**Rationale**: Minimal context cost while sufficient for basic functionality. The two characters allow potential future extension (one for each boundary convention: entry vs exit).

### Q2: What information does each escape character encode?

**Decision: Fixed escape character** (`_` only currently used)

The escape character signals:
- A raw section follows
- Length is encoded in the next 3 Z85 characters
- No endianness/convention metadata (simplified design)

The second character (`~`) is reserved for future use (e.g., different boundary conventions or special modes).

**Rationale**: Simplicity. The current design hardcodes one convention. Future versions could multiplex information via character choice.

### Q3: How are disambiguation bits laid out?

**Decision: Implicit in Z85 block structure**

Entry boundaries emit partial Z85 characters (1-3 chars for 1-3 bytes). Stability is checked at encode time:
- ~68% of byte values produce stable leading characters (no disambiguation needed)
- Unstable cases are detected and the encoder avoids mid-block cuts or uses Z85 encoding

Exit boundaries use trailing Z85 characters, disambiguated via already-decoded raw context (zero cost in escape budget).

**Rationale**: Leverages Z85's mathematical properties. No explicit disambiguation bytes needed; context provides resolution.

### Q4: Raw block internal layout - syntax and ordering

**Decision: Escape-first prefix pattern**

Layout for each raw section:
1. **ESCAPE** (1 byte): `_` (0x5F)
2. **LENGTH** (3 Z85 chars): encodes raw byte count in base-85 (supports up to 614,125 bytes)
3. **PARTIAL Z85 CHARS** (0-3 chars): for entry boundary bytes (if mid-block entry)
4. **RAW DATA** (N bytes): the actual bytes, unencoded
5. **PARTIAL Z85 CHARS** (0-3 chars): for exit boundary bytes (if mid-block exit)

This is a fixed, deterministic layout that allows the decoder to parse forward without lookahead.

**Rationale**: Length-before-data is required for forward-only decoding. Z85-encoded length ensures all output remains in the printable ASCII set.

## Length Encoding Detail

The 3-character Z85 encoding of length:

```
length = d0 * 85² + d1 * 85 + d2
where each dᵢ ∈ [0, 84] maps to a Z85 alphabet character
```

This supports lengths from 0 to 614,125 (more than sufficient). The 3-character minimum uses all of the extended Z85 alphabet, ensuring printable output only.

## Raw Byte Eligibility (R3)

Default policy: Bytes in the range **0x20-0x7E** (printable ASCII) are eligible for raw passthrough, except the escape character itself (`_`, 0x5F).

This ensures:
- Readable text appears literally in the output
- Escape sequences can't be confused with data
- The decoder never sees ambiguous character sequences

The encoder is conservative: it only uses raw sections when the benefits are clear. The decoder accepts any byte value in raw sections (length-prefixed sections are unambiguous).

## Mid-Block Boundary Handling

### Entry Boundaries (raw section starts mid-block)

When a raw section begins after K bytes of a Z85 block (K ∈ {1,2,3}):
1. Encoder emits K+1 leading Z85 characters for those bytes (with padding zeros)
2. Checks if boundary byte is "stable" (leading digit independent of padding)
3. If stable, proceeds with raw section; if not, falls back to full Z85 encoding

Stability rates (uniform random data):
- 1-byte entry: 68% stable (174/256)
- 2-byte entry: 89% stable
- 3-byte entry: 96% stable

### Exit Boundaries (raw section ends mid-block)

When a raw section ends with K bytes to go in a Z85 block (K ∈ {1,2,3}):
1. Encoder emits K trailing Z85 characters for those bytes
2. Trailing character values are disambiguated using already-decoded raw bytes via modular arithmetic
3. No disambiguation bits needed (trailing digits = sum of all bytes mod 85)

This asymmetry (entry vs exit) is mathematically justified by Z85's structure: trailing characters naturally encode a symmetric function of all block bytes.

## Building & Testing

### Build

```bash
cargo build --release
```

### Run Tests

```bash
cargo test
```

Test coverage includes:
- Standard Z85 baseline (partial blocks, all byte values)
- Raw section basics (various lengths and alignments)
- **Mid-block boundaries** (all entry/exit combinations)
- Non-aligned lengths
- Position invariant verification
- Escape character detection
- Edge cases (consecutive raw sections, large inputs, alternating patterns)
- Round-trip correctness (primary test criterion)

All 66 tests should pass.

### Run with Output

```bash
cargo test -- --nocapture
```

## Implementation Notes

### Encoder Strategy

The encoder is **opportunistic**: it scans the input for runs of eligible bytes (≥4 bytes minimum) and encodes them as raw sections. The greedy algorithm:

1. Find next run of eligible bytes
2. If run ≥ 4 bytes, check if raw encoding is beneficial
3. For mid-block cuts, validate boundary byte stability
4. Emit raw section or fall back to Z85

This is simple and effective; it's not optimal (e.g., won't reorganize data), but it handles all cases correctly.

### Decoder Strategy

The decoder is **simple and strict**: it reads forward, alternating between Z85 blocks and raw sections based on character values:
- Non-Z85 character → escape sequence (read 3-char length, then raw data)
- Z85 character → Z85 block or partial block

No lookahead needed; length is always explicit before raw data.

### Partial Block Decoding

Partial blocks (1-4 Z85 characters for 1-3 bytes) are decoded via brute force search: try all possible input bytes, encode them with zero-padding, and check if the partial Z85 characters match. This is simple and correct, though not optimal for speed. Performance is acceptable for typical input sizes.

## Limitations & Future Work

1. **No maximum length enforcement**: Raw sections can be arbitrarily long (encode uses single section)
2. **Single escape character**: Only `_` is used; `~` reserved for future use
3. **Conservative stability checking**: Encoder is conservative on mid-block cuts (could be more aggressive with proper disambiguation)
4. **No configuration**: Hardcoded escape characters and byte eligibility (could be parameterized)

## Compatibility

- **Input**: Arbitrary binary data (0 to 2^32 bytes)
- **Output**: Printable ASCII (Z85 alphabet + escape characters)
- **Superset of Z85**: Any standard Z85 output is valid extended Z85
- **Not a drop-in replacement**: Extended Z85 decoder is backward-compatible with standard Z85, but standard Z85 decoder will reject extended output (escape characters are non-Z85)

## References

- [ZeroMQ RFC 32: Z85 Encoding](https://rfc.zeromq.org/spec/32/)
- [DESIGN-CONSTRAINTS.md](./DESIGN-CONSTRAINTS.md) - Detailed analysis and design rationale
- [TEST-EXPECTATIONS.md](./TEST-EXPECTATIONS.md) - Expected test cases
