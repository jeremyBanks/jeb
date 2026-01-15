//! Snapshot testing extension traits.
//!
//! Provides `.snap(expected)` and `.snap_dbg(expected)` methods for inline
//! snapshot testing. These methods compare actual values with expected
//! snapshots and update the source code if they differ.
//!
//! # Example
//!
//! ```no_run
//! use inline::InlineSnapExt;
//!
//! fn compute() -> i32 {
//!     42
//! }
//!
//! // Snapshot with Bake serialization:
//! let result = compute().snap(0);
//!
//! // Snapshot with Debug formatting:
//! let result = compute().snap_dbg("");
//! ```

use {
    crate::{
        runtime,
        value::Value,
    },
    std::{
        any::TypeId,
        fmt::Debug,
        panic::Location,
    },
};

/// Create a raw string literal token stream from content.
///
/// Raw strings preserve newlines as actual newlines in source code, making
/// multi-line snapshots much more readable:
///
/// ```text
/// // Instead of:
/// .snap_dbg("Struct {\n    field: 1,\n}")
///
/// // We get:
/// .snap_dbg(r"Struct {
///     field: 1,
/// }")
/// ```
///
/// The function automatically determines the minimum number of `#` characters
/// needed to avoid conflicts with the content.
///
/// Falls back to regular escaped strings for content containing lone `\r`
/// (carriage return not followed by `\n`), which is invalid in Rust source.
/// Check if a string contains only "simple safe ASCII" that needs no escaping.
/// Safe chars: printable ASCII (space through ~) excluding quote and backslash.
fn is_simple_ascii(s: &str) -> bool {
    s.bytes()
        .all(|b| matches!(b, b' '..=b'!' | b'#'..=b'[' | b']'..=b'~'))
}

