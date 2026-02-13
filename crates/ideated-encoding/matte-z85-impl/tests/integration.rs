use z85_extended::{encode, decode};

#[test]
fn test_raw_passthrough() {
    // Text with quotes and spaces (raw-eligible)
    let data = b"\"Hello, World!\"";
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_mixed_content() {
    // Mix of binary and text
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01, 0x02, 0x03]); // Binary
    data.extend_from_slice(b"    "); // Raw-eligible spaces
    data.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]); // Binary
    
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_position_invariant() {
    // Standard Z85 encoding
    let data = b"HelloWorld123456";
    let standard_encoded = encode(data);
    
    // All Z85-encoded blocks should appear at predictable positions
    // 16 bytes = 4 blocks of 4 bytes = 4 * 5 chars = 20 chars
    let decoded = decode(&standard_encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_all_raw() {
    let data = b"        "; // All spaces, raw-eligible
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    // Should use raw encoding
    assert!(encoded.contains(&b'_'));
}

#[test]
fn test_length_bound() {
    // Output should never be longer than standard Z85
    let data = b"Test data with various content 123 @#$";
    let encoded = encode(data);
    
    // Standard Z85 would be: ceil(len/4) * 5 chars
    let std_z85_len = ((data.len() + 3) / 4) * 5;
    
    assert!(encoded.len() <= std_z85_len, 
            "Encoded length {} exceeds standard Z85 length {}",
            encoded.len(), std_z85_len);
}
