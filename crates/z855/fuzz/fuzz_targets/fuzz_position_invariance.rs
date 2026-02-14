#![no_main]

use libfuzzer_sys::fuzz_target;
use z855::encode;

fuzz_target!(|data: &[u8]| {
    // Position invariance (DESIGN-CONSTRAINTS.md §3 P1):
    // Z85-encoded blocks must appear at exact same character positions
    // as they would in standard Z85 encoding.
    
    if data.is_empty() {
        return;
    }
    
    let encoded = encode(data);
    
    // Find Z85-encoded sections (continuous non-escape characters)
    let mut in_z85 = false;
    let mut z85_start = 0;
    let mut char_pos = 0;
    
    for (i, &byte) in encoded.iter().enumerate() {
        let is_z85_char = matches!(byte, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'.' | b'-');
        let is_escape = matches!(byte, b'_' | b'~' | b'`' | b'|' | b',' | b';');
        
        if is_z85_char && !in_z85 {
            // Start of Z85 section
            in_z85 = true;
            z85_start = i;
        } else if !is_z85_char && in_z85 {
            // End of Z85 section - verify position invariance
            let z85_len = i - z85_start;
            
            // Z85 encodes 4 bytes → 5 chars
            // Position of this section should align with standard Z85 encoding
            // Character position should be multiple of 5, or we're mid-block
            
            // If we have a complete 5-char block, it should be at a multiple-of-5 position
            if z85_len >= 5 && z85_len % 5 == 0 {
                assert!(
                    char_pos % 5 == 0,
                    "Z85 block at char position {} is not aligned (should be multiple of 5)",
                    char_pos
                );
            }
            
            char_pos += z85_len;
            in_z85 = false;
        }
        
        // Track character position (not byte position - escapes are multi-char)
        if is_escape {
            char_pos += 1; // Escape char itself
            // Following char is data, but it's not a Z85 position
        }
    }
});
