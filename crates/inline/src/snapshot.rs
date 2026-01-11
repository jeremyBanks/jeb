//! Snapshot testing extension traits.
//!
//! Provides `.snap(expected)` and `.snap_dbg(expected)` methods for inline
//! snapshot testing. These methods compare actual values with expected snapshots
//! and update the source code if they differ.
//!
//! # Example
//!
//! ```no_run
//! use inline::InlineSnapExt;
//!
//! fn compute() -> i32 { 42 }
//!
//! // Snapshot with Bake serialization:
//! let result = compute().snap(0);
//!
//! // Snapshot with Debug formatting:
//! let result = compute().snap_dbg("");
//! ```

use std::{fmt::Debug, panic::Location};

use crate::{runtime, value::Value};

/// Extension trait for inline snapshot testing.
///
/// Provides two snapshot methods:
/// - `.snap(expected)` - For types implementing `Value` (Bake + Clone + PartialEq)
/// - `.snap_dbg(expected)` - For types implementing `Debug`, compares debug output
///
/// Both methods:
/// 1. Compare the actual value with the expected value
/// 2. If they differ and we're in Write mode, update the source file
/// 3. Return the actual value (self)
pub trait InlineSnapExt: Sized {
    /// Compare this value against an expected snapshot using Bake serialization.
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
    /// let result = compute().snap_dbg("expected debug output");
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

        // Values differ - check if we should update the source
        if mode.can_write() {
            // Resolve the source file path
            let file_path = runtime::resolve_source_path(location.file());

            // Bake the actual value to tokens
            let baked = databake::Bake::bake(&self, &Default::default());

            // Update the source file
            if let Err(e) =
                runtime::update_source_file(&file_path, location.line(), location.column(), baked)
            {
                eprintln!(
                    "inline::snap: failed to update source at {}:{}:{}: {}",
                    location.file(),
                    location.line(),
                    location.column(),
                    e
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

        // Values differ - check if we should update the source
        if mode.can_write() {
            // Resolve the source file path
            let file_path = runtime::resolve_source_path(location.file());

            // Create a string literal token for the debug output
            let baked = quote::quote! { #actual_dbg };

            // Update the source file
            if let Err(e) =
                runtime::update_source_file(&file_path, location.line(), location.column(), baked)
            {
                eprintln!(
                    "inline::snap_dbg: failed to update source at {}:{}:{}: {}",
                    location.file(),
                    location.line(),
                    location.column(),
                    e
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
    fn test_snap_returns_actual() {
        // Always returns the actual value, not the expected
        let result = 100.snap(42);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_snap_dbg_equal_values() {
        let result = 42.snap_dbg("42");
        assert_eq!(result, 42);
    }

    #[test]
    fn test_snap_dbg_returns_actual() {
        let result = 100.snap_dbg("42");
        assert_eq!(result, 100);
    }
}
