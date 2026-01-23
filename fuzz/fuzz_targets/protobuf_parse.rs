//! Fuzz test: parse arbitrary bytes as protobuf wire format.
//!
//! This tests that the parser handles all inputs gracefully without panicking.

#![no_main]

use libfuzzer_sys::fuzz_target;
use protobuf_wire::Message;

fuzz_target!(|data: &[u8]| {
    // Should never panic, only return Ok or Err
    let _ = Message::parse(data);
});
