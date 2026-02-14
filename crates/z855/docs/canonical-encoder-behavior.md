# Canonical Encoder Behavior

> **Note:** This document describes the *canonical* encoder behavior for the Rust and TypeScript implementations. These are design choices to ensure both encoders produce **identical output** for the same input. **The decoder does not enforce these choices** - any valid encoding that satisfies the length invariant will decode correctly.

## Overview

Extended Z855 allows multiple valid encodings for the same input. For example, 8 safe bytes could be encoded as:
- Standard Z85 (10 chars)
- Two 4-byte passthroughs: `,[4 bytes],[4 bytes]` (10 chars)
- One 8-byte passthrough: `8|[8 bytes]` (10 chars, assuming block-aligned)

All produce valid output that decodes to the same bytes. The canonical encoder makes deterministic choices so that:
1. Rust and TypeScript produce identical output
2. Output is reasonably compact
3. Human-readable portions are preserved when possible

## Position Preference: Bit-Reversal Sort

When multiple positions are valid for an extended passthrough (5/6/7-byte escapes), the encoder prefers positions that maximize alignment to power-of-2 boundaries.

### Algorithm

For each candidate position `P` for a K-byte passthrough starting at block offset:

1. Compute `start` = absolute byte offset where passthrough begins
2. Compute `end` = start + K - 1 (last byte of passthrough)
3. Compute `rev_start` = bit_reverse(start)
4. Compute `rev_end` = bit_reverse(end)
5. Sort key = `(min(rev_start, rev_end), max(rev_start, rev_end))`
6. Try positions in ascending sort key order

### Why Bit Reversal?

Positions aligned to power-of-2 boundaries have trailing zeros in binary:
- Position 0: `0b...0000` (aligned to everything)
- Position 4: `0b...0100` (aligned to 4)
- Position 8: `0b...1000` (aligned to 8)
- Position 3: `0b...0011` (not aligned)

Bit reversal turns trailing zeros into leading zeros:
- `bit_reverse(0)` = 0 (smallest)
- `bit_reverse(4)` = has leading zeros
- `bit_reverse(3)` = has leading ones (larger)

So aligned positions naturally sort first without hardcoding specific offsets.

### Example

For a 7-byte passthrough (`~`) at input position 5:
- Candidate positions P = 0, 1, 2, 3 (within the Z85 block)
- For P=1: start=5, end=11, sort_key = (bit_rev(5), bit_rev(11))
- For P=2: start=6, end=12, sort_key = (bit_rev(6), bit_rev(12))
- Position 12 has more trailing zeros than 11, so P=2 might be preferred

The encoder tries positions in sort key order until one satisfies the length invariant.

## Escape Preference Order

When encoding a run of safe bytes, the encoder tries escapes in this order:

1. **`0|` (rest-of-input)** - When safe bytes extend to end of input. Only case where output can be shorter than standard Z85.

2. **`|` with length prefix (8+ bytes)** - For long runs of safe bytes not at end of input.

3. **`~` (7 bytes)** - Extended passthrough, 1 char shorter than standard Z85.

4. **`_` (6 bytes)** - Extended passthrough, 1 char shorter than standard Z85.

5. **`;` (5 bytes)** - Extended passthrough, 1 char shorter than standard Z85.

6. **`,` block-aligned (4 bytes)** - At position 0 of output block, no ambiguity.

7. **`,` non-aligned (4 bytes)** - At position 1-4, requires canonical minimum.

8. **Standard Z85** - Fallback when no passthrough applies.

## Length Invariant

The canonical encoder maintains the length invariant:

```
encode(input).length == ceil(input.length * 5 / 4)
```

Exception: `0|` at end of input can produce shorter output.

When a passthrough escape would violate this invariant (produce more characters than standard Z85), the encoder skips it and tries the next option.

## Fallback Behavior

If a longer escape doesn't fit (e.g., 6-byte passthrough violates length invariant), the encoder tries shorter escapes:

- Try `_` (6 bytes) → doesn't fit
- Try `;` (5 bytes) → doesn't fit
- Try `,` (4 bytes) → fits, use it
- Or fall back to standard Z85

This ensures maximum passthrough usage while respecting the length invariant.

## Decoder Tolerance

The decoder accepts ANY valid encoding:
- Different position choices
- Different escape choices
- Missing or extra padding (for `|` escape)
- Non-canonical minimum selections (for `,` at non-aligned positions)

The decoder does not validate that the encoder made "optimal" choices. It simply interprets whatever escape sequences are present.

## Implementation Notes

### Rust
- `bit_reverse()` uses `u64::reverse_bits()`
- Position candidates sorted with `sort_by_key()`

### TypeScript
- `bitReverse()` uses BigInt for 64-bit precision
- Standard bit manipulation pattern for reversal

Both implementations must produce identical output for cross-language compatibility.
