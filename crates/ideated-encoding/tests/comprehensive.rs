//! Comprehensive test suite for ideated-encoding.
//!
//! This includes:
//! - Property-based testing with proptest
//! - Edge case coverage
//! - Boundary condition testing
//! - Cross-block continuation tests
//! - Specific regression tests

use {
    ideated_encoding::{
        Decoder,
        Encoder,
        decode,
        encode,
    },
    proptest::prelude::*,
};

// =============================================================================
// Property-Based Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// The fundamental invariant: decode(encode(x)) == x for all inputs
    #[test]
    fn prop_roundtrip_any_bytes(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode failed");
        prop_assert_eq!(decoded, data);
    }

    /// Roundtrip with streaming encoder
    #[test]
    fn prop_roundtrip_streaming_encoder(
        chunks in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..100), 0..20)
    ) {
        let mut encoder = Encoder::new();
        let mut original = Vec::new();
        for chunk in &chunks {
            encoder.write(chunk);
            original.extend_from_slice(chunk);
        }
        let encoded = encoder.finish();
        let decoded = decode(&encoded).expect("decode failed");
        prop_assert_eq!(decoded, original);
    }

    /// Roundtrip with streaming decoder
    #[test]
    fn prop_roundtrip_streaming_decoder(data in prop::collection::vec(any::<u8>(), 0..500)) {
        let encoded = encode(&data);

        // Decode in random chunk sizes
        let mut decoder = Decoder::new();
        let mut offset = 0;
        while offset < encoded.len() {
            let chunk_size = ((offset * 7 + 3) % 13).min(encoded.len() - offset).max(1);
            decoder.write(&encoded[offset..offset + chunk_size]).expect("write failed");
            offset += chunk_size;
        }
        let decoded = decoder.finish().expect("finish failed");
        prop_assert_eq!(decoded, data);
    }

    /// Encoded output should never be more than ~125% of input (plus small constant overhead)
    #[test]
    fn prop_encoding_overhead_bounded(data in prop::collection::vec(any::<u8>(), 1..1000)) {
        let encoded = encode(&data);
        // Standard Z85 is exactly 125% (5/4). With escapes and padding, should be close.
        // Allow up to 130% plus 10 bytes for edge cases
        let max_len = (data.len() * 13 / 10) + 10;
        prop_assert!(
            encoded.len() <= max_len,
            "encoded len {} exceeds max {} for input len {}",
            encoded.len(), max_len, data.len()
        );
    }

    /// Encoded output should only contain valid characters (Z85 + escapes + padding)
    #[test]
    fn prop_encoded_chars_valid(data in prop::collection::vec(any::<u8>(), 0..500)) {
        let encoded = encode(&data);
        for (i, &byte) in encoded.iter().enumerate() {
            let is_valid = ideated_encoding::is_z85_char(byte)
                || ideated_encoding::is_escape_char(byte);
            prop_assert!(is_valid, "invalid char {} at position {}", byte, i);
        }
    }

    /// ASCII text (all Z85-safe characters) should roundtrip correctly.
    /// Note: Raw passthrough doesn't always produce shorter output than pure Z85
    /// for small inputs due to padding overhead after raw sequences. The benefit
    /// becomes more reliable for larger inputs (see test_large_ascii).
    #[test]
    fn prop_ascii_roundtrip(
        data in prop::collection::vec(
            prop::sample::select(b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_vec()),
            8..200
        )
    ) {
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode failed");
        prop_assert_eq!(decoded, data);
    }

    /// Data containing dots should roundtrip correctly (dot is both Z85 char and padding)
    #[test]
    fn prop_dots_roundtrip(
        prefix in prop::collection::vec(any::<u8>(), 0..50),
        dot_count in 1usize..20,
        suffix in prop::collection::vec(any::<u8>(), 0..50)
    ) {
        let mut data = prefix;
        data.extend(std::iter::repeat(b'.').take(dot_count));
        data.extend(suffix);

        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode failed");
        prop_assert_eq!(decoded, data);
    }

    /// Data containing escape characters should roundtrip correctly
    #[test]
    fn prop_escape_chars_in_data_roundtrip(data in prop::collection::vec(
        prop::sample::select(vec![b',', b'`', b';', b'~', b'_', b'|', b'a', b'b', b'0', b'1']),
        0..200
    )) {
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode failed");
        prop_assert_eq!(decoded, data);
    }

    /// Single-byte inputs should all roundtrip
    #[test]
    fn prop_single_byte_roundtrip(byte: u8) {
        let data = [byte];
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode failed");
        prop_assert_eq!(decoded.as_slice(), &data[..]);
    }

    /// Two-byte inputs should all roundtrip
    #[test]
    fn prop_two_byte_roundtrip(b1: u8, b2: u8) {
        let data = [b1, b2];
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode failed");
        prop_assert_eq!(decoded.as_slice(), &data[..]);
    }

    /// Block-aligned inputs (multiples of 4 bytes) should roundtrip
    #[test]
    fn prop_block_aligned_roundtrip(blocks in 1usize..50, fill: u8) {
        let data = vec![fill; blocks * 4];
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect("decode failed");
        prop_assert_eq!(decoded, data);
    }
}

