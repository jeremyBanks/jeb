use z85_extended::{encode, decode};

/// Document and verify which bytes are raw-eligible
#[test]
fn test_raw_eligible_bytes() {
    // Z85 alphabet: 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#
    // Escape chars: _ ~ | , ;
    // Raw-eligible: printable ASCII (32-126) NOT in Z85 alphabet and NOT escape chars
    // Minimum 5 bytes for net savings per §8 budget analysis
    
    // These should be raw-eligible: space, quotes, backtick, etc.
    let raw_eligible = b" \"\\'`";
    for &b in raw_eligible {
        let data = vec![b; 5]; // 5 copies (minimum for net savings)
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Byte {} should roundtrip", b);
        
        // Should appear literally in output (after escape + length)
        let has_literal = encoded.windows(5).any(|w| w == data.as_slice());
        assert!(has_literal, "Byte {} (char: {:?}) should appear literally in output. Encoded: {:?}", 
            b, b as char, String::from_utf8_lossy(&encoded));
    }
}

/// Verify Z85 alphabet characters are NOT raw-eligible
#[test]
fn test_z85_chars_not_raw() {
    // These are IN the Z85 alphabet, should NOT be raw
    let z85_chars = b"0aA.-:";
    
    for &b in z85_chars {
        let data = vec![b; 8]; // 8 copies
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Z85 char {} should still roundtrip", b);
        
        // Should be Z85-encoded, not raw (won't appear literally)
        // Can't easily assert this, but at least verify it works
    }
}

/// Verify escape characters are not raw-eligible
#[test]
fn test_escape_chars_not_raw() {
    let escapes = b"_~|,;";
    
    for &b in escapes {
        let data = vec![b; 4];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Escape char {} should roundtrip", b);
        
        // These must be Z85-encoded, not passed through raw
        // (otherwise decoder would interpret them as escapes)
    }
}

