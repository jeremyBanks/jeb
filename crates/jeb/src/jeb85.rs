use crate::{
    ASCII_INLINE_TEXT_LUT, Z85_LUT, div_exact, usize_eq,
    z85::{
        BASE_85, BLOCK_BYTES_4, BLOCK_DIGITS_5, BLOCK_DIGITS_BY_BYTES, encode_z85_block,
        encode_z85_compact, can_encode_compactly, encoded_z85_length, decode_z85_block,
    },
};



/// Raw sequence prefix for exactly 4 bytes
pub const RAW_PREFIX_4: u8 = b'_';
/// Raw sequence prefix for exactly 6 bytes
pub const RAW_PREFIX_6: u8 = b'~';
/// Raw sequence prefix for variable-length look-back encoding
pub const RAW_PREFIX_VAR: u8 = b'|';

/// When this encoding is used to convert binary data into line of text, our
/// implementation limits each line to 80 digits.
pub const TARGET_LINE_SIZE_DIGITS: usize = 80;
/// When this encoding is split into 80 digit lines, each line contains 64 bytes
/// of data, which has a good chance of some alignment with binary data.
pub const TARGET_LINE_SIZE_BYTES: usize = usize_eq(
    64,
    div_exact(TARGET_LINE_SIZE_DIGITS * BLOCK_BYTES_4, BLOCK_DIGITS_5),
);

/// Attempts to encode a 4-byte block using mid-block transition (Z85 prefix + raw suffix).
/// Returns Some((encoded, actual_length)) if successful, None if not beneficial.
///
/// Pattern: <Z85 chars>|<raw bytes> (total 5 chars)
///
/// This works when:
/// - First N bytes (1-3) have a value that fits in fewer than standard Z85 chars
/// - Remaining 4-N bytes are all ASCII-safe
/// - Total: prefix_z85_chars + 1 (|) + (4-N) raw bytes = 5
fn try_encode_mid_block_transition(block: &[u8; BLOCK_BYTES_4]) -> Option<[u8; BLOCK_DIGITS_5]> {
    // Try each possible split point (1, 2, or 3 bytes)
    for prefix_bytes in 1..=3 {
        let suffix_bytes = BLOCK_BYTES_4 - prefix_bytes;

        // Check if suffix bytes are all ASCII-safe
        let suffix_safe = block[prefix_bytes..].iter()
            .all(|b| ASCII_INLINE_TEXT_LUT[*b as usize]);

        if !suffix_safe {
            continue;
        }

        // Calculate prefix value (big-endian)
        let mut prefix_value = 0u32;
        for &byte in &block[..prefix_bytes] {
            prefix_value = (prefix_value << 8) | u32::from(byte);
        }

        // Check if prefix can be compactly encoded
        if let Some(prefix_chars) = can_encode_compactly(prefix_value, prefix_bytes) {
            // Total length must be 5: prefix_chars + 1 (|) + suffix_bytes
            if prefix_chars + 1 + suffix_bytes == BLOCK_DIGITS_5 {
                // Encode it!
                let (z85_block, _) = encode_z85_compact(prefix_value);
                let z85_start = BLOCK_DIGITS_5 - prefix_chars;

                let mut result = [0u8; BLOCK_DIGITS_5];
                // Copy Z85 prefix
                result[..prefix_chars].copy_from_slice(&z85_block[z85_start..]);
                // Add delimiter
                result[prefix_chars] = RAW_PREFIX_VAR;
                // Copy raw suffix
                result[prefix_chars + 1..].copy_from_slice(&block[prefix_bytes..]);

                return Some(result);
            }
        }
    }

    None
}

