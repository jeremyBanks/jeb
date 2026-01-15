#![allow(
    non_snake_case,
    unused_variables
)]
use inline::*;

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


 */

/// (We'll be focusing primarily on byte-oriented encodings that can produce
/// ASCII-safe output.)

/// One of the simplest as most common ways is hexadecimal ("hex", base 16).
/// Each byte is eight bits, which evenly divides into two hex digits.

/// bits, and each hex digit represents four bits, so we just output two
/// hex digits for each byte.
}

macro_rules! literate {
    // Base case - no more tokens to process
    (@process [] $($output:tt)*) => {
        $($output)*
    };

    // Match a doc comment and convert to eprintln
    (@process [#[doc = $doc:literal] $($rest:tt)*] $($output:tt)*) => {
        literate!(@process [$($rest)*] $($output)* eprintln!($doc);)
    };

    // Match a let statement
    (@process [let $p:pat = $e:expr ; $($rest:tt)*] $($output:tt)*) => {
        literate!(@process [$($rest)*] $($output)* eprintln!(">>> let {} = {};", stringify!($p), stringify!($e)); let $p = $e;)
    };

    // Match an expression statement (ending with ;)
    (@process [$e:expr ; $($rest:tt)*] $($output:tt)*) => {
        literate!(@process [$($rest)*] $($output)* eprintln!(">>> {};", stringify!($e)); $e;)
    };

    // Match a trailing expression (no semicolon)
    (@process [$e:expr] $($output:tt)*) => {
        $($output)* eprintln!(">>> {}", stringify!($e)); $e
    };

    // Main entry: create the test function
    ($($body:tt)*) => {
        #[test]
        fn literate_test() {
            literate!(@process [$($body)*]);
        }
    };
}
use literate;
