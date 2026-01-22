//! Fuzz test: serde Value through JSON roundtrip.
//!
//! Generates arbitrary Values, serializes to JSON, parses back.
//! Note: JSON roundtrips are lossy (type info lost, float rounding),
//! so we just test that the operations don't panic.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_value::Value;

fuzz_target!(|value: Value| {
    // Serialize Value to JSON
    let json = match serde_json::to_string(&value) {
        Ok(j) => j,
        Err(_) => return, // Some values can't be JSON serialized
    };

    // Parse JSON back to Value - should not panic
    let parsed: Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(_) => return,
    };

    // Re-serialize should not panic
    let _ = serde_json::to_string(&parsed).expect("re-serialize failed");
});