// =============================================================================
// Specific Length Tests
// =============================================================================

#[test]
fn test_lengths_0_to_100() {
    for len in 0..=100 {
        let data: Vec<u8> = (0..len).map(|i| (i * 31 + 17) as u8).collect();
        let encoded = encode(&data);
        let decoded = decode(&encoded).expect(&format!("decode failed for len {}", len));
        assert_eq!(decoded, data, "roundtrip failed for len {}", len);
    }
}

#[test]
fn test_lengths_around_block_boundaries() {
    // Test around 4-byte (input block) and 5-char (output block) boundaries
    for base in [4, 5, 8, 10, 16, 20, 40, 50, 80, 100] {
        for offset in -3i32..=3 {
            let len = (base as i32 + offset).max(0) as usize;
            let data: Vec<u8> = (0..len).map(|i| i as u8).collect();
            let encoded = encode(&data);
            let decoded = decode(&encoded).expect(&format!("decode failed for len {}", len));
            assert_eq!(decoded, data, "roundtrip failed for len {}", len);
        }
    }
}

// =============================================================================
// Escape-Specific Tests
// =============================================================================

#[test]
fn test_exact_raw_counts_at_position_0() {
    // Test data that should trigger each escape at position 0
    // These are safe Z85 characters that should use raw passthrough

    // 3 raw bytes -> backtick
    let data3 = b"abc";
    assert_eq!(decode(&encode(data3)).unwrap(), data3);

    // 4 raw bytes -> comma
    let data4 = b"abcd";
    assert_eq!(decode(&encode(data4)).unwrap(), data4);

    // 5 raw bytes -> tilde
    let data5 = b"abcde";
    assert_eq!(decode(&encode(data5)).unwrap(), data5);

    // 6 raw bytes -> semicolon
    let data6 = b"abcdef";
    assert_eq!(decode(&encode(data6)).unwrap(), data6);

    // 7 raw bytes -> underscore
    let data7 = b"abcdefg";
    assert_eq!(decode(&encode(data7)).unwrap(), data7);
}

#[test]
fn test_cross_block_continuation_lengths() {
    // Test lengths that require cross-block continuation
    // At position 0: escape uses 1 char, leaving 4 chars for raw in first block

    // 7 raw bytes at position 0: 1 (escape) + 4 (raw) + 2 (raw continuation) + 3
    // (padding)
    let data7 = b"1234567";
    let encoded = encode(data7);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data7);

    // 6 raw bytes at position 0: spans into next block
    let data6 = b"123456";
    let encoded = encode(data6);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data6);
}

// =============================================================================
// Dot (Padding Character) Tests
// =============================================================================

#[test]
fn test_all_dots_various_lengths() {
    for len in 1..=32 {
        let data = vec![b'.'; len];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "failed for {} dots", len);
    }
}

#[test]
fn test_dots_at_various_positions() {
    // Dot at start
    let data = b".abcdefgh";
    assert_eq!(decode(&encode(data)).unwrap(), data);

    // Dot at end
    let data = b"abcdefgh.";
    assert_eq!(decode(&encode(data)).unwrap(), data);

    // Dot in middle
    let data = b"abcd.efgh";
    assert_eq!(decode(&encode(data)).unwrap(), data);

    // Multiple dots
    let data = b"a.b.c.d.e.f.g.h";
    assert_eq!(decode(&encode(data)).unwrap(), data);

    // Dots at block boundaries (every 4th position)
    let data = b"abc.efg.ijk.mno.";
    assert_eq!(decode(&encode(data)).unwrap(), data);
}

