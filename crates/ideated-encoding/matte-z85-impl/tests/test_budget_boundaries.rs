use z85_extended::{encode, decode};

/// §8: 4 bytes is break-even (budget=1), no net savings
/// Encoder should NOT use raw passthrough (we require 5+ bytes)
#[test]
fn test_4_byte_no_raw_passthrough() {
    // Exactly 4 raw-eligible bytes
    let data = b"    "; // 4 spaces
    let encoded = encode(data);
    
    println!("4-byte test:");
    println!("Input: {:?}", data);
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // Should be pure Z85 (no escape chars)
    let has_escape = encoded.iter().any(|&b| matches!(b, b'_' | b'~' | b'|' | b',' | b';'));
    assert!(!has_escape, 
        "4 bytes (break-even) should use Z85, not raw passthrough. Encoded: {:?}",
        String::from_utf8_lossy(&encoded));
    
    // Standard Z85: 4 bytes → 5 chars
    assert_eq!(encoded.len(), 5, "4 bytes should encode to 5 Z85 chars");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    println!("✓ 4 bytes correctly uses Z85 (no raw passthrough)");
}

/// §8: 5 bytes = budget 2, first net savings
#[test]
fn test_5_byte_first_savings() {
    // Exactly 5 raw-eligible bytes (minimum for raw passthrough)
    let data = b"     "; // 5 spaces
    let encoded = encode(data);
    
    println!("\n5-byte test:");
    println!("Encoded: {:?}", String::from_utf8_lossy(&encoded));
    
    // Should use raw passthrough (has escape)
    let has_escape = encoded.iter().any(|&b| matches!(b, b'_' | b'~' | b'|' | b',' | b';'));
    assert!(has_escape, "5 bytes should use raw passthrough");
    
    // Raw: 1 (escape) + 1 (len) + 5 (data) = 7 bytes
    // Z85: ceil(5 * 5/4) = 7 bytes
    // Equal! But we prefer raw for transparency
    assert_eq!(encoded.len(), 7);
    
    // Should contain literal spaces
    let encoded_str = String::from_utf8_lossy(&encoded);
    assert!(encoded_str.contains("     "), "Should contain literal spaces");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    println!("✓ 5 bytes uses raw passthrough (first net savings)");
}

