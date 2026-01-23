//! Fuzz test: serde Value coercions.
//!
//! Generates arbitrary Values and attempts to deserialize them into various
//! concrete types, exercising the coercion logic. We verify that coercions
//! either succeed or fail gracefully (no panics).

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::Deserialize;
use serde_value::{from_value, Value};
use std::collections::HashMap;

fuzz_target!(|value: Value| {
    // Try deserializing to various integer types (tests integer coercion)
    let _ = from_value::<i8>(value.clone());
    let _ = from_value::<i16>(value.clone());
    let _ = from_value::<i32>(value.clone());
    let _ = from_value::<i64>(value.clone());
    let _ = from_value::<i128>(value.clone());
    let _ = from_value::<u8>(value.clone());
    let _ = from_value::<u16>(value.clone());
    let _ = from_value::<u32>(value.clone());
    let _ = from_value::<u64>(value.clone());
    let _ = from_value::<u128>(value.clone());

    // Try deserializing to float types (tests f32->f64 coercion)
    let _ = from_value::<f32>(value.clone());
    let _ = from_value::<f64>(value.clone());

    // Try deserializing to char (tests string->char, u32->char coercion)
    let _ = from_value::<char>(value.clone());

    // Try deserializing to String (tests bytes->string, char->string coercion)
    let _ = from_value::<String>(value.clone());

    // Try deserializing to Vec<u8> (tests bytes->seq coercion)
    let _ = from_value::<Vec<u8>>(value.clone());

    // Try deserializing to bytes via serde_bytes-style (tests seq->bytes coercion)
    #[derive(Deserialize)]
    struct BytesWrapper(#[serde(with = "serde_bytes")] Vec<u8>);
    let _ = from_value::<BytesWrapper>(value.clone());

    // Try deserializing to HashMap (tests seq-of-pairs->map coercion)
    let _ = from_value::<HashMap<String, Value>>(value.clone());
    let _ = from_value::<HashMap<i32, i32>>(value.clone());

    // Try deserializing to Vec of pairs (tests map->seq-of-pairs coercion)
    let _ = from_value::<Vec<(String, Value)>>(value.clone());
    let _ = from_value::<Vec<(Value, Value)>>(value.clone());

    // Try deserializing to bool
    let _ = from_value::<bool>(value.clone());

    // Try deserializing to Option
    let _ = from_value::<Option<i32>>(value.clone());
    let _ = from_value::<Option<String>>(value.clone());

    // Try deserializing to unit
    let _ = from_value::<()>(value.clone());

    // Try deserializing to tuple
    let _ = from_value::<(i32, String)>(value.clone());
    let _ = from_value::<(u8, u8, u8)>(value.clone());

    // If we get here without panicking, the coercion logic is sound
});

mod serde_bytes {
    use serde::Deserializer;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(BytesVisitor)
    }

    struct BytesVisitor;

    impl<'de> serde::de::Visitor<'de> for BytesVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("bytes")
        }

        fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Vec<u8>, E> {
            Ok(v.to_vec())
        }

        fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<Vec<u8>, E> {
            Ok(v)
        }
    }
}
