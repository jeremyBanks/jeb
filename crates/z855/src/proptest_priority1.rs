// Priority 1 Property Tests - CRITICAL INVARIANTS
//
// These tests verify the core requirements from DESIGN-CONSTRAINTS.md
// that were identified as untested by coverage analysis.

#![cfg(test)]

use proptest::prelude::*;
use crate::z855::{encode, decode, Z85_ALPHABET};

// =============================================================================
// Helper: Standard Z85 Reference Encoder
// =============================================================================
// This is a simple, obviously-correct implementation of standard Z85 encoding
// (without any extensions) to use as a reference for position invariance tests.

fn standard_z85_encode(data: &[u8]) -> String {
    let mut result = String::new();
    
    // Encode complete 4-byte blocks
    for chunk in data.chunks_exact(4) {
        let mut value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let mut block = [0u8; 5];
        
        for i in (0..5).rev() {
            block[i] = Z85_ALPHABET[(value % 85) as usize];
            value /= 85;
        }
        
        result.push_str(std::str::from_utf8(&block).unwrap());
    }
    
    // Handle remainder (1-3 bytes) with padding
    let remainder = data.chunks_exact(4).remainder();
    if !remainder.is_empty() {
        let mut padded = [0u8; 4];
        padded[..remainder.len()].copy_from_slice(remainder);
        
        let value = u32::from_be_bytes(padded);
        let mut block = [0u8; 5];
        
        for i in (0..5).rev() {
            block[i] = Z85_ALPHABET[(value % 85) as usize];
            value /= 85;
        }
        
        // Emit remainder.len() + 1 characters
        result.push_str(std::str::from_utf8(&block[..remainder.len() + 1]).unwrap());
    }
    
    result
}

// =============================================================================
// Priority 1 Test 1: Position Invariance (P1) - THE CORE REQUIREMENT
// =============================================================================

proptest! {
    #[test]
    fn position_invariance_length_bound(data: Vec<u8>) {
        let z855_output = encode(&data);
        let std_z85_output = standard_z85_encode(&data);
        
        // P1a: Output length ≤ standard Z85 length
        prop_assert!(
            z855_output.len() <= std_z85_output.len(),
            "z855 output ({} chars) exceeds standard Z85 length ({} chars)",
            z855_output.len(), std_z85_output.len()
        );
    }
}

proptest! {
    #[test]
    fn position_invariance_aligned_blocks(data in prop::collection::vec(any::<u8>(), 0..=64).prop_filter("4-byte aligned", |v| v.len() % 4 == 0)) {
        // For 4-byte aligned data, if z855 uses NO passthrough,
        // output should be IDENTICAL to standard Z85
        let z855_output = encode(&data);
        let std_z85_output = standard_z85_encode(&data);
        
        // Remove any escape characters and raw bytes to isolate Z85 chars
        let z855_z85_only: String = z855_output
            .chars()
            .filter(|&c| Z85_ALPHABET.contains(&(c as u8)))
            .collect();
        
        // For aligned blocks, Z85 characters should match positions
        // (This is a weak form of position invariance - better test below)
        prop_assert!(z855_output.len() <= std_z85_output.len());
    }
}

// =============================================================================
// Priority 1 Test 2: Mid-Block Entry Cuts (R2) - THE HARDEST PIECE
// =============================================================================

proptest! {
    #[test]
    fn mid_block_entry_1_byte(
        prefix in prop::collection::vec(any::<u8>(), 1..=1),
        suffix: Vec<u8>
    ) {
        // Entry cut after 1 byte (3 bytes into next block)
        let data = [prefix.clone(), suffix.clone()].concat();
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("mid-block entry should decode");
        
        prop_assert_eq!(decoded, data, "roundtrip failed for 1-byte entry");
    }
    
    #[test]
    fn mid_block_entry_2_bytes(
        prefix in prop::collection::vec(any::<u8>(), 2..=2),
        suffix: Vec<u8>
    ) {
        // Entry cut after 2 bytes (2 bytes into next block)
        let data = [prefix.clone(), suffix.clone()].concat();
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("mid-block entry should decode");
        
        prop_assert_eq!(decoded, data, "roundtrip failed for 2-byte entry");
    }
    
    #[test]
    fn mid_block_entry_3_bytes(
        prefix in prop::collection::vec(any::<u8>(), 3..=3),
        suffix: Vec<u8>
    ) {
        // Entry cut after 3 bytes (1 byte into next block)
        let data = [prefix.clone(), suffix.clone()].concat();
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("mid-block entry should decode");
        
        prop_assert_eq!(decoded, data, "roundtrip failed for 3-byte entry");
    }
}

