// Property-based tests for z855 encoding/decoding
//
// These tests verify invariants that should hold for ALL inputs,
// using randomly generated test cases via proptest.

#![cfg(test)]

use proptest::prelude::*;
use crate::z855::{encode, decode};

// =============================================================================
// Property 1: Roundtrip Invariant
// =============================================================================
// For ANY byte sequence, decode(encode(x)) should equal x

proptest! {
    #[test]
    fn roundtrip_arbitrary_bytes(data: Vec<u8>) {
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode should succeed on encoder output");
        prop_assert_eq!(decoded, data, "roundtrip failed");
    }

    #[test]
    fn roundtrip_small_inputs(data in prop::collection::vec(any::<u8>(), 0..=20)) {
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode should succeed on encoder output");
        prop_assert_eq!(decoded, data, "roundtrip failed for small input");
    }

    #[test]
    fn roundtrip_aligned_blocks(data in prop::collection::vec(any::<u8>(), 0..=64).prop_filter("must be 4-byte aligned", |v| v.len() % 4 == 0)) {
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode should succeed on encoder output");
        prop_assert_eq!(decoded, data, "roundtrip failed for aligned input");
    }
}

// =============================================================================
// Property 2: Length Bounds
// =============================================================================
// Encoded length should never exceed theoretical maximum

fn max_encoded_len(input_len: usize) -> usize {
    // Worst case: no passthrough possible, all Z85 encoding
    // 4 bytes -> 5 chars, rounded up
    ((input_len + 3) / 4) * 5
}

proptest! {
    #[test]
    fn encoded_length_bounded(data: Vec<u8>) {
        let encoded = encode(&data);
        let max_len = max_encoded_len(data.len());
        prop_assert!(
            encoded.len() <= max_len,
            "encoded length {} exceeds max {} for input length {}",
            encoded.len(), max_len, data.len()
        );
    }
}

// =============================================================================
// Property 3: Decoder Never Panics
// =============================================================================
// Decoder should return Result, never panic, even on garbage input

proptest! {
    #[test]
    fn decoder_never_panics_ascii(input: String) {
        // Try to decode arbitrary ASCII strings
        // Should either succeed or return Err, never panic
        let _ = decode(&input);
    }

    #[test]
    fn decoder_never_panics_with_escapes(
        prefix in "[,;~_|]{0,3}",
        suffix in "[,;~_|]{0,3}",
        middle: Vec<u8>
    ) {
        // Generate strings with escape characters and arbitrary bytes
        let mut test_input = prefix;
        for &byte in &middle {
            test_input.push(byte as char);
        }
        test_input.push_str(&suffix);
        
        let _ = decode(&test_input);
    }
}

// =============================================================================
// Property 4: Transparency (Standard Z85 Compatibility)
// =============================================================================
// Data that could be encoded as pure Z85 should roundtrip identically

// Generate data that benefits from passthrough (printable ASCII)
fn printable_ascii() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(0x20u8..=0x7E, 0..100)
}

proptest! {
    #[test]
    fn transparency_preserves_data(data in printable_ascii()) {
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode should succeed");
        prop_assert_eq!(decoded, data, "transparency broken");
    }
}

// =============================================================================
// Property 5: Decoder Accepts Liberal Input
// =============================================================================
// The decoder should accept ANY bytes in passthrough sections

proptest! {
    #[test]
    fn decoder_liberal_comma_passthrough(
        prefix: Vec<u8>,
        raw_bytes in prop::collection::vec(any::<u8>(), 4..=4), // exactly 4 bytes
        suffix: Vec<u8>
    ) {
        // Construct: encode(prefix) + "," + raw_bytes + encode(suffix)
        let mut input = encode(&prefix);
        input.push(',');
        for &byte in &raw_bytes {
            input.push(byte as char);
        }
        input.push_str(&encode(&suffix));
        
        // Should decode without panic (may succeed or fail depending on validity)
        let result = decode(&input);
        
        // If it succeeds, should contain the raw bytes somewhere
        if let Ok(decoded) = result {
            // The raw bytes should appear in the output
            // (exact position depends on prefix length)
            prop_assert!(
                decoded.len() >= raw_bytes.len(),
                "decoded output too short"
            );
        }
    }

    #[test]
    fn decoder_liberal_tilde_passthrough(
        raw_bytes in prop::collection::vec(any::<u8>(), 7..=7) // exactly 7 bytes
    ) {
        // Construct: "~" + raw_bytes
        let mut input = String::from("~");
        for &byte in &raw_bytes {
            input.push(byte as char);
        }
        
        // Should decode without panic
        let _ = decode(&input);
    }

    #[test]
    fn decoder_liberal_pipe_passthrough(
        raw_bytes in prop::collection::vec(any::<u8>(), 8..=100) // 8+ bytes
    ) {
        // Construct: "|" + length_byte + raw_bytes
        let len_byte = (raw_bytes.len() - 8) as u8;
        let mut input = String::from("|");
        input.push(len_byte as char);
        for &byte in &raw_bytes {
            input.push(byte as char);
        }
        
        // Should decode without panic
        let _ = decode(&input);
    }
}

// =============================================================================
// Property 6: Encoder Produces Valid Output
// =============================================================================
// Everything the encoder produces should be decodable

proptest! {
    #[test]
    fn encoder_output_always_decodable(data: Vec<u8>) {
        let encoded = encode(&data);
        decode(&encoded).expect("encoder output must be decodable");
    }
}

// =============================================================================
// Property 7: Idempotent Encoding
// =============================================================================
// encode(x) should be deterministic (same input -> same output)

proptest! {
    #[test]
    fn encoding_is_deterministic(data: Vec<u8>) {
        let encoded1 = encode(&data);
        let encoded2 = encode(&data);
        prop_assert_eq!(encoded1, encoded2, "encoding should be deterministic");
    }
}

// =============================================================================
// Property 8: Empty Input Edge Case
// =============================================================================

proptest! {
    #[test]
    fn empty_input_roundtrips() {
        let data: Vec<u8> = vec![];
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("empty input should decode");
        prop_assert_eq!(decoded, data, "empty input roundtrip failed");
    }
}

// =============================================================================
// Property 9: Single-Byte Inputs
// =============================================================================

proptest! {
    #[test]
    fn single_byte_roundtrips(byte: u8) {
        let data = vec![byte];
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("single byte should decode");
        prop_assert_eq!(decoded, data, "single byte roundtrip failed");
    }
}

// =============================================================================
// Property 10: Large Inputs (Stress Test)
// =============================================================================

#[test]
fn large_input_roundtrips() {
    proptest!(ProptestConfig::with_cases(10), |(data in prop::collection::vec(any::<u8>(), 1000..10000))| {
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("large input should decode");
        prop_assert_eq!(decoded, data, "large input roundtrip failed");
    });
}
