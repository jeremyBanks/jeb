# JEB85 Design

## Goals

Opportunistic semitranslucent binary-to-text encodings that:
1. Preserve human-readable ASCII text as literal characters
2. Encode binary data efficiently using base-85
3. Maintain predictable block alignment (4 bytes → 5 characters)
4. Support arbitrary-length ASCII sequences with minimal overhead

## Base

85 (base-85 encoding)

## Digits

Z85 alphabet (86 total characters):
```
0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#
```

## Special Characters

- `|` - Raw data delimiter and count prefix
- `_` - Reserved for future use
- `~` - Reserved for future use

## Encoding Scheme

### Simple Raw Blocks (≤5 bytes)
```
|test    → 4 bytes of raw ASCII
```

### Length-Prefixed Raw (6+ bytes)
```
X|[data...]

where X is a Z85 digit encoding the count:
- If X ≥ 8: Direct count (8-84 bytes)
- If X = 0-7: Indirect count (read X more Z85 digits, decode as base-85)
```

### Examples

**Direct count:**
```
8|hello wo               → 8 bytes
b|hello world            → 11 bytes (b=11 in Z85)
```

**Indirect count (for counts outside 8-84 range):**
```
2|1f[100 bytes of text]  → read 2 Z85 digits "1f" = 100
3|abc[large count]       → read 3 Z85 digits
```

### Zero-Padding

When byte count isn't divisible by 4, the final block is zero-padded:
```
6|hello!
  → "hell" + "o!" + 0x00 + 0x00
```

This elegantly handles:
- Null-terminated strings
- Partial blocks
- Any non-aligned ASCII sequence

## Why This Works

**Character budget:**
```
Simple: |test = 5 chars (no savings)
Length: 8|testtest = 10 chars (break even)
Length: b|hello world = 13 vs 14 chars Z85 (saves 1)
```

**Minimum 6 bytes** ensures we don't use length-prefix when it's not beneficial.

**The 0-7 trick** allows unlimited length encoding:
- 1 digit: up to 84 bytes
- 2 digits: up to 7,224 bytes
- 3 digits: up to 614K bytes
- 7 digits: up to 2.8 billion bytes

## Decision Points

### Why 6-byte minimum?
Below 6 bytes, the overhead of encoding length equals or exceeds savings.

### Why look-back for `|`?
Allows decoder to determine context:
- Z85_digit + `|` → length-prefixed raw
- `|` alone → simple 4-byte raw
- Z85_digit + Z85_digit → standard Z85

### Why zero-padding?
Handles null terminators without special cases. The decoder just reads N bytes and stops, unused bytes in the block are implicitly zero.

## Data Analysis Results

From analyzing real binaries:
- **17.29%** of blocks in jeb binary could use raw encoding
- **1.84%** additional blocks have ASCII + trailing nulls (now handled!)
- **34.78%** of text file blocks are consecutive ASCII
- **Space savings:** 8-16% for typical mixed content

## Future Extensions

Reserved characters `~` and `_` could enable:
- Bit-level partial blocks (even finer granularity)
- Compression mode indicators
- Alternative character sets
- Metadata embedding

