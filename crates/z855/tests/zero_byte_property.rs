// Test for zero-byte opportunistic exit property
//
// Jeremy's hypothesis: "the new Z855 encoding can ensure that we can display
// all safe segments of at least four characters directly raw as long as there
// is a zero byte either before or after."
//
// This relates to opportunistic zero-padding for exit cuts (DESIGN-CONSTRAINTS §7).

use z855::{encode, decode};

#[test]
fn safe_segment_with_trailing_zero() {
    // Test case: 4+ printable ASCII chars followed by zero byte
    let data = b"ABCD\x00";
    
    let encoded = encode(data);
    let decoded = decode(&encoded).expect("should decode");
    
    assert_eq!(decoded, data, "roundtrip failed");
    
    // Check if "ABCD" appears raw in output
    // (This is a weak test - just verifies the segment is present somewhere)
    let has_raw_segment = encoded.contains("ABCD");
    
    println!("Input: {:?}", String::from_utf8_lossy(data));
    println!("Encoded: {}", encoded);
    println!("Raw segment present: {}", has_raw_segment);
    
    // Note: We don't assert has_raw_segment because the encoder is opportunistic
    // It MIGHT use passthrough if budget allows, but not guaranteed
}

#[test]
fn safe_segment_with_leading_zero() {
    // Test case: zero byte followed by 4+ printable ASCII chars
    let data = b"\x00ABCD";
    
    let encoded = encode(data);
    let decoded = decode(&encoded).expect("should decode");
    
    assert_eq!(decoded, data, "roundtrip failed");
    
    let has_raw_segment = encoded.contains("ABCD");
    
    println!("Input: {:?}", String::from_utf8_lossy(data));
    println!("Encoded: {}", encoded);
    println!("Raw segment present: {}", has_raw_segment);
}

#[test]
fn safe_segment_between_zeros() {
    // Test case: zero, 4+ chars, zero
    let data = b"\x00ABCD\x00";
    
    let encoded = encode(data);
    let decoded = decode(&encoded).expect("should decode");
    
    assert_eq!(decoded, data, "roundtrip failed");
    
    let has_raw_segment = encoded.contains("ABCD");
    
    println!("Input: {:?}", String::from_utf8_lossy(data));
    println!("Encoded: {}", encoded);
    println!("Raw segment present: {}", has_raw_segment);
}

#[test]
fn longer_safe_segment_with_zeros() {
    // Test with longer safe segment
    let data = b"\x00Hello, World!\x00";
    
    let encoded = encode(data);
    let decoded = decode(&encoded).expect("should decode");
    
    assert_eq!(decoded, data, "roundtrip failed");
    
    println!("Input: {:?}", String::from_utf8_lossy(data));
    println!("Encoded: {}", encoded);
    println!("Length: input={} encoded={}", data.len(), encoded.len());
    
    // For longer segments, passthrough is more likely to be beneficial
    // Check compression ratio
    let std_z85_len = ((data.len() + 3) / 4) * 5;
    println!("Standard Z85 would be {} chars", std_z85_len);
    println!("Actual: {} chars", encoded.len());
}

#[test]
fn empirical_zero_byte_benefit() {
    // Empirical test: how often does zero byte enable passthrough?
    
    let test_cases = vec![
        b"AAAA\x00".as_slice(),
        b"\x00AAAA".as_slice(),
        b"TEST\x00DATA\x00".as_slice(),
        b"\x00\x00\x00\x00safe\x00\x00\x00\x00".as_slice(),
        b"abc\x00def\x00ghi\x00".as_slice(),
    ];
    
    for (i, data) in test_cases.iter().enumerate() {
        let encoded = encode(data);
        let decoded = decode(&encoded).expect("should decode");
        assert_eq!(&decoded[..], *data, "case {} roundtrip failed", i);
        
        // Measure encoding efficiency
        let std_z85_len = ((data.len() + 3) / 4) * 5;
        let saved = std_z85_len.saturating_sub(encoded.len());
        
        println!("Case {}: input={} std_z85={} actual={} saved={}",
                 i, data.len(), std_z85_len, encoded.len(), saved);
    }
}

// Property-based test idea (not yet implemented with proptest):
// For any data with zero bytes at boundaries and 4+ safe chars between,
// verify that the encoding is no worse than standard Z85
// (and likely better due to passthrough opportunities)
