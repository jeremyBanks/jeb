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
}
