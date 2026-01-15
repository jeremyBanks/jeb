# jeb-bytes-as-text Specification

[def jeb-bat @children] The `jeb-bytes-as-text` crate provides basic binary text
encoding logic with support for several different alphabets such as base64url
and Z85.

---

## Generic Encoding

[def jeb-bat.generic @children] Defines a generalized model for encoding binary
data into text using an arbitrary alphabet. This model abstracts over common
schemes like Base64, Base32, and Z85.

[def jeb-bat.generic.alphabet] An alphabet is an ordered sequence of `N`
distinct characters, where `N` is the base of the encoding. Each character
represents a value from `0` to `N-1`.

The alphabet size `N` must be greater than 1. Any base greater than 1 is
mathematically valid, though bases below 2 or above 256 have limited practical
utility.

[def jeb-bat.generic.block] Encoding operates on fixed-size blocks. This allows
the encoding to process data in chunks rather than bit-by-bit.

An encoding scheme is defined by two constants derived from its alphabet size
`N`:

1. **Input Block (`B`)**: The number of bytes consumed per step.
2. **Output Block (`C`)**: The number of characters produced per step.

These are the smallest integers where the output capacity (`N` to the power of
`C`) is large enough to hold the input possibilities (`256` to the power of
`B`).

[def jeb-bat.generic.encode-block] To encode a block of `B` bytes:

1. Read `B` bytes from the input.
2. Treat these bytes as a big-endian unsigned integer called `V`.
3. Convert `V` into a base-`N` number.
4. Map each base-`N` digit to a character using the alphabet.
5. If the result has fewer than `C` digits, add leading zeros (represented by
   the first character in the alphabet) until the length is `C`.

[def jeb-bat.generic.partial-blocks] When the input length is not a multiple of
`B`, the final chunk of data is smaller than a full block. This is called a
partial block of size `b` (where `b < B`).

To encode a partial block:

1. **Pad**: Add `B - b` zero bytes to the end (right side) of the partial input
   to create a full block of `B` bytes.
2. **Encode**: Encode this padded block normally to produce `C` characters.
3. **Truncate**: Keep only the first `c` characters, where `c` is calculated as
   `ceil(b * C / B)`.

This "pad with zeros, truncate output" strategy ensures that the significant
data (the `b` bytes) is preserved in the most significant characters of the
output.

[def jeb-bat.generic.decode-partial] To decode a partial chunk of `c` characters
(where `c < C`):

1. **Pad**: Add `C - c` padding characters to the end. The padding character is
   the character representing the value `0` (typically the first character in
   the alphabet).
2. **Decode**: Decode this padded block normally to produce `B` bytes.
3. **Truncate**: Keep only the first `b` bytes, where `b` is calculated as
   `floor(c * B / C)`.

The formulas `c = ceil(b * C / B)` (encoding) and `b = floor(c * B / C)`
(decoding) are exact inverses for all valid partial blocks.

However, not all character counts are valid: a partial block must encode at
least 1 byte and at most `B - 1` bytes. This means only specific values of `c`
correspond to valid encoded data. For example, in Base64 (B=3, C=4), valid
partial block sizes are 2 or 3 characters; 1 character cannot represent any
valid partial block.

---

## Alphabets

[def jeb-bat.alphabets @children] Specific alphabet definitions for common
encoding schemes.

### Base64url

[def jeb-bat.alphabets.base64url @children] The Base64url alphabet is a URL-safe
variant of Base64 encoding, defined in RFC 4648.

[def jeb-bat.alphabets.base64url.chars] The alphabet consists of 64 characters
in this exact order:

```
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_
```

Where `A` represents value 0, `B` represents 1, through `_` representing 63.

[def jeb-bat.alphabets.base64url.params] Using the generic encoding model:

- `N = 64` (alphabet size)
- `B = 3` (bytes per block)
- `C = 4` (characters per block)

### Z85

[def jeb-bat.alphabets.z85 @children] The Z85 alphabet is defined in the ZeroMQ
ZMQ RFC 32 specification for encoding binary data.

[def jeb-bat.alphabets.z85.chars] The alphabet consists of 85 characters in this
exact order:

```
0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#
```

Where `0` represents value 0, `1` represents 1, through `#` representing 84.

[def jeb-bat.alphabets.z85.params] Using the generic encoding model:

- `N = 85` (alphabet size)
- `B = 4` (bytes per block)
- `C = 5` (characters per block)

---

## Translucent Encodings

[def jeb-bat.translucent @children] Translucent encodings extend a base encoding
(like Z85) to allow preserving "safe" ASCII text in its raw, readable form,
while maintaining strict compatibility with the base encoding's block structure.

### Alignment Preservation

[def jeb-bat.translucent.alignment] A critical constraint of translucent
encodings is **Alignment Preservation**. Any encoded data must appear at the
same character position in the distinct output stream as it would in a pure,
standard encoded stream.

To achieve this, raw data text chunks (including their escape prefixes and
padding) must consume exactly the same number of characters as the standard
encoding would use for those same bytes. For Z85 (4 bytes → 5 chars), this means
every raw chunk must have a length (prefix + data + padding) that is a multiple
of 5.

_Exception_: The final chunk of the stream does not need to align, as no encoded
blocks follow it.

### Example Prefix

[def jeb-bat.translucent.example] For demonstration purposes, consider a prefix
`~` that indicates the following 4 bytes are raw ASCII.

- Input: `~` + 4 raw bytes.
- Total Chars: 1 (prefix) + 4 (data) = 5 characters.
- Alignment: Since Z85 maps 4 bytes to 5 characters, this prefix maintains
  perfect block alignment.

### Mid-Block Transitions & Local Stability

[def jeb-bat.translucent.mid-block] Switching from encoded to raw mode in the
middle of a block (e.g., after 1 byte of a 4-byte block) is risky because Z85
output characters depend on all bytes in the block.

To ensure **Local Behavior** (no backtracking or non-local output changes), a
mid-block transition is permitted **only if the emitted partial characters are
stable**.

**The Zero-Bit Rule**: We can only emit characters that are _fully determined_
by the bytes we have already seen.

- Conceptually: If we treat the missing future bytes as all-zeros (minimum
  value) or all-ones (maximum value), do we get the same leading characters?
- If yes, those leading characters are **stable** and can be emitted safely
  before switching to raw mode.
- If no, the characters are **unstable** (they depend on the future bytes). We
  MUST NOT switch to raw mode yet; we must continue encoding until we reach a
  stable boundary.

For Z85, the stability of output characters depends on how many bytes are known:

- **1 Byte Known**: First character stable in **~68%** of cases.
- **2 Bytes Known**: First 2-3 characters stable in **~89%** of cases.
- **3 Bytes Known**: First 3-4 characters stable in **~96%** of cases.

**Technical Detail**: Stability is determined by whether the range of possible
values for the unknown bytes can cross a character boundary.

- Let `V_min` be the value computed with valid bytes and trailing zeros.
- Let `V_max` be the value computed with valid bytes and trailing ones (0xFF).
- The characters are stable if `Encode(V_min)` produces the same leading
  characters as `Encode(V_max)`.
- For Z85's first character: `divisor = 85^4 (52,200,625)`. The "uncertainty
  range" added by 3 unknown bytes is `2^24 - 1` (~16.7 million).
- If `V_min % divisor` is less than `divisor - (2^24 - 1)`, then adding the
  maximum uncertainty cannot push the value to the next multiple. Thus, the
  character is **stable**.
- If `V_min % divisor` is potentially close enough to the boundary, the
  character is **unstable**.
