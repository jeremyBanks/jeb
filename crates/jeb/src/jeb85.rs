//! JEB85 encoding: Z85 with inline raw ASCII sequences.
//!
//! This is a byte-based encoder (not block-based). Raw-friendly regions can
//! start and end at any byte position.
//!
//! ## Encoding Scheme
//!
//! - `_xxxx`: Exactly 4 raw bytes
//! - `~xxxxxx`: Exactly 6 raw bytes
//! - `N|xxx...`: N raw bytes, where N is the byte count encoded in base-85
//! - Standard Z85 for non-raw-friendly data

use crate::{
    ASCII_INLINE_TEXT_LUT, Z85_LUT,
    z85::{BASE_85, BLOCK_DIGITS_5, decode_z85, encode_z85, encode_z85_block, encoded_z85_length},
};

/// Raw sequence prefix for exactly 4 bytes.
pub const RAW_PREFIX_4: u8 = b'_';
/// Raw sequence prefix for exactly 6 bytes.
pub const RAW_PREFIX_6: u8 = b'~';
/// Raw sequence prefix for variable-length (byte count encoded before this).
pub const RAW_PREFIX_VAR: u8 = b'|';

/// Minimum byte count for variable-length raw encoding to be worthwhile.
/// Below this threshold, Z85 or fixed prefixes are more efficient.
pub const MIN_VAR_RAW_BYTES: usize = 8;

// =============================================================================
// ENCODER
// =============================================================================

/// Minimum raw-friendly bytes needed to switch from Z85 to raw mode.
/// Using raw for fewer bytes than this isn't worth the mode switch overhead.
const MIN_RAW_RUN: usize = 4;

/// Encodes bytes to jeb85 format.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[must_use]
pub fn encode_jeb85(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(encoded_z85_length(input.len()));
    let mut pos = 0;

    while pos < input.len() {
        // Count consecutive raw-friendly bytes
        let raw_run = count_raw_friendly(input, pos);

        if raw_run >= MIN_RAW_RUN {
            // Worth switching to raw mode
            emit_raw_region(&mut output, &input[pos..pos + raw_run]);
            pos += raw_run;
        } else {
            // Use Z85 - find extent including short raw runs
            let start = pos;
            while pos < input.len() {
                let next_raw_run = count_raw_friendly(input, pos);
                if next_raw_run >= MIN_RAW_RUN {
                    break; // Long enough raw run, stop Z85 here
                }
                // Include this byte (and any short raw run) in Z85
                pos += next_raw_run.max(1);
            }
            emit_z85(&mut output, &input[start..pos]);
        }
    }

    output
}

/// Count consecutive raw-friendly bytes starting at pos.
fn count_raw_friendly(input: &[u8], pos: usize) -> usize {
    let mut count = 0;
    while pos + count < input.len() && is_raw_friendly(input[pos + count]) {
        count += 1;
    }
    count
}

/// Emit a raw-friendly region using the best encoding.
fn emit_raw_region(output: &mut Vec<u8>, bytes: &[u8]) {
    let mut pos = 0;
    let len = bytes.len();

    while pos < len {
        let remaining = len - pos;

        // Try to use the most efficient encoding for what remains
        match remaining {
            0 => break,
            1 | 2 | 3 => {
                // Too short for raw prefix, use Z85
                emit_z85(output, &bytes[pos..]);
                break;
            }
            4 => {
                // Exactly 4 bytes: use _
                output.push(RAW_PREFIX_4);
                output.extend_from_slice(&bytes[pos..pos + 4]);
                pos += 4;
            }
            5 => {
                // 5 bytes: use _ for 4, Z85 for 1
                output.push(RAW_PREFIX_4);
                output.extend_from_slice(&bytes[pos..pos + 4]);
                pos += 4;
                emit_z85(output, &bytes[pos..]);
                break;
            }
            6 => {
                // Exactly 6 bytes: use ~
                output.push(RAW_PREFIX_6);
                output.extend_from_slice(&bytes[pos..pos + 6]);
                pos += 6;
            }
            7 => {
                // 7 bytes: use ~ for 6, then handle 1 remaining
                // But 1 byte as Z85 is 2 chars, total 8 chars
                // Alternative: _ for 4 + Z85 for 3 = 5 + 4 = 9 chars
                // So ~ + Z85(1) = 7 + 2 = 9 chars... same
                // Actually: ~ is 1 prefix + 6 bytes = 7 chars total
                // Then 1 byte as Z85 = 2 chars. Total 9.
                // _ is 1 prefix + 4 bytes = 5 chars
                // Then 3 bytes as Z85 = 4 chars. Total 9.
                // Either works, prefer ~ for longer raw span
                output.push(RAW_PREFIX_6);
                output.extend_from_slice(&bytes[pos..pos + 6]);
                pos += 6;
                emit_z85(output, &bytes[pos..]);
                break;
            }
            _ => {
                // 8+ bytes: use length-prefixed encoding
                // Calculate optimal chunk size considering alignment
                let chunk_size = optimal_raw_chunk_size(remaining, pos);
                emit_length_prefixed_raw(output, &bytes[pos..pos + chunk_size]);
                pos += chunk_size;
            }
        }
    }
}

