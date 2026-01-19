#![recursion_limit = "512"]
#![allow(
    non_snake_case,
    unused
)]
use {
    inline::*,
    jeb::testing::literate,
};

literate! {
/**
    # Encoding bytes as text

    There are a lot of different ways to encode bytes/binary data as text, with
    different trade-offs and use cases. We're going to describe several options
    and discuss the decisions and trade-offs in their designs. Some properties
    we may look at include:

    - **Context compatibility:** What characters does the encoding use in its
      output? Where can the encoded data be used as-is without further encoding?
    - **Offset stability:** Will a given byte offset in the input data always
      map to the same character offset in the output data, or can this differ
      based on preceding bytes? Do those characters themselves have variable
      byte lengths when encoded as UTF-8?
    - **Overhead:** How many more characters than bytes do we need to encode a
      given input? How many bytes do those characters themselves require when
      encoded in UTF-8? Can this differ depending on the input data? If we ever
      describe this with a single number, then we're assuming randomly uniformly
      distributed input data for simplicity.
    - **Transparency:** How meaningful is the encoded data when viewed as text?
      Are any patterns in the binary data visible in the encoded text? Are any
      values passed through in a way that is meaningful to a reader?
    - **Ordering:** will two encoded values retain the same relative
      lexicographic ordering as the original bytes, will the sort order be
      preserved?

    We'll see that different encodings make different trade-offs in this space,
    particularly around block alignment (how the encoding groups bytes) and
    numeric transparency (whether small integers remain recognizable).

    ## Latin-1 passthrough

    The simplest possible way to encode binary data as text is just to covert
    byte values directly to Unicode code points. This is sometimes referred to
    Latin-1 passthrough, because this is equivalent to the legacy Latin-1
    (ISO-8859-1) text encoding whose characters now make up the first 256
    Unicode code points (the Basic Latin (ASCII) and Latin-1 Supplement blocks).

    https://en.wikipedia.org/wiki/ISO/IEC_8859-1

    This has the great virtue of simplicity; it's a natural, almost canonical
    choice. That makes is a good option when you just need to store bytes as
    text due to data format limitations, and there are no other considerations.
    However, it may not be a good choice if you need to embed or operate on the
    encoded data as text.

    - **Context compatibility:** nonexistent. This encoding includes control
      characters which may not be expected or well-supported (e.g. `\x00` `NUL`,
      `\x07` `BEL`, `\x7F` `DEL`). It also includes essentially every character
      with special meaning in other encodings or syntaxes (e.g. `\` backslash,
      `"` double quote). These encoded values will practically always require an
      additional layer of escaping or framing to be used in text contexts.
    - **Offset stability:** stable as characters, unstable as UTF-8. Each input
      byte maps exactly to one output character, but half of those characters
      require two bytes when encoded as UTF-8.
    - **Overhead::** 0 as characters, +50% as UTF-8. The consistent 1 byte:1
      character ratio is good, but half of those characters requiring two bytes
      when encoded as UTF-8 is not good.
    - **Text transparency:** this provides provides great transparency in theory
      because all ASCII characters in the input data are passed through
      unchanged. However, the broad context compatibility issues limit the cases
      where this can actually be taken advantage of.
    - **Ordering:** preserved.

    ## Binary

    Binary encoding—where each byte becomes eight `0` or `1` characters—isn't
    practical for production, but it's useful as a pedagogical tool and for
    human input/output in contexts like this document.
*/
static BINARY: &[u8; 2] = b"01";
/**
    Because we're speaking in terms of byte-oriented encoding (not
    bit-oriented), we need to specify whether the most-significant-bits/
    higher-order-bits (`128` and down) come first (called "big-endian") or the
    least-significant-bits/lower-order-bits (`1` and up) come first (called
    "little-endian"). We follow the standard choice: big-endian, for consistency
    with the way normal decimal numbers are written in code and math.
*/
    fn to_binary(bytes: impl AsRef<[u8]>) -> String {
        let bytes = bytes.as_ref();
        let len = bytes.len() * 8;
        let mut result = String::with_capacity(len);
        for byte in bytes {
            for bit in 0..8 {
                result.push(BINARY[((*byte as usize) >> (7 - bit)) & 0x1] as char);
            }
        }
        result
    }

    to_binary([  0_u8]).is("00000000");
    to_binary([  1_u8]).is("00000001");
    to_binary([  2_u8]).is("00000010");
    to_binary([  3_u8]).is("00000011");
    to_binary([128_u8]).is("10000000");
    to_binary([255_u8]).is("11111111");
    255_u16.to_be_bytes().is([0_u8, 255_u8]);
    256_u16.to_be_bytes().is([1_u8,   0_u8]);
    to_binary(255_u16.to_be_bytes()).is("0000000011111111");
    to_binary(256_u16.to_be_bytes()).is("0000000100000000");
    to_binary(256_u32.to_be_bytes()).is("00000000000000000000000100000000");

/**
    ## Hexadecimal

    One of the most common ways to encode binary data as text is hexadecimal
    ("hex", base 16). Each byte (8 bits, a value from 0 to 255) is represented
    by two hexadecimal digits (4 bits, each representing a value from 0 to 15),
    extending the decimal digits (`0` to `9`) with the first six letters of the
    alphabet (`A` to `F`).

    https://datatracker.ietf.org/doc/html/rfc4648#section-8
*/
    static HEX: &[u8; 16] = b"0123456789ABCDEF";
/**
    This is a common choice for binary values that may be directly manually
    edited by humans.

    Cleanly splitting each byte in half keeps this encoding quite simple, with
    only one significant design question: which half comes first in the text
    output, the high 4 bits (representing 16, 32, 64, and 128) or the low 4
    bits (representing 1, 2, 4, and 8)? This property is a form of endianness,
    and for hex the answer is always "big endian" (the high 4 bits come first),
    aligning with how decimal numbers are typically written.

    Encoding is quite simple: just pull out the bits, and use them to index into
    the alphabet.
*/
    fn to_hex(bytes: impl AsRef<[u8]>) -> String {
        let bytes = bytes.as_ref();
        let mut result = String::new();
        for byte in bytes {
            let high = byte >> 4; // == byte / 16
            let low = byte & 0xF; // == byte % 16
            result.push(HEX[high as usize] as char);
            result.push(HEX[low as usize] as char);
        }
        result
    }

    to_hex(Vec::<u8>::from_iter(0x00..=0x20)).is(
        "000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F20");

    to_hex(from_binary("00000000"        )).is("00"  );
    to_hex(from_binary("00000001"        )).is("01"  );
    to_hex(from_binary("10000000"        )).is("80"  );
    to_hex(from_binary("11111111"        )).is("FF"  );
    to_hex(from_binary("0000000000000000")).is("0000");
    to_hex(from_binary("0000000000000001")).is("0001");
    to_hex(from_binary("1000000000000000")).is("8000");
    to_hex(from_binary("1111111111111111")).is("FFFF");
/**
    - **Context compatibility:** as good as it gets. It only uses digits and a
      handful of letters, and typically not case-sensitive.
    - **Offset stability:** fully stable.
    - **Overhead:** +100%, as bad as it gets. You wouldn't pick hex for its
      efficiency.
    - **Transparency:** pretty good for numeric/binary data. Zeros are `00` and
      it's not that difficult to interpret positive integers. Text is of course
      unrecognizable.
    - **Ordering:** preserved. The hex alphabet is in ascending order (`0-9`,
      then `A-F`), so lexicographic comparison of hex strings matches numeric
      comparison of the underlying bytes.

    ## Beyond byte boundaries

    Hexadecimal has a special property: 4 bits per character divides evenly into
    8 bits per byte. This means each byte maps to exactly two characters, with
    no leftover bits and no need to group multiple bytes together.

    Other bases don't divide so cleanly. Base 64 uses 6 bits per character, and
    base 85 uses log₂(85) ≈ 6.4 bits per character. Since these don't divide 8
    evenly, these encodings must work on multi-byte blocks. Base 64 works on
    3-byte (24-bit) blocks, and base 85 works on 4-byte (32-bit) blocks.

    This raises a question: what happens when the input length isn't a multiple
    of the block size? Different encodings handle this differently, and we'll
    see the details as we go.
*/

/**
    ## Base 64

    One of the most common choices for encoding binary data as text in
    production use cases where human-readability is not a priority is a base 64
    encoding. We're specifically considering base64url, the standard version
    which uses a URL-safe alphabet and no padding.

    https://datatracker.ietf.org/doc/html/rfc4648#section-5
 */
    let BASE64_URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
/**
    WRITE SOMETHING: 4 characters per 3 bytes, doesn't line up with single-byte
    boundaries, and 3 isn't a power of two so it doesn't line up super-cleanly
    with any data structures.

    XXX: how does it handle partial blocks? That's key to so many things!
    And is it big endian?

    - **Context compatibility:** very good if using the base64url alphabet. It
      doesn't require encoding it any standard string contexts. The only minor
      conflicts are that the minus (`-`) character is sometimes used as a
      delimiter and may not be valid in some identifier-like contexts.
    - **Offset stability:** fully stable.
    - **Overhead:** +33%,

    ## Z85

    It's officially defined as requiring 4-byte (32-bit) blocks, but we can use
    the same approach as base 64 to support partial blocks.

    https://rfc.zeromq.org/spec/32/
 */
    let Z85: &[u8; 85] =
        b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";



}

// The decoding implementations are mostly down here, outside of the literate
// block because that's too much code.

fn from_binary(text: impl AsRef<str>) -> Vec<u8> {
    let text = text.as_ref();
    let len = text.len() / 8;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let index = i * 8;
        let byte = u8::from_str_radix(&text[index..index + 8], 2).unwrap();
        result.push(byte);
    }
    result
}

fn from_hex(text: impl AsRef<str>) -> Vec<u8> {
    let text = text.as_ref();
    let len = text.len() / 2;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let index = i * 2;
        let byte = u8::from_str_radix(&text[index..index + 2], 16).unwrap();
        result.push(byte);
    }
    result
}