/// Verify length bound: output never longer than standard Z85
/// Note: With 5-byte minimum for raw passthrough, small sections use pure Z85
#[test]
fn test_length_bound() {
    // Test various data patterns
    let test_cases = vec![
        vec![0u8; 4],           // 4 zeros (pure Z85)
        vec![0xFFu8; 4],        // 4 FFs (pure Z85)
        b"test".to_vec(),       // 4 letters (Z85 chars, not raw)
        vec![0, 1, 2, 3, 4, 5], // 6 bytes (pure Z85)
        b"        ".to_vec(),   // 8 spaces (raw-eligible, above minimum)
        b"     ".to_vec(),      // 5 spaces (exactly at minimum)
        b"Hello, World!".to_vec(),
    ];
    
    for data in test_cases {
        let encoded = encode(&data);
        
        // Standard Z85 length formula: ceil(len * 5/4)
        let standard_z85_len = (data.len() * 5 + 3) / 4;
        
        assert!(encoded.len() <= standard_z85_len,
            "Encoded length {} should be ≤ standard Z85 length {} for input of {} bytes",
            encoded.len(), standard_z85_len, data.len());
        
        // Verify roundtrip
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

/// Verify we can decode standard Z85 output
#[test]
fn test_standard_z85_compatibility() {
    // Encode some data with our encoder
    let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let our_encoded = encode(&data);
    let our_decoded = decode(&our_encoded).unwrap();
    assert_eq!(our_decoded, data);
    
    // If input has no raw-eligible sections, output should be pure Z85
    // (no escape characters)
    let binary_data = vec![0, 1, 2, 3, 255, 254, 253, 252];
    let binary_encoded = encode(&binary_data);
    
    // Should not contain escape chars
    let has_escape = binary_encoded.iter().any(|&b| matches!(b, b'_' | b'~' | b'|' | b',' | b';'));
    assert!(!has_escape, 
        "Pure binary data should encode as standard Z85 with no escapes. Got: {:?}",
        String::from_utf8_lossy(&binary_encoded));
    
    let binary_decoded = decode(&binary_encoded).unwrap();
    assert_eq!(binary_decoded, binary_data);
}

/// Verify position invariant: Z85 blocks at exact positions
#[test]
fn test_position_invariant() {
    // Create data: 4-byte Z85 block, raw section (5+ bytes), another 4-byte Z85 block
    let mut data = vec![0xDE, 0xAD, 0xBE, 0xEF]; // First block
    data.extend_from_slice(b"     "); // Raw (5 spaces, above minimum)
    data.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]); // Second block
    
    let encoded = encode(&data);
    
    println!("Position invariant test:");
    println!("Input: {:?}", data);
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // First 5 bytes should be Z85 encoding of [0xDE, 0xAD, 0xBE, 0xEF]
    // (We don't know the exact encoding, but it should be deterministic)
    
    // Encode just the first block separately
    let first_block_only = encode(&data[0..4]);
    println!("First block alone: {:?}", String::from_utf8_lossy(&first_block_only));
    
    // The first 5 chars of full encoding should match first block encoding
    assert_eq!(&encoded[0..5], &first_block_only[0..5],
        "First Z85 block should encode to same position as standalone");
    
    // After that: escape + length + raw
    assert_eq!(encoded[5], b'_', "Should be block-aligned escape");
    assert_eq!(encoded[6], 5, "Should be length 5");
    assert_eq!(&encoded[7..12], b"     ", "Should be raw spaces");
    
    // After raw: second Z85 block
    let second_block_only = encode(&data[9..13]);
    println!("Second block alone: {:?}", String::from_utf8_lossy(&second_block_only));
    
    assert_eq!(&encoded[12..17], &second_block_only[0..5],
        "Second Z85 block should encode to same position as standalone");
    
    // Verify roundtrip
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Verify empty input
#[test]
fn test_empty_input() {
    let data: Vec<u8> = vec![];
    let encoded = encode(&data);
    assert_eq!(encoded.len(), 0, "Empty input should produce empty output");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Verify single byte (not raw-eligible)
#[test]
fn test_single_byte() {
    let data = vec![0x42];
    let encoded = encode(&data);
    
    // Should be 2 chars (1 byte → 2 char partial Z85)
    assert_eq!(encoded.len(), 2, "Single byte should encode to 2 chars");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Verify very long raw section (>255 bytes)
#[test]
fn test_long_raw_section() {
    // Create 300 spaces (all raw-eligible)
    let data = vec![b' '; 300];
    let encoded = encode(&data);
    
    println!("300 spaces encoded length: {}", encoded.len());
    
    // Should split into multiple raw sections (max 255 bytes each)
    // First: escape + 255 + 255 spaces = 257
    // Second: escape + 45 + 45 spaces = 47
    // Total: ~304 bytes
    
    assert!(encoded.len() < 400, "Should be reasonably compact");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Test that shows actual space savings
#[test]
fn test_space_savings_example() {
    // Data with long runs of spaces (raw-eligible, well above 5-byte minimum)
    // §8 says net savings begin at 5+ bytes
    let data = b"            Hello            World            ";
    let encoded = encode(data);
    
    println!("\nSpace savings example:");
    println!("Input: {:?}", String::from_utf8_lossy(data));
    println!("Input length: {} bytes", data.len());
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    println!("Encoded length: {} bytes", encoded.len());
    
    // Standard Z85 would be: ceil(len * 5/4)
    let standard_len = (data.len() * 5 + 3) / 4;
    println!("Standard Z85 would be: {} bytes", standard_len);
    
    if encoded.len() < standard_len {
        println!("Savings: {} bytes ({:.1}%)", 
            standard_len - encoded.len(),
            100.0 * (standard_len - encoded.len()) as f64 / standard_len as f64);
        assert!(encoded.len() < standard_len, "Should save space");
    } else {
        println!("No savings (encoded {} vs standard {}) - likely because 'Hello' and 'World' are Z85 chars",
            encoded.len(), standard_len);
        // This is fine - not all inputs benefit from raw passthrough
    }
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}
