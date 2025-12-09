# JEB85 Specification

## Overview

JEB85 is a binary-to-text encoding that opportunistically preserves ASCII-safe text as literal characters while encoding binary data using Z85 (base-85). This creates "semitranslucent" encodings where human-readable text remains readable.

## Character Set

### Z85 Digits (85 characters)
```
0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#
```

Each digit represents a value 0-84.

### Special Characters (3 characters)
- `_` (underscore) - Prefix for exactly 4 bytes of raw data
- `~` (tilde) - Prefix for exactly 6 bytes of raw data
- `|` (pipe) - Look-back length-prefixed raw sequences

### ASCII-Safe Characters (95 characters)
Characters that can appear as literal raw data:
```
 !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~
```

Note: Space (0x20) through tilde (0x7E), excluding control characters and excluding tab/newline/carriage return.

## Block Structure

JEB85 processes data in 4-byte blocks, encoding each block as 5 characters. This maintains consistent alignment and predictable output length.

### Standard Z85 Block Encoding
A 4-byte block is interpreted as a big-endian 32-bit integer and encoded in base-85:

```
Input:  [b₃, b₂, b₁, b₀]  (4 bytes)
Value:  v = (b₃ << 24) | (b₂ << 16) | (b₁ << 8) | b₀
Output: 5 Z85 digits representing v in base-85
```

For blocks with fewer than 4 bytes, pad with zeros on the right and use fewer output digits:
- 1 byte → 2 digits
- 2 bytes → 3 digits
- 3 bytes → 4 digits
- 4 bytes → 5 digits

## Raw ASCII Encoding

JEB85 uses three different strategies for encoding consecutive ASCII-safe bytes, choosing the most efficient based on length.

### Fixed 4-Byte Sequences: `_`

When exactly 4 consecutive ASCII-safe bytes are encountered:
```
Format: _xxxx

Example:
Input:  "test" (4 bytes)
Output: "_test" (5 chars)
```

This is equivalent in length to Z85 encoding but preserves readability.

### Fixed 6-Byte Sequences: `~`

When exactly 6 consecutive ASCII-safe bytes are encountered:
```
Format: ~xxxxxx

Example:
Input:  "hello!" (6 bytes)
Output: "~hello!" (7 chars)
```

This saves 1 character compared to Z85 (which would use 8 chars for 6 bytes).

### Variable-Length Sequences: Look-Back `|`

For longer ASCII sequences where look-back encoding is beneficial:

```
Format: [D₁...Dₙ]|[data bytes...]

Where:
- D₁...Dₙ are Z85 digits encoding the byte count in base-85
- The last digit Dₙ has a special constraint
- The number of digits is encoded in the last digit's low 2 bits
```

#### Look-Back Decoding Algorithm

When the decoder encounters `|` after Z85 digits:

1. Read the last Z85 digit Dₙ before the `|`
2. Decode Dₙ to get value V (0-84)
3. Compute N = V & 3 (low 2 bits, gives 0-3)
4. Read N additional digits backwards (total of N+1 digits)
5. Decode all N+1 digits as a big-endian base-85 number = length L
6. Read L raw bytes following the `|`

#### Look-Back Encoding Constraint

For N digits encoding length L, the encoder must satisfy:
```
last_digit_value & 3 == (N - 1)
```

This constraint partitions the encoding space by modulo 4, allowing the decoder to determine how many digits to read.

#### Look-Back Examples

```
Example 1: 8 bytes
  N=1 digit needed (8 fits in 1 digit)
  Need: last_digit & 3 == 0
  Z85[8] = '8', and 8 & 3 = 0 ✓
  Output: "8|testtest" (10 chars)

Example 2: 100 bytes
  100 in base-85 = 1×85 + 15 = "1e" or "1f" or "1g" or "1h"
  N=2 digits needed
  Need: last_digit & 3 == 1
  Z85[15] = 'e', 15 & 3 = 3 ✗
  Z85[16] = 'f', 16 & 3 = 0 ✗
  Z85[17] = 'g', 17 & 3 = 1 ✓
  Output: "1g|" + 100 bytes (103 chars)
```

### Chunked Encoding for Other Lengths

For lengths that don't benefit from look-back encoding, JEB85 uses chunked encoding:

```
8 bytes:  "_test_test" (10 chars) using two 4-byte chunks
10 bytes: "_test~hello!" (17 chars) using 4-byte + 6-byte chunks
12 bytes: "~hello~world!" (14 chars) using two 6-byte chunks (or three 4-byte)
```

The encoder chooses the optimal chunking strategy to minimize output length.

## Mid-Block Transitions

When a 4-byte block has a mix of binary and ASCII, JEB85 can use mid-block transitions:

```
Format: [Z85 prefix]|[ASCII suffix]

Total length: exactly 5 characters

Example:
Input:  [0x05, 'a', 'b', 'c']
Output: "5|abc"
        ↑   ↑↑↑
        │   └── 3 ASCII bytes
        └────── 1 byte (value 5) encoded as Z85 '5'
```

Mid-block transitions require:
1. First N bytes (1-3) encode compactly in Z85
2. Remaining 4-N bytes are ASCII-safe
3. Total output is exactly 5 characters

This preserves partial readability while maintaining block alignment.

## Encoding Algorithm

