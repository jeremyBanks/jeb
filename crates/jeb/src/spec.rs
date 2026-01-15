#![allow(
    non_snake_case,
    unused_variables
)]
use inline::*;


#[test]
fn literate() {
    // # Encoding bytes as text

    // There are a lot of different ways to encode binary data as text, with
    // different trade-offs and use cases. The most common considerations as
    // size/efficiency (how many characters are required to encode a given
    // number of bytes), and what characters are used in the encoded
    // representation (determining contexts where the encoded data can be used).

    // ## Latin-1 passthrough

    // The simplest possible way to encode binary data as text is just to covert
    // byte values directly to Unicode code points. This is sometimes referred
    // to **Latin-1 passthrough**, because this is equivalent to the legacy
    // "Latin-1" text encoding whose characters now make up the first 256
    // Unicode code points (the "Basic Latin" (ASCII) and
    // "Latin-1 Supplement" blocks).

    // (We'll be focusing primarily on byte-oriented encodings that can produce
    // ASCII-safe output.)

    // One of the simplest as most common ways is hexadecimal ("hex", base 16).
    // Each byte is eight bits, which evenly divides into two hex digits.

    // bits, and each hex digit represents four bits, so we just output two
    // hex digits for each byte.

    let _ = ();
}
