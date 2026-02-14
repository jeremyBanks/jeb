# Extended Passthrough: 5, 6, and 7-Byte Escapes

> **Note:** This document represents the current best understanding of the planned encoding scheme. Details may be incorrect or outdated.

## Overview

We extend the passthrough mechanism beyond 4-byte blocks:

| Escape | Raw Bytes | Standard Z85 Length | Escape Length | Difference |
|--------|-----------|---------------------|---------------|------------|
| `,`    | 4         | 5 chars             | 5 chars       | 0          |
| `;`    | 5         | 7 chars             | 6 chars       | -1         |
| `_`    | 6         | 8 chars             | 7 chars       | -1         |
| `~`    | 7         | 9 chars             | 8 chars       | -1         |

The 5/6/7-byte escapes are each **1 character shorter** than standard Z85 encoding.

## The Key Insight

With 4-byte passthrough (`,`), non-aligned positions create ambiguity because we don't have enough Z85 characters to fully determine the interrupted blocks. This required the complex "canonical minimum" scheme.

With 5/6/7-byte escapes, we **save 1 character**. We can spend this saved character by outputting one additional Z85 digit for the partial block. This extra digit is enough to **fully disambiguate** the encoding.

Result: **No canonical minimum constraint needed.** These escapes work at any position.

## Why the Math Works

### Standard Z85 Encoding Lengths

Z85 encodes N bytes into ceil(N × 5/4) characters:
- 4 bytes → 5 chars
- 5 bytes → 7 chars (4→5 + 1→2)
- 6 bytes → 8 chars (4→5 + 2→3)
- 7 bytes → 9 chars (4→5 + 3→4)

### Escape Encoding Lengths

Escape + raw bytes:
- `;` + 5 bytes = 6 chars
- `_` + 6 bytes = 7 chars
- `~` + 7 bytes = 8 chars

Each is exactly 1 char shorter than standard Z85.

### Spending the Saved Character

When using these escapes at a non-block-aligned position, we interrupt a Z85 block. With 4-byte passthrough, we had:

```
[P chars of before block] [,] [4 raw bytes] [(5-P) chars of after block]
```

The partial encodings (P chars and 5-P chars) left ambiguity.

With 5-byte passthrough, we have 1 extra char to spend:

```
[P+1 chars of before block] [;] [5 raw bytes] [(5-P) chars of after block]
```

Or equivalently, we could add it to the after block:

```
[P chars of before block] [;] [5 raw bytes] [(5-P+1) chars of after block]
```

The extra character is simply the next character from what standard Z85 would have produced. It's not new information - it's just including more of the deterministic encoding.

## Ambiguity Resolution

### Recall: 4-Byte Passthrough Ambiguity

With position P for 4-byte passthrough, the "before" block had:
- P=1: ~2 bits ambiguity (3-4 possible values)
- P=2: ~3.2 bits ambiguity (9-10 possible values)
- P=3: ~4.8 bits ambiguity (28-29 possible values)
- P=4: ~6.4 bits ambiguity (85 possible values)

Each additional Z85 character provides ~6.4 bits of information (log2(85) ≈ 6.4).

### 5/6/7-Byte Passthrough: No Ambiguity

By adding 1 character (~6.4 bits) to the partial encoding:
- P=1 + 1 char → 2 chars: 2 bits ambiguity - 6.4 bits = **fully resolved**
- P=2 + 1 char → 3 chars: 3.2 bits ambiguity - 6.4 bits = **fully resolved**
- P=3 + 1 char → 4 chars: 4.8 bits ambiguity - 6.4 bits = **fully resolved**
- P=4 + 1 char → 5 chars: Full block = **fully determined**

The extra character is always sufficient to eliminate ambiguity.

## Encoding Algorithm

For 5-byte passthrough (`;`) at position P in a block:

1. **Compute the before block's standard Z85 encoding** (5 chars: C0, C1, C2, C3, C4)
2. **Output P+1 characters**: C0 through C_P (one more than with 4-byte passthrough)
3. **Output the escape**: `;`
4. **Output the 5 raw bytes**
5. **Continue with the after block** using (5-P) characters of its encoding

Total output: (P+1) + 1 + 5 + (5-P) = 12 characters for 9 input bytes

Standard Z85 for 9 bytes: ceiling(9 × 5/4) = 12 characters ✓

### Example: 5-byte passthrough at P=2

Input: 9 bytes `[B0, B1, B2, B3, B4, B5, B6, B7, B8]`

Where bytes B2-B6 are safe and we want passthrough.

