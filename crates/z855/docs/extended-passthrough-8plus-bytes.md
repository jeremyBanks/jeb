# Extended Passthrough: 8+ Byte Escapes with `|`

> **Note:** This document represents the current best understanding of the planned encoding scheme. Details may be incorrect or outdated.

## Overview

For raw passthrough of 8 or more bytes, we use the `|` escape character with a variable-length prefix encoding. This is fundamentally different from the fixed-length escapes (`,;_~`):

| Escape | Bytes | Prefix | Structure |
|--------|-------|--------|-----------|
| `,` | 4 | None | `,[4 raw bytes]` |
| `;` | 5 | None | `;[5 raw bytes]` |
| `_` | 6 | None | `_[6 raw bytes]` |
| `~` | 7 | None | `~[7 raw bytes]` |
| `\|` | 8+ | Variable-length | `[prefix]\|[raw bytes][padding]\|` |

## The Variable-Length Prefix

### Encoding Scheme

The prefix encodes the raw byte count using a base-42 variable-length integer with continuation bits:

- Each prefix digit is a Z85 character with value 0-83 (not 84)
- Values 0-41: **terminal digit** (no continuation)
- Values 42-83: **continuation digit** (subtract 42 to get the base-42 value, continue reading)

### Reading the Prefix (Decoder)

The decoder reads the prefix **backwards** from the `|`:

1. See `|` in the input stream
2. Look at the immediately preceding character (value V₀)
3. If V₀ < 42: This is the only prefix digit. Length = V₀. Done.
4. If V₀ >= 42: This digit contributes (V₀ - 42) as the **least significant** base-42 digit. Continue.
5. Look at the next preceding character (value V₁)
6. If V₁ < 42: This is the final (most significant) digit. Done.
7. If V₁ >= 42: Continue reading backwards...

### Interpreting the Value (Big-Endian)

Although we read digits backwards (least significant first), we interpret them in **big-endian** order.

**Example:** Stream contains `ABC|` where:
- A has Z85 value 10
- B has Z85 value 45 (45 >= 42, so continuation)
- C has Z85 value 50 (50 >= 42, so continuation)

Reading backwards from `|`: C, B, A
- C: 50 >= 42 → contributes (50 - 42) = 8 as least significant digit, continue
- B: 45 >= 42 → contributes (45 - 42) = 3, continue
- A: 10 < 42 → terminal, contributes 10 as most significant digit

**Value = 10 × 42² + 3 × 42 + 8 = 17,640 + 126 + 8 = 17,774 bytes**

### Prefix Length Examples

| Length (bytes) | Prefix digits | Example prefix value breakdown |
|----------------|---------------|-------------------------------|
| 0-41 | 1 | Direct value |
| 42-1,763 | 2 | V₁ × 42 + V₀ |
| 1,764-74,087 | 3 | V₂ × 42² + V₁ × 42 + V₀ |
| 74,088-3,111,695 | 4 | V₃ × 42³ + V₂ × 42² + V₁ × 42 + V₀ |

Maximum with 4 blocks of buffering: well beyond 2⁵³, effectively unlimited.

## Length Value Semantics

### Special Values

| Length | Meaning |
|--------|---------|
| 0 | **"Rest of input is raw"** - Special case, see below |
| 1-7 | **Decoding error** - Use `,;_~` escapes for these |
| 8+ | That many raw bytes follow |

### The Zero-Length Special Case (`0|`)

When the length prefix is 0, it means **all remaining input is passed through raw**:

```
...preceding encoded data...0|raw bytes until end of input
```

**Critical property:** This is the **only** case where the encoded output can be **shorter** than standard Z85 encoding. There's no need to pad to the standard length when there's no following content.

**Example:** Input is 100 bytes of safe characters.
- Standard Z85: 125 characters
- With `0|`: 2 + 100 = 102 characters (shorter!)

## Output Structure for 8+ Bytes

### General Form

```
[prefix digits][|][raw bytes][padding][|]
```

- **Prefix digits:** Minimal encoding of the length (base-42 with continuation)
- **First `|`:** Marks end of prefix
- **Raw bytes:** Exactly `length` bytes of literal data
- **Padding:** Period (`.`) characters to maintain length invariant
- **Final `|`:** Aesthetic terminator for padding (may change in future)

### Padding

Padding ensures the total output length equals standard Z85 encoding length.

**Calculation:**
- Standard Z85 for N bytes = ⌈N × 5/4⌉ characters
- Our encoding uses: prefix_len + 1 + raw_len + padding_len + 1
- padding_len = ⌈N × 5/4⌉ - prefix_len - 1 - raw_len - 1

**Padding format:** `.....` followed by `|`

**Note:** Padding is purely aesthetic and ignored by the decoder. It may change in future versions. For some input sizes (e.g., 8 bytes), there is no room for padding.

### Length Examples

| Raw bytes | Z85 length | Prefix | First `\|` | Raw | Padding | Final `\|` | Total |
|-----------|------------|--------|------------|-----|---------|------------|-------|
| 8 | 10 | 1 | 1 | 8 | 0 | 0 | 10 ✓ |
| 9 | 12 | 1 | 1 | 9 | 0 | 1 | 12 ✓ |
| 10 | 13 | 1 | 1 | 10 | 0 | 1 | 12 ✗ |

Wait, let me recalculate...

For 10 raw bytes:
- Standard Z85: ⌈10 × 5/4⌉ = ⌈12.5⌉ = 13 chars
- Our encoding: 1 (prefix) + 1 (`|`) + 10 (raw) = 12 chars
- Padding needed: 13 - 12 = 1 char (just the final `|`)

