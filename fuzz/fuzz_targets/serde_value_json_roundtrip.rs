//! Fuzz test: serde Value through JSON roundtrip.
//!
//! Generates arbitrary Values, serializes to JSON, parses back,
//! and checks the result. Note: not all Values survive JSON roundtrip
//! (JSON loses type information), so we only check it doesn't panic.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_value::Value;

fuzz_target!(|value: Value| {
    // Serialize Value to JSON - should not panic
    let json = match serde_json::to_string(&value) {
        Ok(j) => j,
        Err(_) => return, // Some values can't be JSON serialized
    };

    // Parse JSON back to Value - should not panic
    let _: Result<Value, _> = serde_json::from_str(&json);
});
