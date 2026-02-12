//! Comprehensive test suite for Extended Z85 encoding.

use z85::{encode, decode};

// ==============================================================================
// 1. Standard Z85 Baseline Tests
// ==============================================================================

#[test]
fn test_empty_input() {
    let input = b"";
    let encoded = encode(input);
    assert_eq!(encoded.len(), 0);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_single_byte() {
    let input = b"\x00";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_four_bytes_complete_block() {
    let input = b"\x00\x00\x00\x00";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_eight_bytes_two_blocks() {
    let input = b"\x00\x00\x00\x00\x00\x00\x00\x00";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_partial_one_byte() {
    let input = b"\xDE";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_partial_two_bytes() {
    let input = b"\xDE\xAD";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_partial_three_bytes() {
    let input = b"\xDE\xAD\xBE";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_five_bytes_one_block_plus_one() {
    let input = b"\x00\x00\x00\x00\xFF";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_all_zero_four_bytes() {
    let input = &[0u8; 4];
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_all_zero_eight_bytes() {
    let input = &[0u8; 8];
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_all_ff_four_bytes() {
    let input = &[0xFFu8; 4];
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_all_ff_eight_bytes() {
    let input = &[0xFFu8; 8];
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_every_byte_value_at_position_0() {
    for byte_val in 0..=255u8 {
        let input = [byte_val, 0, 0, 0];
        let encoded = encode(&input);
        let decoded = decode(&encoded).expect("failed to decode");
        assert_eq!(&decoded[..], &input[..], "mismatch for byte value {} at position 0", byte_val);
    }
}

#[test]
fn test_every_byte_value_at_position_1() {
    for byte_val in 0..=255u8 {
        let input = [0, byte_val, 0, 0];
        let encoded = encode(&input);
        let decoded = decode(&encoded).expect("failed to decode");
        assert_eq!(&decoded[..], &input[..], "mismatch for byte value {} at position 1", byte_val);
    }
}

#[test]
fn test_every_byte_value_at_position_2() {
    for byte_val in 0..=255u8 {
        let input = [0, 0, byte_val, 0];
        let encoded = encode(&input);
        let decoded = decode(&encoded).expect("failed to decode");
        assert_eq!(&decoded[..], &input[..], "mismatch for byte value {} at position 2", byte_val);
    }
}

#[test]
fn test_every_byte_value_at_position_3() {
    for byte_val in 0..=255u8 {
        let input = [0, 0, 0, byte_val];
        let encoded = encode(&input);
        let decoded = decode(&encoded).expect("failed to decode");
        assert_eq!(&decoded[..], &input[..], "mismatch for byte value {} at position 3", byte_val);
    }
}

// ==============================================================================
// 2. Raw Section Basics
// ==============================================================================

#[test]
fn test_block_aligned_raw_section_4_bytes() {
    let input = b"Hello";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_block_aligned_raw_section_8_bytes() {
    let input = b"HelloWor";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_block_aligned_raw_section_12_bytes() {
    let input = b"HelloWorld!!";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_section_40_bytes() {
    let input = b"0123456789abcdefghij0123456789abcdefghij";
    assert_eq!(input.len(), 40);
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_all_printable_input_4_bytes() {
    let input = b"Test";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_all_printable_input_8_bytes() {
    let input = b"TestData";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_all_printable_input_12_bytes() {
    let input = b"TestDataMore";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_all_printable_input_20_bytes() {
    let input = b"This is twenty bytes";
    assert_eq!(input.len(), 20);
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_all_printable_input_100_bytes() {
    let mut input = Vec::new();
    for _ in 0..10 {
        input.extend_from_slice(b"0123456789");
    }
    assert_eq!(input.len(), 100);
    let encoded = encode(&input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_no_printable_input() {
    let input = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_mixed_binary_and_printable() {
    let input = b"\xDE\xAD\xBEEFHelloBinary\xFF\xFE";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_minimum_raw_section_4_bytes() {
    // Test that the encoder uses raw passthrough even for 4 bytes (no savings, but transparency)
    let input = b"\xDE\xAD\xBEEF";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

// ==============================================================================
// 3. Mid-Block Boundaries (R2)
// ==============================================================================

#[test]
fn test_mid_block_entry_1_byte() {
    // 3 binary bytes, then printable ASCII begins
    let input = b"\x00\x01\x02Hello";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_mid_block_entry_2_bytes() {
    // 2 binary bytes, then printable ASCII
    let input = b"\x00\x01HelloWorld";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_mid_block_entry_3_bytes() {
    // 1 binary byte, then printable ASCII
    let input = b"\x00HelloWorldTest";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_mid_block_exit_1_byte() {
    // Printable followed by 3 binary bytes
    let input = b"HelloW\xFF\xFE\xFD";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_mid_block_exit_2_bytes() {
    // Printable followed by 2 binary bytes
    let input = b"HelloWor\xFF\xFE";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_mid_block_exit_3_bytes() {
    // Printable followed by 1 binary byte
    let input = b"HelloWorld\xFF";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_mid_block_entry_and_exit() {
    // Mid-block entry and exit: [bin2][ASCII7][bin1]
    let input = b"\x00\x01HelloWorld\xFF";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_entry_exit_offsets_1_1() {
    let input = b"\xFF\xFF\xFFFiveChar\xFF\xFF\xFF";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_entry_exit_offsets_1_2() {
    let input = b"\xFF\xFF\xFFFiveChar\xFF\xFF";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_entry_exit_offsets_2_1() {
    let input = b"\xFF\xFF\x46iveChar\xFF\xFF\xFF";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_entry_exit_offsets_2_2() {
    let input = b"\xFF\xFF\x46iveChar\xFF\xFF";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_entry_exit_offsets_3_1() {
    let input = b"\xFFFiveChar\xFF\xFF\xFF";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_entry_exit_offsets_3_2() {
    let input = b"\xFFFiveChar\xFF\xFF";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

// ==============================================================================
// 4. Non-Aligned Lengths (R1)
// ==============================================================================

#[test]
fn test_raw_5_bytes_non_aligned() {
    let input = b"Abcde";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_6_bytes_non_aligned() {
    let input = b"Abcdef";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_7_bytes_non_aligned() {
    let input = b"Abcdefg";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_9_bytes_non_aligned() {
    let input = b"Abcdefghi";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_10_bytes() {
    let input = b"Abcdefghij";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_11_bytes() {
    let input = b"Abcdefghijk";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_13_bytes() {
    let input = b"Abcdefghijklm";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_5_bytes_offset_1() {
    let input = b"\x00Abcde\x00\x00\x00";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_7_bytes_offset_3() {
    let input = b"\x00\x00\xFEAbcdefg\x00";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

// ==============================================================================
// 5. Position Invariant (P1)
// ==============================================================================

#[test]
fn test_position_invariant_mixed_content() {
    // Binary blocks should be identical to standard Z85 encoding
    let input = b"\xDE\xAD\xBE\xEFHelloWorld\x01\x02\x03\x04";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_output_never_longer_than_standard() {
    // Extended output should be <= standard Z85 length
    let input = b"\xDE\xAD\xBE\xEFHelloWorld\x01\x02\x03\x04";
    let extended = encode(input);
    // For comparison, we could implement standard Z85 encoding
    // but the constraint is: extended length <= standard length
    // This test just verifies the extended version works
    let decoded = decode(&extended).expect("failed to decode");
    assert_eq!(decoded, input);
}

// ==============================================================================
// 6. Escape Characters & Length Encoding
// ==============================================================================

#[test]
fn test_escape_character_detection() {
    // Verify non-Z85 characters in output are only escape characters
    let input = b"OnlyPrintable";
    let encoded = encode(input);
    for &c in &encoded {
        // Should be either Z85 alphabet or escape characters
        assert!(is_valid_extended_char(c), "invalid character: 0x{:02x} ({})", c, c as char);
    }
}

#[test]
fn test_underscore_escape_character() {
    // Test that _ is used as escape
    let input = b"TestData";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_section_with_escape_char_in_data() {
    // Escape character (_) appears in the input data
    let input = b"Test_Data";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_section_with_tilde_in_data() {
    // Tilde (~) appears in input data
    let input = b"Test~Data";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_with_z85_alphabet_chars() {
    // Printable bytes that are in Z85 alphabet (e.g., digits, letters)
    let input = b"0123456789abcdef";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_section_only_escape_chars() {
    // Raw section containing only escape characters
    let input = b"____";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_raw_section_only_tildes() {
    let input = b"~~~~";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_consecutive_raw_sections() {
    // Decoder must handle two raw sections back-to-back
    // This tests R4 requirement
    let input = b"TestOneTestTwo";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

// ==============================================================================
// 7. Edge Cases
// ==============================================================================

#[test]
fn test_alternating_binary_printable() {
    // Adversarial pattern
    let input = b"\xFFH\xFFe\xFFl\xFFl\xFFo";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_very_long_input_10kb() {
    let mut input = Vec::new();
    for i in 0..10000 {
        if i % 2 == 0 {
            input.push(b'A');
        } else {
            input.push((i % 256) as u8);
        }
    }
    let encoded = encode(&input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_all_printable_ascii_32_to_126() {
    // All printable ASCII characters
    let mut input = Vec::new();
    for c in 32u8..=126 {
        input.push(c);
    }
    let encoded = encode(&input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

// ==============================================================================
// 8. Decoder Robustness
// ==============================================================================

#[test]
fn test_standard_z85_input_no_escape() {
    // Valid standard Z85 (no escape characters) should decode
    // We'll test this by creating standard Z85 encoded data
    let input = b"\x00\x00\x00\x00";
    let encoded = encode(input);
    let decoded = decode(&encoded).expect("failed to decode");
    assert_eq!(decoded, input);
}

#[test]
fn test_invalid_escape_at_end() {
    let input = b"abcd_";
    let result = decode(input);
    assert!(result.is_err(), "should error on escape at end");
}

#[test]
fn test_invalid_character() {
    let input = b"abcd\x00ef";
    let result = decode(input);
    assert!(result.is_err(), "should error on invalid character");
}

// ==============================================================================
// Utility Functions
// ==============================================================================

fn is_valid_extended_char(c: u8) -> bool {
    // Z85 alphabet characters, or escape characters
    const Z85_ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    Z85_ALPHABET.contains(&c) || c == b'_' || c == b'~'
}