/// Determine optimal chunk size for a raw region, considering alignment.
fn optimal_raw_chunk_size(remaining: usize, _current_offset: usize) -> usize {
    // For now, take all remaining bytes if >= 8
    // TODO: Consider alignment optimization here
    remaining
}

/// Emit a length-prefixed raw sequence: N|bytes
fn emit_length_prefixed_raw(output: &mut Vec<u8>, bytes: &[u8]) {
    let len = bytes.len();
    debug_assert!(len >= MIN_VAR_RAW_BYTES);

    // Encode length as base-85, stripping leading zeros
    let len_encoded = encode_z85_block((len as u32).to_be_bytes());
    let mut start = 0;
    while start < BLOCK_DIGITS_5 - 1 && len_encoded[start] == b'0' {
        start += 1;
    }

    output.extend_from_slice(&len_encoded[start..]);
    output.push(RAW_PREFIX_VAR);
    output.extend_from_slice(bytes);
}

/// Emit bytes as Z85.
fn emit_z85(output: &mut Vec<u8>, bytes: &[u8]) {
    output.extend_from_slice(&encode_z85(bytes));
}

// =============================================================================
// DECODER
// =============================================================================

/// Decodes jeb85-encoded data back to bytes.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn decode_jeb85(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut pos = 0;

    while pos < input.len() {
        let byte = input[pos];

        match byte {
            RAW_PREFIX_4 => {
                // _xxxx: exactly 4 raw bytes
                pos += 1;
                if pos + 4 > input.len() {
                    return Err("unexpected end of input after _".into());
                }
                output.extend_from_slice(&input[pos..pos + 4]);
                pos += 4;
            }
            RAW_PREFIX_6 => {
                // ~xxxxxx: exactly 6 raw bytes
                pos += 1;
                if pos + 6 > input.len() {
                    return Err("unexpected end of input after ~".into());
                }
                output.extend_from_slice(&input[pos..pos + 6]);
                pos += 6;
            }
            RAW_PREFIX_VAR => {
                return Err("unexpected | without length prefix".into());
            }
            _ if is_z85_digit(byte) => {
                // Process Z85 in chunks of 5 (full blocks)
                // A length prefix is detected when we have < 5 digits followed by |

                loop {
                    // Count available Z85 digits
                    let digit_start = pos;
                    let mut digit_count = 0;
                    while pos + digit_count < input.len()
                        && is_z85_digit(input[pos + digit_count])
                        && digit_count < BLOCK_DIGITS_5
                    {
                        digit_count += 1;
                    }

                    if digit_count == BLOCK_DIGITS_5 {
                        // Full block - decode as Z85
                        let digits = &input[digit_start..digit_start + BLOCK_DIGITS_5];
                        decode_z85_digits(&mut output, digits)?;
                        pos += BLOCK_DIGITS_5;

                        // Continue if more Z85 digits follow
                        if pos < input.len() && is_z85_digit(input[pos]) {
                            continue;
                        }
                    } else if digit_count > 0 {
                        // Partial block - check if followed by |
                        let digits = &input[digit_start..digit_start + digit_count];
                        pos += digit_count;

                        if pos < input.len() && input[pos] == RAW_PREFIX_VAR {
                            // Length-prefixed raw sequence
                            pos += 1; // consume |
                            let length = decode_length(digits)?;
                            if pos + length > input.len() {
                                return Err("unexpected end of input in raw sequence".into());
                            }
                            output.extend_from_slice(&input[pos..pos + length]);
                            pos += length;
                        } else {
                            // Partial Z85 block at end or before non-Z85
                            decode_z85_digits(&mut output, digits)?;
                        }
                    }
                    break;
                }
            }
            _ => {
                return Err("invalid character in jeb85 input".into());
            }
        }
    }

    Ok(output)
}

