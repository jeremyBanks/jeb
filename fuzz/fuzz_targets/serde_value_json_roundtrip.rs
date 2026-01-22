//! Fuzz test: serde Value through JSON roundtrip.
//!
//! Generates arbitrary Values, serializes to JSON, parses back,
//! and checks stability. Note: not all Values survive JSON roundtrip
//! exactly (JSON loses type information), but a second roundtrip should
//! be stable.

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

    // Second roundtrip should be stable
    let json2 = serde_json::to_string(&parsed1).expect("re-serialize failed");
    let parsed2: Value = serde_json::from_str(&json2).expect("re-parse failed");

    assert_eq!(parsed1, parsed2, "JSON roundtrip not stable");
    assert_eq!(json1, json2, "JSON output not stable");
});
