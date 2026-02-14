#![no_main]

use libfuzzer_sys::fuzz_target;
use z855::{decode, encode};

fuzz_target!(|data: &[u8]| {
    // Roundtrip property: decode(encode(data)) == data
    let encoded = encode(data);
    let decoded = decode(&encoded);
    
    assert_eq!(
        data, &decoded[..],
        "roundtrip failed: {} bytes encoded to {} bytes, decoded to {} bytes",
        data.len(),
        encoded.len(),
        decoded.len()
    );
    
    // Length bound property: encoded length <= original + overhead
    // From DESIGN-CONSTRAINTS.md: worst case is ~1.25x + small constant
    let max_len = data.len() + (data.len() / 4) + 10;
    assert!(
        encoded.len() <= max_len,
        "encoded length {} exceeds bound {} for input length {}",
        encoded.len(),
        max_len,
        data.len()
    );
});