/// §8: 6-8 bytes = budget 2
#[test]
fn test_6_to_8_bytes_budget_2() {
    for len in 6..=8 {
        let data = vec![b' '; len];
        let encoded = encode(&data);
        
        // Should use raw passthrough
        let has_escape = encoded.iter().any(|&b| matches!(b, b'_' | b'~' | b'|' | b',' | b';'));
        assert!(has_escape, "{} bytes should use raw passthrough", len);
        
        // Standard Z85 would be: ceil(len * 5/4)
        let standard_len = (len * 5 + 3) / 4;
        
        // Raw: 1 + 1 + len
        let raw_len = 2 + len;
        
        println!("{} bytes: raw={}, Z85={}, encoded={}", len, raw_len, standard_len, encoded.len());
        assert!(encoded.len() <= standard_len, "{} bytes should not exceed standard Z85", len);
        
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
    println!("✓ 6-8 bytes (budget=2) use raw passthrough");
}

/// §8: 9-12 bytes = budget 3 (comfortable)
#[test]
fn test_9_to_12_bytes_budget_3() {
    for len in 9..=12 {
        let data = vec![b' '; len];
        let encoded = encode(&data);
        
        // Should definitely use raw passthrough (comfortable budget)
        let has_escape = encoded.iter().any(|&b| matches!(b, b'_' | b'~' | b'|' | b',' | b';'));
        assert!(has_escape, "{} bytes should use raw passthrough", len);
        
        let standard_len = (len * 5 + 3) / 4;
        let raw_len = 2 + len;
        
        println!("{} bytes: raw={}, Z85={}, encoded={}, savings={}", 
            len, raw_len, standard_len, encoded.len(), standard_len - encoded.len());
        
        // Should save significant space
        assert!(encoded.len() < standard_len, "{} bytes should save space", len);
        
        // Budget = 3 means we have 3 chars of overhead budget
        // Using 2 (escape + len), so comfortable
        assert_eq!(standard_len - len, 3, "{} bytes should have budget=3", len);
        
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
    println!("✓ 9-12 bytes (budget=3) use raw passthrough efficiently");
}

/// §8: 13-16 bytes = budget 4 (very comfortable)
#[test]
fn test_13_to_16_bytes_budget_4() {
    for len in 13..=16 {
        let data = vec![b' '; len];
        let encoded = encode(&data);
        
        let standard_len = (len * 5 + 3) / 4;
        let raw_len = 2 + len;
        
        println!("{} bytes: raw={}, Z85={}, savings={}", 
            len, raw_len, standard_len, standard_len - raw_len);
        
        assert!(encoded.len() <= raw_len, "{} bytes should use raw", len);
        
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
    println!("✓ 13-16 bytes (budget=4) very comfortable");
}

/// Test exact budget calculation formula
#[test]
fn test_budget_formula() {
    // Budget = ⌈N × 5/4⌉ - N
    
    let test_cases = vec![
        (4, 1),   // 5 - 4 = 1
        (5, 2),   // 7 - 5 = 2
        (6, 2),   // 8 - 6 = 2
        (7, 2),   // 9 - 7 = 2
        (8, 2),   // 10 - 8 = 2
        (9, 3),   // 12 - 9 = 3
        (12, 3),  // 15 - 12 = 3
        (13, 4),  // 17 - 13 = 4
        (16, 4),  // 20 - 16 = 4
    ];
    
    println!("\nBudget calculations:");
    println!("N bytes | Z85 chars | Budget");
    println!("--------|-----------|-------");
    
    for (n, expected_budget) in test_cases {
        let z85_len = (n * 5 + 3) / 4;
        let budget = z85_len - n;
        
        println!("{:7} | {:9} | {:6}", n, z85_len, budget);
        assert_eq!(budget, expected_budget, 
            "{} bytes should have budget {}", n, expected_budget);
    }
    
    println!("✓ Budget formula verified");
}

/// Test that budget is used efficiently for mid-block cuts
/// 
/// FIXED: Added budget check to encoder - now respects P1 position invariant
#[test]
fn test_midblock_budget_usage() {
    // 1 byte (stable) + 5 spaces = total 6 bytes
    // Standard Z85: 8 chars
    // With mid-block entry:
    //   - 2 chars (partial for 1 byte)
    //   - 1 char (escape)
    //   - 1 byte (length)
    //   - 5 bytes (raw)
    // Total: 2 + 1 + 1 + 5 = 9 bytes
    
    let data = b"\x32     "; // 0x32 = 50 < 174 (stable)
    let encoded = encode(data);
    
    println!("\nMid-block budget test:");
    println!("Encoded length: {}", encoded.len());
    println!("Standard Z85: 8 chars");
    
    // With budget check: encoder detects mid-block would violate P1
    // Falls back to pure Z85 (8 chars)
    assert!(encoded.len() <= 8, "Must not exceed standard Z85 (P1 violation)");
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    
    if encoded.len() == 8 {
        println!("✓ Encoder correctly avoided mid-block (would violate P1)");
    } else {
        println!("✓ Mid-block used within budget");
    }
}

/// Test large N: budget approaches N/4
#[test]
fn test_large_budget() {
    let data = vec![b' '; 100];
    let encoded = encode(&data);
    
    // Standard Z85: ceil(100 * 5/4) = 125 chars
    let standard_len = (100 * 5 + 3) / 4;
    assert_eq!(standard_len, 125);
    
    // Budget = 125 - 100 = 25
    // This is 25% of N, which is ~N/4
    let budget = standard_len - 100;
    println!("\n100 bytes: standard Z85={}, budget={} ({}%)", 
        standard_len, budget, budget * 100 / 100);
    
    assert_eq!(budget, 25);
    
    // Raw encoding: 1 + 1 + 100 = 102 bytes
    // Savings: 125 - 102 = 23 bytes (92% of budget used)
    assert!(encoded.len() <= standard_len);
    
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    println!("✓ Large N has abundant budget (~N/4)");
}
