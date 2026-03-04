//! Hex-b1032 encoding for generation indices.
//!
//! Similar to b1032 but for hex commit hashes:
//! - 0-9999: decimal representation
//! - 10000-65535: hex representation, skipping all-digit values
//! - 65536+: hex representation, no constraints

/// Check if a 4-digit hex value (0x0000-0xFFFF) is "good" (contains at least one letter A-F).
#[cfg(test)]
fn is_good_hex(v: u32) -> bool {
    debug_assert!(v <= 0xFFFF);
    // Extract the 4 hex digits
    let d0 = (v >> 12) & 0xF;
    let d1 = (v >> 8) & 0xF;
    let d2 = (v >> 4) & 0xF;
    let d3 = v & 0xF;
    
    // Good if at least one digit is A-F (>= 0xA)
    d0 >= 0xA || d1 >= 0xA || d2 >= 0xA || d3 >= 0xA
}

/// Count how many "good" hex values exist in [0, v].
fn good_hex_count(v: u32) -> u32 {
    debug_assert!(v <= 0xFFFF);
    
    // Total values up to v
    let total = v + 1;
    
    // Bad values are those where all 4 digits are 0-9
    // Bad count in [0, v] = how many values have all digits < 0xA
    let d0 = (v >> 12) & 0xF;
    let d1 = (v >> 8) & 0xF;
    let d2 = (v >> 4) & 0xF;
    let d3 = v & 0xF;
    
    // Count bad values using same logic as b1032's bad_leq
    let mut tight = 1u32;
    let mut loose = 0u32;
    
    for lim in [d0, d1, d2, d3] {
        let mut nt = 0u32;
        let mut nl = 0u32;
        
        // Count values 0-9 (decimal digits)
        for x in 0..0xA {  // 0 through 9
            if x > lim {
                break;
            }
            if x == lim {
                nt += tight;
            } else {
                nl += tight;
            }
        }
        nl += loose * 10; // Each loose position can be any of 0-9
        
        tight = nt;
        loose = nl;
    }
    
    let bad_count = tight + loose;
    total - bad_count
}

/// Find the k-th good hex value (0-indexed) in the range [0, 0xFFFF].
fn unrank_good_hex(k: u32) -> u32 {
    debug_assert!(k < 0x10000 - 10000, "k out of range for 4-digit hex");
    
    let mut lo = 0u32;
    let mut hi = 0xFFFF;
    let target = k + 1;
    
    while lo < hi {
        let mid = (lo + hi) / 2;
        if good_hex_count(mid) >= target {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    
    lo
}

/// Encode a generation index using hex-b1032 scheme.
///
/// - 0-9999: decimal
/// - 10000-65535: hex (skipping all-digit values)
/// - 65536+: hex (no constraints)
pub fn encode_generation_index(gen_idx: u32) -> String {
    if gen_idx <= 9999 {
        // Decimal range
        format!("{}", gen_idx)
    } else if gen_idx <= 65535 {
        // Hex range with skipping
        // Direct hex values start at 39322 (0x999A)
        if gen_idx >= 39322 {
            // Past all the bad values, use direct hex
            format!("{:04X}", gen_idx)
        } else {
            // Map to k-th good value
            let k = gen_idx - 10000;
            let hex_val = unrank_good_hex(k);
            format!("{:04X}", hex_val)
        }
    } else {
        // Large values, no constraint
        format!("{:X}", gen_idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_good_hex() {
        // All digits (bad)
        assert!(!is_good_hex(0x0000));
        assert!(!is_good_hex(0x1234));
        assert!(!is_good_hex(0x9999));
        
        // Has at least one letter (good)
        assert!(is_good_hex(0xA000));
        assert!(is_good_hex(0x0A00));
        assert!(is_good_hex(0x00A0));
        assert!(is_good_hex(0x000A));
        assert!(is_good_hex(0xABCD));
        assert!(is_good_hex(0x999A));
        assert!(is_good_hex(0xFFFF));
    }

    #[test]
    fn test_good_hex_count() {
        // No good values at 0
        assert_eq!(good_hex_count(0), 0);
        
        // First good value is at 0x000A (10)
        assert_eq!(good_hex_count(0x0009), 0);
        assert_eq!(good_hex_count(0x000A), 1);
        
        // At 0x9999, there are 39322 total values, 10000 bad (all-digit) ones
        // So 29322 good values
        assert_eq!(good_hex_count(0x9999), 39322 - 10000);
        
        // At 0x999A, more good values
        assert!(good_hex_count(0x999A) > good_hex_count(0x9999));
        
        // At 0xFFFF, should have total - 10000 good values
        assert_eq!(good_hex_count(0xFFFF), 0x10000 - 10000);
    }

    #[test]
    fn test_encode_decimal_range() {
        assert_eq!(encode_generation_index(0), "0");
        assert_eq!(encode_generation_index(42), "42");
        assert_eq!(encode_generation_index(9999), "9999");
    }

    #[test]
    fn test_encode_hex_range() {
        // First value in hex range
        let encoded_10000 = encode_generation_index(10000);
        // Should be first good hex >= 0x2710
        // 0x2710 = "2710" (all digits, bad)
        // First good should be something like "271A" or similar
        assert!(encoded_10000.len() == 4);
        assert!(encoded_10000.chars().any(|c| matches!(c, 'A'..='F')));
        
        // Should not produce all-digit values
        for idx in 10000..10100 {
            let encoded = encode_generation_index(idx);
            assert!(encoded.chars().any(|c| matches!(c, 'A'..='F')),
                    "idx {} produced all-digit hex: {}", idx, encoded);
        }
    }

    #[test]
    fn test_encode_large_values() {
        assert_eq!(encode_generation_index(65536), "10000");
        assert_eq!(encode_generation_index(0x100000), "100000");
    }

    #[test]
    fn test_no_ambiguity() {
        // Decimal range produces values without letters
        for idx in 0..=9999 {
            let encoded = encode_generation_index(idx);
            assert!(encoded.chars().all(|c| c.is_ascii_digit()));
        }
        
        // Hex range (10000-65535) produces values with at least one letter
        for idx in (10000..20000).step_by(100) {
            let encoded = encode_generation_index(idx);
            if idx <= 65535 {
                assert!(encoded.chars().any(|c| matches!(c, 'A'..='F')));
            }
        }
    }
}
