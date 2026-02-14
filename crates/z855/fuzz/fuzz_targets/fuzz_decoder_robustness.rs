#![no_main]

use libfuzzer_sys::fuzz_target;
use z855::decode;

fuzz_target!(|data: &[u8]| {
    // Decoder must never panic on any input
    // This is a critical safety property - decoder should be liberal
    let _ = decode(data);
    
    // If we get here without panicking, the test passes
    // The decoder may return any bytes, but must not crash
});