```
For each 4-byte block in the input:
  1. Check if all 4 bytes are ASCII-safe
  2. If yes, accumulate in raw_buffer
  3. If no:
     a. Try mid-block transition if block has ASCII suffix
     b. If mid-block fails, flush raw_buffer and encode block as Z85
  4. At end of data or when switching to binary, flush raw_buffer:
     - Exactly 4 bytes → use _xxxx
     - Exactly 6 bytes → use ~xxxxxx
     - Other lengths:
       * Try look-back encoding
       * If not beneficial, use chunked encoding (_, ~, or Z85)
```

## Decoding Algorithm

```
Parse character by character:
  1. If char is '_':
     - Read next 4 bytes as raw ASCII

  2. If char is '~':
     - Read next 6 bytes as raw ASCII

  3. If char is Z85 digit:
     - Collect consecutive Z85 digits (up to 5)
     - Check what follows:
       * If '|':
         + Check if mid-block transition (5 total chars)
         + Otherwise, decode as look-back length encoding
       * Otherwise:
         + Decode as standard Z85 block (5 digits → 4 bytes)

  4. Standalone '|':
     - Error: invalid encoding
```

## Examples

### Example 1: Short ASCII (4 bytes)
```
Input:  "test"
Output: "_test"
Length: 5 chars (same as Z85, but readable)
```

### Example 2: Medium ASCII (6 bytes)
```
Input:  "hello!"
Output: "~hello!"
Length: 7 chars (vs 8 for Z85, saves 12.5%)
```

### Example 3: Long ASCII (100 bytes)
```
Input:  100 bytes of ASCII text
Output: "1g|" + 100 bytes of text
Length: 103 chars (vs 125 for Z85, saves 17.6%)
```

### Example 4: Mixed Binary and ASCII
```
Input:  [0xFF, 0xFF, 0xFF, 0xFF, 't', 'e', 's', 't']
Output: "#####_test"
        ←Z85→ ←─4─→
Length: 10 chars
```

### Example 5: Mid-Block Transition
```
Input:  [0x05, 'a', 'b', 'c']
Output: "5|abc"
Length: 5 chars (maintains block alignment)
```

### Example 6: Multiple Chunks
```
Input:  "hello world!" (12 bytes)
Output: "~hello~world!"
        ←──6──→←──6──→
Length: 14 chars (vs 15 for Z85)
```

## Encoding Efficiency

**Space efficiency** compared to standard Z85:

| ASCII Bytes | JEB85 Output | Z85 Output | Savings |
|-------------|--------------|------------|---------|
| 4           | 5 chars      | 5 chars    | 0%      |
| 6           | 7 chars      | 8 chars    | 12.5%   |
| 8           | 10 chars     | 10 chars   | 0%      |
| 12          | 14 chars     | 15 chars   | 6.7%    |
| 16          | 17 chars     | 20 chars   | 15%     |
| 100         | 103 chars    | 125 chars  | 17.6%   |

**For typical text files:** 30-40% of data can be preserved as literal ASCII, reducing overall encoded size by 8-15% while maintaining readability.

## Constraints and Edge Cases

1. **Look-back constraint:** For N digits, last digit value & 3 must equal (N-1)
2. **Mid-block detection:** Exactly 5 total characters and compact Z85 prefix
3. **Chunking optimization:** Encoder chooses between _, ~, look-back, and Z85 based on efficiency
4. **Minimum lengths:**
   - `_` used only for exactly 4 bytes
   - `~` used only for exactly 6 bytes
   - Look-back used when beneficial over chunking

5. **Ambiguity resolution:**
   - `_` and `~` are always followed by fixed-length data
   - `|` after Z85 digits distinguishes mid-block from look-back by checking compact encoding validity
   - Standard Z85 blocks never contain `_`, `~`, or `|`

## Maximum Encodable Lengths

With look-back encoding:
- 1 digit: up to 84 bytes
- 2 digits: up to 7,224 bytes (85² - 1)
- 3 digits: up to 614,124 bytes (85³ - 1)
- 4 digits: up to 52,200,624 bytes (85⁴ - 1)
- 5 digits: up to ~4.4 billion bytes

## Compatibility

JEB85 is **not** a superset of Z85 due to the special meaning of `_`, `~`, and `|`:
- Z85-encoded data containing these characters may decode differently in JEB85
- JEB85-encoded data without `_`, `~`, or `|` may be valid Z85
- For safety, always specify which encoding is being used

## Implementation Notes

### Encoder Optimizations

1. **Chunking Strategy**: When flushing raw buffer, prefer:
   - Multiples of 4: use all `_` prefixes
   - Multiples of 6: use all `~` prefixes
   - Mixed: compare chunking vs look-back and choose optimal

2. **Look-Back Search**: Try increasing digit counts until constraint is satisfied
   - Start with minimum digits needed for the value
   - Increment until `last_digit & 3 == (num_digits - 1)`

3. **Mid-Block Priority**: Always check mid-block before flushing buffer
   - Provides optimal compression for mixed blocks
   - Maintains 5-char block alignment

### Decoder Robustness

1. **Validate lengths**: Check sufficient bytes available before reading
2. **Distinguish patterns**: Mid-block vs look-back vs standard Z85
3. **Handle partial blocks**: Final block may have fewer than 5 Z85 digits
