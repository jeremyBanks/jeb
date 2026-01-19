# Extended Z85 with Raw Passthrough - Design Notes

This document describes extensions to Z85 encoding that allow raw (unencoded) byte sequences to pass through while maintaining the standard Z85 overhead guarantees.

## Core Principle

Standard Z85 encoding has +25% overhead (4 bytes → 5 chars). Our extensions maintain this overhead by using escape characters that mark transitions between encoded and raw data.

**Critical invariant:** Following content must not be affected by the presence of raw content before it. Raw bytes use 1:1 character mapping, so they "save" space compared to encoded bytes. Padding is inserted after raw sequences to maintain the +25% overhead and preserve alignment.

## Escape Characters

We use 6 characters outside Z85's alphabet:

- `,` (comma)
- `` ` `` (backtick)
- `;` (semicolon)
- `~` (tilde)
- `_` (underscore)
- `|` (pipe)

These characters have **position-dependent meanings** within 5-character blocks.

## Position-Dependent Escape Meanings

### At Beginning of Block (Position 0)

No prefix bytes can exist before the escape, so no endianness interpretation is needed:

| Escape | Raw bytes following |
|--------|---------------------|
| `` ` `` | 3 bytes |
| `,` | 4 bytes |
| `~` | 5 bytes |
| `;` | 6 bytes |
| `_` | 7 bytes |

### In Middle of Block (Positions 1-3)

Prefix bytes exist before the escape and must be interpreted with endianness:

(Position 4 is invalid - escape characters cannot appear at the last position of a block)

| Escape | Raw bytes | Prefix endianness | Notes |
|--------|-----------|-------------------|-------|
| `,` | 4 bytes | Little-endian (LE) | Preferred when all prefix bytes are zero |
| `` ` `` | 4 bytes | Big-endian (BE) | |
| `;` | 6 bytes | Little-endian (LE) | Preferred when all prefix bytes are zero |
| `~` | 6 bytes | Big-endian (BE) | |
| `_` | 7 bytes | Little-endian (LE) | No BE variant (ran out of chars) |

**Position 1 special case:** Only 1 prefix byte exists, so BE and LE are identical (single byte has no byte order).

### The `|` Escape (8+ bytes)

The `|` character works at any position and signals a **backward-looking length encoding**:

1. The decoder buffers up to one block (5 chars) to detect `|`
2. When `|` is encountered, read backwards up to 4 Z85 digits before it
3. These digits encode the length in a variable-length base-42 encoding (see below)
4. The digits that encoded the length are **reinterpreted as part of the raw sequence prefix** (not as encoded data)

## Prefix Encoding

### Standard Prefixes (3-7 byte escapes)

The Z85 characters before the escape encode the **low-order bits** of the prefix bytes, with high-order bits implicitly zero:

- 1 char: byte 0 = digit value (0-84)
- 2 chars: u16 BE or LE < 7,225 (85²)
- 3 chars: u24 BE or LE < 614,125 (85³)

This maximizes coverage for zero-heavy data.

### Coverage Rates

For uniformly random data:
- 1 byte: ~33% (values < 85)
- 2 bytes: ~11% (values < 7,225)
- 3 bytes: ~3.7% (values < 614,125)

For zero-heavy data (small integers, padding, null-delimited strings), coverage is much higher.

### Endianness Benefits

**Little-endian (`,`, `;`)** works well for:
- ASCII text with null delimiters: `[0x48, 0x00]` → LE: 0x0048 = 72 < 7,225 ✓
- UTF-16LE text
- Trailing zeros (padding at end)
- x86/ARM little-endian integers

**Big-endian (`` ` ``, `~`)** works well for:
- Leading zeros (padding at start)
- Network byte order integers
- Big-endian file formats

Example: `[0x48, 0x00, 0x65, 0x00]` ("H\0e\0")
- Big-endian: 0x4800 = 18,432 > 7,225 ✗
- Little-endian: 0x0048 = 72 < 7,225 ✓

## Block Structure and Cross-Block Continuation

The stream is divided into **fixed 5-char blocks**. Block boundaries never change.