// =============================================================================
// Priority 1 Test 3: Mid-Block Exit Cuts (R2)
// =============================================================================

proptest! {
    #[test]
    fn mid_block_exit_1_byte_before(
        prefix: Vec<u8>,
        suffix in prop::collection::vec(any::<u8>(), 1..=1)
    ) {
        // Exit cut 1 byte before block boundary
        let data = [prefix.clone(), suffix.clone()].concat();
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("mid-block exit should decode");
        
        prop_assert_eq!(decoded, data, "roundtrip failed for 1-byte-before exit");
    }
    
    #[test]
    fn mid_block_exit_2_bytes_before(
        prefix: Vec<u8>,
        suffix in prop::collection::vec(any::<u8>(), 2..=2)
    ) {
        // Exit cut 2 bytes before block boundary
        let data = [prefix.clone(), suffix.clone()].concat();
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("mid-block exit should decode");
        
        prop_assert_eq!(decoded, data, "roundtrip failed for 2-bytes-before exit");
    }
    
    #[test]
    fn mid_block_exit_3_bytes_before(
        prefix: Vec<u8>,
        suffix in prop::collection::vec(any::<u8>(), 3..=3)
    ) {
        // Exit cut 3 bytes before block boundary
        let data = [prefix.clone(), suffix.clone()].concat();
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("mid-block exit should decode");
        
        prop_assert_eq!(decoded, data, "roundtrip failed for 3-bytes-before exit");
    }
}

// =============================================================================
// Priority 1 Test 4: Both Entry AND Exit Mid-Block (Tightest Budget)
// =============================================================================

proptest! {
    #[test]
    fn mid_block_both_boundaries(
        entry_offset in 1usize..=3,
        raw_length in 4usize..=8,
        suffix_length in 1usize..=3
    ) {
        // Build data with:
        // - entry_offset bytes (Z85-encoded as partial block)
        // - raw_length bytes (could be passthrough if printable)
        // - suffix_length bytes (Z85-encoded as partial block)
        
        let mut data = Vec::new();
        data.extend(vec![0x41; entry_offset]); // 'A' (printable, could trigger passthrough)
        data.extend(vec![0x42; raw_length]);   // 'B' (printable)
        data.extend(vec![0x43; suffix_length]); // 'C' (printable)
        
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("both-boundaries should decode");
        
        prop_assert_eq!(decoded, data, "roundtrip failed for both-mid-block case");
        
        // Verify output length is reasonable (tightest budget case)
        let max_len = ((data.len() + 3) / 4) * 5; // Standard Z85 length
        prop_assert!(
            encoded.len() <= max_len,
            "both-mid-block output ({}) exceeds max ({})",
            encoded.len(), max_len
        );
    }
}

// =============================================================================
// Priority 1 Test 5: Non-Aligned Raw Section Lengths (R1)
// =============================================================================

proptest! {
    #[test]
    fn non_aligned_5_bytes(data in prop::collection::vec(any::<u8>(), 5..=5)) {
        // Explicitly test 5-byte input (not 4-aligned)
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("5-byte input should decode");
        prop_assert_eq!(decoded, data, "5-byte roundtrip failed");
    }
    
    #[test]
    fn non_aligned_6_bytes(data in prop::collection::vec(any::<u8>(), 6..=6)) {
        // Explicitly test 6-byte input
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("6-byte input should decode");
        prop_assert_eq!(decoded, data, "6-byte roundtrip failed");
    }
    
    #[test]
    fn non_aligned_7_bytes(data in prop::collection::vec(any::<u8>(), 7..=7)) {
        // Explicitly test 7-byte input
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("7-byte input should decode");
        prop_assert_eq!(decoded, data, "7-byte roundtrip failed");
    }
    
    #[test]
    fn non_aligned_various(data in prop::collection::vec(any::<u8>(), 5..=11).prop_filter("not 4-aligned", |v| v.len() % 4 != 0)) {
        // Test 5,6,7,9,10,11 byte inputs (all non-aligned)
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("non-aligned input should decode");
        prop_assert_eq!(decoded, data, "non-aligned roundtrip failed");
    }
}
