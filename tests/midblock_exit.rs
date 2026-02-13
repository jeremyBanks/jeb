use z85_extended::{encode, decode};

#[test]
fn test_midblock_exit_opportunistic() {
    // Test case: raw section followed by bytes that can exit mid-block
    // The bytes [0, 0] can exit opportunistically because encode_partial([0,0]) == encode_block([0,0,0,0])[..3]
    
    // Build test data: 
    // - 4 spaces (raw-eligible, will be raw section)
    // - followed by [0, 0] which should exit mid-block if opportunistic check passes
    let data = b"    \x00\x00";
    
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();
    
    assert_eq!(decoded, data, "Roundtrip failed");
    
    // The encoding should be shorter than full Z85 due to raw passthrough
    // Full Z85 would be: ceil(6 * 5/4) = 8 chars
    // With raw+exit: escape(1) + len(1) + raw(4) + partial_exit(3) = 9 chars
    // Actually this might not save characters, but it should still work
    
    println!("Input: {:?}", data);
    println!("Encoded ({} bytes): {:?}", encoded.len(), String::from_utf8_lossy(&encoded));
    println!("Decoded: {:?}", decoded);
}

#[test]
fn test_midblock_exit_vs_block_aligned() {
    // Create data where mid-block exit is possible
    // Using zeros because encode_partial([0]) == encode_block([0,0,0,0])[..2] = "00"
    
    let data1 = b"     \x00"; // 5 spaces + 1 zero
    let data2 = b"    \x00";  // 4 spaces + 1 zero
    
    for data in [data1 as &[u8], data2 as &[u8]] {
        let encoded = encode(data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Roundtrip failed for {:?}", data);
    }
}

#[test]
fn test_midblock_exit_complex() {
    // More complex case: multiple sections with opportunistic exits
    //Data: "test"(4 raw) + [0](1 byte, can exit) + " hello"(6 raw, but starts mid-block)
    
    let data = b"test\x00 hello";
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();
    
    assert_eq!(decoded, data, "Complex roundtrip failed");
    
    println!("\nComplex test:");
    println!("Input: {:?}", String::from_utf8_lossy(data));
    println!("Encoded ({} bytes): {:?}", encoded.len(), String::from_utf8_lossy(&encoded));
}