When an escape indicates more raw bytes than remain in the current block, the raw bytes continue into the next block. The decoder tracks "raw bytes remaining" as state.

Example:
```
Block 1 (chars 0-4): AB,cd
  - Decode AB as LE u16 → 2 prefix bytes (must be < 7,225)
  - , signals: 4 raw bytes follow
  - cd = 2 raw bytes
  - State: 2 raw bytes remaining

Block 2 (chars 5-9): efGHI
  - ef = 2 raw bytes (satisfies remaining count)
  - GHI = 3-char partial block → 2 encoded bytes
  - State: 0 raw bytes remaining
```

## Partial Block Padding

Partial blocks (< 4 bytes) are padded at the **beginning** (high-order positions) with zeros:
- `GHI` (3 chars encoding 2 bytes) treats input as `[0, 0, byte0, byte1]`
- The encoded characters map to the low-order bytes

**Advantage:** If encoding decisions change, the same bytes produce the same trailing characters since they occupy low-order positions.

## Raw Sequence Padding

Raw bytes use 1:1 character mapping (more efficient than 5:4 encoding). This creates "extra space" that must be filled with padding to maintain the +25% overhead invariant:

Example:
- 100 raw bytes = 100 chars of output
- Standard Z85 would use: 125 chars
- Gap: 25 chars