/// Encodes a length using the look-back scheme.
///
/// Returns the Z85 digits that encode the length, where the last digit's
/// low 2 bits indicate how many additional digits to read backwards.
///
/// The decoder will:
/// 1. Read the last digit D
/// 2. Compute N = D & 3 (0-3 additional digits)
/// 3. Read N more digits backwards (total N+1 digits)
/// 4. Decode all N+1 digits as base-85 number = length
///
/// For this to work: last_digit & 3 must equal (total_digits - 1)
fn encode_length_lookback(length: usize) -> Vec<u8> {
    let length_u32 = length as u32;

    // Determine minimum number of digits needed
    let min_digits = if length_u32 <= 84 {
        1
    } else if length_u32 <= 7_224 {
        2
    } else if length_u32 <= 614_124 {
        3
    } else if length_u32 <= 52_200_624 {
        4
    } else {
        5
    };

    // Try each possible number of digits starting from minimum
    // We need to find N where (last_digit & 3) == (N - 1)
    for num_digits in min_digits..=5 {
        // Encode length in base-85 with exactly num_digits digits
        let z85_block = encode_z85_block(length_u32.to_be_bytes());
        let start = BLOCK_DIGITS_5 - num_digits;

        // Check if last digit satisfies constraint
        let last_digit_char = z85_block[BLOCK_DIGITS_5 - 1];
        let last_digit_value = Z85_LUT[last_digit_char as usize] as usize;

        if (last_digit_value & 3) == (num_digits - 1) {
            // Found valid representation!
            return z85_block[start..].to_vec();
        }
    }

    // If we get here, we couldn't find a valid representation with ≤5 digits
    // This shouldn't happen for reasonable lengths, but fall back to max digits
    let z85_block = encode_z85_block(length_u32.to_be_bytes());
    z85_block.to_vec()
}

/// Flushes the raw buffer using the appropriate encoding scheme.
///
/// Uses one of three schemes:
/// - Exactly 4 bytes: `_xxxx`
/// - Exactly 6 bytes: `~xxxxxx`
/// - Other lengths: look-back `|` encoding if beneficial, otherwise break into chunks
fn flush_raw_buffer(output: &mut Vec<u8>, raw_buffer: &mut Vec<u8>) {
    let byte_count = raw_buffer.len();

    if byte_count == 0 {
        return;
    }

    // Use fixed 4-byte prefix
    if byte_count == 4 {
        output.push(RAW_PREFIX_4);
        output.extend_from_slice(raw_buffer);
        raw_buffer.clear();
        return;
    }

    // Use fixed 6-byte prefix
    if byte_count == 6 {
        output.push(RAW_PREFIX_6);
        output.extend_from_slice(raw_buffer);
        raw_buffer.clear();
        return;
    }

    // For other lengths, try look-back encoding if beneficial
    // Calculate how much space each encoding would use
    let prefix_digits = encode_length_lookback(byte_count);
    let lookback_length = prefix_digits.len() + 1 + byte_count;

    // Calculate optimal encoding by breaking into chunks with _, ~
    // Prefer multiples of 4 to avoid Z85 overhead on remainder
    let optimal_chunk_length = if byte_count % 4 == 0 {
        // Multiple of 4: use all _ prefixes
        (byte_count / 4) * 5
    } else if byte_count % 6 == 0 {
        // Multiple of 6: use all ~ prefixes
        (byte_count / 6) * 7
    } else {
        // Mixed: try to minimize Z85 usage
        let chunks_6 = byte_count / 6;
        let chunks_4 = (byte_count % 6) / 4;
        let remainder = byte_count % 4;

        let opt1 = chunks_6 * 7 + chunks_4 * 5 + if remainder > 0 { BLOCK_DIGITS_BY_BYTES[remainder] } else { 0 };

        // Alternative: use all 4-byte chunks
        let chunks_4_alt = byte_count / 4;
        let remainder_alt = byte_count % 4;
        let opt2 = chunks_4_alt * 5 + if remainder_alt > 0 { BLOCK_DIGITS_BY_BYTES[remainder_alt] } else { 0 };

        std::cmp::min(opt1, opt2)
    };

    // Use look-back if it's shorter than chunked encoding
    if byte_count >= 5 && lookback_length < optimal_chunk_length {
        output.extend_from_slice(&prefix_digits);
        output.push(RAW_PREFIX_VAR);
        output.extend_from_slice(raw_buffer);
        raw_buffer.clear();
        return;
    }

    // Use chunked encoding with _, ~, and Z85
    // Prefer 4-byte chunks to avoid Z85 overhead
    let mut pos = 0;
    while pos < byte_count {
        let remaining = byte_count - pos;

        // Check if using ~ leaves a bad remainder
        if remaining >= 6 && remaining % 6 == 0 {
            // Use ~ for exactly divisible by 6
            output.push(RAW_PREFIX_6);
            output.extend_from_slice(&raw_buffer[pos..pos + 6]);
            pos += 6;
        } else if remaining >= 4 {
            // Use _ for 4 bytes
            output.push(RAW_PREFIX_4);
            output.extend_from_slice(&raw_buffer[pos..pos + 4]);
            pos += 4;
        } else {
            // Use Z85 for < 4 bytes
            let mut byte_block = [0x00; BLOCK_BYTES_4];
            byte_block[..remaining].copy_from_slice(&raw_buffer[pos..pos + remaining]);
            let encoded_length = BLOCK_DIGITS_BY_BYTES[remaining];
            let encoded_block = encode_z85_block(byte_block);
            output.extend_from_slice(&encoded_block[..encoded_length]);
            pos += remaining;
        }
    }

    raw_buffer.clear();
}

