//! Snapshot testing extension trait.
//!
//! Provides a `.snapshot(expected)` method that can be called on any value
//! implementing `Value`. The method compares the actual value with the expected
//! snapshot and updates the source code if they differ.
//!
//! # Example
//!
//! ```no_run
//! use inline::Snapshot;
//!
//! fn compute() -> i32 { 42 }
//!
//! // In a test:
//! let result = compute().snapshot(0);  // Expected value in source
//! // If compute() returns 42 but source says 0, the source is updated to 42
//! ```

use std::panic::Location;

use crate::{runtime, value::Value};

/// Extension trait for snapshot testing on any `Value` type.
///
/// This trait provides a `.snapshot(expected)` method that:
/// 1. Compares the actual value (`self`) with the expected value
/// 2. If they differ and we're in Write mode, updates the source file
/// 3. Returns the actual value
///
/// The expected value in the source code is updated to match the actual value,
/// making this ideal for snapshot testing workflows.
pub trait Snapshot: Sized {
    /// Compare this value against an expected snapshot.
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
    /// use inline::Snapshot;
    ///
    /// let actual = some_computation();
    /// let result = actual.snapshot(expected_value);
    /// // result == actual, and source is updated if actual != expected_value
    /// ```
    #[track_caller]
    fn snapshot(self, expected: Self) -> Self
    where
        Self: Value + 'static;
}

impl<T> Snapshot for T {
    #[track_caller]
    fn snapshot(self, expected: Self) -> Self
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
                    "inline::snapshot: failed to update source at {}:{}:{}: {}",
                    location.file(),
                    location.line(),
                    location.column(),
                    e
                );
            }
        } else if mode.should_reject_write() {
            // In Reject/Verify mode, we might want to panic or warn
            // For now, just note the mismatch silently
            // TODO: Consider panicking in Verify mode?
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_equal_values() {
        // When values are equal, just returns self
        let result = 42.snapshot(42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_snapshot_returns_actual() {
        // Always returns the actual value, not the expected
        // File updates only happen in Write mode; in tests we're typically in Verify mode
        // which doesn't write, so this just tests the return value behavior
        let result = 100.snapshot(42);
        assert_eq!(result, 100);
    }
}