The encoder inserts padding after raw sequences (possibly underscore characters, but decoder doesn't care about padding content) to maintain alignment so following content starts at the correct position.

## Variable-Length Encoding for `|` (8+ bytes)

The `|` escape uses a **backward-looking base-42 encoding** for the length.

### Algorithm

1. Read backwards from `|` up to 4 Z85 digits (stop at block boundary)
2. For each digit d (read right-to-left):
   - Subtract 1: `value = d - 1` (range 0-83 for digit 0-84)
   - If `value >= 42`: continuation flag is set, more digits follow
   - Extract length contribution: `value % 42` (range 0-41)
   - Accumulate in base-42: multiply previous total by 42, add contribution
3. Special case for first digit when not block-aligned: use only 21 values (0-20) for length, remaining bit encodes endianness (LE if < 21, BE if >= 21 after halving)

### Worked Example: Encoding Length 50

We want to encode a raw sequence of 50 bytes using `|`.

**Step 1: Convert 50 to base-42**
- 50 = 1×42 + 8
- Base-42 digits: [1, 8] (high-order to low-order)

**Step 2: Encode with continuation flags**

Physical layout before `|`: `[digit for 1][digit for 8]|`

The rightmost digit (immediately before `|`) is processed first during decoding:
- **Rightmost (8):** Must signal continuation (more digits to the left)
  - 8 + 42 (continuation flag) = 50
  - 50 + 1 (encoding offset) = 51 → Z85[51]

- **Left of rightmost (1):** Final digit, no continuation
  - 1 + 0 (no continuation) = 1
  - 1 + 1 (encoding offset) = 2 → Z85[2]

**Output:** `Z85[2]Z85[51]|` followed by 50 raw bytes

**Decoding:**
1. See `|`, read backward from position before `|`
2. First digit: Z85[51] → `51-1=50` → `50≥42` (continuation set) → contribution: `50%42=8` → accumulator = 8
3. Second digit: Z85[2] → `2-1=1` → `1<42` (no continuation, stop) → accumulator = `1×42 + 8 = 50`
4. Length is 50 bytes
5. Output literal bytes of Z85[2] and Z85[51] (first 2 of the 50 raw bytes)
6. Read next 48 bytes as raw

### Worked Example: Encoding Length 10 at Non-Block-Aligned Position

If `|` appears at position 3 (not block-aligned), we have prefix bytes and need to encode endianness.

Physical layout: `A B (Z85[11]) |` (positions 0,1,2,3)

Length 10, little-endian:
- 10 < 42 (single base-42 digit)
- For non-block-aligned first digit, split value to encode endianness:
  - Values 0-20 → little-endian
  - Values 21-41 → big-endian (after halving: `(value-21)` gives length contribution)
- For LE with length 10: use 10 directly
- Add encoding offset: `10 + 1 = 11` → Z85[11]

**Output:** `ABZ85[11]|` followed by remaining raw bytes

**Decoding:**
1. See `|` at position 3
2. Read backward: find Z85[11] at position 2, B at position 1, A at position 0
3. Decode Z85[11]: `11-1=10` → `10<42` (no continuation) → `10<21` (LE)
4. Length = 10 bytes, endianness = LE
5. Output literal bytes: A, B, Z85[11] as first 3 of the 10 raw bytes
6. Read next 7 bytes as raw

Note: The prefix bytes A and B are reinterpreted as part of the raw output, not as an encoded prefix value in this case.

### Special Length Values

- **Length 0:** Infinite length (until end of stream). Only used when encoder knows stream ends. No memory overhead for decoder.
- **Length 1-7:** Invalid (would use simpler escapes)
- **Length 8+:** Valid, represents number of raw bytes to follow

### Padding for Long Sequences

For lengths ≥ 12, padding is needed after the raw bytes to maintain alignment. The padding ensures following encoded content appears at correct block-aligned positions.

## Encoder Strategy

The encoder uses a lookahead buffer (recommended ~64 bytes, configurable for larger buffers when beneficial):

1. **Maximize raw block length** - choose the escape form that yields the longest contiguous raw passthrough
2. **Tiebreaker: prefer earlier positions** - if multiple forms give equal raw length, prefer position 0 > 1 > 2 > 3
3. **Infinite length:** Only use when stream end is known (large buffer or end-of-input)

## Decoder Algorithm

```
state = normal
raw_bytes_remaining = 0

for each 5-char block:
    if raw_bytes_remaining > 0:
        consume min(5, raw_bytes_remaining) chars as raw bytes
        raw_bytes_remaining -= chars consumed
        decode remaining chars (if any) as standard or escaped
    else:
        scan block for escape characters
        if escape found:
            decode prefix (if any)
            set raw_bytes_remaining based on escape type
            consume raw bytes from remainder of block
        else if standard Z85:
            decode 5 chars → 4 bytes
```

### Key Clarifications

**No reinterpretation in standard escapes (`,` `` ` `` `;` `~` `_`):**
The decoder scans for escape characters FIRST, before any Z85 decoding. When an escape is found:
- Characters before the escape are decoded as base-85 integers (not as a standard Z85 block)
- This is a different decoding path, not a reinterpretation

Example: `AB,cd`
- Scan block, find `,` at position 2
- Decode `AB` as base-85 integer: `A=10, B=11` → `10*85 + 11 = 861` → u16 little-endian bytes
- `,` signals 4 raw bytes follow
- `cd` provides 2 raw bytes
- 2 raw bytes continue in next block

**The `|` escape IS reinterpretation:**
Unlike other escapes, the characters before `|` serve a dual purpose:
1. Their Z85 digit VALUES encode the length (base-42 encoding)
2. Their BYTE representations become part of the raw output

Example: `ABC|` (simple case, not block-end)
- Read `ABC` backward, decode as base-42 to get length N
- Then output the literal bytes `A`, `B`, `C` as part of the N raw bytes
- The remaining (N-3) raw bytes follow in the stream

This is why it's called "reinterpretation" - the same characters are interpreted as both length metadata AND content.

**Position 4 is invalid:**
Escape characters cannot appear at position 4 (the last position in a 5-char block). This is an encoding error. Valid positions are 0-3 only.

## Invariants

1. **Overhead preserved:** All escape forms maintain or improve upon +25% overhead
2. **Fixed block boundaries:** 5-char blocks never change alignment
3. **Unambiguous escapes:** No escape character appears in standard Z85 output
4. **Length-determined:** No scanning needed; escape position/type determines length
5. **Unrestricted raw content:** Can contain any byte, including escape characters
6. **No cascading changes:** Following content unaffected by preceding raw sequences (via padding)

## Character Safety Notes

Escape characters are chosen to minimize conflicts:

- `,`: Generally safe
- `` ` ``: Unsafe in shell `${}` and markdown inline code (but Z85 has same issue)
- `;`, `~`, `_`, `|`: Generally safe in most contexts

Space character available if needed but avoided (breaks monospace alignment). No other whitespace used (tabs, newlines) to preserve alignment properties.
