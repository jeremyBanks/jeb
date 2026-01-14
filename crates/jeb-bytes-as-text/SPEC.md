# jeb-bytes-as-text Specification

[def jeb-bat @children]
The `jeb-bytes-as-text` crate provides basic binary text encoding logic with support for several different alphabets such as base64url and Z85.

---

## Generic Encoding

[def jeb-bat.generic @children]
Defines a generalized model for encoding binary data into text using an arbitrary alphabet. This model abstracts over common schemes like Base64, Base32, and Z85.

[def jeb-bat.generic.alphabet]
An alphabet is an ordered sequence of `N` distinct characters, where `N` is the base of the encoding. Each character represents a value from `0` to `N-1`.

The alphabet size `N` must be greater than 1. Any base greater than 1 is mathematically valid, though bases below 2 or above 256 have limited practical utility.

[def jeb-bat.generic.block]
Encoding operates on fixed-size blocks. This allows the encoding to process data in chunks rather than bit-by-bit.

An encoding scheme is defined by two constants derived from its alphabet size `N`:
1.  **Input Block (`B`)**: The number of bytes consumed per step.
2.  **Output Block (`C`)**: The number of characters produced per step.

These are the smallest integers where the output capacity (`N` to the power of `C`) is large enough to hold the input possibilities (`256` to the power of `B`).

[def jeb-bat.generic.encode-block]
To encode a block of `B` bytes:
1.  Read `B` bytes from the input.
2.  Treat these bytes as a big-endian unsigned integer called `V`.
3.  Convert `V` into a base-`N` number.
4.  Map each base-`N` digit to a character using the alphabet.
5.  If the result has fewer than `C` digits, add leading zeros (represented by the first character in the alphabet) until the length is `C`.

[def jeb-bat.generic.partial-blocks]
When the input length is not a multiple of `B`, the final chunk of data is smaller than a full block. This is called a partial block of size `b` (where `b < B`).

To encode a partial block:
1.  **Pad**: Add `B - b` zero bytes to the end (right side) of the partial input to create a full block of `B` bytes.
2.  **Encode**: Encode this padded block normally to produce `C` characters.
3.  **Truncate**: Keep only the first `c` characters, where `c` is calculated as `ceil(b * C / B)`.

This "pad with zeros, truncate output" strategy ensures that the significant data (the `b` bytes) is preserved in the most significant characters of the output.

[def jeb-bat.generic.decode-partial]
To decode a partial chunk of `c` characters (where `c < C`):
1.  **Pad**: Add `C - c` padding characters to the end. The padding character is the character representing the value `0` (typically the first character in the alphabet).
2.  **Decode**: Decode this padded block normally to produce `B` bytes.
3.  **Truncate**: Keep only the first `b` bytes, where `b` is calculated as `floor(c * B / C)`.

The formulas `c = ceil(b * C / B)` (encoding) and `b = floor(c * B / C)` (decoding) are exact inverses for all valid partial blocks.

However, not all character counts are valid: a partial block must encode at least 1 byte and at most `B - 1` bytes. This means only specific values of `c` correspond to valid encoded data. For example, in Base64 (B=3, C=4), valid partial block sizes are 2 or 3 characters; 1 character cannot represent any valid partial block.

---

## Alphabets

[def jeb-bat.alphabets @children]
Specific alphabet definitions for common encoding schemes.

### Base64url

[def jeb-bat.alphabets.base64url @children]
The Base64url alphabet is a URL-safe variant of Base64 encoding, defined in RFC 4648.

[def jeb-bat.alphabets.base64url.chars]
The alphabet consists of 64 characters in this exact order:
```
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_
```
Where `A` represents value 0, `B` represents 1, through `_` representing 63.

[def jeb-bat.alphabets.base64url.params]
Using the generic encoding model:
- `N = 64` (alphabet size)
- `B = 3` (bytes per block)
- `C = 4` (characters per block)

### Z85

[def jeb-bat.alphabets.z85 @children]
The Z85 alphabet is defined in the ZeroMQ ZMQ RFC 32 specification for encoding binary data.

[def jeb-bat.alphabets.z85.chars]
The alphabet consists of 85 characters in this exact order:
```
0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#
```
Where `0` represents value 0, `1` represents 1, through `#` representing 84.

[def jeb-bat.alphabets.z85.params]
Using the generic encoding model:
- `N = 85` (alphabet size)
- `B = 4` (bytes per block)
- `C = 5` (characters per block)

---

## Translucent Encodings

[def jeb-bat.translucent @children]
Translucent encodings extend a base encoding (like Z85) to allow preserving "safe" ASCII text in its raw, readable form, while maintaining strict compatibility with the base encoding's block structure.

### Alignment Preservation

[def jeb-bat.translucent.alignment]
A critical constraint of translucent encodings is **Alignment Preservation**.
Any encoded data must appear at the same character position in the distinct output stream as it would in a pure, standard encoded stream.

To achieve this, raw data text chunks (including their escape prefixes and padding) must consume exactly the same number of characters as the standard encoding would use for those same bytes.
For Z85 (4 bytes → 5 chars), this means every raw chunk must have a length (prefix + data + padding) that is a multiple of 5.

*Exception*: The final chunk of the stream does not need to align, as no encoded blocks follow it.

### Raw Data Prefixes

[def jeb-bat.translucent.prefixes]
Common prefix schemes for Z85:

1.  **`_` (Underscore)**: Prefix for exactly 4 bytes of raw data.
    *   Input: 4 bytes.
    *   Output: `_` + 4 bytes = 5 characters.
    *   Matches Z85 block size (4 bytes → 5 chars). **Preserves Alignment.**

2.  **`~` (Tilde)**: Prefix for exactly 6 bytes of raw data.
    *   Input: 6 bytes.
    *   Output: `~` + 6 bytes = 7 characters.
    *   **Does NOT preserve alignment** (7 is not a multiple of 5).
    *   Allowed *only* at the end of the stream.

3.  **`|` (Pipe)**: Variable-length or special escape.
    *   Used for mid-block transitions or other counts.
    *   Must employ padding to satisfy alignment if followed by more data.

### Mid-Block Transitions & Local Stability

[def jeb-bat.translucent.mid-block]
Switching from encoded to raw mode in the middle of a block (e.g., after 1 byte of a 4-byte block) is risky because Z85 output characters depend on all bytes in the block.

To ensure **Local Behavior** (no backtracking or non-local output changes), a mid-block transition is permitted **only if the emitted partial characters are stable**.

**The Zero-Bit Rule**:
A partial block can be emitted (and the mode switched) only if treating the remaining unseen bytes as zeros produces the *exact same* output characters as treating them as any other value.

*   Mathematically: `Encode(CurrentBytes + 00...)` must yield stable leading characters that do not change based on the value of the missing bytes.
*   For Z85: `b0` determines `c0` stably in ~68% of cases. In the other 32%, `c0` depends on lower bits.
*   **Constraint**: The encoder MUST NOT switch to raw mode mid-block if the current partial state is unstable. It must continue encoding until a stable boundary (or full block) is reached.