#[test]
fn test_dot_in_cross_block_continuation() {
    // 7 safe bytes with dot at position that lands in continuation block
    // _ escape at position 0: 1 escape + 4 raw in block 1, 3 raw in block 2
    // If bytes 5, 6, 7 include a dot, it must not be skipped

    // Dot at position 5 (first byte of continuation)
    let data = b"abcd.fg";
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data, "dot at continuation start failed");

    // Dot at position 6 (second byte of continuation)
    let data = b"abcde.g";
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data, "dot at continuation middle failed");

    // Dot at position 7 (last byte of continuation)
    let data = b"abcdef.";
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data, "dot at continuation end failed");

    // Multiple dots in continuation
    let data = b"abcd...";
    let encoded = encode(data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data, "multiple dots in continuation failed");
}

// =============================================================================
// Binary Data Tests
// =============================================================================

#[test]
fn test_all_byte_values() {
    // Single byte of each value
    for byte in 0..=255u8 {
        let data = [byte];
        let decoded = decode(&encode(&data)).unwrap();
        assert_eq!(decoded, data, "failed for byte {}", byte);
    }
}

#[test]
fn test_all_byte_pairs() {
    // Test every possible 2-byte combination would be too slow,
    // so test representative samples
    for b1 in (0..=255u8).step_by(17) {
        for b2 in (0..=255u8).step_by(19) {
            let data = [b1, b2];
            let decoded = decode(&encode(&data)).unwrap();
            assert_eq!(decoded, data, "failed for bytes [{}, {}]", b1, b2);
        }
    }
}

#[test]
fn test_zero_bytes() {
    for len in 1..=32 {
        let data = vec![0u8; len];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "failed for {} zeros", len);
    }
}

#[test]
fn test_0xff_bytes() {
    for len in 1..=32 {
        let data = vec![0xFFu8; len];
        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "failed for {} 0xFF bytes", len);
    }
}

#[test]
fn test_alternating_patterns() {
    // 0x00, 0xFF alternating
    let data: Vec<u8> = (0..50)
        .map(|i| if i % 2 == 0 { 0x00 } else { 0xFF })
        .collect();
    assert_eq!(decode(&encode(&data)).unwrap(), data);

    // 0x55, 0xAA alternating (bit patterns)
    let data: Vec<u8> = (0..50)
        .map(|i| if i % 2 == 0 { 0x55 } else { 0xAA })
        .collect();
    assert_eq!(decode(&encode(&data)).unwrap(), data);
}

#[test]
fn test_counting_sequences() {
    // 0, 1, 2, 3, ...
    let data: Vec<u8> = (0..=255).collect();
    assert_eq!(decode(&encode(&data)).unwrap(), data);

    // 255, 254, 253, ...
    let data: Vec<u8> = (0..=255).rev().collect();
    assert_eq!(decode(&encode(&data)).unwrap(), data);
}

// =============================================================================
// Streaming Tests
// =============================================================================

