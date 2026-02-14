#![no_main]

use libfuzzer_sys::fuzz_target;
use z855::decode;

fuzz_target!(|data: &[u8]| {
    // Decoder must never panic on any input
    // This is a critical safety property - decoder should be liberal
    
    // Try to decode as UTF-8 string (decoder expects &str)
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = decode(s);
    }
    
    // If we get here without panicking, the test passes
    // The decoder may return any bytes or error, but must not crash
});
