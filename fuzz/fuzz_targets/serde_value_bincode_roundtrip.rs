//! Fuzz test: serde Value through bincode roundtrip.
//!
//! Generates arbitrary Values, serializes to bincode, deserializes back,
//! and verifies the result matches. Bincode is a binary format designed
//! for Rust and should preserve all type information.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_value::Value;

fuzz_target!(|value: Value| {
    // Serialize Value to bincode
    let bytes = match bincode::serialize(&value) {
        Ok(b) => b,
        Err(_) => return, // Some values can't be serialized
    };

    // Deserialize back to Value
    let parsed: Value = match bincode::deserialize(&bytes) {
        Ok(v) => v,
        Err(_) => return,
    };

    // Bincode should preserve Value exactly
    assert_eq!(value, parsed, "bincode roundtrip mismatch");
});