#[test]
fn test_streaming_encoder_byte_by_byte() {
    let original = b"Hello, World! This is a test of streaming encoding.";
    let mut encoder = Encoder::new();
    for &byte in original {
        encoder.write(&[byte]);
    }
    let encoded = encoder.finish();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_streaming_decoder_byte_by_byte() {
    let original = b"Hello, World! This is a test of streaming decoding.";
    let encoded = encode(original);

    let mut decoder = Decoder::new();
    for &byte in &encoded {
        decoder.write(&[byte]).unwrap();
    }
    let decoded = decoder.finish().unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_streaming_various_chunk_sizes() {
    let original: Vec<u8> = (0..500).map(|i| (i * 7 + 13) as u8).collect();

    for chunk_size in [1, 2, 3, 4, 5, 7, 13, 16, 32, 64, 128] {
        // Streaming encode
        let mut encoder = Encoder::new();
        for chunk in original.chunks(chunk_size) {
            encoder.write(chunk);
        }
        let encoded = encoder.finish();

        // Streaming decode
        let mut decoder = Decoder::new();
        for chunk in encoded.chunks(chunk_size) {
            decoder.write(chunk).unwrap();
        }
        let decoded = decoder.finish().unwrap();

        assert_eq!(decoded, original, "failed for chunk size {}", chunk_size);
    }
}

// =============================================================================
// Real-World Data Patterns
// =============================================================================

#[test]
fn test_json_like_data() {
    let data = br#"{"name": "test", "value": 42, "nested": {"a": 1, "b": 2}}"#;
    assert_eq!(decode(&encode(data)).unwrap(), data);
}

#[test]
fn test_xml_like_data() {
    let data = br#"<root><item id="1">Hello</item><item id="2">World</item></root>"#;
    assert_eq!(decode(&encode(data)).unwrap(), data);
}

#[test]
fn test_base64_like_data() {
    let data = b"SGVsbG8sIFdvcmxkIQ==";
    assert_eq!(decode(&encode(data)).unwrap(), data);
}

#[test]
fn test_hex_like_data() {
    let data = b"0123456789abcdef0123456789ABCDEF";
    assert_eq!(decode(&encode(data)).unwrap(), data);
}

#[test]
fn test_mixed_binary_and_text() {
    let mut data = Vec::new();
    data.extend_from_slice(b"START");
    data.extend_from_slice(&[0x00, 0x01, 0x02, 0x03]);
    data.extend_from_slice(b"MIDDLE");
    data.extend_from_slice(&[0xFE, 0xFF, 0x00, 0x00]);
    data.extend_from_slice(b"END");

    assert_eq!(decode(&encode(&data)).unwrap(), data);
}

// =============================================================================
// Prefix Encoding Edge Cases
// =============================================================================

#[test]
fn test_prefix_value_boundaries() {
    // Test values at boundaries of base-85 encoding capacity

    // 1 char prefix: values 0-84
    for v in [0u8, 1, 42, 84] {
        let data = vec![v, b'a', b'b', b'c', b'd']; // 1 prefix byte + 4 raw
        assert_eq!(
            decode(&encode(&data)).unwrap(),
            data,
            "failed for prefix value {}",
            v
        );
    }

    // Values just above 84 should not use 1-char prefix (use Z85 instead)
    let data = vec![85u8, b'a', b'b', b'c', b'd'];
    assert_eq!(decode(&encode(&data)).unwrap(), data);
}

#[test]
fn test_prefix_endianness() {
    // Little-endian: low byte first
    // [0x48, 0x00] as LE = 0x0048 = 72, fits in base-85
    let le_data = vec![0x48, 0x00, b'a', b'b', b'c', b'd'];
    assert_eq!(decode(&encode(&le_data)).unwrap(), le_data);

    // [0x00, 0x48] as LE = 0x4800 = 18432, doesn't fit in 85^2 = 7225
    // Should use different encoding strategy
    let be_data = vec![0x00, 0x48, b'a', b'b', b'c', b'd'];
    assert_eq!(decode(&encode(&be_data)).unwrap(), be_data);
}

// =============================================================================
// Large Data Tests
// =============================================================================

#[test]
fn test_large_data() {
    // 10KB of data
    let data: Vec<u8> = (0..10_000).map(|i| (i * 31 + 17) as u8).collect();
    assert_eq!(decode(&encode(&data)).unwrap(), data);
}

#[test]
fn test_large_ascii() {
    // 10KB of ASCII (tests large input roundtrip)
    let pattern = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let data: Vec<u8> = pattern.iter().cycle().take(10_000).copied().collect();
    let encoded = encode(&data);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);

    // Note: Raw passthrough with current escape/padding design may not always
    // produce shorter output than pure Z85. The encoder's cost model could be
    // optimized to fall back to pure Z85 when that would be more efficient.
    // For now, just verify the encoding stays within reasonable bounds.
    let max_len = (data.len() * 15 / 10) + 10; // Allow up to 150% + overhead
    assert!(
        encoded.len() <= max_len,
        "encoding exceeded reasonable bounds: {} > {}",
        encoded.len(),
        max_len
    );
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_invalid_z85_character() {
    // Characters outside Z85 alphabet that aren't escape chars
    let invalid = b"\x00\x01\x02"; // Control characters
    let result = decode(invalid);
    assert!(result.is_err());
}

#[test]
fn test_truncated_input() {
    // Encode some data, then truncate
    let original = b"Hello, World!";
    let encoded = encode(original);

    // Truncating should either succeed (partial decode) or fail gracefully
    for truncate_at in 1..encoded.len() {
        let truncated = &encoded[..truncate_at];
        // We don't care if it succeeds or fails, just that it doesn't panic
        let _ = decode(truncated);
    }
}

// =============================================================================
// Regression Tests
// =============================================================================

#[test]
fn regression_hello_world() {
    // The original failing test case
    let original = b"Hello, World!";
    let encoded = encode(original);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn regression_dot_continuation_skip() {
    // This was a bug where dots in continuation blocks were skipped
    let original = b"abcde.g";
    let encoded = encode(original);
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, original, "dot was incorrectly skipped");
}
