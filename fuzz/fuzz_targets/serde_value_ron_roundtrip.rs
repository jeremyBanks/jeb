//! Fuzz test: serde Value through RON roundtrip.
//!
//! Generates arbitrary Values, serializes to RON, parses back,
//! and verifies the result matches. RON (Rusty Object Notation)
//! preserves Rust type information better than JSON.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_value::Value;

fuzz_target!(|value: Value| {
    // Serialize Value to RON
    let ron_str = match ron::to_string(&value) {
        Ok(s) => s,
        Err(_) => return, // Some values can't be RON serialized
    };

    // Parse RON back to Value
    let parsed: Value = match ron::from_str(&ron_str) {
        Ok(v) => v,
        Err(_) => return,
    };

    // RON should preserve Value through roundtrip
    assert_eq!(value, parsed, "RON roundtrip mismatch");
});
