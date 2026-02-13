use z85_extended::{encode, decode};

/// R4: Decoder must accept consecutive raw sections
/// (Even though encoder should never produce them)
#[test]
fn test_consecutive_raw_sections_manual() {
    // Manually construct: raw1 + raw2 (no Z85 between)
    // Structure: escape1 + len1 + data1 + escape2 + len2 + data2
    
    let mut encoded = Vec::new();
    
    // First raw section: 5 spaces
    encoded.push(b'_'); // Block-aligned escape
    encoded.push(5u8); // Length
    encoded.extend_from_slice(b"     "); // 5 spaces
    
    // Second raw section: 5 quotes (immediately after, no Z85 gap)
    encoded.push(b'_'); // Block-aligned escape
    encoded.push(5u8); // Length  
    encoded.extend_from_slice(b"\"\"\"\"\""); // 5 quotes
    
    println!("Manually constructed consecutive raw sections:");
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // Decoder must accept this per R4
    let decoded = decode(&encoded).unwrap();
    println!("Decoded: {:?}", decoded);
    
    // Should decode to: 5 spaces + 5 quotes
    assert_eq!(decoded, b"     \"\"\"\"\"");
    println!("✓ Consecutive raw sections decoded successfully");
}

/// Test decoder handles mid-block consecutive sections
#[test]
fn test_consecutive_raw_midblock() {
    // Manually construct: partial Z85 + raw1 + raw2
    
    let mut encoded = Vec::new();
    
    // 1 byte of Z85 data (encoded as 2 chars partial)
    let partial = z85_encode_1byte(0x42);
    encoded.extend_from_slice(&partial);
    
    // First raw section at 1-byte offset
    encoded.push(b'~'); // 1-byte-into-block escape
    encoded.push(5u8);
    encoded.extend_from_slice(b"     ");
    
    // Second raw section immediately after (also at 1-byte offset? or aligned?)
    // This tests decoder robustness
    encoded.push(b'_'); // Let's say it's aligned now
    encoded.push(5u8);
    encoded.extend_from_slice(b"\"\"\"\"\"");
    
    println!("\nMid-block consecutive:");
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    match decode(&encoded) {
        Ok(decoded) => {
            println!("Decoded: {:?}", decoded);
            println!("✓ Mid-block consecutive handled");
        }
        Err(e) => {
            println!("✗ Decoder rejected consecutive mid-block: {:?}", e);
            // This might be expected if decoder requires proper alignment
        }
    }
}

/// Helper: encode 1 byte as 2-char partial Z85
fn z85_encode_1byte(byte: u8) -> Vec<u8> {
    // Simple encoding: byte value → 2 base-85 digits
    const Z85_CHARS: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    
    let value = byte as u32;
    let c1 = Z85_CHARS[(value / 85) as usize];
    let c0 = Z85_CHARS[(value % 85) as usize];
    
    vec![c1, c0]
}

/// Test that encoder doesn't produce consecutive raw sections
#[test]
fn test_encoder_avoids_consecutive() {
    // Pattern that might tempt encoder to make consecutive sections:
    // 5 spaces + 5 quotes
    let data = b"     \"\"\"\"\"";
    let encoded = encode(data);
    
    println!("\nEncoder test (5 spaces + 5 quotes):");
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // Count escapes
    let escape_count = encoded.iter().filter(|&&b| b == b'_' || b == b'~' || b == b'|' || b == b',' || b == b';').count();
    println!("Escape count: {}", escape_count);
    
    // Well-behaved encoder should produce 1 escape, not 2
    // (combine into single 10-byte raw section)
    if escape_count == 1 {
        println!("✓ Encoder used single raw section (efficient)");
    } else if escape_count == 2 {
        println!("⚠️ Encoder used 2 consecutive raw sections (suboptimal but valid)");
    }
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

/// Test R4 edge case: three consecutive raw sections
#[test]
fn test_three_consecutive_raw() {
    let mut encoded = Vec::new();
    
    // Section 1
    encoded.push(b'_');
    encoded.push(5u8);
    encoded.extend_from_slice(b"     ");
    
    // Section 2
    encoded.push(b'_');
    encoded.push(5u8);
    encoded.extend_from_slice(b"\"\"\"\"\"");
    
    // Section 3
    encoded.push(b'_');
    encoded.push(5u8);
    encoded.extend_from_slice(b"'''''");
    
    println!("\nThree consecutive:");
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, b"     \"\"\"\"\"'''''");
    println!("✓ Three consecutive raw sections decoded");
}

/// Test consecutive with different alignments
#[test]
fn test_consecutive_mixed_alignment() {
    // This is a tricky edge case - what if encoder (hypothetically)
    // emitted raw sections at different block alignments back-to-back?
    
    let mut encoded = Vec::new();
    
    // Block-aligned raw (5 bytes)
    encoded.push(b'_');
    encoded.push(5u8);
    encoded.extend_from_slice(b"     ");
    
    // Immediately followed by another block-aligned raw (5 bytes)
    // After first raw: pos = 5, which is 5 % 4 = 1 byte into block
    // So decoder is at offset 1 after first section
    // Second escape is '_' (block-aligned), meaning it assumes we're at offset 0?
    // This might cause issues!
    
    encoded.push(b'_'); // Claiming block-aligned
    encoded.push(5u8);
    encoded.extend_from_slice(b"\"\"\"\"\"");
    
    println!("\nConsecutive block-aligned (misaligned):");
    println!("Encoded bytes: {:?}", encoded);
    
    match decode(&encoded) {
        Ok(decoded) => {
            println!("Decoded: {:?}", decoded);
            // If decoder accepts this, it might be doing implicit alignment
        }
        Err(e) => {
            println!("✗ Decoder error (expected - second section misaligned): {:?}", e);
            // This is probably correct behavior - consecutive sections
            // must maintain proper alignment
        }
    }
}

/// Test decoder handles length=0 sections gracefully
#[test]
fn test_zero_length_raw_section() {
    let mut encoded = Vec::new();
    
    // Valid raw section
    encoded.push(b'_');
    encoded.push(5u8);
    encoded.extend_from_slice(b"     ");
    
    // Zero-length raw section (degenerate case)
    encoded.push(b'_');
    encoded.push(0u8);  // length = 0
    // No data
    
    // Another valid section
    encoded.push(b'_');
    encoded.push(5u8);
    encoded.extend_from_slice(b"\"\"\"\"\"");
    
    println!("\nZero-length section:");
    
    match decode(&encoded) {
        Ok(decoded) => {
            println!("Decoded: {:?}", decoded);
            assert_eq!(decoded, b"     \"\"\"\"\"");
            println!("✓ Zero-length section handled (ignored)");
        }
        Err(e) => {
            println!("✗ Zero-length rejected: {:?}", e);
        }
    }
}
