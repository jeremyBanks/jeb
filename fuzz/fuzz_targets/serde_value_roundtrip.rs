//! Fuzz test: serde Value internal roundtrip.
//!
//! Generates arbitrary Values, deserializes them as Value (via from_value),
//! serializes back (via to_value), and verifies the result matches.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_value::{from_value, to_value, Value};

fuzz_target!(|value: Value| {
    // Deserialize Value as Value - tests the Deserializer impl
    let deserialized: Value = match from_value(value.clone()) {
        Ok(v) => v,
        Err(_) => return,
    };

    // Serialize back to Value - tests the Serialize impl
    let reserialized = match to_value(&deserialized) {
        Ok(v) => v,
        Err(_) => return,
    };

    // Roundtrip should be lossless
    assert_eq!(value, deserialized, "from_value mismatch");
    assert_eq!(deserialized, reserialized, "to_value mismatch");
});
