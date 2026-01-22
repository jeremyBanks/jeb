//! Fuzz test: serde Value through JSON roundtrip.
//!
//! Generates arbitrary Values, serializes to JSON, parses back,
//! and checks stability. Note: not all Values survive JSON roundtrip
//! exactly (JSON loses type information, f32→f64, etc.), but after
//! one roundtrip the Value should be stable.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_value::Value;

fuzz_target!(|value: Value| {
    // Serialize Value to JSON
    let json1 = match serde_json::to_string(&value) {
        Ok(j) => j,
        Err(_) => return, // Some values can't be JSON serialized
    };

    // Parse JSON back to Value
    let parsed1: Value = match serde_json::from_str(&json1) {
        Ok(v) => v,
        Err(_) => return,
    };

    // Second roundtrip - Value should be stable
    let json2 = serde_json::to_string(&parsed1).expect("re-serialize failed");
    let parsed2: Value = serde_json::from_str(&json2).expect("re-parse failed");

    // Value equality should be stable after one roundtrip
    // (we don't check JSON string equality because float formatting may vary)
    assert_eq!(parsed1, parsed2, "JSON roundtrip Value not stable");
});