/// Decodes JEB85-encoded data back to bytes.
///
/// Handles:
/// - Standard Z85 blocks (5 digits → 4 bytes)
/// - Fixed 4-byte raw: `_xxxx`
/// - Fixed 6-byte raw: `~xxxxxx`
/// - Mid-block transitions: `X|yyy` where X is compact Z85
/// - Look-back variable-length: `[digits]|data` where last digit's low 2 bits encode digit count
///
/// # Errors
///
/// Returns an error if the input contains invalid characters or is malformed.
pub fn decode_jeb85(encoded: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut output = Vec::new();
    let mut pos = 0;

    while pos < encoded.len() {
        let ch = encoded[pos];

        // Check for fixed-length raw prefixes first
        if ch == RAW_PREFIX_4 {
            // Fixed 4-byte raw sequence: _xxxx
            pos += 1;
            if pos + 4 > encoded.len() {
                return Err("decode_jeb85: insufficient bytes for _xxxx");
            }
            output.extend_from_slice(&encoded[pos..pos + 4]);
            pos += 4;
            continue;
        }

        if ch == RAW_PREFIX_6 {
            // Fixed 6-byte raw sequence: ~xxxxxx
            pos += 1;
            if pos + 6 > encoded.len() {
                return Err("decode_jeb85: insufficient bytes for ~xxxxxx");
            }
            output.extend_from_slice(&encoded[pos..pos + 6]);
            pos += 6;
            continue;
        }

        // Check if this is a Z85 digit
        let digit_value = Z85_LUT[ch as usize];

        if digit_value < BASE_85 as u8 {
            // Collect consecutive Z85 digits (up to 5 for a block)
            let mut z85_buf = [0u8; BLOCK_DIGITS_5];
            let mut z85_len = 0;

            while pos < encoded.len() && z85_len < BLOCK_DIGITS_5 {
                let ch = encoded[pos];
                let val = Z85_LUT[ch as usize];

                if val < BASE_85 as u8 {
                    z85_buf[z85_len] = ch;
                    z85_len += 1;
                    pos += 1;
                } else {
                    break;
                }
            }

            // Check what follows the Z85 digits
            if pos < encoded.len() && encoded[pos] == RAW_PREFIX_VAR {
                // This could be either a mid-block transition or look-back encoding
                // Mid-block: prefix_digits + | + suffix_bytes = 5 total
                // Look-back: length_digits + | + data_bytes

                // First, check if this could be a mid-block transition
                // Mid-block requires: 1-3 Z85 digits + | + 1-3 raw bytes = 5 total
                let ascii_suffix_len = BLOCK_DIGITS_5 - z85_len - 1;

                if z85_len >= 1 && z85_len <= 3 && ascii_suffix_len >= 1 && ascii_suffix_len <= 3 {
                    // Decode the Z85 prefix to get the value
                    let mut prefix_value = 0u32;
                    for i in 0..z85_len {
                        let val = Z85_LUT[z85_buf[i] as usize];
                        prefix_value = prefix_value * (BASE_85 as u32) + (val as u32);
                    }

                    let prefix_bytes = if z85_len == 1 { 1 }
                        else if z85_len == 2 { 2 }
                        else { 3 };

                    // Check if this value can be compactly encoded
                    if can_encode_compactly(prefix_value, prefix_bytes).map_or(false, |chars| chars == z85_len) {
                        // Check if we have enough bytes for the suffix
                        if pos + 1 + ascii_suffix_len <= encoded.len() {
                            // Valid mid-block transition!
                            pos += 1; // Skip '|'

                            // Output prefix bytes (big-endian)
                            let prefix_byte_slice = &prefix_value.to_be_bytes()[BLOCK_BYTES_4 - prefix_bytes..];
                            output.extend_from_slice(prefix_byte_slice);

                            // Output raw suffix bytes
                            output.extend_from_slice(&encoded[pos..pos + ascii_suffix_len]);
                            pos += ascii_suffix_len;
                            continue;
                        }
                    }
                }

                // Not a mid-block transition, decode as look-back length encoding
                // All Z85 digits represent the length in base-85
                let mut length = 0u32;
                for i in 0..z85_len {
                    let val = Z85_LUT[z85_buf[i] as usize];
                    length = length * (BASE_85 as u32) + (val as u32);
                }

                pos += 1; // Skip '|'

                // Read 'length' raw bytes
                if pos + length as usize > encoded.len() {
                    return Err("decode_jeb85: insufficient bytes for look-back raw sequence");
                }
                output.extend_from_slice(&encoded[pos..pos + length as usize]);
                pos += length as usize;
            } else {
                // Standard Z85 block (no '|' follows)
                if z85_len == BLOCK_DIGITS_5 {
                    // Full block
                    let decoded = decode_z85_block(z85_buf)?;
                    output.extend_from_slice(&decoded);
                } else if z85_len > 0 {
                    // Partial block at end of input
                    for i in z85_len..BLOCK_DIGITS_5 {
                        z85_buf[i] = b'0';
                    }
                    let decoded = decode_z85_block(z85_buf)?;

                    // Only take bytes corresponding to original digit count
                    let byte_count = match z85_len {
                        1 => 1,
                        2 => 1,
                        3 => 2,
                        4 => 3,
                        _ => 4,
                    };
                    output.extend_from_slice(&decoded[..byte_count]);
                }
            }
        } else {
            // Invalid character
            return Err("decode_jeb85: invalid character");
        }
    }

    Ok(output)
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[must_use]
pub fn encode_jeb85(bytes: &[u8]) -> Vec<u8> {
    let encoded_length = encoded_z85_length(bytes.len());
    let mut output = Vec::with_capacity(encoded_length);

    let mut raw_buffer = Vec::<u8>::new();

    for bytes in bytes.chunks(BLOCK_BYTES_4) {
        // Fast path: all bytes are ASCII-safe, accumulate for raw encoding
        if bytes.iter().all(|b| ASCII_INLINE_TEXT_LUT[*b as usize]) {
            raw_buffer.extend_from_slice(bytes);
            continue;
        }

        // We have non-ASCII bytes, so flush any pending raw buffer first
        if !raw_buffer.is_empty() {
            flush_raw_buffer(&mut output, &mut raw_buffer);
        }

        // Prepare block for encoding (pad if needed)
        let byte_length = bytes.len();
        let mut byte_block = [0x00; BLOCK_BYTES_4];
        byte_block[..byte_length].copy_from_slice(bytes);

        // Try mid-block transition if this is a full 4-byte block
        if byte_length == BLOCK_BYTES_4 {
            if let Some(mid_block) = try_encode_mid_block_transition(&byte_block) {
                output.extend_from_slice(&mid_block);
                continue;
            }
        }

        // Fall back to standard Z85 encoding
        let encoded_length = BLOCK_DIGITS_BY_BYTES[bytes.len()];
        let encoded_block = encode_z85_block(byte_block);
        let encoded = &encoded_block[..encoded_length];

        output.extend_from_slice(encoded);
    }

    if !raw_buffer.is_empty() {
        flush_raw_buffer(&mut output, &mut raw_buffer);
    }

    debug_assert!(output.len() <= encoded_length);

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mid_block_transition_1_byte_prefix() {
        // [0x05, 'a', 'b', 'c'] should encode as "5|abc"
        // Value 5 fits in 1 Z85 char, 3 ASCII bytes follow
        let block = [0x05, b'a', b'b', b'c'];
        let result = try_encode_mid_block_transition(&block);

        assert!(result.is_some());
        let encoded = result.unwrap();
        assert_eq!(&encoded, b"5|abc");
    }

    #[test]
    fn test_mid_block_transition_2_byte_prefix() {
        // [0x01, 0x00, 'h', 'i'] should encode as "31|hi"
        // Value 256 (0x0100) fits in 2 Z85 chars, 2 ASCII bytes follow
        let block = [0x01, 0x00, b'h', b'i'];
        let result = try_encode_mid_block_transition(&block);

        assert!(result.is_some());
        let encoded = result.unwrap();
        assert_eq!(&encoded, b"31|hi");
    }

    #[test]
    fn test_mid_block_transition_3_byte_prefix() {
        // [0x01, 0x00, 0x00, 'x'] should encode as "961|x"
        // Value 65536 (0x010000) fits in 3 Z85 chars, 1 ASCII byte follows
        let block = [0x01, 0x00, 0x00, b'x'];
        let result = try_encode_mid_block_transition(&block);

        assert!(result.is_some());
        let encoded = result.unwrap();
        assert_eq!(&encoded, b"961|x");
    }

    #[test]
    fn test_mid_block_transition_fails_non_ascii_suffix() {
        // [0xFF, 0xFF, 0xFF, 0xFF] - all binary, no ASCII suffix at any split point
        let block = [0xFF, 0xFF, 0xFF, 0xFF];
        let result = try_encode_mid_block_transition(&block);

        assert!(result.is_none());
    }

    #[test]
    fn test_mid_block_transition_fails_value_too_large() {
        // [0xFF, 'a', 'b', 'c'] - value 255 needs 2 chars, doesn't fit pattern
        // 1 char + 1 (|) + 3 bytes = 5, but 255 needs 2 chars
        let block = [0xFF, b'a', b'b', b'c'];
        let result = try_encode_mid_block_transition(&block);

        assert!(result.is_none());
    }

    #[test]
    fn test_mid_block_transition_fails_all_binary() {
        // [0x00, 0x01, 0x02, 0x03] - all binary, no ASCII suffix
        let block = [0x00, 0x01, 0x02, 0x03];
        let result = try_encode_mid_block_transition(&block);

        assert!(result.is_none());
    }

    #[test]
    fn test_encode_jeb85_with_mid_block_transition() {
        // Test that encode_jeb85 uses mid-block transition
        let input = vec![0x05, b'a', b'b', b'c'];
        let encoded = encode_jeb85(&input);

        assert_eq!(&encoded, b"5|abc");
    }

    #[test]
    fn test_encode_jeb85_pure_ascii_short() {
        // Pure ASCII below 6 bytes uses Z85
        let input = b"test".to_vec();
        let encoded = encode_jeb85(&input);

        // Should be Z85 encoded: 5 chars for 4 bytes
        assert_eq!(encoded.len(), 5);
    }

    #[test]
    fn test_encode_jeb85_mixed_blocks() {
        // Mix of mid-block transition and pure ASCII (below 6-byte threshold)
        let input = vec![
            0x05, b'a', b'b', b'c',  // Mid-block: "5|abc"
            b't', b'e', b's', b't',   // Pure ASCII: uses Z85 (4 bytes < 6)
        ];
        let encoded = encode_jeb85(&input);

        // Should be: "5|abc" + Z85("test") = 5 + 5 = 10 chars
        assert_eq!(encoded.len(), 10);
        assert_eq!(&encoded[..5], b"5|abc");
    }

    #[test]
    fn test_encode_jeb85_round_trip_compatibility() {
        // Ensure mid-block encoded data has correct length (5 chars per 4 bytes)
        let inputs = vec![
            vec![0x05, b'x', b'y', b'z'],
            vec![0x01, 0x00, b'a', b'b'],
            vec![0x01, 0x00, 0x00, b'!'],
        ];

        for input in inputs {
            let encoded = encode_jeb85(&input);
            // 4 bytes should encode to exactly 5 characters
            assert_eq!(encoded.len(), 5, "Input: {:?}, Encoded: {:?}", input, encoded);
        }
    }

    #[test]
    fn test_mid_block_boundary_values() {
        // Test boundary values for each prefix size

        // Max 1-char value (84) with 3 ASCII bytes
        let block = [0x54, b'a', b'b', b'c']; // 0x54 = 84
        let result = try_encode_mid_block_transition(&block);
        assert!(result.is_some());

        // Just over 1-char max (85) - should fail for 1-char encoding
        let block = [0x55, b'a', b'b', b'c']; // 0x55 = 85
        let result = try_encode_mid_block_transition(&block);
        // This should fail because 85 needs 2 chars: 2 + 1 + 3 = 6 > 5
        assert!(result.is_none());
    }

    // ===== New Scheme Tests: _, ~, and Look-back | =====

    #[test]
    fn test_flush_raw_buffer_4_bytes_uses_underscore() {
        // 4 bytes uses _xxxx encoding
        let mut output = Vec::new();
        let mut raw_buffer = b"test".to_vec();

        flush_raw_buffer(&mut output, &mut raw_buffer);

        assert_eq!(&output[..1], b"_");
        assert_eq!(&output[1..], b"test");
        assert!(raw_buffer.is_empty());
    }

    #[test]
    fn test_flush_raw_buffer_6_bytes_uses_tilde() {
        // 6 bytes uses ~xxxxxx encoding
        let mut output = Vec::new();
        let mut raw_buffer = b"hello!".to_vec();

        flush_raw_buffer(&mut output, &mut raw_buffer);

        assert_eq!(&output[..1], b"~");
        assert_eq!(&output[1..], b"hello!");
        assert!(raw_buffer.is_empty());
    }

    #[test]
    fn test_flush_raw_buffer_8_bytes_chunked() {
        // 8 bytes uses chunked encoding with _ (two 4-byte chunks)
        let mut output = Vec::new();
        let mut raw_buffer = b"testtest".to_vec();

        flush_raw_buffer(&mut output, &mut raw_buffer);

        // Should be _test_test (10 chars)
        assert_eq!(&output, b"_test_test");
        assert!(raw_buffer.is_empty());
    }

    #[test]
    fn test_flush_raw_buffer_3_bytes_fallback() {
        // 3 bytes should fall back to Z85 encoding
        let mut output = Vec::new();
        let mut raw_buffer = b"abc".to_vec();

        flush_raw_buffer(&mut output, &mut raw_buffer);

        // Should be Z85 encoded (4 digits for 3 bytes)
        assert_eq!(output.len(), 4);
        assert!(raw_buffer.is_empty());
    }

    #[test]
    fn test_flush_raw_buffer_5_bytes_fallback() {
        // 5 bytes should fall back to Z85 encoding
        let mut output = Vec::new();
        let mut raw_buffer = b"hello".to_vec();

        flush_raw_buffer(&mut output, &mut raw_buffer);

        // Should be Z85 encoded (7 digits for 5 bytes: 4 bytes + 1 byte)
        assert_eq!(output.len(), 7);
        assert!(raw_buffer.is_empty());
    }

    #[test]
    fn test_encode_jeb85_4_bytes_ascii() {
        // 4 ASCII bytes should use _xxxx
        let input = b"test".to_vec();
        let encoded = encode_jeb85(&input);

        assert_eq!(&encoded, b"_test");
    }

    #[test]
    fn test_encode_jeb85_6_bytes_ascii() {
        // 6 ASCII bytes should use ~xxxxxx
        let input = b"hello!".to_vec();
        let encoded = encode_jeb85(&input);

        assert_eq!(&encoded, b"~hello!");
    }

    #[test]
    fn test_encode_jeb85_8_bytes_ascii() {
        // 8 ASCII bytes uses chunked encoding with two _ prefixes
        let input = b"testtest".to_vec();
        let encoded = encode_jeb85(&input);

        // Should be _test_test (10 chars)
        assert_eq!(&encoded, b"_test_test");
    }

    #[test]
    fn test_encode_jeb85_no_special_prefix_for_short() {
        // 3 bytes of ASCII should use Z85 encoding
        let input = b"abc".to_vec();
        let encoded = encode_jeb85(&input);

        // Should NOT start with _, ~, or contain |
        assert_ne!(encoded[0], b'_');
        assert_ne!(encoded[0], b'~');
        assert!(!encoded.contains(&b'|'));
        assert_eq!(encoded.len(), 4); // Z85: 4 chars for 3 bytes
    }

    #[test]
    fn test_encode_jeb85_mixed_with_underscore() {
        // Binary followed by 4 ASCII bytes
        let input = vec![0xFF, 0xFF, 0xFF, 0xFF, b't', b'e', b's', b't'];
        let encoded = encode_jeb85(&input);

        // First 4 bytes are binary (5 Z85 chars), then 4 ASCII bytes (_test)
        assert!(encoded.len() == 10); // 5 + 5 (_test)
        assert_eq!(&encoded[5..], b"_test");
    }

    // ===== Decoder Tests =====

    #[test]
    fn test_decode_rejects_standalone_pipe() {
        // Standalone | should be rejected
        let encoded = b"|test";
        let result = decode_jeb85(encoded);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_underscore_4_bytes() {
        // _test should decode to "test"
        let encoded = b"_test";
        let decoded = decode_jeb85(encoded).unwrap();
        assert_eq!(decoded, b"test");
    }

    #[test]
    fn test_decode_tilde_6_bytes() {
        // ~hello! should decode to "hello!"
        let encoded = b"~hello!";
        let decoded = decode_jeb85(encoded).unwrap();
        assert_eq!(decoded, b"hello!");
    }

    #[test]
    fn test_decode_lookback_round_trip() {
        // Test look-back encoding with a very long ASCII sequence
        // where look-back becomes beneficial over chunked encoding
        let data = vec![b'x'; 100];

        let encoded = encode_jeb85(&data);
        let decoded = decode_jeb85(&encoded).unwrap();
        assert_eq!(decoded, data);

        // Verify that for such a long sequence, look-back is used
        assert!(encoded.contains(&b'|'));
    }

    #[test]
    fn test_decode_mid_block_transition() {
        // 5|abc should decode to [0x05, 'a', 'b', 'c']
        let encoded = b"5|abc";
        let decoded = decode_jeb85(encoded).unwrap();
        assert_eq!(decoded, vec![0x05, b'a', b'b', b'c']);
    }

    #[test]
    fn test_round_trip_underscore_4_bytes() {
        let input = b"test".to_vec();
        let encoded = encode_jeb85(&input);
        assert_eq!(&encoded, b"_test");
        let decoded = decode_jeb85(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_round_trip_tilde_6_bytes() {
        let input = b"hello!".to_vec();
        let encoded = encode_jeb85(&input);
        assert_eq!(&encoded, b"~hello!");
        let decoded = decode_jeb85(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_round_trip_lookback_8_bytes() {
        let input = b"testtest".to_vec();
        let encoded = encode_jeb85(&input);
        let decoded = decode_jeb85(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_round_trip_lookback_long() {
        // Longer ASCII sequence to test look-back encoding
        let input = b"hello world from jeb85".to_vec();
        let encoded = encode_jeb85(&input);
        let decoded = decode_jeb85(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_round_trip_mid_block() {
        let input = vec![0x05, b'a', b'b', b'c'];
        let encoded = encode_jeb85(&input);
        let decoded = decode_jeb85(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_round_trip_mixed_content() {
        // Binary + 4 ASCII bytes (should use _ encoding)
        let input = vec![0xFF, 0xFF, 0xFF, 0xFF, b't', b'e', b's', b't'];
        let encoded = encode_jeb85(&input);
        let decoded = decode_jeb85(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_round_trip_mixed_content_6_bytes() {
        // Binary + 6 ASCII bytes (should use ~ encoding)
        let input = vec![0xFF, 0xFF, 0xFF, 0xFF, b'h', b'e', b'l', b'l', b'o', b'!'];
        let encoded = encode_jeb85(&input);
        let decoded = decode_jeb85(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_round_trip_binary_only() {
        // Pure binary should use standard Z85
        let input = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
        let encoded = encode_jeb85(&input);
        let decoded = decode_jeb85(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_round_trip_long_ascii() {
        // Long ASCII sequence
        let input = b"The quick brown fox jumps over the lazy dog".to_vec();
        let encoded = encode_jeb85(&input);
        let decoded = decode_jeb85(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_decode_error_invalid_character() {
        // Character not in Z85 alphabet and not valid prefix
        let encoded = b"hello\x00world";
        let result = decode_jeb85(encoded);
        assert!(result.is_err());
    }
}
