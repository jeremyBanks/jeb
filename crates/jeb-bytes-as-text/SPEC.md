# jeb-bytes-as-text Specification

[def jeb-bat @children]
The `jeb-bytes-as-text` crate provides basic binary text encoding logic with support for several different alphabets such as base64url and Z85.

---

## Generic Encoding

[def jeb-bat.generic @children]
Defines a generalized model for encoding binary data into text using an arbitrary alphabet. This model abstracts over common schemes like Base64, Base32, and Z85.

[def jeb-bat.generic.alphabet]
An alphabet is an ordered sequence of `N` distinct characters, where `N` is the base of the encoding. Each character represents a value from `0` to `N-1`.

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
When the input length is not a multiple of `B`, the final chunk of data is smaller than a full block. This is called a partial block.
(Details on padding/extrapolation to be defined next).
