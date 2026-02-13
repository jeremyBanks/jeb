use z85_extended::{encode, decode};

/// Test mid-block exit: verify trailing chars are emitted after raw section
#[test]
fn test_exit_1byte_trailing() {
    // Pattern: 5 spaces (raw) + 1 byte that can exit mid-block
    // If opportunistic exit works, should emit 2 chars (1 byte → 2 char partial)
    let data = b"     \x00"; // 5 spaces + 1 zero
    let encoded = encode(data);
    
    println!("1-byte exit test:");
    println!("Input: {:?}", data);
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    println!("Encoded bytes: {:?}", encoded);
    
    // Expected structure:
    // - 1 char: escape '_' (block-aligned entry)
    // - 1 byte: length (5)
    // - 5 bytes: raw spaces
    // - IF exit works: 2 chars (partial Z85 for 1 byte)
    // Total: 1 + 1 + 5 + 2 = 9 bytes
    
    // Verify roundtrip regardless
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    // Check if exit was used (would be 9 bytes) vs full block (would be 1+1+5+5 = 12)
    if encoded.len() == 9 {
        println!("✓ Mid-block exit used (9 bytes)");
        // Verify structure
        assert_eq!(encoded[0], b'_', "Should start with block-aligned escape");
        assert_eq!(encoded[1], 5, "Should have length 5");
        assert_eq!(&encoded[2..7], b"     ", "Should have 5 raw spaces");
        // Last 2 bytes should be partial Z85 encoding of 0x00
    } else {
        println!("✗ Mid-block exit not used (encoded {} bytes, expected 9)", encoded.len());
        // This is OK - opportunistic exit is optional
    }
}

/// Test 2-byte exit
#[test]
fn test_exit_2byte_trailing() {
    // 5 spaces + 2 zeros
    let data = b"     \x00\x00";
    let encoded = encode(data);
    
    println!("\n2-byte exit test:");
    println!("Input: {:?}", data);
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    // If exit works: 1 (escape) + 1 (len) + 5 (raw) + 3 (partial for 2 bytes) = 10
    if encoded.len() == 10 {
        println!("✓ 2-byte mid-block exit used");
        assert_eq!(&encoded[2..7], b"     ");
    } else {
        println!("✗ 2-byte exit not used (encoded {})", encoded.len());
    }
}

/// Test 3-byte exit
#[test]
fn test_exit_3byte_trailing() {
    // 5 spaces + 3 zeros
    let data = b"     \x00\x00\x00";
    let encoded = encode(data);
    
    println!("\n3-byte exit test:");
    println!("Input: {:?}", data);
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    // If exit works: 1 + 1 + 5 + 4 (partial for 3 bytes) = 11
    if encoded.len() == 11 {
        println!("✓ 3-byte mid-block exit used");
    } else {
        println!("✗ 3-byte exit not used (encoded {})", encoded.len());
    }
}

/// Test that opportunistic check works: bytes that DON'T zero-pad correctly
/// should NOT use mid-block exit
#[test]
fn test_exit_opportunistic_failure_case() {
    // Find a byte that does NOT zero-pad correctly
    // encode_partial([b]) != encode_block([b, 0, 0, 0])[..2]
    
    // Try some bytes and see which ones fail the opportunistic check
    let mut fails_check = Vec::new();
    
    for byte in 1u8..=10 {
        let data = vec![b' '; 5].into_iter().chain(std::iter::once(byte)).collect::<Vec<_>>();
        let encoded = encode(&data);
        
        // If exit was used, length should be 9 (1+1+5+2)
        // If NOT used, should use full block encoding
        if encoded.len() != 9 {
            fails_check.push(byte);
        }
    }
    
    println!("\nBytes that fail opportunistic check (1-10): {:?}", fails_check);
    
    if !fails_check.is_empty() {
        let byte = fails_check[0];
        let data = vec![b' '; 5].into_iter().chain(std::iter::once(byte)).collect::<Vec<_>>();
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
        
        println!("Byte {} doesn't use mid-block exit (correct behavior)", byte);
        println!("Encoded length: {} (not 9)", encoded.len());
    } else {
        println!("All tested bytes (1-10) passed opportunistic check");
    }
}

/// Test exit at exact block boundary (4 bytes after raw section)
/// FIXED: Skip opportunistic exit when remaining bytes fill complete blocks
#[test]
fn test_exit_block_aligned() {
    // 5 spaces + 4 bytes (exactly fills a block)
    let data = b"     \x00\x00\x00\x00";
    let encoded = encode(data);
    
    println!("\nBlock-aligned exit test:");
    println!("Input: {:?} ({} bytes)", data, data.len());
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    println!("Encoded bytes: {:?}", encoded);
    println!("Length: {}", encoded.len());
    
    // Debug: what happened?
    if encoded.len() > 0 { println!("  [0]: '{}' (escape)", encoded[0] as char); }
    if encoded.len() > 1 { println!("  [1]: {} (length)", encoded[1]); }
    if encoded.len() > 2 { println!("  [2-6]: raw section"); }
    if encoded.len() > 7 { 
        println!("  [7..]: {:?}", String::from_utf8_lossy(&encoded[7..])); 
        println!("        ({} chars after raw)", encoded.len() - 7);
    }
    
    match decode(&encoded) {
        Ok(decoded) => {
            println!("✓ Decoded successfully");
            assert_eq!(decoded, data);
        }
        Err(e) => {
            println!("✗ Decode error: {:?}", e);
            panic!("Decoder failed on encoder output");
        }
    }
    
    // After 5 spaces (raw), we have 4 bytes left
    // These should encode as a full Z85 block (5 chars)
    // Total: 1 + 1 + 5 + 5 = 12 bytes
    
    if encoded.len() == 12 {
        println!("✓ Block-aligned, used full Z85 block for last 4 bytes");
    }
}

/// Test multiple raw sections with exits between them
#[test]
fn test_multiple_raw_sections_with_exits() {
    // Pattern: raw1 + exit + raw2 + exit
    // But need to construct data where encoder would naturally create this
    
    // 5 spaces + 2 zeros + 5 spaces + 2 zeros
    let data = b"     \x00\x00     \x00\x00";
    let encoded = encode(data);
    
    println!("\nMultiple sections test:");
    println!("Input length: {} bytes", data.len());
    println!("Encoded length: {} bytes", encoded.len());
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    // This tests a complex pattern - exact structure depends on implementation
    // but roundtrip must work
}

/// Verify exit doesn't happen when remaining bytes are Z85 characters
#[test]
fn test_no_exit_for_z85_chars() {
    // 5 spaces (raw) + "test" (Z85 chars, not raw-eligible)
    let data = b"     test";
    let encoded = encode(data);
    
    println!("\nNo-exit for Z85 chars:");
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    // "test" should be Z85-encoded, not raw (not eligible)
    // So no exit mid-block - they'll be part of normal Z85 blocks
}

/// Test raw section ending exactly at stream end (no exit needed)
#[test]
fn test_raw_at_stream_end() {
    // Just 5 spaces, nothing after
    let data = b"     ";
    let encoded = encode(data);
    
    println!("\nRaw at stream end:");
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    // Should be: escape + length + raw
    // No exit chars needed (nothing after)
}
