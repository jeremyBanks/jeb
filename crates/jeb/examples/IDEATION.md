# Extended Z85 with Raw Passthrough - Design Notes

This document describes extensions to Z85 encoding that allow raw (unencoded) byte sequences to pass through while maintaining the standard Z85 overhead guarantees.

## Core Principle

Standard Z85 encoding has +25% overhead (4 bytes → 5 chars). Our extensions maintain this overhead by using escape characters that mark transitions between encoded and raw data.

## Escape Characters

We use characters outside Z85's alphabet to mark raw passthrough:

- `` ` `` (backtick) - marks raw bytes with **big-endian** prefix interpretation
- `,` (comma) - marks raw bytes with **little-endian** prefix interpretation

Visual mnemonic: backtick is "high" (big-endian), comma is "low" (little-endian).

## 4-Byte Raw Sequences

The escape character can appear at positions 0-3 within a 5-char block:

| Form | Prefix bytes | Raw bytes | Total | Output chars |
|------|--------------|-----------|-------|--------------|
| `` `abcd`` | 0 | 4 | 4 | 5 |
| ``A`bcd`` | 1 | 3 (+1 in next block) | 4 | 5 |
| ``AB`cd`` | 2 | 2 (+2 in next block) | 4 | 5 |
| ``ABC`d`` | 3 | 1 (+3 in next block) | 4 | 5 |

Same for `,` with little-endian interpretation.

### Prefix Encoding

The Z85 characters before the escape encode the **low-order bits** of the prefix bytes, with high-order bits implicitly zero. This maximizes coverage for zero-heavy data.

- ``A`bcd``: A's digit value (0-84) is byte 0 directly
- ``AB`cd``: Decode as base-85 u16 < 7225 (85²)
- ``ABC`d``: Decode as base-85 u24 < 614125 (85³)

### Coverage

For random data:
- 1 byte: ~33% (values < 85)
- 2 bytes: ~11% (values < 7225)
- 3 bytes: ~3.7% (values < 614125)

For zero-heavy data (small integers, padding, null-terminated strings), coverage is much higher.

### Endianness Choice

**Big-endian (`` ` ``)** works well for:
- Leading zeros (padding at start)
- Network byte order integers
- Big-endian file formats

**Little-endian (`,`)** works well for:
- ASCII text with null delimiters (e.g., `[0x48, 0x00]` for "H\0")
- UTF-16LE text
- Trailing zeros (padding at end)
- x86/ARM little-endian integers

Example: `[0x48, 0x00, 0x65, 0x00]` ("H\0e\0")
- Big-endian: 0x4800 = 18,432 > 7225 ✗
- Little-endian: 0x0048 = 72 < 7225 ✓

## Block Structure and State

The stream is divided into **fixed 5-char blocks**. Block boundaries never change—this ensures that changes to one block don't cascade to following blocks.

When an escape marker indicates more raw bytes than remain in the current block, the raw bytes continue into the next block. The decoder tracks "raw bytes remaining" as state across blocks.

Example:
```
Block 1 (chars 0-4): AB`cd
  - Decode AB → 2 prefix bytes (must be < 7225)
  - ` signals: next 4 bytes in output are raw
  - cd = 2 raw bytes
  - State: 2 raw bytes remaining

Block 2 (chars 5-9): efGHI
  - ef = 2 raw bytes (satisfies remaining count)
  - GHI = 3-char partial block → 2 encoded bytes
  - State: 0 raw bytes remaining
```

## Partial Block Padding

Partial blocks (< 4 bytes) are padded at the **beginning** (high-order positions) with zeros. This means:
- `GHI` (3 chars encoding 2 bytes) treats the bytes as `[0, 0, byte0, byte1]`
- The encoded characters map to the low-order bytes

**Advantage:** If the encoding decision changes (raw vs encoded), the same bytes will produce the same trailing characters, because they occupy the low-order positions.

## Encoder Strategy

The encoder uses a lookahead buffer (~64 bytes recommended) and:

1. **Maximize raw block length** - choose the escape form that yields the longest contiguous raw passthrough
2. **Tiebreaker: prefer earlier positions** - if multiple forms give equal raw length, prefer position 0 > 1 > 2 > 3

This gives predictable/canonical output and better alignment properties.

## Extended Sequences (5-7 bytes) - TENTATIVE

We have 6 available escape characters. Two are allocated (`` ` `` and `,`). The remaining 4 could be used for:

- Character for 5-byte raw sequences
- Character for 6-byte raw sequences
- Character for 7-byte raw sequences
- Character for 8+ byte extended escapes (two-character escape sequence)

### Open Questions

1. **Position support:** Can these work at any position (not just 0)? The same cross-block continuation logic should apply.

2. **Endianness:** Single choice (LE?) or support both? For long raw sequences, endianness may be irrelevant since we're mostly bypassing encoding anyway.

3. **Character count:**
   - 5 bytes → 6 chars (1 escape + 5 raw)
   - 6 bytes → 7 chars (1 escape + 6 raw)
   - 7 bytes → 8 chars (1 escape + 7 raw)

   These match standard Z85 overhead (n bytes → n+1 chars for partial blocks).

## Invariants

- Overhead matches or improves upon standard Z85
- Fixed block boundaries (5 chars) never change
- No escape character appears in standard Z85 output
- Length always determined by escape marker position
- Raw content unrestricted (can contain any byte, including escape characters)
- Cross-block continuation uses simple "bytes remaining" counter

## Character Safety Notes

Current escape characters are chosen to minimize conflicts:

- `` ` ``: Unsafe in shell `${}` context and markdown inline code, but Z85 has same issue
- `,`: Generally safe
- `~`, `;`, `_`, `|`: Available for future use
- Space: Reluctantly available if needed, but breaks monospace alignment

We avoid other whitespace (tabs, newlines) to preserve alignment properties.
