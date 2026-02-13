use z85_extended::{encode, decode};

/// Property: All valid input should roundtrip
#[test]
fn property_roundtrip() {
    // Test random patterns
    for seed in 0..100 {
        let len = (seed % 50) + 1; // 1-50 bytes
        let data: Vec<u8> = (0..len).map(|i| ((seed + i * 17) % 256) as u8).collect();
        
        let encoded = encode(&data);
        match decode(&encoded) {
            Ok(decoded) => {
                assert_eq!(decoded, data, 
                    "Roundtrip failed for seed {} (len {})", seed, len);
            }
            Err(e) => {
                panic!("Decode failed for seed {} (len {}): {:?}\nEncoded: {:?}", 
                    seed, len, e, String::from_utf8_lossy(&encoded));
            }
        }
    }
}

/// Property: Output length never exceeds standard Z85
#[test]
fn property_length_bound() {
    for len in 1..100 {
        // Test various byte patterns
        for pattern in [0u8, 255, 32, 127, 50, 100, 174, 173] {
            let data = vec![pattern; len];
            let encoded = encode(&data);
            let standard_z85_len = (len * 5 + 3) / 4;
            
            assert!(encoded.len() <= standard_z85_len,
                "Length violation: {} bytes ({} pattern) → {} chars, standard Z85 = {}",
                len, pattern, encoded.len(), standard_z85_len);
        }
    }
}

/// Property: Raw-eligible bytes should appear literally when beneficial
#[test]
fn property_transparency() {
    // Long runs of spaces should be raw
    let data = vec![b' '; 20];
    let encoded = encode(&data);
    
    // Should contain literal spaces (after escape+length header)
    let encoded_str = String::from_utf8_lossy(&encoded);
    assert!(encoded_str.contains("     "), // At least 5 spaces visible
        "Long space run should be transparent. Encoded: {:?}", encoded_str);
}

/// Property: Alternating raw-eligible / Z85 content
#[test]
fn property_mixed_content() {
    // Pattern: raw + Z85 + raw + Z85
    for _ in 0..10 {
        let mut data = Vec::new();
        data.extend_from_slice(b"     "); // Raw
        data.extend_from_slice(b"test"); // Z85
        data.extend_from_slice(b"     "); // Raw
        data.extend_from_slice(b"1234"); // Z85
        
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

/// Property: Edge lengths (boundaries)
#[test]
fn property_boundary_lengths() {
    // Test lengths near 4-byte boundaries
    for base_len in [4, 8, 12, 16, 20, 24, 28, 32] {
        for offset in 0..5 {
            let len = base_len + offset;
            let data = vec![b' '; len]; // Raw-eligible
            
            let encoded = encode(&data);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, data, "Failed at length {}", len);
        }
    }
}

/// Property: Non-printable bytes never appear raw
#[test]
fn property_non_printable_encoded() {
    for byte in 0..32u8 {
        let data = vec![byte; 10];
        let encoded = encode(&data);
        
        // Encoded should NOT contain the raw byte value
        assert!(!encoded.contains(&byte),
            "Non-printable byte {} appeared in output", byte);
        
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
    
    // DEL and high bytes
    for byte in [127u8, 128, 200, 255] {
        let data = vec![byte; 10];
        let encoded = encode(&data);
        assert!(!encoded.contains(&byte),
            "Non-printable byte {} appeared in output", byte);
    }
}

/// Property: Stability - small input changes produce valid output
#[test]
fn property_stability() {
    let base = vec![b' '; 20];
    
    // Flip each byte and verify still valid
    for i in 0..base.len() {
        let mut data = base.clone();
        data[i] = b'x'; // Change to Z85 char
        
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Failed when flipping byte {}", i);
    }
}

/// Property: Concatenation of encoded sections
#[test]
fn property_concatenation() {
    // Encode two parts separately, then together
    let part1 = b"test     ";
    let part2 = b"     hello";
    
    let encoded1 = encode(part1);
    let encoded2 = encode(part2);
    
    let mut combined = part1.to_vec();
    combined.extend_from_slice(part2);
    let encoded_combined = encode(&combined);
    
    // Sanity: all should decode
    decode(&encoded1).unwrap();
    decode(&encoded2).unwrap();
    decode(&encoded_combined).unwrap();
    
    // Note: encoded_combined might not equal encoded1 + encoded2
    // because encoder makes different decisions based on full context
}

/// Property: Empty sections
#[test]
fn property_empty() {
    let data = b"";
    let encoded = encode(data);
    assert_eq!(encoded.len(), 0, "Empty input should encode to empty");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Property: Single bytes of various values
#[test]
fn property_single_bytes() {
    for byte in 0..=255u8 {
        let data = vec![byte];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Single byte {} failed", byte);
    }
}

/// Property: Large inputs (stress test)
#[test]
fn property_large_inputs() {
    for len in [100, 255, 256, 300, 500, 1000] {
        let data = vec![b' '; len]; // Raw-eligible
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Failed at length {}", len);
        
        // Check standard Z85 length bound
        let standard_len = (len * 5 + 3) / 4;
        assert!(encoded.len() <= standard_len,
            "Length bound violated at {} bytes", len);
    }
}

/// Property: Escape characters never appear raw
#[test]
fn property_escape_chars_encoded() {
    for &escape in b"_~|,;" {
        let data = vec![escape; 10];
        let encoded = encode(&data);
        
        // Count escapes in encoded output
        let escape_count = encoded.iter().filter(|&&b| b == escape).count();
        
        // Should have escapes (for raw sections), but not 10 in a row
        // The data bytes themselves should be Z85-encoded, not raw
        assert!(escape_count < 10,
            "Escape char {} appeared too many times (likely raw)", escape as char);
        
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}

/// Property: Decoder rejects invalid input
#[test]
fn property_decoder_rejects_invalid() {
    // Invalid Z85 character (not in alphabet, not an escape, not raw-eligible)
    // Use a control character that would appear in what should be Z85 context
    // Newline (10) is control char, definitely not in Z85
    let invalid = b"0000\n0000"; // \n not valid Z85 char
    match decode(invalid) {
        Ok(_) => panic!("Should reject control char in Z85 context"),
        Err(e) => println!("✓ Correctly rejected: {:?}", e),
    }
    
    // Truncated after escape
    let truncated = b"_"; // Escape but no length
    assert!(decode(truncated).is_err(), "Should reject truncated escape");
    
    // Truncated raw section
    let truncated_raw = b"_\x0AABCDE"; // Says 10 bytes, only gives 5
    assert!(decode(truncated_raw).is_err(), "Should reject truncated raw");
}

/// Property: Position invariant - Z85 blocks at correct positions
#[test]
fn property_position_invariant() {
    // Pattern: 8 Z85 bytes (2 blocks) + 10 spaces (raw) + 4 Z85 bytes (1 block)
    let data = b"testdata          more";
    let encoded = encode(data);
    
    // First 10 chars should be pure Z85 (2 blocks of 4 bytes each)
    // Standard Z85 for "testdata": same as first 10 chars of encoded
    let just_prefix = b"testdata";
    let prefix_encoded = encode(just_prefix);
    
    // First 10 chars should match (position invariant)
    assert_eq!(&encoded[0..10], &prefix_encoded[0..10],
        "Position invariant violated - Z85 blocks not at same positions");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}