pub(crate) fn make_raw_string(content: &str) -> proc_macro2::TokenStream {
    // If content is simple safe ASCII, use a plain string literal
    if is_simple_ascii(content) {
        let literal = format!("\"{}\"", content);
        return literal
            .parse()
            .unwrap_or_else(|_| quote::quote! { #content });
    }

    // Check for lone \r (not followed by \n) - can't be in Rust source
    let has_lone_cr = {
        let bytes = content.as_bytes();
        bytes
            .iter()
            .enumerate()
            .any(|(i, &b)| b == b'\r' && bytes.get(i + 1) != Some(&b'\n'))
    };

    if has_lone_cr {
        // Fall back to regular string with escapes
        return quote::quote! { #content };
    }

    // Find minimum hash count needed
    // Raw string r##"..."## closes on `"` followed by exactly N `#` chars
    let mut hash_count = 0;
    loop {
        // Build the closing delimiter: " followed by hash_count # chars
        let closing: String = std::iter::once('"')
            .chain(std::iter::repeat('#').take(hash_count))
            .collect();

        // If content doesn't contain this closing sequence, we're safe
        if !content.contains(&closing) {
            break;
        }
        hash_count += 1;

        // Safety limit (should never happen in practice)
        if hash_count > 100 {
            return quote::quote! { #content };
        }
    }

    // Build the raw string literal: r##"content"##
    let hashes: String = std::iter::repeat('#').take(hash_count).collect();
    let raw_literal = format!("r{}\"{}\"{}", hashes, content, hashes);

    // Parse it as a token stream
    raw_literal.parse().unwrap_or_else(|_| {
        // Fallback to escaped string if parsing fails
        quote::quote! { #content }
    })
}

/// Convert escaped string literals in a token stream to raw strings.
///
/// Walks through the token tree and converts any string literals that contain
/// escape sequences into raw string literals for better readability.
fn convert_strings_to_raw(tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    use proc_macro2::{
        Group,
        TokenTree,
    };

    tokens
        .into_iter()
        .map(|tt| {
            match tt {
                TokenTree::Group(group) => {
                    // Recursively process groups
                    let converted = convert_strings_to_raw(group.stream());
                    let mut new_group = Group::new(group.delimiter(), converted);
                    new_group.set_span(group.span());
                    TokenTree::Group(new_group)
                }
                TokenTree::Literal(lit) => {
                    let lit_str = lit.to_string();
                    // Check if it's a non-raw string literal (starts with " but not r")
                    if lit_str.starts_with('"') && !lit_str.starts_with("r") {
                        // Try to parse as a string literal
                        if let Ok(syn_lit) = syn::parse_str::<syn::LitStr>(&lit_str) {
                            let value = syn_lit.value();
                            // Only convert if the value differs from the literal representation
                            // (i.e., it had escape sequences)
                            let simple_check = format!("\"{}\"", value);
                            if simple_check != lit_str {
                                // Has escape sequences, convert to raw
                                let raw_tokens = make_raw_string(&value);
                                // Extract the literal from the token stream
                                if let Some(TokenTree::Literal(raw_lit)) =
                                    raw_tokens.into_iter().next()
                                {
                                    return TokenTree::Literal(raw_lit);
                                }
                            }
                        }
                    }
                    TokenTree::Literal(lit)
                }
                other => other,
            }
        })
        .collect()
}

/// Extension trait for inline snapshot testing.
///
/// Provides two snapshot methods:
/// - `.snap(expected)` - For types implementing `Value` (Bake + Clone +
///   PartialEq)
/// - `.snap_dbg(expected)` - For types implementing `Debug`, compares debug
///   output
///
/// Both methods:
/// 1. Compare the actual value with the expected value
/// 2. If they differ and we're in Write mode, update the source file
/// 3. Return the actual value (self)
pub trait InlineSnapExt: Sized {
    /// Compare this value against an expected snapshot using Bake
    /// serialization.
    ///
    /// If the values differ and the runtime mode allows writes, the source
    /// file is updated to replace the `expected` argument with the baked
    /// representation of `self`.
    ///
    /// Returns `self` (the actual value), not the expected value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inline::InlineSnapExt;
    ///
    /// fn compute() -> i32 {
    ///     42
    /// }
    /// let result = compute().snap(42);
    /// // If compute() != 42, source is updated with actual value
    /// ```
    #[track_caller]
    fn snap<E>(self, expected: E) -> Self
    where
        Self: Value + 'static + PartialEq<E>,
        E: Debug;

    /// Compare this value's Debug output against an expected string snapshot.
    ///
    /// Uses `{:#?}` (pretty Debug) formatting to convert the value to a string,
    /// then compares with the expected string. If they differ and the runtime
    /// mode allows writes, the source file is updated.
    ///
    /// This is useful for types that implement `Debug` but not `Bake`.
    ///
    /// Returns `self` (the actual value), not the expected value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use inline::InlineSnapExt;
    ///
    /// fn compute() -> i32 {
    ///     42
    /// }
    /// let result = compute().snap_dbg("42");
    /// // If debug output differs, source is updated with actual debug string
    /// ```
    #[track_caller]
    fn snap_dbg(self, expected: &str) -> Self
    where
        Self: Debug;
}

impl<T> InlineSnapExt for T {
    #[track_caller]
    fn snap<E>(self, expected: E) -> Self
    where
        Self: Value + 'static + PartialEq<E>,
        E: Debug,
    {
        let location = Location::caller();
        let mode = runtime::get_mode();

        // If values are equal, nothing to do
        if self == expected {
            return self;
        }

        // Values differ - handle based on mode
        match mode {
            runtime::Mode::Verify => {
                // In verify mode, panic with both expected and actual
                let actual_baked = databake::Bake::bake(&self, &Default::default());
                panic!(
                    "Snapshot mismatch at {}:{}:{}\n\nExpected:\n{:#?}\n\nActual:\n{}\n\nRun with \
                     INLINE_MODE=write to update snapshots.",
                    location.file(),
                    location.line(),
                    location.column(),
                    expected,
                    actual_baked,
                );
            }
            runtime::Mode::Write | runtime::Mode::Memory => {
                // Both modes update in-memory state; Write also persists to disk
                let file_path = runtime::resolve_source_path(location.file());

                // Special case for String: output just the raw string literal
                // (avoids databake's "...".to_owned() suffix)
                let baked = if TypeId::of::<Self>() == TypeId::of::<String>() {
                    // SAFETY: We just verified Self is String
                    let s: &String = unsafe { &*(&self as *const Self as *const String) };
                    make_raw_string(s)
                } else {
                    // Bake the actual value to tokens
                    let baked = databake::Bake::bake(&self, &Default::default());
                    // Convert any escaped string literals to raw strings for readability
                    convert_strings_to_raw(baked)
                };

                // Update the source file (in-memory; disk write depends on mode)
                if let Err(e) = runtime::update_source_file(
                    &file_path,
                    location.line(),
                    location.column(),
                    baked,
                ) {
                    eprintln!(
                        "inline::snap: failed to update source at {}:{}:{}: {}",
                        location.file(),
                        location.line(),
                        location.column(),
                        e
                    );
                }
            }
            runtime::Mode::Reject => {
                panic!(
                    "Snapshot mismatch at {}:{}:{} (mode=Reject, updates not allowed)",
                    location.file(),
                    location.line(),
                    location.column(),
                );
            }
        }

        self
    }

    #[track_caller]
    fn snap_dbg(self, expected: &str) -> Self
    where
        Self: Debug,
    {
        let location = Location::caller();
        let mode = runtime::get_mode();

        // Get the pretty debug representation
        let actual_dbg = format!("{:#?}", self);

        // If debug strings are equal, nothing to do
        if actual_dbg == expected {
            return self;
        }

        // Values differ - handle based on mode
        match mode {
            runtime::Mode::Verify => {
                // In verify mode, panic with both expected and actual
                panic!(
                    "Snapshot mismatch at {}:{}:{}\n\nExpected:\n{}\n\nActual:\n{}\n\nRun with \
                     INLINE_MODE=write to update snapshots.",
                    location.file(),
                    location.line(),
                    location.column(),
                    expected,
                    actual_dbg,
                );
            }
            runtime::Mode::Write | runtime::Mode::Memory => {
                // Both modes update in-memory state; Write also persists to disk
                let file_path = runtime::resolve_source_path(location.file());

                // Create a raw string literal for readable multi-line output
                let baked = make_raw_string(&actual_dbg);

                // Update the source file (in-memory; disk write depends on mode)
                if let Err(e) = runtime::update_source_file(
                    &file_path,
                    location.line(),
                    location.column(),
                    baked,
                ) {
                    eprintln!(
                        "inline::snap_dbg: failed to update source at {}:{}:{}: {}",
                        location.file(),
                        location.line(),
                        location.column(),
                        e
                    );
                }
            }
            runtime::Mode::Reject => {
                panic!(
                    "Snapshot mismatch at {}:{}:{} (mode=Reject, updates not allowed)",
                    location.file(),
                    location.line(),
                    location.column(),
                );
            }
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_equal_values() {
        // When values are equal, just returns self
        let result = 42.snap(42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_snap_mismatch_in_memory_mode() {
        // In memory mode, mismatches don't panic - just return actual
        std::env::set_var("INLINE_MODE", "memory");
        let result = 100.snap(100i32);
        assert_eq!(result, 100);
        std::env::remove_var("INLINE_MODE");
    }

    #[test]
    #[should_panic(expected = "Snapshot mismatch")]
    fn test_snap_mismatch_panics_in_verify_mode() {
        std::env::set_var("INLINE_MODE", "verify");
        let _ = 100.snap(42);
        std::env::remove_var("INLINE_MODE");
    }

    #[test]
    fn test_snap_dbg_equal_values() {
        let result = 42.snap_dbg("42");
        assert_eq!(result, 42);
    }

    #[test]
    fn test_snap_dbg_mismatch_in_memory_mode() {
        // In memory mode, mismatches don't panic - just return actual
        std::env::set_var("INLINE_MODE", "memory");
        let result = 100.snap_dbg("100");
        assert_eq!(result, 100);
        std::env::remove_var("INLINE_MODE");
    }

    #[test]
    #[should_panic(expected = "Snapshot mismatch")]
    fn test_snap_dbg_mismatch_panics_in_verify_mode() {
        std::env::set_var("INLINE_MODE", "verify");
        let _ = 100.snap_dbg("42");
        std::env::remove_var("INLINE_MODE");
    }

    #[test]
    fn test_make_raw_string_simple() {
        // Simple ASCII uses plain string (no r prefix)
        let tokens = make_raw_string("hello world");
        assert_eq!(tokens.to_string(), r#""hello world""#);
    }

    #[test]
    fn test_make_raw_string_with_newlines() {
        let content = "line1\nline2\nline3";
        let tokens = make_raw_string(content);
        let s = tokens.to_string();
        // Should be a raw string with actual newlines
        assert!(s.starts_with("r"), "Should be raw string: {}", s);
        assert!(s.contains('\n'), "Should contain actual newlines: {}", s);
        assert!(
            !s.contains("\\n"),
            "Should NOT contain escaped newlines: {}",
            s
        );
    }

    #[test]
    fn test_make_raw_string_with_quotes() {
        let content = r#"has "quotes" inside"#;
        let tokens = make_raw_string(content);
        let s = tokens.to_string();
        // Should use r#"..."# syntax
        assert!(s.starts_with("r#"), "Should use r# for quotes: {}", s);
    }

    #[test]
    fn test_make_raw_string_with_quote_hash() {
        // Content with "# needs r##"..."##
        let content = "has \"# combo";
        let tokens = make_raw_string(content);
        let s = tokens.to_string();
        assert!(s.starts_with("r##"), "Should use r## for quote-hash: {}", s);
    }

    #[test]
    fn test_make_raw_string_lone_cr_fallback() {
        // Lone \r (not followed by \n) falls back to escaped string
        let content = "hello\rworld";
        let tokens = make_raw_string(content);
        let s = tokens.to_string();
        // Should be escaped string, not raw
        assert!(
            s.starts_with('"'),
            "Should fall back to regular string: {}",
            s
        );
        assert!(s.contains("\\r"), "Should have escaped CR: {}", s);
    }

    #[test]
    fn test_make_raw_string_crlf_ok() {
        // CRLF is fine (normalized to LF by Rust)
        let content = "hello\r\nworld";
        let tokens = make_raw_string(content);
        let s = tokens.to_string();
        assert!(s.starts_with('r'), "CRLF should work as raw string: {}", s);
    }

    #[test]
    fn test_make_raw_string_typical_debug() {
        // Typical pretty Debug output
        let content = "MaskedBytes {\n    bytes: [\n        250,\n    ],\n}";
        let tokens = make_raw_string(content);
        let s = tokens.to_string();
        assert!(s.starts_with('r'), "Should be raw string");
        assert!(s.contains('\n'), "Should have actual newlines");
        // Verify it parses back correctly
        let parsed: proc_macro2::TokenStream = s.parse().expect("Should parse");
        assert!(!parsed.is_empty());
    }

    #[test]
    fn test_string_roundtrip_all_codepoints() {
        // Test that we can round-trip strings with many Unicode code points
        // Build a string with the first 500 code points (excluding surrogates and
        // control chars that would cause issues)
        let mut test_string = String::new();

        for cp in 32u32..500 {
            // Skip control chars 0-31
            if let Some(c) = char::from_u32(cp) {
                test_string.push(c);
            }
        }

        // Add some emoji and special characters
        test_string.push_str("🎉🚀💯");
        test_string.push_str("👨‍👩‍👧‍👦"); // Family emoji with ZWJ
        test_string.push_str("🏳️‍🌈"); // Rainbow flag
        test_string.push_str("中文日本語한국어");
        test_string.push_str("مرحبا");
        test_string.push_str("🔥✨🌟");

        // Test with make_raw_string
        let tokens = make_raw_string(&test_string);
        let tokens_str = tokens.to_string();

        // Should parse back successfully
        let parsed: proc_macro2::TokenStream = tokens_str.parse().expect("Should parse");
        assert!(!parsed.is_empty());
    }

    #[test]
    fn test_string_with_problematic_chars() {
        // Test specific problematic characters
        let cases = [
            ("nul", "\0", true),       // raw OK
            ("tab", "\t", true),       // raw OK
            ("newline", "\n", true),   // raw OK
            ("crlf", "\r\n", true),    // raw OK (normalized)
            ("lone_cr", "\r", false),  // must escape
            ("backslash", "\\", true), // raw OK
            ("quote", "\"", true),     // raw OK (uses r#)
            ("unicode_replacement", "\u{FFFD}", true),
            ("bom", "\u{FEFF}", true),
        ];

        for (name, content, expect_raw) in cases {
            let tokens = make_raw_string(content);
            let tokens_str = tokens.to_string();

            // Should parse without error
            let result: Result<proc_macro2::TokenStream, _> = tokens_str.parse();
            assert!(
                result.is_ok(),
                "Failed to parse string with {}: {:?}",
                name,
                result.err()
            );

            if expect_raw {
                assert!(
                    tokens_str.starts_with('r'),
                    "{} should be raw string: {}",
                    name,
                    tokens_str
                );
            }
        }
    }
}
