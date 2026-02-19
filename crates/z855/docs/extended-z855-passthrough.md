# Extended Z855: Raw Passthrough Encoding

> **Note:** This document represents the current best understanding of the planned encoding scheme as of the time of writing. Details may be incorrect or outdated. Always refer to the actual implementation and tests as the source of truth.

## Background: Standard Z85

Z85 encodes binary data as printable ASCII using an 85-character alphabet:
```
0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#
```

**Encoding:**
- 4 input bytes are interpreted as a big-endian 32-bit unsigned integer V
- V is converted to base-85, producing 5 characters
- Ratio: 4 bytes → 5 characters (25% overhead)

**Arbitrary length:** Trailing 1-3 bytes are encoded as 2-4 characters respectively.

## Z855 Extension: Raw Passthrough

### Goal

Allow sections of input that consist of "safe" characters to pass through unencoded, preserving human readability while maintaining the same output length as standard Z85.

### Safe Characters

The safe character set (90 characters):
```
0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_
```

This is the Z85 alphabet plus five additional characters: `,;|~_`

### The Escape Character

We use `,` (comma) as the escape character to signal raw passthrough.

### Block-Aligned Passthrough

When `,` appears at position 0 of a 5-character output block:
- The next 4 characters are raw input bytes (not Z85-encoded)
- Total: 5 characters (same as standard Z85 for 4 bytes)
- **Lossless and unambiguous**

Example:
- Input: `test` (4 bytes: 0x74 0x65 0x73 0x74)
- Standard Z85: `By/Jn`
- Passthrough: `,test`

The encoder may choose either representation. The decoder handles both.

## Non-Aligned Passthrough (Advanced)

### The Challenge

When the 4 raw bytes we want to pass through don't align with Z85 block boundaries, the `,` must appear at a non-zero position within a 5-character block. This interrupts the Z85 encoding of the surrounding blocks.

### Why It's Lossy

Z85 encodes 4 bytes (32 bits) into 5 characters. Each character carries approximately log2(85) ≈ 6.4 bits of information. If we only have P characters from a block, we only have ~P×6.4 bits, which is insufficient to fully reconstruct 32 bits.

However, the raw passthrough bytes themselves provide additional constraints, significantly reducing the ambiguity.

### Information Analysis by Position

When `,` appears at position P (1 ≤ P ≤ 4) within a 5-character block:

**First interrupted block** (has P high-order Z85 chars, raw provides last 4-P input bytes):

| P | Z85 chars | Unknown bytes | Known from raw | Possible values | Bits ambiguous |
|---|-----------|---------------|----------------|-----------------|----------------|
| 1 | 1 | B0 (1 byte) | B1,B2,B3 | ~3-4 | ~1.6-2.0 |
| 2 | 2 | B0,B1 (2 bytes) | B2,B3 | ~9-10 | ~3.2 |
| 3 | 3 | B0,B1,B2 (3 bytes) | B3 | ~28-29 | ~4.8 |
| 4 | 4 | all 4 bytes | none | 85 | ~6.4 |

**Second interrupted block** (has 5-P low-order Z85 chars, raw provides first P input bytes):

The constraint is much tighter here because we know the high-order bytes and have the low-order base-85 digits. In most cases, this fully determines the block:

| P | Z85 chars | Possible values | Bits ambiguous |
|---|-----------|-----------------|----------------|
| 1 | 4 | ~0-1 | ~0 |
| 2 | 3 | ~0-1 | ~0 |
| 3 | 2 | ~0-1 | ~0 |
| 4 | 1 | 1 (fully known) | 0 |

**Total ambiguity:** The second block is almost always fully determined, so total ambiguity equals the first block's ambiguity.

### The Asymmetry Explained

Why is the "before" block ambiguous but the "after" block usually determined?

- **Before block:** We have high-order Z85 digits and know low-order input bytes. The Z85 digits constrain V to a range of size 85^(5-P). We must find which V values in this range have the known low-order bytes. Multiple values typically qualify.

- **After block:** We have low-order Z85 digits (V mod 85^(5-P)) and know high-order input bytes. The known high bytes fix most of V's value. The modular constraint then usually pins down the remaining low bytes uniquely.

### Shifting the Ambiguity

For the same 4 raw bytes, we can place `,` at position P or P+1:

- **Position P:** Ambiguity in the "before" block
- **Position P+1:** Ambiguity in the "after" block (which was the "before" block's successor)

The total bits of ambiguity remains the same, but we choose which block bears it.

## Canonical Values and Encoding Rules

### The Problem

Non-aligned passthrough is inherently ambiguous: multiple input values decode to the same output. We need deterministic behavior.

### The Solution: Canonical Minimum

For each ambiguous block, we define the **canonical value** as the **minimum** among all possible 32-bit block values (interpreting bytes as big-endian unsigned integers).

**Decoding rule:** Always output the canonical (minimum) value.

**Encoding rule:** Only use non-aligned passthrough if the actual block value equals the canonical minimum. Otherwise, use standard Z85.

### Implications

- Decoding is deterministic and unambiguous
- Encoding is selective: non-aligned passthrough only works ~10-11% of the time (roughly 1 in 9-10 for typical ambiguity levels)
- Round-trip consistency: encode → decode → same value (when passthrough is used)

### Optimization: Try Both Positions

Since we can shift `,` by one position to move ambiguity between blocks, the encoder can:

1. Try position P: check if "before" block value is canonical
2. If not, try position P+1: check if "after" block value is canonical
3. Use whichever works; fall back to standard Z85 if neither does

This roughly doubles the success rate for non-aligned passthrough (~20% instead of ~10%).

**Constraint:** This requires at least one more output character after the passthrough, so it doesn't apply at the very end of data.

## Decoder Behavior Summary

1. At a block boundary, check if next char is `,`
2. If yes: take next 4 bytes as literal output (no validation of content)
3. If no: decode as standard Z85

The decoder never needs to know the safe character set. It trusts that well-formed input only uses `,` escapes where appropriate.

## Encoder Behavior Summary

1. Process input in 4-byte chunks
2. For each chunk, if all 4 bytes are safe characters:
   - If block-aligned: may use `,XXXX` passthrough
   - If non-aligned: check if block value is canonical minimum; if so, may use passthrough
   - For non-aligned: try both `,` positions if possible
3. Otherwise: use standard Z85 encoding

## Length Invariant

All encodings produce exactly the same output length as standard Z85:
- 4 input bytes → 5 output characters (whether Z85 or `,XXXX`)
- n input bytes → ceiling(n × 5/4) output characters

## Test Case Format

Test cases use multiple files to represent valid encodings:
- `X.input` — raw input bytes
- `X.encoded` — standard Z85 encoding (always present)
- `X.encoded-Y` — alternative valid encodings
- `X.encoded-expected` — if present, encoder must produce exactly this

For error cases, `.input` contains `<error />` (exactly, no newline).