Standard Z85:
- Block 0 (`B0-B3`) → `ABCDE` (5 chars)
- Partial block (`B4-B8`) → `FGHI` (5 bytes → ... wait, that's the second block)

Let me redo this more carefully:
- Before block: `B0, B1, B2, B3` → standard Z85 `ABCDE`
- After block: `B4, B5, B6, B7, B8` → standard Z85 would be 7 chars, but we only need the trailing portion

With `;` at position 2:
- Output: `ABC` (3 chars = P+1 = 2+1)
- Output: `;`
- Output: `B2, B3, B4, B5, B6` (5 raw bytes)
- Output: remaining Z85 chars for bytes B7, B8

Wait, this needs more careful analysis of which bytes are where.

## Detailed Position Analysis

Let's be precise about byte positions and block boundaries.

### Input Stream and Block Boundaries

```
Input bytes:    B0  B1  B2  B3 | B4  B5  B6  B7 | B8  B9  ...
Block:          [  Block 0   ] | [  Block 1   ] | [Block 2]
Z85 output:     [   5 chars  ] | [   5 chars  ] | [ ... ]
```

### 5-Byte Passthrough Spanning Blocks

If we want to pass through bytes B2-B6 (5 consecutive bytes):

```
Input:     B0  B1 | B2  B3  B4  B5  B6 | B7  B8  ...
                  |<-- 5 raw bytes -->|
```

These 5 bytes span from Block 0 (B2, B3) into Block 1 (B4, B5, B6).

The escape `;` appears at some position in the output stream. Let's say position P within Block 0's output.

**Output structure:**
```
[P+1 Z85 chars from Block 0] [;] [B2 B3 B4 B5 B6] [remaining Z85 for B7, B8, ...]
```

The P+1 chars are the first P+1 characters of what `encode(B0, B1, B2, B3)` would produce.

The remaining chars after the raw bytes encode the data that comes after B6.

## Length Verification

### Output Structure

For K-byte passthrough (K = 5, 6, or 7) at position P within a block:

```
[(P+1) Z85 chars] [escape] [K raw bytes] [(5-P) Z85 chars]
```

Compare to 4-byte passthrough:
```
[P Z85 chars] [,] [4 raw bytes] [(5-P) Z85 chars]
```

The difference: K-byte passthrough uses **P+1** chars for the before block (one more than 4-byte).

### Length Calculations

**4-byte passthrough (8 input bytes = 10 chars):**
- P + 1 + 4 + (5-P) = **10 chars** ✓

**5-byte passthrough (9 input bytes = 12 chars):**
- (P+1) + 1 + 5 + (5-P) = **12 chars** ✓

**6-byte passthrough (10 input bytes = 13 chars):**
- (P+1) + 1 + 6 + (5-P) = **13 chars** ✓

**7-byte passthrough (11 input bytes = 14 chars):**
- (P+1) + 1 + 7 + (5-P) = **14 chars** ✓

All lengths match standard Z85 encoding.

### Concrete Example: 5-byte passthrough

Input: 9 bytes `[B0, B1, B2, B3, B4, B5, B6, B7, B8]`

Want to pass through bytes B2-B6 (5 bytes). These span block 0 (B2,B3) and block 1 (B4,B5,B6).

Standard Z85 would produce:
- Block 0 (B0-B3) → `C0 C1 C2 C3 C4` (5 chars)
- Block 1 (B4-B7) → `D0 D1 D2 D3 D4` (5 chars)
- Partial (B8) → `E0 E1` (2 chars)
- Total: 12 chars

With `;` at position P=2 (raw starts at byte 2):
- Output `C0 C1 C2` (P+1 = 3 chars from block 0)
- Output `;`
- Output `B2 B3 B4 B5 B6` (5 raw bytes)
- Output `D3 D4` (5-P = 3 chars from block 1... wait, need to recalculate)

Actually, the after-block chars depend on what remains. After the 5 raw bytes (B2-B6), we have B7, B8 remaining. These encode to 3 chars.

Revised output:
- `C0 C1 C2` (3 chars) - first 3 chars of block 0's encoding
- `;` (1 char)
- `B2 B3 B4 B5 B6` (5 raw bytes)
- 3 chars encoding B7, B8

Total: 3 + 1 + 5 + 3 = 12 chars ✓

The key insight: **the extra char (C2) fully determines block 0** when combined with knowing B2, B3 from the raw passthrough.

## Comparison to 4-Byte Passthrough

| Aspect | 4-byte (`,`) | 5/6/7-byte (`;_~`) |
|--------|--------------|---------------------|
| Length vs Z85 | Same | 1 char shorter |
| Extra char for partial | No | Yes |
| Ambiguity | Yes (need canonical min) | No |
| Position constraints | Canonical minimum | Any position |
| Implementation complexity | High | Lower |

## Decoder Behavior

1. See `;` at position P+1 within output → 5-byte passthrough
2. The P+1 chars before `;` are the first P+1 Z85 digits of the before block
3. Take next 5 bytes as literal output
4. Continue decoding from position P+1+1+5

The decoder can fully reconstruct the before block because it has P+1 chars (enough to eliminate ambiguity), plus it knows some bytes from the raw passthrough that overlap with the before block.

## Safe Character Set

Same as for 4-byte passthrough:
```
0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_
```

The escape characters `;_~` are included in the safe set, so they can appear within raw passthrough data.

## Future: 8+ Byte Escapes

A scheme for 8+ byte passthrough will be added later. Details TBD.
