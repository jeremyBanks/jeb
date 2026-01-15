#![allow(
    non_snake_case,
    unused
)]
use {
    bat::PrettyPrinter,
    inline::*,
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

fn print_doc_block(doc_strings: &[&str]) {
    if doc_strings.is_empty() {
        return;
    }

    // Group doc strings: multi-line strings (/** */) are processed alone,
    // consecutive single-line strings (///) are grouped together
    let mut i = 0;
    while i < doc_strings.len() {
        let s = doc_strings[i];
        if s.contains('\n') {
            // Multi-line block comment - process alone
            print_single_doc_group(&[s]);
            i += 1;
        } else {
            // Single-line - gather consecutive single-line strings
            let start = i;
            while i < doc_strings.len() && !doc_strings[i].contains('\n') {
                i += 1;
            }
            print_single_doc_group(&doc_strings[start..i]);
        }
    }
}

fn print_single_doc_group(doc_strings: &[&str]) {
    if doc_strings.is_empty() {
        return;
    }

    // Join all doc strings with newlines, then split into lines
    let combined = doc_strings.join("\n");
    let lines: Vec<&str> = combined
        .lines()
        .map(|line| if line.trim().is_empty() { "" } else { line })
        .collect();
    let mut lines: &[&str] = &lines;

    // Strip one leading and one trailing empty line if present
    if lines.first().map(|s| s.is_empty()).unwrap_or(false) {
        lines = &lines[1..];
    }
    if lines.last().map(|s| s.is_empty()).unwrap_or(false) {
        lines = &lines[..lines.len() - 1];
    }

    if lines.is_empty() {
        return;
    }

    // Find minimum leading whitespace among non-empty lines
    let min_indent = lines
        .iter()
        .filter(|line| !line.is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Build dedented text
    let dedented: String = lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                ""
            } else {
                &line[min_indent..]
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Print with bat markdown highlighting (add trailing newline to content)
    let content = format!("{}\n", dedented);
    eprintln!();
    PrettyPrinter::new()
        .input_from_bytes(content.as_bytes())
        .language("markdown")
        .print()
        .unwrap();
}

fn print_code(code: &str) {
    // Add trailing newline to content
    let content = format!("{}\n", code);
    eprintln!();
    PrettyPrinter::new()
        .input_from_bytes(content.as_bytes())
        .language("rust")
        .print()
        .unwrap();
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
        literate!(@process [$($rest)*] [] $($output)* print_doc_block(&[$($buf),*]); print_code(concat!("let ", stringify!($p), " = ", stringify!($e), ";")); let $p = $e;)
    };

    // Hit an expression statement - flush buffer first, then process
    (@process [$e:expr ; $($rest:tt)*] [$($buf:literal),*] $($output:tt)*) => {
        literate!(@process [$($rest)*] [] $($output)* print_doc_block(&[$($buf),*]); print_code(concat!(stringify!($e), ";")); $e;)
    };

    // Hit a trailing expression - flush buffer first, then process
    (@process [$e:expr] [$($buf:literal),*] $($output:tt)*) => {
        $($output)* print_doc_block(&[$($buf),*]); print_code(stringify!($e)); $e
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
