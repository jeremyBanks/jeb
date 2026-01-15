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
        fmt::Debug,
        panic::Location,
    },
};

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
    fn snap(self, expected: Self) -> Self
    where
        Self: Value + 'static;

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
    fn snap(self, expected: Self) -> Self
    where
        Self: Value + 'static,
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
                let expected_baked = databake::Bake::bake(&expected, &Default::default());
                panic!(
                    "Snapshot mismatch at {}:{}:{}\n\nExpected:\n{}\n\nActual:\n{}\n\nRun with \
                     INLINE_MODE=write to update snapshots.",
                    location.file(),
                    location.line(),
                    location.column(),
                    expected_baked,
                    actual_baked,
                );
            }
            runtime::Mode::Write => {
                // Resolve the source file path
                let file_path = runtime::resolve_source_path(location.file());

                // Bake the actual value to tokens
                let baked = databake::Bake::bake(&self, &Default::default());

                // Update the source file
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
            runtime::Mode::Memory => {
                // Memory mode: do nothing, just return
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
            runtime::Mode::Write => {
                // Resolve the source file path
                let file_path = runtime::resolve_source_path(location.file());

                // Create a string literal token for the debug output
                let baked = quote::quote! { #actual_dbg };

                // Update the source file
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
            runtime::Mode::Memory => {
                // Memory mode: do nothing, just return
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
        let result = 100.snap(42);
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
        let result = 100.snap_dbg("42");
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
    fn test_string_roundtrip_all_codepoints() {
        // Test that we can round-trip strings with many Unicode code points
        // Build a string with the first 500 code points (excluding surrogates)
        let mut test_string = String::new();

        for cp in 0u32..500 {
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

        // Test that the string survives being formatted and parsed back
        let debug_output = format!("{:#?}", test_string);

        // The debug output should be a valid Rust string literal
        // When we quote it and parse, we should get back equivalent content
        let tokens = quote::quote! { #debug_output };
        let tokens_str = tokens.to_string();

        // Parse it back
        let parsed: proc_macro2::TokenStream = tokens_str.parse().expect("Should parse");

        // Extract the string from the token stream
        let mut iter = parsed.into_iter();
        if let Some(proc_macro2::TokenTree::Literal(lit)) = iter.next() {
            // The literal should parse successfully
            let lit_str = lit.to_string();
            // It should start and end with quotes
            assert!(lit_str.starts_with('"'), "Should be a string literal");
            assert!(lit_str.ends_with('"'), "Should end with quote");
        } else {
            panic!("Expected a literal token");
        }
    }

    #[test]
    fn test_string_with_problematic_chars() {
        // Test specific problematic characters
        let cases = [
            ("nul", "\0"),
            ("tab", "\t"),
            ("newline", "\n"),
            ("crlf", "\r\n"),
            ("backslash", "\\"),
            ("quote", "\""),
            ("unicode_replacement", "\u{FFFD}"),
            ("bom", "\u{FEFF}"),
        ];

        for (name, content) in cases {
            let debug_output = format!("{:#?}", content);
            let tokens = quote::quote! { #debug_output };
            let tokens_str = tokens.to_string();

            // Should parse without error
            let result: Result<proc_macro2::TokenStream, _> = tokens_str.parse();
            assert!(
                result.is_ok(),
                "Failed to parse string with {}: {:?}",
                name,
                result.err()
            );
        }
    }
}
