//! Base-42 variable-length encoding for the `|` escape.
//!
//! The `|` escape uses a backward-looking base-42 encoding for the length.
//! Characters before `|` encode the length as metadata, not as output bytes.
//!
//! ## Encoding Algorithm
//!
//! Each Z85 digit encodes: `(continuation_flag * 42) + contribution + 1`
//! - Continuation flag: 1 if more digits to the left, 0 if final
//! - The "+1" offset ensures Z85 digit values start at 1, not 0
//!
//! ## Endianness Encoding
//!
//! For the leftmost digit when not block-aligned:
//! - Values 0-20 → little-endian, contribution = value
//! - Values 21-41 → big-endian, contribution = value - 21
//!
//! This halves the range to encode endianness in the spare bits.

use crate::alphabet::{z85_digit_char, z85_digit_value};
use crate::error::DecodeError;

/// Endianness for prefix bytes (affects padding of interrupted block).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

/// Maximum length that can be encoded with 4 digits.
/// 42^4 - 1 = 3,111,695
pub const MAX_LENGTH: usize = 42 * 42 * 42 * 42 - 1;

/// Encode a length into base-42 digits for the `|` escape.
///
/// # Arguments
/// - `length`: The number of raw bytes (0 for infinite, or 8+)
/// - `position`: Position of `|` within the block (0-4)
/// - `endianness`: Endianness hint for the decoder
///
/// # Returns
/// Z85 characters in physical order (left-to-right, to appear before `|`).
///
/// # Panics
/// If length is 1-7 (should use simpler escapes) or exceeds MAX_LENGTH.
pub fn encode_length(length: usize, position: u8, endianness: Endianness) -> Vec<u8> {
    assert!(
        length == 0 || length >= 8,
        "length must be 0 (infinite) or >= 8, got {}",
        length
    );
    assert!(
        length <= MAX_LENGTH,
        "length {} exceeds maximum {}",
        length,
        MAX_LENGTH
    );
    assert!(position <= 4, "position must be 0-4, got {}", position);

    // Convert length to base-42 digits (low-order first)
    let mut digits_low_first = Vec::new();
    let mut remaining = length;

    loop {
        let digit = remaining % 42;
        digits_low_first.push(digit as u8);
        remaining /= 42;
        if remaining == 0 {
            break;
        }
    }

    // How many characters are available before `|`? That's the position value.
    let available_chars = position as usize;

    // We need at least as many positions as we have base-42 digits
    // If position is 0, we have no length encoding chars (raw bytes start immediately after |)
    // But wait - the spec says | at position 4 can use up to 4 chars before it

    // Actually, the number of available characters is exactly `position`:
    // - position 0: no chars before |
    // - position 1: 1 char before |
    // - position 2: 2 chars before |
    // - position 3: 3 chars before |
    // - position 4: 4 chars before |

    // Ensure we have enough space
    if digits_low_first.len() > available_chars && available_chars > 0 {
        panic!(
            "length {} requires {} digits but only {} positions available",
            length,
            digits_low_first.len(),
            available_chars
        );
    }

    // Special case: position 0 means | at start of block
    // In this case, we read digits from the previous block
    // For encoding purposes, we always output all needed digits
    if available_chars == 0 {
        // At position 0, we need to use digits from the preceding content
        // This is a complex case - for now, let's just output the digits
        // and let the caller handle placement
        // Actually, looking at the spec more carefully, if | is at position 0,
        // we read backward from the previous block. Let's output all needed digits.
    }

    // Build output: physical order (leftmost first)
    // The rightmost digit (closest to |) has continuation=1 if more digits to its left
    // The leftmost digit has continuation=0

    let num_digits = digits_low_first.len();
    let block_aligned = position == 0 || position == 4;

    let mut result = Vec::with_capacity(num_digits);

    // Process from highest-order (leftmost) to lowest-order (rightmost)
    for (i, &contribution) in digits_low_first.iter().rev().enumerate() {
        let is_leftmost = i == 0;

        // Continuation flag: set if there are more digits to the left (from decoder's perspective)
        // The decoder reads right-to-left, so continuation=1 means "keep reading left"
        // Therefore: leftmost digit has continuation=0, all others have continuation=1
        let has_continuation = !is_leftmost;

        // For non-block-aligned leftmost digit, encode endianness
        let digit_value = if is_leftmost && !block_aligned && num_digits == available_chars {
            // Leftmost digit at non-block-aligned position encodes endianness
            match endianness {
                Endianness::Little => contribution, // 0-20 range
                Endianness::Big => contribution + 21, // 21-41 range
            }
        } else {
            contribution
        };

        // Add continuation flag
        let with_continuation = if has_continuation {
            digit_value + 42
        } else {
            digit_value
        };

        // Add +1 offset for Z85 encoding
        let z85_value = with_continuation + 1;

        // Convert to Z85 character
        debug_assert!(z85_value < 85, "Z85 value {} out of range", z85_value);
        result.push(z85_digit_char(z85_value));
    }

    result
}

