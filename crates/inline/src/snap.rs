//! Snapshot comparison using `PartialEq` (`==` operator).
//!
//! This module provides the `Snap` type which enables snapshot testing via
//! the `==` operator:
//!
//! ```no_run
//! use inline::snap;
//!
//! let actual = compute_something();
//! snap!("expected value") == actual; // Updates source if mismatch
//! ```
//!
//! The comparison returns `true` if values match or were successfully updated,
//! and panics in Verify/Reject modes if there's a mismatch.

use {
    crate::{
        runtime,
        snapshot::make_raw_string,
        value::Value,
    },
    std::{
        fmt::Debug,
        panic::Location,
    },
};

/// A snapshot value for comparison with `==`.
///
/// Created via the [`snap!`] macro. When compared against an actual value,
/// updates the source code if they don't match (in Write mode) or panics
/// (in Verify mode).
///
/// # Example
///
/// ```no_run
/// use inline::snap;
///
/// let result = compute();
/// snap!(42) == result; // Updates source if result != 42
/// ```
pub struct Snap<T> {
    /// The expected value from source code
    pub value: T,
    file: &'static str,
    line: u32,
    column: u32,
}

impl<T> Snap<T> {
    /// Create a new Snap with the given value.
    ///
    /// Usually called via the [`snap!`] macro which captures the source
    /// location.
    #[track_caller]
    pub fn new(value: T) -> Self {
        let location = Location::caller();
        Self {
            value,
            file: location.file(),
            line: location.line(),
            column: location.column(),
        }
    }

    /// Create a Snap with explicit location (for testing).
    #[doc(hidden)]
    pub fn __new(value: T, file: &'static str, line: u32, column: u32) -> Self {
        Self {
            value,
            file,
            line,
            column,
        }
    }
}

impl<T, E> PartialEq<E> for Snap<T>
where
    T: PartialEq<E> + Debug,
    E: Value + 'static,
{
    #[allow(unsafe_code)]
    fn eq(&self, actual: &E) -> bool {
        use std::any::TypeId;

        // If values match, nothing to do
        if self.value == *actual {
            return true;
        }

        // Values differ - handle based on mode
        let mode = runtime::get_mode();

        match mode {
            runtime::Mode::Verify => {
                let actual_baked = databake::Bake::bake(actual, &Default::default());
                panic!(
                    "Snapshot mismatch at {}:{}:{}\n\nExpected:\n{:#?}\n\nActual:\n{}\n\nRun with \
                     INLINE_MODE=write to update snapshots.",
                    self.file, self.line, self.column, self.value, actual_baked,
                );
            }
            runtime::Mode::Write | runtime::Mode::Memory => {
                // Both modes update in-memory state; Write also persists to disk
                let file_path = runtime::resolve_source_path(self.file);

                // Special case for String: output just the raw string literal
                let baked = if TypeId::of::<E>() == TypeId::of::<String>() {
                    // SAFETY: We just verified E is String
                    let s: &String = unsafe { &*(actual as *const E as *const String) };
                    make_raw_string(s)
                } else {
                    databake::Bake::bake(actual, &Default::default())
                };

                if let Err(e) =
                    runtime::update_source_file(&file_path, self.line, self.column, baked)
                {
                    eprintln!(
                        "inline::snap: failed to update source at {}:{}:{}: {}",
                        self.file, self.line, self.column, e
                    );
                }
            }
            runtime::Mode::Reject => {
                panic!(
                    "Snapshot mismatch at {}:{}:{} (mode=Reject, updates not allowed)",
                    self.file, self.line, self.column,
                );
            }
        }

        true
    }
}

// Implement Eq for Snap when inner type is Eq
impl<T: Eq + Debug> Eq for Snap<T> where Snap<T>: PartialEq {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_eq_matching_values() {
        // When values match, eq returns true
        let snap = Snap::new(42);
        assert!(snap == 42);
    }

    #[test]
    fn test_snap_eq_matching_string() {
        // String can compare against &str
        let snap = Snap::new("hello");
        let actual = "hello".to_string();
        assert!(snap == actual);
    }

    #[test]
    fn test_snap_eq_mismatch_memory_mode() {
        // In memory mode, mismatch still returns true (conceptually updated)
        unsafe {
            std::env::set_var("INLINE_MODE", "memory");
        }
        let snap = Snap::new(100i32);
        assert!(snap == 100); // Different values but returns true in memory mode
        unsafe {
            std::env::remove_var("INLINE_MODE");
        }
    }

    #[test]
    #[should_panic(expected = "Snapshot mismatch")]
    fn test_snap_eq_mismatch_verify_mode() {
        unsafe {
            std::env::set_var("INLINE_MODE", "verify");
        }
        let snap = Snap::new(42);
        let _ = snap == 100; // Should panic
        unsafe {
            std::env::remove_var("INLINE_MODE");
        }
    }
}
