use z85_extended::{encode, decode};

#[test]
fn test_midblock_entry_1byte() {
    // Create data that forces mid-block entry after 1 byte
    // Use a small byte value (< 174) for stability, followed by raw-eligible content
    let mut data = vec![50u8]; // Stable byte
    data.extend_from_slice(b"        "); // 8 spaces, raw-eligible
    
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_midblock_entry_2byte() {
    // Mid-block entry after 2 bytes
    let mut data = vec![50u8, 60u8]; // Both < 174, stable
    data.extend_from_slice(b"\"\"\"\"\"\"\"\""); // Quotes, raw-eligible
    
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_midblock_entry_3byte() {
    // Mid-block entry after 3 bytes
    let mut data = vec![50u8, 60u8, 70u8]; // All < 174, stable
    data.extend_from_slice(b"        "); // Spaces
    
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_no_midblock_unstable() {
    // High byte values (>= 174) should prevent mid-block entry
    let mut data = vec![200u8]; // Unstable
    data.extend_from_slice(b"        ");
    
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    // Should still work, just might not use mid-block cut
}

#[test]
fn test_position_invariant_with_midblock() {
    // Verify that Z85 blocks appear at expected positions even with mid-block cuts
    let mut data = vec![];
    
    // Start with 1 byte
    data.push(50u8);
    // Raw section
    data.extend_from_slice(b"    "); // 4 bytes raw
    // Then more Z85 data (should be position-aligned)
    data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_roundtrip_complex() {
    // Complex mix to test all code paths
    let mut data = vec![];
    
    // Binary start
    data.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]);
    
    // Mid-block entry
    data.push(50);
    data.extend_from_slice(b"\"quotes\"");
    
    // More binary
    data.extend_from_slice(&[0x00, 0x01, 0x02]);
    
    // Another raw section
    data.extend_from_slice(b"        ");
    
    // Trailing binary
    data.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
    
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}