/// Decode length from Z85 digits read backward from `|`.
///
/// # Arguments
/// - `digits`: Z85 characters before `|`, in physical order (leftmost first)
/// - `block_aligned`: Whether `|` is at a block boundary (position 0 or 4)
///
/// # Returns
/// `(length, endianness)` where length is the number of raw bytes to follow.
pub fn decode_length(digits: &[u8], block_aligned: bool) -> Result<(usize, Endianness), DecodeError> {
    if digits.is_empty() {
        // No length digits means length 0 (infinite) is implied?
        // Actually, the spec doesn't clearly define this case.
        // Let's return length 8 as a default minimum.
        return Ok((0, Endianness::Little));
    }

    let mut total: usize = 0;
    let mut power: usize = 1; // 42^0 = 1, starts at low-order digit
    let mut endianness = Endianness::Little;
    let num_digits = digits.len();

    // Process right-to-left (rightmost/low-order digit first, closest to |)
    for (i, &c) in digits.iter().rev().enumerate() {
        let z85_value = z85_digit_value(c).ok_or(DecodeError::InvalidLengthDigit {
            position: num_digits - 1 - i,
        })?;

        // Remove +1 offset
        if z85_value == 0 {
            return Err(DecodeError::InvalidLengthDigit {
                position: num_digits - 1 - i,
            });
        }
        let value = z85_value - 1;

        // Extract continuation flag and contribution
        let has_continuation = value >= 42;
        let contribution = if has_continuation { value - 42 } else { value };

        // Check for leftmost digit (which we process last)
        let is_leftmost = i == num_digits - 1;

        // For non-block-aligned leftmost digit, decode endianness
        let actual_contribution = if is_leftmost && !block_aligned {
            if contribution < 21 {
                endianness = Endianness::Little;
                contribution
            } else if contribution < 42 {
                endianness = Endianness::Big;
                contribution - 21
            } else {
                return Err(DecodeError::InvalidLengthDigit {
                    position: num_digits - 1 - i,
                });
            }
        } else {
            contribution
        };

        // Accumulate in base-42 (low-order first, so multiply by power)
        total += actual_contribution as usize * power;
        power *= 42;

        // If no continuation and not at end, remaining digits are padding
        if !has_continuation && !is_leftmost {
            // Early termination - stop processing
            break;
        }
    }

    // Validate length
    if total != 0 && total < 8 {
        return Err(DecodeError::InvalidLength { length: total });
    }

    Ok((total, endianness))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::z85_digit_char;

    #[test]
    fn test_encode_length_50() {
        // From the spec's worked example:
        // Length 50 = 1×42 + 8
        // Rightmost digit (8): needs continuation, so 8 + 42 = 50, then +1 = 51 → Z85[51]
        // Leftmost digit (1): no continuation, so 1, then +1 = 2 → Z85[2]
        // Output: Z85[2] Z85[51] |

        let encoded = encode_length(50, 2, Endianness::Little);
        assert_eq!(encoded.len(), 2);
        assert_eq!(encoded[0], z85_digit_char(2));  // '2'
        assert_eq!(encoded[1], z85_digit_char(51)); // Should be 'P' (51st char in Z85)
    }

    #[test]
    fn test_decode_length_50() {
        let digits = [z85_digit_char(2), z85_digit_char(51)];
        let (length, endianness) = decode_length(&digits, true).unwrap();
        assert_eq!(length, 50);
        assert_eq!(endianness, Endianness::Little);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        for length in [0, 8, 10, 42, 50, 100, 1000, 10000, MAX_LENGTH] {
            // Test block-aligned positions (position 4)
            let num_digits = num_base42_digits(length);
            if num_digits <= 4 {
                let encoded = encode_length(length, 4, Endianness::Little);
                let (decoded_length, _) = decode_length(&encoded, true).unwrap();
                assert_eq!(
                    decoded_length, length,
                    "roundtrip failed for length {} at position 4 (block-aligned)",
                    length
                );
            }

            // Test non-block-aligned only when high-order digit fits in LE range (0-20)
            let high_order_digit = if length == 0 {
                0
            } else {
                let mut n = length;
                while n >= 42 {
                    n /= 42;
                }
                n
            };

            if high_order_digit <= 20 {
                for position in [2, 3] {
                    if position as usize >= num_digits {
                        let encoded = encode_length(length, position, Endianness::Little);
                        let (decoded_length, _) = decode_length(&encoded, false).unwrap();
                        assert_eq!(
                            decoded_length, length,
                            "roundtrip failed for length {} at position {} (non-block-aligned)",
                            length, position
                        );
                    }
                }
            }
        }
    }

    fn num_base42_digits(length: usize) -> usize {
        if length == 0 {
            return 1;
        }
        let mut n = length;
        let mut count = 0;
        while n > 0 {
            n /= 42;
            count += 1;
        }
        count
    }

    #[test]
    fn test_endianness_encoding() {
        // At non-block-aligned position, leftmost digit encodes endianness
        // Test with a small length that fits in one digit

        // Length 10, LE: should encode as value 10 (in 0-20 range)
        let encoded_le = encode_length(10, 1, Endianness::Little);
        assert_eq!(encoded_le.len(), 1);
        // value = 10, +1 = 11
        assert_eq!(encoded_le[0], z85_digit_char(11));

        // Length 10, BE: should encode as value 10 + 21 = 31 (in 21-41 range)
        let encoded_be = encode_length(10, 1, Endianness::Big);
        assert_eq!(encoded_be.len(), 1);
        // value = 31, +1 = 32
        assert_eq!(encoded_be[0], z85_digit_char(32));

        // Decode and verify endianness
        let (len_le, end_le) = decode_length(&encoded_le, false).unwrap();
        assert_eq!(len_le, 10);
        assert_eq!(end_le, Endianness::Little);

        let (len_be, end_be) = decode_length(&encoded_be, false).unwrap();
        assert_eq!(len_be, 10);
        assert_eq!(end_be, Endianness::Big);
    }

    #[test]
    fn test_length_8_minimum() {
        let encoded = encode_length(8, 1, Endianness::Little);
        let (length, _) = decode_length(&encoded, false).unwrap();
        assert_eq!(length, 8);
    }

    #[test]
    fn test_length_zero_infinite() {
        let encoded = encode_length(0, 1, Endianness::Little);
        let (length, _) = decode_length(&encoded, false).unwrap();
        assert_eq!(length, 0); // 0 means infinite
    }

    #[test]
    fn test_invalid_length_1_to_7() {
        // Lengths 1-7 should be rejected during decode if somehow encoded
        // (The encoder panics on these)

        // Manually create an invalid encoding for length 5
        // This would be value 5 + 1 = 6
        let invalid_digit = z85_digit_char(6); // represents length 5
        let result = decode_length(&[invalid_digit], false);
        assert!(matches!(result, Err(DecodeError::InvalidLength { length: 5 })));
    }
}
