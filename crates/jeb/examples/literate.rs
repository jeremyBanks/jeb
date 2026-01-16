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
    This is a common choice for binary values that may be
    directly manually edited by humans.

    Cleanly splitting each byte in half keeps this encoding quite simple, with
    only one significant design question: which half comes first in the text
    output, the high 4 bits (representing 16, 32, 64, and 128) or the low 4
    bits (representing 1, 2, 4, and 8)? This property is a form of endianness,
    and for hex the answer is always "big endian" (the high 4 bits come first),
    aligning with how decimal numbers are typically written. This makes encoding
    quite simple: just pull out the bits, and use them to index into the
    alphabet.
*/
    fn hex_encode(bytes: &[u8]) -> String {
        let mut result = String::new();
        for byte in bytes {
            let high = (byte & 0xF0) >> 4;
            let low = byte & 0x0F;
            result.push(HEX[high as usize] as char);
            result.push(HEX[low as usize] as char);
        }
        result
    }

    let data = Vec::<u8>::from_iter(0x00..=0x20);
    hex_encode(&data).is("000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F20");
/**
    - **Context compatibility:** as good as it gets. It only uses digits and a
      handful of letters, and typically not case-sensitive.
    - **Offset stability:** fully stable.
    - **Overhead:** +100%, as bad as it gets. You wouldn't pick hex for its
      efficiency.
    - **Transparency:** pretty good for numeric/binary data. Zeros are `00` and
      it's not that difficult to interpret positive integers. Text is of course
      unrecognizable.

    REWORD: other encodings don't line up with byte boundaries. How do they deal
    with partial blocks? It's generalizable! But it does result in output
    sometimes have some wasted bits.

    ## Base 64 (URL-safe)

    https://datatracker.ietf.org/doc/html/rfc4648#section-5
 */
    let BASE64_URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/**
    Z85

    It's officially defined as requiring 4-byte (32-bit) blocks, but we can use
    the same approach as base 64 to support partial blocks.

    https://rfc.zeromq.org/spec/32/
 */
    let Z85: &[u8; 85] =
        b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
/// (We'll be focusing primarily on byte-oriented encodings that can produce
/// ASCII-safe output.)

/// One of the simplest as most common ways is hexadecimal ("hex", base 16).
/// Each byte is eight bits, which evenly divides into two hex digits.

/// bits, and each hex digit represents four bits, so we just output two
/// hex digits for each byte.

println!("test 1!");
println!("test 2!");
println!("{}", {
    println!("test 3a!");
    println!("test 3b!");
    "test 3c!"
});


}
