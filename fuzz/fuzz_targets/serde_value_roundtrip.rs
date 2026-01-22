//! Fuzz test: serde Value internal roundtrip.
//!
//! Generates arbitrary Values, tests roundtrip through from_value/to_value.
//! Note: Some type info is lost (e.g., Tuple → Seq) because serde's data model
//! doesn't distinguish them at runtime. We test stability: a second roundtrip
//! should produce the same result.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_value::{from_value, to_value, Value};

fuzz_target!(|value: Value| {
    // First roundtrip
    let rt1: Value = match from_value(value.clone()) {
        Ok(v) => v,
        Err(_) => return,
    };

    // Second roundtrip - should be stable
    let rt2: Value = match from_value(rt1.clone()) {
        Ok(v) => v,
        Err(_) => panic!("second from_value failed but first succeeded"),
    };

    // After one roundtrip, value should be stable
    assert_eq!(rt1, rt2, "roundtrip not stable");

    // Also test to_value roundtrip
    let serialized = to_value(&rt1).expect("to_value failed");
    assert_eq!(rt1, serialized, "to_value should be identity for Value");
});