| Raw bytes | Z85 length | Prefix | `\|` | Raw | Available for padding |
|-----------|------------|--------|------|-----|-----------------------|
| 8 | 10 | 1 | 1 | 8 | 0 (exact fit) |
| 9 | 12 | 1 | 1 | 9 | 1 |
| 10 | 13 | 1 | 1 | 10 | 1 |
| 20 | 25 | 1 | 1 | 20 | 3 |
| 42 | 53 | 1 | 1 | 42 | 9 |
| 43 | 54 | 2 | 1 | 43 | 8 |
| 100 | 125 | 2 | 1 | 100 | 22 |

## Decoder Algorithm

### Buffering Requirements

The decoder needs to buffer up to **4 blocks** (20 characters) to handle the variable-length prefix. This is because:
- Prefix can span multiple Z85 blocks
- Decoder must see the `|` before it knows how to interpret preceding characters

### Decoding Steps

1. **Buffer input** until a complete block boundary or `|` is encountered
2. **When `|` is seen:**
   a. Read prefix backwards from `|` using the base-42 continuation scheme
   b. Get length L
   c. If L == 0: Output all remaining input as raw, done
   d. If L is 1-7: Decoding error
   e. If L >= 8: Take next L bytes as raw output
   f. Compute and skip padding
   g. Continue decoding

### Computing Bytes Represented by Prefix

The prefix digits themselves represent some input bytes (since they occupy space in the Z85 output). For P prefix digits:
- Those P characters represent some input bytes that are "absorbed" into the escape sequence
- The decoder accounts for this when computing the total input consumed

## Encoder Algorithm

### Limits

- **Maximum decodable length:** 2⁵³ - 1 (JavaScript MAX_SAFE_INTEGER)
- **Maximum encoder segment:** 64 KiB (implementation choice to limit encoder memory)

### Canonical Encoder Behavior

Both Rust and TypeScript encoders must produce **identical output** for all inputs.

**Processing safe byte runs:**

1. Accumulate consecutive safe bytes
2. If run < 8 bytes: Use `,;_~` escapes or standard Z85
3. If run >= 8 bytes:
   - If at end of input: Use `0|` (rest is raw)
   - If run reaches 64 KiB limit:
     - Peek ahead: if at end of input → use `0|`
     - Otherwise → emit length-prefixed escape for 64 KiB, continue
   - Otherwise: Continue accumulating or emit when run ends

**Preference order (longest/most efficient first):**
1. `0|` (rest is raw) - when applicable
2. `|` with length prefix (8+ bytes)
3. `~` (7 bytes)
4. `_` (6 bytes)
5. `;` (5 bytes)
6. `,` (4 bytes, block-aligned)
7. `,` (4 bytes, non-aligned with canonical minimum)
8. Standard Z85

### Prefix Digit Generation

To encode length L as prefix digits:

1. If L < 42: Output single digit with value L
2. Otherwise:
   - Extract base-42 digits: d₀ = L % 42, d₁ = (L / 42) % 42, etc.
   - Output most significant digit as-is (value < 42)
   - Output remaining digits with +42 (continuation bit)

**Example:** L = 100
- 100 = 2 × 42 + 16
- Digits (big-endian): 2, 16
- Encoded: digit with value 2, digit with value (16 + 42) = 58
- Stream: `[char for 2][char for 58]|`

## Safe Character Set

Same as other escapes:
```
0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_
```

All escape characters (`,;_~|`) and padding character (`.`) are in the safe set.

## Examples

### Example 1: 8 bytes (exact fit, no padding)

Input: `abcdefgh` (8 safe bytes)

- Standard Z85: 10 chars
- Prefix: value 8 → single digit (8 < 42) → Z85 char for 8
- Output: `8|abcdefgh` (1 + 1 + 8 = 10 chars) ✓

### Example 2: 20 bytes (with padding)

Input: `abcdefghijklmnopqrst` (20 safe bytes)

- Standard Z85: 25 chars
- Prefix: value 20 → single digit → Z85 char for 20
- Raw: 20 bytes
- Used: 1 + 1 + 20 = 22 chars
- Padding: 25 - 22 = 3 chars → `...|`
- Output: `[20]|abcdefghijklmnopqrst...|` (25 chars) ✓

### Example 3: 100 bytes (multi-digit prefix)

Input: 100 safe bytes

- Standard Z85: 125 chars
- Prefix: value 100 = 2×42 + 16
  - First digit: 2 (terminal)
  - Second digit: 16 + 42 = 58 (continuation)
  - Stream order: `[char 2][char 58]|`
- Used: 2 + 1 + 100 = 103 chars
- Padding: 125 - 103 = 22 chars → `......................|`

### Example 4: Rest-of-input (`0|`)

Input: 50 safe bytes at end of file

- Standard Z85: 63 chars
- With `0|`: 2 + 50 = 52 chars (shorter!)
- No padding needed (nothing follows)

## Interaction with Block Alignment

The `|` escape can appear at any position. The decoder buffers until it sees `|`, then interprets the preceding characters as prefix digits.

After processing a `|` escape (raw bytes + padding), the encoder/decoder continues from the next position, which may or may not be block-aligned. Subsequent escapes handle alignment as documented in other specs.

## Future Considerations

- Padding format (`.` characters, final `|`) is aesthetic and may change
- The 64 KiB encoder limit is an implementation choice, not a protocol limit
- Decoders should accept any valid length up to 2⁵³ - 1
