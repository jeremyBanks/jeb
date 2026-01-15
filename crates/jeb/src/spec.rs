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

fn print_doc_block(lines: &[&str]) {
    if lines.is_empty() {
        return;
    }

    // Strip one leading and one trailing empty line if present
    let mut lines = lines;
    if lines.first().map(|s| s.trim().is_empty()).unwrap_or(false) {
        lines = &lines[1..];
    }
    if lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
        lines = &lines[..lines.len() - 1];
    }

    if lines.is_empty() {
        return;
    }

    // Find minimum leading whitespace among non-empty lines
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Print each line with common indent stripped
    for line in lines {
        if line.trim().is_empty() {
            eprintln!();
        } else {
            eprintln!("{}", &line[min_indent..]);
        }
    }
}

macro_rules! literate {
    // Base case - no more tokens, flush any buffered docs
    (@process [] [$($doc:literal),*] $($output:tt)*) => {
        $($output)*
        print_doc_block(&[$($doc),*]);
    };

    // Accumulate consecutive doc comments into buffer
    (@process [#[doc = $doc:literal] $($rest:tt)*] [$($buf:literal),*] $($output:tt)*) => {
        literate!(@process [$($rest)*] [$($buf,)* $doc] $($output)*)
    };

    // Hit a let statement - flush buffer first, then process
    (@process [let $p:pat = $e:expr ; $($rest:tt)*] [$($buf:literal),*] $($output:tt)*) => {
        literate!(@process [$($rest)*] [] $($output)* print_doc_block(&[$($buf),*]); eprintln!(); eprintln!(">>> let {} = {};", stringify!($p), stringify!($e)); let $p = $e; eprintln!();)
    };

    // Hit an expression statement - flush buffer first, then process
    (@process [$e:expr ; $($rest:tt)*] [$($buf:literal),*] $($output:tt)*) => {
        literate!(@process [$($rest)*] [] $($output)* print_doc_block(&[$($buf),*]); eprintln!(); eprintln!(">>> {};", stringify!($e)); $e; eprintln!();)
    };

    // Hit a trailing expression - flush buffer first, then process
    (@process [$e:expr] [$($buf:literal),*] $($output:tt)*) => {
        $($output)* print_doc_block(&[$($buf),*]); eprintln!(); eprintln!(">>> {}", stringify!($e)); $e
    };

    // Main entry: create the test function with empty buffer
    ($($body:tt)*) => {
        #[test]
        fn literate_test() {
            literate!(@process [$($body)*] []);
        }
    };
}
use literate;
