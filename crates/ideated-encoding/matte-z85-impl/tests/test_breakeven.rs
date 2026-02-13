use z85_extended::encode;

/// Find exact break-even points for mid-block encoding
#[test]
fn find_midblock_breakeven_points() {
    println!("\nFinding mid-block break-even points:\n");
    
    // Test 1-byte partial with increasing raw lengths
    println!("1-byte partial:");
    for raw_len in 5..15 {
        let mut data = vec![0x32u8]; // 1 stable byte
        data.extend(vec![b' '; raw_len]);
        
        let encoded = encode(&data);
        let standard = (data.len() * 5 + 3) / 4;
        let uses_midblock = encoded.contains(&b'~');
        
        println!("  {} spaces: {} chars (std: {}) {}", 
            raw_len, encoded.len(), standard,
            if uses_midblock { "✓ mid-block" } else { "✗ pure Z85" });
    }
    
    // Test 2-byte partial
    println!("\n2-byte partial:");
    for raw_len in 5..15 {
        let mut data = vec![0x32u8, 0x33u8]; // 2 stable bytes
        data.extend(vec![b' '; raw_len]);
        
        let encoded = encode(&data);
        let standard = (data.len() * 5 + 3) / 4;
        let uses_midblock = encoded.contains(&b'|');
        
        println!("  {} spaces: {} chars (std: {}) {}", 
            raw_len, encoded.len(), standard,
            if uses_midblock { "✓ mid-block" } else { "✗ pure Z85" });
    }
    
    // Test 3-byte partial
    println!("\n3-byte partial:");
    for raw_len in 5..15 {
        let mut data = vec![0x32u8, 0x33u8, 0x34u8]; // 3 stable bytes
        data.extend(vec![b' '; raw_len]);
        
        let encoded = encode(&data);
        let standard = (data.len() * 5 + 3) / 4;
        let uses_midblock = encoded.contains(&b',');
        
        println!("  {} spaces: {} chars (std: {}) {}", 
            raw_len, encoded.len(), standard,
            if uses_midblock { "✓ mid-block" } else { "✗ pure Z85" });
    }
}
