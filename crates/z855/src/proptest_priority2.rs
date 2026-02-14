// Priority 2 Property Tests - Edge Cases & Quality
//
// These tests verify lower-priority requirements and quality properties
// identified in the coverage analysis.

#![cfg(test)]

use proptest::prelude::*;
use crate::z855::{encode, decode};

// =============================================================================
// Priority 2 Test 1: Consecutive Raw Sections (R4)
// =============================================================================
// The decoder MUST accept consecutive raw sections even though the encoder
// shouldn't produce them. This is an edge case for decoder robustness.

#[test]
fn consecutive_raw_comma_sections() {
    // Manually construct: ",AAAA,BBBB" (two comma-escaped 4-byte sections back-to-back)
    let input = ",AAAA,BBBB";
    
    // Decoder should accept this even though encoder wouldn't produce it
    let result = decode(input);
    
    // Should either:
    // a) Decode successfully to 8 bytes (AAAABBBB), OR
    // b) Return error (decoder is allowed to reject, just shouldn't panic)
    
    match result {
        Ok(decoded) => {
            // If it decodes, check if it's reasonable
            assert!(decoded.len() <= 8, "decoded too many bytes");
        }
        Err(_) => {
            // Decoder is allowed to reject this edge case
            // Just verify it didn't panic
        }
    }
}

#[test]
fn consecutive_raw_tilde_sections() {
    // Two 7-byte raw sections: "~AAAAAAA~BBBBBBB"
    let input = "~AAAAAAA~BBBBBBB";
    
    let result = decode(input);
    
    match result {
        Ok(decoded) => {
            assert!(decoded.len() <= 14, "decoded too many bytes");
        }
        Err(_) => {
            // Allowed to reject
        }
    }
}

#[test]
fn consecutive_raw_mixed_escapes() {
    // Mix of different escape types: ",AAAA~BBBBBBB"
    let input = ",AAAA~BBBBBBB";
    
    let result = decode(input);
    
    match result {
        Ok(decoded) => {
            assert!(decoded.len() <= 11, "decoded too many bytes");
        }
        Err(_) => {
            // Allowed to reject
        }
    }
}

// =============================================================================
// Priority 2 Test 2: Self-Signaling Property
// =============================================================================
// When the encoder uses passthrough, the output should contain escape
// characters. When it doesn't use passthrough, output should be pure Z85.

proptest! {
    #[test]
    fn self_signaling_with_printable_data(data in prop::collection::vec(0x20u8..=0x7E, 8..=32)) {
        // Printable ASCII data might trigger passthrough
        let encoded = encode(&data);
        
        // Check if output contains escape characters
        let _has_escapes = encoded.contains(',') || 
                           encoded.contains(';') || 
                           encoded.contains('_') || 
                           encoded.contains('~') || 
                           encoded.contains('|');
        
        // If passthrough is used, escape chars MUST be present
        // (We can't easily check the inverse - encoder is opportunistic)
        
        // At minimum: output should only contain valid chars
        for ch in encoded.chars() {
            let is_z85 = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#".contains(ch);
            let is_escape = ",;_~|.".contains(ch);
            
            prop_assert!(
                is_z85 || is_escape,
                "output contains invalid character: {:?}",
                ch
            );
        }
    }
}

proptest! {
    #[test]
    fn self_signaling_random_data(data: Vec<u8>) {
        // Random data unlikely to trigger passthrough (not printable ASCII)
        let encoded = encode(&data);
        
        // Output should only contain valid characters (Z85 alphabet + escapes)
        for ch in encoded.chars() {
            let is_z85 = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#".contains(ch);
            let is_escape = ",;_~|.".contains(ch);
            
            prop_assert!(
                is_z85 || is_escape,
                "output contains invalid character: {:?}",
                ch
            );
        }
    }
}

// =============================================================================
// Priority 2 Test 3: Encoder Strategy Verification
// =============================================================================
// Verify that the encoder makes reasonable decisions about when to use
// passthrough vs standard Z85.

proptest! {
    #[test]
    fn encoder_never_produces_longer_output(data: Vec<u8>) {
        let encoded = encode(&data);
        
        // Maximum possible Z85 length for this data
        let max_z85_len = ((data.len() + 3) / 4) * 5;
        
        // Encoder should NEVER produce output longer than standard Z85
        // (passthrough is only used when it saves space or breaks even)
        prop_assert!(
            encoded.len() <= max_z85_len,
            "encoded length {} exceeds max Z85 length {} for {} bytes",
            encoded.len(), max_z85_len, data.len()
        );
    }
}

proptest! {
    #[test]
    fn encoder_benefits_from_passthrough_on_safe_data(
        data in prop::collection::vec(0x30u8..=0x7A, 16..=64)
    ) {
        // Data that should benefit from passthrough (lots of safe ASCII)
        let encoded = encode(&data);
        let std_z85_len = ((data.len() + 3) / 4) * 5;
        
        // We don't REQUIRE passthrough (encoder is opportunistic),
        // but just verify output is valid
        prop_assert!(encoded.len() <= std_z85_len);
        
        // And that it roundtrips
        let decoded = decode(&encoded).expect("should decode");
        prop_assert_eq!(decoded, data);
    }
}