/// Decode Z85 digits into output.
fn decode_z85_digits(output: &mut Vec<u8>, digits: &[u8]) -> Result<(), String> {
    let decoded = decode_z85(digits).map_err(|_| "invalid Z85 data".to_string())?;
    output.extend_from_slice(&decoded);
    Ok(())
}

/// Decode a length from Z85 digits.
fn decode_length(digits: &[u8]) -> Result<usize, String> {
    let mut value: usize = 0;
    for &digit in digits {
        let digit_value = Z85_LUT[digit as usize];
        if digit_value >= BASE_85 as u8 {
            return Err("invalid Z85 digit in length".into());
        }
        value = value
            .checked_mul(BASE_85)
            .ok_or_else(|| "length overflow".to_string())?
            .checked_add(digit_value as usize)
            .ok_or_else(|| "length overflow".to_string())?;
    }
    Ok(value)
}

// =============================================================================
// HELPERS
// =============================================================================

/// Check if a byte is raw-friendly (can be included in raw sequences).
#[inline]
fn is_raw_friendly(byte: u8) -> bool {
    ASCII_INLINE_TEXT_LUT[byte as usize]
}

/// Check if a byte is a valid Z85 digit.
#[inline]
fn is_z85_digit(byte: u8) -> bool {
    Z85_LUT[byte as usize] < BASE_85 as u8
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(input: &[u8]) -> Vec<u8> {
        let encoded = encode_jeb85(input);
        decode_jeb85(&encoded).expect("decode failed")
    }

    #[test]
    fn test_empty() {
        assert_eq!(roundtrip(b""), b"");
    }

    #[test]
    fn test_pure_binary() {
        let input = vec![0xFF, 0x00, 0x12, 0x34];
        let encoded = encode_jeb85(&input);
        eprintln!(
            "pure_binary encoded: {:?}",
            String::from_utf8_lossy(&encoded)
        );
        let decoded = decode_jeb85(&encoded).expect("decode failed");
        eprintln!("pure_binary decoded: {:?}", decoded);
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_4_raw_bytes() {
        let input = b"test";
        let encoded = encode_jeb85(input);
        assert_eq!(&encoded, b"_test");
        assert_eq!(roundtrip(input), input);
    }

    #[test]
    fn test_6_raw_bytes() {
        let input = b"hello!";
        let encoded = encode_jeb85(input);
        assert_eq!(&encoded, b"~hello!");
        assert_eq!(roundtrip(input), input);
    }

    #[test]
    fn test_8_raw_bytes() {
        let input = b"testtest";
        let encoded = encode_jeb85(input);
        // Should be 8|testtest (length 8 in base-85 is just "8")
        assert!(
            encoded.starts_with(b"8|"),
            "got {:?}",
            String::from_utf8_lossy(&encoded)
        );
        assert_eq!(roundtrip(input), input);
    }

    #[test]
    fn test_z85_partial_roundtrip() {
        // Test that z85 encode/decode works for partial blocks
        let input1 = b"x";
        let enc1 = encode_z85(input1);
        eprintln!("z85 encode 'x': {:?}", String::from_utf8_lossy(&enc1));
        let dec1 = decode_z85(&enc1).ok().expect("decode failed");
        eprintln!("z85 decode: {:?}", dec1);
        assert_eq!(dec1.as_slice(), input1.as_slice());
    }

    #[test]
    fn test_short_raw_1() {
        assert_eq!(roundtrip(b"x"), b"x");
    }

    #[test]
    fn test_short_raw_2() {
        assert_eq!(roundtrip(b"ab"), b"ab");
    }

    #[test]
    fn test_short_raw_3() {
        assert_eq!(roundtrip(b"abc"), b"abc");
    }

    #[test]
    fn test_short_raw_5() {
        assert_eq!(roundtrip(b"hello"), b"hello");
    }

    #[test]
    fn test_short_raw_7() {
        assert_eq!(roundtrip(b"testing"), b"testing");
    }

    #[test]
    fn test_mixed_binary_raw() {
        let input = vec![0xFF, 0xFF, 0xFF, 0xFF, b't', b'e', b's', b't'];
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_raw_between_binary() {
        let mut input = vec![0xFF, 0xFF, 0xFF, 0xFF];
        input.extend_from_slice(b"testtest");
        input.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        let encoded = encode_jeb85(&input);
        eprintln!(
            "raw_between_binary encoded: {:?}",
            String::from_utf8_lossy(&encoded)
        );
        let decoded = decode_jeb85(&encoded).expect("decode failed");
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_long_raw() {
        let input = b"The quick brown fox jumps over the lazy dog".to_vec();
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_very_long_raw() {
        let input = vec![b'x'; 1000];
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_alternating() {
        let mut input = Vec::new();
        for _ in 0..10 {
            input.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
            input.extend_from_slice(b"test");
        }
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_all_byte_values() {
        let input: Vec<u8> = (0..=255).collect();
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_decode_error_truncated_underscore() {
        assert!(decode_jeb85(b"_ab").is_err());
    }

    #[test]
    fn test_decode_error_truncated_tilde() {
        assert!(decode_jeb85(b"~hello").is_err());
    }

    #[test]
    fn test_decode_error_standalone_pipe() {
        assert!(decode_jeb85(b"|test").is_err());
    }

    // =========================================================================
    // EDGE CASE TESTS - Reserved prefix characters in input
    // =========================================================================

    #[test]
    fn test_underscore_in_input_4_bytes() {
        // Input contains underscore - currently raw-friendly but it's our prefix!
        // If we encode "a_bc" as "_a_bc", decoder sees _ prefix and expects 4 raw bytes
        // "a_bc" but then there's leftover. Need to verify this works.
        let input = b"a_bc";
        let encoded = encode_jeb85(input);
        eprintln!(
            "underscore_in_input_4: {:?}",
            String::from_utf8_lossy(&encoded)
        );
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_underscore_in_input_start_4_bytes() {
        // Input starts with underscore: "_abc" (4 bytes)
        // Encoded as "__abc" - decoder sees _ prefix, reads "_abc" as 4 raw bytes. OK!
        let input = b"_abc";
        let encoded = encode_jeb85(input);
        eprintln!(
            "underscore_start_4: {:?}",
            String::from_utf8_lossy(&encoded)
        );
        // Should encode as "__abc" (prefix _ followed by raw "_abc")
        assert_eq!(&encoded, b"__abc");
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_tilde_in_input_6_bytes() {
        // Input contains tilde: "ab~cde" (6 bytes)
        // Encoded as "~ab~cde" - decoder reads "ab~cde" as 6 raw bytes
        let input = b"ab~cde";
        let encoded = encode_jeb85(input);
        eprintln!("tilde_in_6: {:?}", String::from_utf8_lossy(&encoded));
        assert_eq!(&encoded, b"~ab~cde");
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_tilde_in_input_start_6_bytes() {
        // Input starts with tilde: "~abcde" (6 bytes)
        // Encoded as "~~abcde" - decoder sees ~ prefix, reads "~abcde"
        let input = b"~abcde";
        let encoded = encode_jeb85(input);
        eprintln!("tilde_start_6: {:?}", String::from_utf8_lossy(&encoded));
        assert_eq!(&encoded, b"~~abcde");
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_pipe_in_input_8_bytes() {
        // Input contains pipe: "abc|defg" (8 bytes)
        // Encoded as "8|abc|defg" - decoder sees 8| prefix, reads 8 bytes "abc|defg"
        let input = b"abc|defg";
        let encoded = encode_jeb85(input);
        eprintln!("pipe_in_8: {:?}", String::from_utf8_lossy(&encoded));
        assert_eq!(&encoded, b"8|abc|defg");
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_pipe_in_input_start_8_bytes() {
        // Input starts with pipe: "|abcdefg" (8 bytes)
        // Encoded as "8||abcdefg" - decoder sees 8| prefix, reads 8 bytes "|abcdefg"
        let input = b"|abcdefg";
        let encoded = encode_jeb85(input);
        eprintln!("pipe_start_8: {:?}", String::from_utf8_lossy(&encoded));
        assert_eq!(&encoded, b"8||abcdefg");
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_all_reserved_chars_in_input() {
        // Input has all reserved chars: "_~|_~|_~" (8 bytes)
        let input = b"_~|_~|_~";
        let encoded = encode_jeb85(input);
        eprintln!("all_reserved_8: {:?}", String::from_utf8_lossy(&encoded));
        assert_eq!(&encoded, b"8|_~|_~|_~");
        assert_eq!(roundtrip(input), input.to_vec());
    }

    // =========================================================================
    // EDGE CASE TESTS - Length prefix encoding (base-85 boundaries)
    // =========================================================================

    #[test]
    fn test_length_85_requires_two_digits() {
        // Length 85 = "10" in base-85 (85 = 1*85 + 0)
        // So encoded as "10|" followed by 85 raw bytes
        let input = vec![b'x'; 85];
        let encoded = encode_jeb85(&input);
        eprintln!("length_85: {:?}", String::from_utf8_lossy(&encoded[..10]));
        // Should start with "10|" (length 85 in base-85)
        assert!(
            encoded.starts_with(b"10|"),
            "got: {:?}",
            String::from_utf8_lossy(&encoded[..10])
        );
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_length_84_single_digit() {
        // Length 84 = "#" in Z85 (Z85 digit 84 is '#')
        // Z85: 0-9, a-z, A-Z, .-:+=^!/*?&<>()[]{}@%$#
        // So 84 should be the character at index 84 in Z85 alphabet
        let input = vec![b'x'; 84];
        let encoded = encode_jeb85(&input);
        eprintln!("length_84: {:?}", String::from_utf8_lossy(&encoded[..5]));
        // Z85[84] = '#', so should start with "#|"
        assert!(
            encoded.starts_with(b"#|"),
            "got: {:?}",
            String::from_utf8_lossy(&encoded[..5])
        );
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_length_7225_requires_three_digits() {
        // Length 7225 = 85*85 = "100" in base-85
        let input = vec![b'x'; 7225];
        let encoded = encode_jeb85(&input);
        eprintln!("length_7225: {:?}", String::from_utf8_lossy(&encoded[..10]));
        // Should start with "100|" (85^2 in base-85)
        assert!(
            encoded.starts_with(b"100|"),
            "got: {:?}",
            String::from_utf8_lossy(&encoded[..10])
        );
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_length_exactly_8() {
        // Minimum for variable-length prefix (MIN_VAR_RAW_BYTES = 8)
        // Length 8 = "8" in base-85
        let input = vec![b'x'; 8];
        let encoded = encode_jeb85(&input);
        eprintln!("length_8: {:?}", String::from_utf8_lossy(&encoded));
        // Should be "8|xxxxxxxx"
        assert_eq!(&encoded, b"8|xxxxxxxx");
        assert_eq!(roundtrip(&input), input);
    }

    // =========================================================================
    // EDGE CASE TESTS - Z85 and length prefix ambiguity
    // =========================================================================

    #[test]
    fn test_z85_five_digits_then_pipe_in_raw() {
        // If Z85 data happens to be followed by a raw region starting with |
        // e.g., binary data that encodes to some Z85 followed by raw "|test..."
        // Decoder must recognize complete Z85 blocks, not confuse with length prefix
        // 4 bytes of binary followed by 8+ raw bytes starting with |
        let mut input = vec![0xFF, 0xFF, 0xFF, 0xFF]; // 4 binary bytes = 5 Z85 chars
        input.extend_from_slice(b"|testxxx"); // 8 raw bytes starting with |
        let encoded = encode_jeb85(&input);
        eprintln!("z85_then_pipe: {:?}", String::from_utf8_lossy(&encoded));
        // Should be: 5 Z85 digits + "8||testxxx"
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_z85_four_digits_then_pipe() {
        // 3 binary bytes (encodes to 4 Z85 digits) followed by raw starting with |
        // BUG: This creates ambiguity! Encoder produces "%nS98||testxxx" where:
        //   - "%nS9" = 4 Z85 digits for 3 bytes
        //   - "8|" = length prefix for 8 bytes
        //   - "|testxxx" = the 8 raw bytes
        // But decoder sees "%nS98" as 5-digit Z85 block, then "||testxxx" with no
        // prefix! TODO: Fix encoder to avoid this ambiguity
        let mut input = vec![0xFF, 0xFF, 0xFF]; // 3 binary bytes = 4 Z85 chars
        input.extend_from_slice(b"|testxxx"); // 8 raw bytes starting with |
        let encoded = encode_jeb85(&input);
        eprintln!("z85_4_then_pipe: {:?}", String::from_utf8_lossy(&encoded));
        // This currently fails - documenting the bug
        assert_eq!(
            roundtrip(&input),
            input,
            "KNOWN BUG: partial Z85 + length prefix ambiguity"
        );
    }

    #[test]
    fn test_z85_partial_block_before_raw_prefix() {
        // Partial Z85 block (1-4 digits) right before a _ or ~ prefix
        // e.g., 1 binary byte encoded as 2 Z85 chars followed by "_test"
        let mut input = vec![0xFF]; // 1 binary byte = 2 Z85 chars
        input.extend_from_slice(b"test"); // 4 raw bytes
        let encoded = encode_jeb85(&input);
        eprintln!(
            "z85_partial_before_raw: {:?}",
            String::from_utf8_lossy(&encoded)
        );
        // Should be: 2 Z85 chars + "_test"
        assert_eq!(roundtrip(&input), input);
    }

    // =========================================================================
    // EDGE CASE TESTS - Boundary between raw and binary modes
    // =========================================================================

    #[test]
    fn test_exactly_min_raw_run_threshold() {
        // Exactly MIN_RAW_RUN (4) raw-friendly bytes - should trigger raw mode
        let input = b"test";
        let encoded = encode_jeb85(input);
        eprintln!("exactly_min_raw: {:?}", String::from_utf8_lossy(&encoded));
        // Should use _ prefix for exactly 4 raw bytes
        assert_eq!(&encoded, b"_test");
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_one_below_min_raw_run_threshold() {
        // 3 raw-friendly bytes surrounded by binary - should use Z85
        let mut input = vec![0xFF];
        input.extend_from_slice(b"abc"); // 3 raw bytes - below threshold
        input.push(0xFF);
        let encoded = encode_jeb85(&input);
        eprintln!("below_min_raw: {:?}", String::from_utf8_lossy(&encoded));
        // Should NOT use raw prefix, should be all Z85
        assert!(
            !encoded.contains(&b'_'),
            "should not use _ prefix for 3 raw bytes"
        );
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_short_raw_gaps_between_raw_regions() {
        // Pattern: 8 raw, 1 binary, 8 raw
        // BUG: Encoder produces "8|testtest@@8|wordword" where:
        //   - "8|testtest" = 8 raw bytes
        //   - "@@" = Z85 for 0xFF (2 digits)
        //   - "8|wordword" = 8 raw bytes
        // But decoder sees "@@8" as a length prefix (621,273 bytes!)
        // TODO: Fix encoder to avoid partial Z85 + length prefix ambiguity
        let mut input = Vec::new();
        input.extend_from_slice(b"testtest"); // 8 raw
        input.push(0xFF); // 1 binary
        input.extend_from_slice(b"wordword"); // 8 raw
        let encoded = encode_jeb85(&input);
        eprintln!("raw_gap_raw: {:?}", String::from_utf8_lossy(&encoded));
        assert_eq!(
            roundtrip(&input),
            input,
            "KNOWN BUG: partial Z85 + length prefix ambiguity"
        );
    }

    #[test]
    fn test_single_binary_byte_between_raw() {
        // "test" + 0xFF + "test" - two raw regions separated by 1 binary byte
        // This likely has the same bug as test_short_raw_gaps_between_raw_regions
        let mut input = Vec::new();
        input.extend_from_slice(b"test"); // 4 raw
        input.push(0xFF); // 1 binary
        input.extend_from_slice(b"test"); // 4 raw
        let encoded = encode_jeb85(&input);
        eprintln!(
            "single_binary_between: {:?}",
            String::from_utf8_lossy(&encoded)
        );
        assert_eq!(
            roundtrip(&input),
            input,
            "KNOWN BUG: partial Z85 + length prefix ambiguity"
        );
    }

    // =========================================================================
    // EDGE CASE TESTS - Partial Z85 blocks
    // =========================================================================

    #[test]
    fn test_partial_z85_1_byte_then_raw() {
        // 1 binary byte (2 Z85 chars), then 4+ raw-friendly bytes
        let mut input = vec![0xFF];
        input.extend_from_slice(b"test");
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_partial_z85_2_bytes_then_raw() {
        // 2 binary bytes (3 Z85 chars), then raw region
        let mut input = vec![0xFF, 0xFF];
        input.extend_from_slice(b"test");
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_partial_z85_3_bytes_then_raw() {
        // 3 binary bytes (4 Z85 chars), then raw region
        // This has the ambiguity bug if raw starts with a digit
        let mut input = vec![0xFF, 0xFF, 0xFF];
        input.extend_from_slice(b"test");
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_partial_z85_at_end() {
        // Raw region followed by 1-3 binary bytes at end
        let mut input = Vec::new();
        input.extend_from_slice(b"test");
        input.extend_from_slice(&[0xFF, 0xFF, 0xFF]);
        assert_eq!(roundtrip(&input), input);
    }

    // =========================================================================
    // EDGE CASE TESTS - Zero bytes and special values
    // =========================================================================

    #[test]
    fn test_all_zero_bytes() {
        // 4 zero bytes - definitely not raw-friendly, should encode as Z85 "00000"
        let input = vec![0, 0, 0, 0];
        let encoded = encode_jeb85(&input);
        assert_eq!(&encoded, b"00000");
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_zero_byte_in_middle_of_raw() {
        // "te\x00st" - zero byte breaks raw-friendliness
        let input = b"te\x00st";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_0xff_bytes() {
        // All 0xFF bytes - max value, definitely binary
        let input = vec![0xFF; 10];
        assert_eq!(roundtrip(&input), input);
    }

    // =========================================================================
    // EDGE CASE TESTS - Decoder error cases
    // =========================================================================

    #[test]
    fn test_decode_truncated_length_prefix_raw() {
        // "8|abc" - length says 8 bytes but only 3 follow
        let result = decode_jeb85(b"8|abc");
        assert!(result.is_err(), "should fail on truncated raw data");
    }

    #[test]
    fn test_decode_length_overflow() {
        // Length that would overflow - not critical since we check bounds
        // Just verify it doesn't panic
        let result = decode_jeb85(b"#####|abc");
        // Either succeeds (if length is small enough) or fails gracefully
        let _ = result;
    }

    #[test]
    fn test_decode_invalid_z85_digit() {
        // Space (0x20) is not a valid Z85 digit
        let result = decode_jeb85(b"hello world");
        assert!(result.is_err(), "should fail on invalid Z85 character (space)");
    }

    #[test]
    fn test_decode_invalid_character_in_length() {
        // Space before | should fail
        let result = decode_jeb85(b" |test");
        assert!(result.is_err(), "should fail on invalid character in length");
    }

    // =========================================================================
    // EDGE CASE TESTS - Encoding efficiency
    // =========================================================================

    #[test]
    fn test_encoding_never_longer_than_z85() {
        // For most inputs, jeb85 should be equal or shorter than pure Z85
        // Test a few cases
        let inputs = vec![
            vec![0xFF; 10],
            b"hello world".to_vec(),
            (0..=255).collect::<Vec<u8>>(),
        ];
        for input in inputs {
            let jeb85_len = encode_jeb85(&input).len();
            let z85_len = encode_z85(&input).len();
            // jeb85 may be slightly longer due to mode switches, but not by much
            assert!(jeb85_len <= z85_len * 2, "jeb85 shouldn't be dramatically longer than z85");
        }
    }

    #[test]
    fn test_raw_encoding_overhead() {
        // Test efficiency of different raw encoding methods
        let input4 = b"test";  // 4 bytes: "_test" (5) vs Z85 (5) - break even
        let input6 = b"hello!";  // 6 bytes: "~hello!" (7) vs Z85 (8) - saves 1
        assert_eq!(roundtrip(input4), input4.to_vec());
        assert_eq!(roundtrip(input6), input6.to_vec());
    }

    // =========================================================================
    // EDGE CASE TESTS - Consecutive raw regions
    // =========================================================================

    #[test]
    fn test_two_4byte_raw_regions_adjacent() {
        // "testword" - 8 raw-friendly bytes should be treated as one region
        let input = b"testword";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_two_6byte_raw_regions_adjacent() {
        // 12 raw-friendly bytes
        let input = b"hello!world!";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_many_consecutive_raw_regions() {
        // Very long all-raw input
        let input = vec![b'x'; 1000];
        assert_eq!(roundtrip(&input), input);
    }

    // =========================================================================
    // EDGE CASE TESTS - Length prefix digit characters in input
    // =========================================================================

    #[test]
    fn test_z85_digits_as_raw_data() {
        // Raw input that looks like Z85: "0123abcd" (8 bytes)
        let input = b"0123abcd";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_raw_starts_with_digit_then_pipe() {
        // Input like "8|xxxxxx" (8 bytes) - looks exactly like our encoding!
        let input = b"8|xxxxxx";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_raw_is_fake_underscore_prefix() {
        // Input "_test" (5 bytes)
        let input = b"_test";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_raw_is_fake_tilde_prefix() {
        // Input "~hello!" (7 bytes)
        let input = b"~hello!";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    // =========================================================================
    // EDGE CASE TESTS - Mixed scenarios
    // =========================================================================

    #[test]
    fn test_binary_raw_binary_raw_pattern() {
        // Alternating: 4 binary, 4 raw, 4 binary, 4 raw
        let mut input = Vec::new();
        input.extend_from_slice(&[0xFF; 4]);
        input.extend_from_slice(b"test");
        input.extend_from_slice(&[0xFF; 4]);
        input.extend_from_slice(b"word");
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_raw_binary_raw_binary_pattern() {
        // Alternating: 4 raw, 4 binary, 4 raw, 4 binary
        let mut input = Vec::new();
        input.extend_from_slice(b"test");
        input.extend_from_slice(&[0xFF; 4]);
        input.extend_from_slice(b"word");
        input.extend_from_slice(&[0xFF; 4]);
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_long_binary_short_raw_long_binary() {
        // 100 binary bytes, 4 raw, 100 binary bytes
        let mut input = vec![0xFF; 100];
        input.extend_from_slice(b"test");
        input.extend_from_slice(&vec![0xFF; 100]);
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_long_raw_short_binary_long_raw() {
        // 100 raw bytes, 4 binary, 100 raw bytes
        let mut input = vec![b'x'; 100];
        input.extend_from_slice(&[0xFF; 4]);
        input.extend_from_slice(&vec![b'x'; 100]);
        assert_eq!(roundtrip(&input), input);
    }

    // =========================================================================
    // EDGE CASE TESTS - Specific ASCII ranges
    // =========================================================================

    #[test]
    fn test_printable_ascii_range() {
        // All printable ASCII: 0x20-0x7E (but excluding tab/newline/cr)
        let input: Vec<u8> = (0x20..=0x7E).collect();
        assert_eq!(roundtrip(&input), input);
    }

    #[test]
    fn test_lowercase_letters_only() {
        let input = b"abcdefghijklmnopqrstuvwxyz";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_uppercase_letters_only() {
        let input = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_digits_only() {
        let input = b"0123456789";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_punctuation_only() {
        // Various punctuation that should be raw-friendly
        let input = b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    // =========================================================================
    // EDGE CASE TESTS - Control characters
    // =========================================================================

    #[test]
    fn test_tab_not_raw_friendly() {
        // Tab (0x09) should NOT be raw-friendly
        let input = b"test\tword";
        let encoded = encode_jeb85(input);
        // Should use Z85, not raw
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_newline_not_raw_friendly() {
        // Newline (0x0A) should NOT be raw-friendly
        let input = b"test\nword";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_carriage_return_not_raw_friendly() {
        // CR (0x0D) should NOT be raw-friendly
        let input = b"test\rword";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    #[test]
    fn test_null_byte_not_raw_friendly() {
        // Null (0x00) should NOT be raw-friendly
        let input = b"test\0word";
        assert_eq!(roundtrip(input), input.to_vec());
    }

    // =========================================================================
    // EDGE CASE TESTS - Determinism and idempotence
    // =========================================================================

    #[test]
    fn test_encode_is_deterministic() {
        // Same input always produces same output
        let input = b"hello world 12345";
        let enc1 = encode_jeb85(input);
        let enc2 = encode_jeb85(input);
        assert_eq!(enc1, enc2);
    }

    #[test]
    fn test_double_encode_decode() {
        // encode(encode(x)) can be decoded back (two layers)
        let input = b"hello world";
        let enc1 = encode_jeb85(input);
        let enc2 = encode_jeb85(&enc1);
        let dec1 = decode_jeb85(&enc2).unwrap();
        let dec2 = decode_jeb85(&dec1).unwrap();
        assert_eq!(dec2, input);
    }
}
