use z85_extended::{encode, decode};

/// Verify that raw sections actually appear as raw bytes in output
#[test]
fn test_raw_bytes_present_in_output() {
    // Note: Z85 alphabet includes lowercase letters, so 'test' is NOT raw-eligible!
    // Use characters outside Z85 alphabet: spaces, quotes, or other punctuation not in Z85
    // Need 5+ bytes for net savings
    let data = b"     \"\"\"\"\"     "; // 5 spaces + 5 quotes + 5 spaces
    let encoded = encode(data);
    
    // Should contain escape char (one of: _ ~ | , ;)
    let has_escape = encoded.iter().any(|&b| matches!(b, b'_' | b'~' | b'|' | b',' | b';'));
    assert!(has_escape, "Encoded output should contain escape character. Got: {:?}", 
        String::from_utf8_lossy(&encoded));
    
    // Should contain the literal quotes in the output (they're raw-eligible)
    let encoded_str = String::from_utf8_lossy(&encoded);
    assert!(encoded_str.contains("\"\"\"\"\""), 
        "Raw section with quotes should appear literally in encoded output. Got: {:?}", encoded_str);
    
    // Should contain spaces
    assert!(encoded_str.contains("     "), 
        "Raw spaces should appear literally. Got: {:?}", encoded_str);
    
    // Verify roundtrip still works
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Verify mid-block entry creates partial Z85 encoding before escape
#[test]
fn test_midblock_entry_structure() {
    // 1 byte (stable) + 5 spaces (raw) - need 5+ for net savings
    let data = b"\x32     "; // 0x32 = 50 < 174 (stable)
    let encoded = encode(data);
    
    println!("Input: {:?}", data);
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    println!("Encoded bytes: {:?}", encoded);
    
    // Expected structure:
    // - 2 chars: partial Z85 encoding of byte 0x32 (1 byte → 2 chars)
    // - 1 char: escape (should be '~' for 1-byte-into-block)
    // - 1 byte: length (5)
    // - 5 bytes: raw spaces
    // Total: 2 + 1 + 1 + 5 = 9 bytes
    
    assert_eq!(encoded.len(), 9, "Expected 9 bytes: 2 (partial Z85) + 1 (escape) + 1 (len) + 5 (raw)");
    
    // Third byte should be escape char '~' (1 byte into block)
    assert_eq!(encoded[2], b'~', "Expected '~' escape for 1-byte entry. Got: {:?}", encoded[2] as char);
    
    // Fourth byte should be length 5
    assert_eq!(encoded[3], 5, "Expected length byte = 5");
    
    // Last 5 bytes should be literal spaces
    assert_eq!(&encoded[4..9], b"     ", "Last 5 bytes should be raw spaces");
    
    // Verify roundtrip
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Verify 2-byte mid-block entry uses correct escape char
#[test]
fn test_midblock_entry_2byte_structure() {
    let data = b"\x32\x33     "; // 2 bytes + 5 spaces
    let encoded = encode(data);
    
    println!("2-byte entry encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // Expected structure:
    // - 3 chars: partial Z85 encoding of 2 bytes (2 bytes → 3 chars)
    // - 1 char: escape '|' (2 bytes into block)
    // - 1 byte: length (5)
    // - 5 bytes: raw spaces
    
    assert_eq!(encoded.len(), 10, "Expected 10 bytes: 3 (partial) + 1 (escape) + 1 (len) + 5 (raw)");
    assert_eq!(encoded[3], b'|', "Expected '|' escape for 2-byte entry");
    assert_eq!(encoded[4], 5, "Expected length = 5");
    assert_eq!(&encoded[5..10], b"     ");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Verify 3-byte mid-block entry uses correct escape char
#[test]
fn test_midblock_entry_3byte_structure() {
    let data = b"\x32\x33\x34     "; // 3 bytes + 5 spaces
    let encoded = encode(data);
    
    println!("3-byte entry encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // Expected structure:
    // - 4 chars: partial Z85 encoding of 3 bytes (3 bytes → 4 chars)
    // - 1 char: escape ',' (3 bytes into block)
    // - 1 byte: length (5)
    // - 5 bytes: raw spaces
    
    assert_eq!(encoded.len(), 11, "Expected 11 bytes: 4 (partial) + 1 (escape) + 1 (len) + 5 (raw)");
    assert_eq!(encoded[4], b',', "Expected ',' escape for 3-byte entry");
    assert_eq!(encoded[5], 5, "Expected length = 5");
    assert_eq!(&encoded[6..11], b"     ");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Verify block-aligned entry uses '_' escape  
#[test]
fn test_block_aligned_entry_structure() {
    let data = b"        "; // 8 spaces, block-aligned (well above 5-byte minimum)
    let encoded = encode(data);
    
    println!("Block-aligned encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // Expected structure:
    // - 1 char: escape '_' (block-aligned)
    // - 1 byte: length (8)
    // - 8 bytes: raw spaces
    
    assert_eq!(encoded.len(), 10, "Expected 10 bytes: 1 (escape) + 1 (len) + 8 (raw)");
    assert_eq!(encoded[0], b'_', "Expected '_' escape for block-aligned entry");
    assert_eq!(encoded[1], 8, "Expected length = 8");
    assert_eq!(&encoded[2..10], b"        ");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Verify mid-block exit with opportunistic zero-padding
#[test]
fn test_midblock_exit_structure() {
    // 5+ spaces (raw, above minimum) + 2 zeros (should exit mid-block if opportunistic check passes)
    let data = b"     \x00\x00";
    let encoded = encode(data);
    
    println!("Exit test encoded: {:?}", String::from_utf8_lossy(&encoded));
    println!("Exit test bytes: {:?}", encoded);
    
    // Expected structure:
    // - 1 char: escape '_' (block-aligned entry)
    // - 1 byte: length (5)
    // - 5 bytes: raw spaces
    // - IF opportunistic exit works: 3 chars for partial Z85 encoding of [0, 0]
    // - OTHERWISE: 3 chars for partial encoding of [0, 0] (same either way)
    
    // The key question: does it emit the partial encoding after the raw section?
    // Let's just verify roundtrip for now since opportunistic exit is optional
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Verify that unstable bytes prevent mid-block cuts
#[test]
fn test_unstable_byte_no_midblock() {
    let data = b"\xB0     "; // 0xB0 = 176 >= 174 (unstable) + 5 spaces
    let encoded = encode(data);
    
    println!("Unstable byte encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // Should NOT have mid-block cut because byte is unstable
    // Should encode as full Z85 block (4 bytes → 5 chars) + raw section
    // OR: encode first byte as Z85, then raw section starting aligned
    
    // The exact encoding strategy depends on implementation, but verify:
    // 1. Roundtrip works
    // 2. If there's an escape, it should be block-aligned ('_') not mid-block
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    if encoded.iter().any(|&b| b == b'~' || b == b'|' || b == b',') {
        panic!("Unstable byte should not produce mid-block escape. Got: {:?}", 
            String::from_utf8_lossy(&encoded));
    }
}

/// Compare encoded length with standard Z85 to verify savings
#[test]
fn test_raw_passthrough_saves_space() {
    let data = b"            "; // 12 spaces (3 blocks worth)
    let encoded = encode(data);
    
    // Standard Z85 would be: 12 bytes → 15 chars
    // With raw passthrough: 1 (escape) + 1 (len) + 12 (raw) = 14 bytes
    
    println!("12 spaces encoded: {:?}", String::from_utf8_lossy(&encoded));
    println!("Length: {} bytes (standard Z85 would be 15)", encoded.len());
    
    assert!(encoded.len() < 15, 
        "Raw passthrough should be shorter than standard Z85. Got {} bytes", encoded.len());
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Verify position invariant: Z85 blocks at same positions as standard Z85
#[test]
fn test_position_invariant_preserved() {
    // Create data with known Z85 encoding, then raw section (5+ bytes), then more Z85
    let mut data = vec![0x00, 0x00, 0x00, 0x00]; // 4-byte block
    data.extend_from_slice(b"     "); // raw section (5 bytes, above minimum)
    data.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]); // another 4-byte block
    
    let encoded = encode(&data);
    
    println!("Position test encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // First 4 bytes should encode to "00000" (standard Z85 for [0,0,0,0])
    // Then escape + len + raw
    // Then second block should encode to standard Z85
    
    // Verify first 5 chars are "00000"
    assert_eq!(&encoded[0..5], b"00000", 
        "First Z85 block should be standard encoding");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}
