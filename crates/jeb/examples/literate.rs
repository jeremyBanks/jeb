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

    There are a lot of different ways to encode binary data as text, with
    different trade-offs and use cases. The most common considerations as
    size/efficiency (how many characters are required to encode a given
    number of bytes), and what characters are used in the encoded
    representation (determining contexts where the encoded data can be
    used).

    ## Latin-1 passthrough

    The simplest possible way to encode binary data as text is just to
    covert byte values directly to Unicode code points. This is
    sometimes referred to **Latin-1 passthrough**, because this is
    equivalent to the legacy "Latin-1" text encoding whose characters
    now make up the first 256 Unicode code points (the "Basic Latin"
    (ASCII) and "Latin-1 Supplement" blocks).

    This has the great virtue of simplicity; it's a natural, almost canonical
    choice. That makes is a good option when you just need to store bytes as
    text due to data format limitations, and there are no other considerations.

    However, it's not a very good choice if you actually need to use the encoded
    data as text where text is actually expected. It includes control
    characters which may not be expected or well-supported (e.g. `\x00` `NUL`,
    `\x07` `BEL`, `\x7F` `DEL`). It includes essentially every character with
    special meaning in other encodings or programming languages (e.g. `\`
    backslash, `"` double quote, `$` dollar sign), so these encoded values can
    rarely be embedded without an additional layer of escaping or framing.
 */

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
