//! Fuzz test: protobuf wire format roundtrip.
//!
//! Generates arbitrary Messages, serializes them, parses them back,
//! and verifies the result matches the original.

#![no_main]

use libfuzzer_sys::fuzz_target;
use protobuf_wire::Message;

fuzz_target!(|msg: Message| {
    // Serialize should succeed for any valid Message from Arbitrary
    let bytes = match msg.serialize() {
        Ok(b) => b,
        Err(_) => return, // Invalid field number, skip
    };

    // Parse should succeed for bytes we just serialized
    let parsed = Message::parse(&bytes).expect("failed to parse serialized message");

    // Roundtrip should be lossless
    assert_eq!(msg, parsed, "roundtrip mismatch");
});
