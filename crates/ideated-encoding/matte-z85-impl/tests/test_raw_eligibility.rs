use z85_extended::{encode, decode};

/// Z85 alphabet for reference (characters that should NOT be raw-eligible)
const Z85_ALPHABET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/// Escape characters (should NOT be raw-eligible)
const ESCAPE_CHARS: &[u8] = b"_~|,;";

/// Test all printable ASCII for raw eligibility
#[test]
fn test_comprehensive_printable_ascii_eligibility() {
    // Printable ASCII: 32-126 (space through tilde)
    // Raw-eligible: printable AND not in Z85 alphabet AND not escape char
    
    let mut raw_eligible = Vec::new();
    let mut not_eligible = Vec::new();
    
    for byte in 32u8..=126 {
        let is_z85 = Z85_ALPHABET.as_bytes().contains(&byte);
        let is_escape = ESCAPE_CHARS.contains(&byte);
        
        if !is_z85 && !is_escape {
            raw_eligible.push(byte);
        } else {
            not_eligible.push(byte);
        }
    }
    
    println!("\nRaw-eligible characters ({}):", raw_eligible.len());
    println!("{:?}", String::from_utf8_lossy(&raw_eligible));
    
    println!("\nNOT raw-eligible ({}):", not_eligible.len());
    println!("Z85 alphabet: {}", Z85_ALPHABET);
    println!("Escapes: {:?}", String::from_utf8_lossy(ESCAPE_CHARS));
    
    // Test each raw-eligible byte (need 5+ for passthrough)
    for &byte in &raw_eligible {
        let data = vec![byte; 5];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Byte {} (char: {:?}) should roundtrip", byte, byte as char);
        
        // Should appear literally in output (after escape + length)
        let has_literal = encoded.windows(5).any(|w| w == data.as_slice());
        assert!(has_literal, 
            "Raw-eligible byte {} (char: {:?}) should appear literally. Encoded: {:?}",
            byte, byte as char, String::from_utf8_lossy(&encoded));
    }
    
    // Verify we found the expected raw-eligible bytes
    // Expected: space, quotes, some punctuation not in Z85
    assert!(raw_eligible.contains(&b' '), "Space should be raw-eligible");
    assert!(raw_eligible.contains(&b'"'), "Quote should be raw-eligible");
    assert!(raw_eligible.contains(&b'\''), "Apostrophe should be raw-eligible");
    assert!(raw_eligible.contains(&b'`'), "Backtick should be raw-eligible");
    
    // Verify Z85 chars and escapes are NOT in raw-eligible
    for byte in b"0aA.-_~" {
        assert!(!raw_eligible.contains(byte), 
            "Byte {} (char: {}) should NOT be raw-eligible", byte, *byte as char);
    }
}

/// Test that non-printable bytes are NEVER passed through raw
#[test]
fn test_non_printable_always_z85() {
    // Test control characters (0-31)
    for byte in 0u8..32 {
        let data = vec![byte; 5];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Non-printable byte {} should roundtrip", byte);
        
        // Should NOT appear literally (should be Z85-encoded)
        let has_literal = encoded.windows(5).any(|w| w == data.as_slice());
        assert!(!has_literal,
            "Non-printable byte {} should be Z85-encoded, not raw. Encoded: {:?}",
            byte, String::from_utf8_lossy(&encoded));
    }
    
    // Test DEL (127)
    let data = vec![127u8; 5];
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    assert!(!encoded.windows(5).any(|w| w == data.as_slice()),
        "DEL (127) should be Z85-encoded, not raw");
    
    // Test high bytes (128-255)
    for byte in [128u8, 200, 255] {
        let data = vec![byte; 5];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "High byte {} should roundtrip", byte);
        
        let has_literal = encoded.windows(5).any(|w| w == data.as_slice());
        assert!(!has_literal,
            "High byte {} should be Z85-encoded, not raw", byte);
    }
}

/// Test mixed eligible/ineligible content
#[test]
fn test_mixed_eligible_ineligible() {
    // Pattern: Z85 chars + raw-eligible + Z85 chars
    let data = b"test     world"; // 'test' and 'world' are Z85 chars, spaces are raw
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    println!("\nMixed content:");
    println!("Input: {:?}", String::from_utf8_lossy(data));
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // The 5+ spaces in the middle should be raw
    let encoded_str = String::from_utf8_lossy(&encoded);
    assert!(encoded_str.contains("     "),
        "Raw-eligible spaces should appear literally");
}

/// Test that escape characters are NOT raw-eligible
#[test]
fn test_escape_chars_not_raw_eligible() {
    // Escape chars: _ ~ | , ;
    for &byte in ESCAPE_CHARS {
        let data = vec![byte; 5];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Escape char {} should roundtrip", byte);
        
        // Must be Z85-encoded (if passed through raw, decoder would interpret as escape)
        let has_literal = encoded.windows(5).any(|w| w == data.as_slice());
        assert!(!has_literal,
            "Escape char {} (char: {:?}) should be Z85-encoded, not raw. Encoded: {:?}",
            byte, byte as char, String::from_utf8_lossy(&encoded));
    }
}

/// Verify the exact count of raw-eligible bytes
#[test]
fn test_raw_eligible_count() {
    let mut count = 0;
    
    for byte in 32u8..=126 {
        let is_z85 = Z85_ALPHABET.as_bytes().contains(&byte);
        let is_escape = ESCAPE_CHARS.contains(&byte);
        
        if !is_z85 && !is_escape {
            count += 1;
        }
    }
    
    // Printable ASCII: 95 total (32-126 inclusive)
    // Z85 alphabet: 85 chars
    // Escapes: 5 chars
    // But Z85 and escapes may overlap... let's check
    
    let z85_count = Z85_ALPHABET.len();
    let escape_count = ESCAPE_CHARS.len();
    
    // Check if any escapes are in Z85 alphabet
    let overlap = ESCAPE_CHARS.iter()
        .filter(|&&b| Z85_ALPHABET.as_bytes().contains(&b))
        .count();
    
    println!("\nCounts:");
    println!("Total printable ASCII (32-126): 95");
    println!("Z85 alphabet: {}", z85_count);
    println!("Escape chars: {}", escape_count);
    println!("Overlap (escapes in Z85): {}", overlap);
    println!("Expected raw-eligible: 95 - {} - {} + {} = {}", 
        z85_count, escape_count, overlap, 95 - z85_count - escape_count + overlap);
    println!("Actual raw-eligible: {}", count);
    
    assert_eq!(count, 95 - z85_count - escape_count + overlap,
        "Raw-eligible count should match calculation");
}
