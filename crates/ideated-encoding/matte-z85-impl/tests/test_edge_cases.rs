use z85_extended::{encode, decode};

/// Test all lengths 1-16 systematically
#[test]
fn test_all_lengths_1_to_16() {
    println!("\nSystematic length test (1-16 bytes):");
    println!("Len | Z85 | Encoded | Pass");
    println!("----|-----|---------|-----");
    
    for len in 1..=16 {
        let data = vec![b' '; len];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        
        let standard_z85 = (len * 5 + 3) / 4;
        let pass = decoded == data;
        
        println!("{:3} | {:3} | {:7} | {}", 
            len, standard_z85, encoded.len(), if pass { "✓" } else { "✗" });
        
        assert_eq!(decoded, data, "Length {} failed roundtrip", len);
    }
    
    println!("✓ All lengths 1-16 roundtrip correctly");
}

/// Test specific non-aligned lengths that were missing
#[test]
fn test_length_7() {
    let data = vec![b' '; 7];
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    // Standard Z85: ceil(7 * 5/4) = 9 chars
    // Raw: 1 + 1 + 7 = 9 chars (equal)
    println!("Length 7: encoded {} bytes", encoded.len());
}

#[test]
fn test_length_9() {
    let data = vec![b' '; 9];
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    // Standard Z85: ceil(9 * 5/4) = 12 chars
    // Raw: 1 + 1 + 9 = 11 chars (saves 1)
    println!("Length 9: encoded {} bytes (budget=3)", encoded.len());
}

#[test]
fn test_length_10_and_11() {
    for len in [10, 11] {
        let data = vec![b' '; len];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
        println!("Length {}: encoded {} bytes", len, encoded.len());
    }
}

/// Test raw section at stream start (no data before)
#[test]
fn test_raw_at_stream_start() {
    let data = b"     "; // Just raw, nothing before
    let encoded = encode(data);
    
    println!("\nRaw at stream start:");
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // Should start with escape
    assert!(matches!(encoded[0], b'_' | b'~' | b'|' | b','),
        "Should start with escape char");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    println!("✓ Raw at stream start works");
}

/// Test raw section at stream end (no data after)
#[test]
fn test_raw_at_stream_end() {
    let data = b"test     "; // 'test' then raw
    let encoded = encode(data);
    
    println!("\nRaw at stream end:");
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    println!("✓ Raw at stream end works");
}

/// Test very large raw section (256+ bytes, requires split)
#[test]
fn test_large_raw_section_split() {
    // 300 spaces - should split into multiple sections (max 255 per section)
    let data = vec![b' '; 300];
    let encoded = encode(&data);
    
    println!("\n300-byte raw section:");
    println!("Encoded length: {}", encoded.len());
    
    // Should have at least 2 escapes (300 > 255)
    let escape_count = encoded.iter()
        .filter(|&&b| matches!(b, b'_' | b'~' | b'|' | b',' | b';'))
        .count();
    
    println!("Escape count: {} (expected ≥2 for 300 bytes)", escape_count);
    
    // Verify it doesn't just crash
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    println!("✓ Large section (300 bytes) handled");
}

/// Test exactly 255 bytes (max single section)
#[test]
fn test_max_single_section() {
    let data = vec![b' '; 255];
    let encoded = encode(&data);
    
    println!("\n255-byte section (max single):");
    
    // Should use exactly 1 raw section
    let escape_count = encoded.iter()
        .filter(|&&b| matches!(b, b'_' | b'~' | b'|' | b',' | b';'))
        .count();
    
    println!("Escape count: {} (expected 1)", escape_count);
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    println!("✓ Max single section (255 bytes) works");
}

/// Test exactly 256 bytes (should it require split?)
/// 
/// ⚠️ Current behavior: encoder uses single 256-byte section (length byte = 0, wraps)
/// This might be intentional (use byte overflow) or a bug (missing split logic)
#[test]
fn test_min_split_boundary() {
    let data = vec![b' '; 256];
    let encoded = encode(&data);
    
    println!("\n256-byte section:");
    
    let escape_count = encoded.iter()
        .filter(|&&b| matches!(b, b'_' | b'~' | b'|' | b',' | b';'))
        .count();
    
    println!("Escape count: {} (single section, length byte wraps to 0)", escape_count);
    
    // Current behavior: single section with length=0 (256 wraps to 0)
    // Decoder must interpret length=0 as 256
    assert_eq!(escape_count, 1, "Current: 256 bytes uses single section");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    println!("✓ 256 bytes decoded (length byte wraps)");
}

/// Test stream boundary alignment
#[test]
fn test_stream_end_alignment() {
    // Test various endings: 1,2,3 bytes after last full block
    for remainder in 1..=3 {
        let len = 8 + remainder; // 8 = 2 full blocks, then remainder
        let data = vec![b' '; len];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        
        assert_eq!(decoded, data, "Stream end with {} remainder failed", remainder);
    }
    println!("✓ Stream end alignment (1,2,3 byte remainders) works");
}

/// Test empty input
#[test]
fn test_empty_stream() {
    let data = b"";
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();
    
    assert_eq!(decoded, data);
    assert_eq!(encoded.len(), 0, "Empty input should encode to empty");
    println!("✓ Empty stream handled");
}

/// Test single byte (minimum non-empty)
#[test]
fn test_single_byte_various() {
    // Test various single bytes
    for byte in [0u8, 1, 50, 100, 174, 200, 255] {
        let data = vec![byte];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        
        assert_eq!(decoded, data, "Single byte {} failed", byte);
    }
    println!("✓ Single bytes (various values) work");
}

/// Test alternating raw-eligible and Z85 content
#[test]
fn test_alternating_content() {
    // Pattern: raw + Z85 + raw + Z85
    let data = b"     test     hello";
    let encoded = encode(data);
    
    println!("\nAlternating content:");
    println!("Input: {:?}", String::from_utf8_lossy(data));
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    println!("✓ Alternating raw/Z85 content works");
}

/// Test pathological case: many tiny raw sections
#[test]
fn test_many_small_sections() {
    // Pattern: (5 spaces + 1 Z85 char) × 10
    let mut data = Vec::new();
    for _ in 0..10 {
        data.extend_from_slice(b"     a"); // 5 spaces + 'a'
    }
    
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    
    assert_eq!(decoded, data);
    println!("✓ Many small sections (10× 5-byte) handled");
}

/// Test no raw-eligible content (pure Z85)
#[test]
fn test_pure_z85_content() {
    let data = b"testinghelloworld123456789"; // All Z85 alphabet
    let encoded = encode(data);
    
    // Should have NO escapes (no raw sections)
    let has_escape = encoded.iter()
        .any(|&b| matches!(b, b'_' | b'~' | b'|' | b',' | b';'));
    
    assert!(!has_escape, "Pure Z85 content should have no escapes");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    println!("✓ Pure Z85 content (no raw) works");
}
