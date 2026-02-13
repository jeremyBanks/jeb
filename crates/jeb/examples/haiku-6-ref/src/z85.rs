//! Standard Z85 encoding/decoding (baseline, no raw sections).
//!
//! This implements the RFC 32 Z85 encoding with support for arbitrary input lengths
//! (not just multiples of 4 bytes).

use crate::error::{Error, Result};

pub const Z85_ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
pub const Z85_DECODE_TABLE: [u8; 256] = generate_decode_table();

const fn generate_decode_table() -> [u8; 256] {
    let mut table = [255u8; 256];
    let alphabet = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
    let mut i = 0;
    while i < alphabet.len() {
        table[alphabet[i] as usize] = i as u8;
        i += 1;
    }
    table
}

pub fn is_z85_char(c: u8) -> bool {
    Z85_DECODE_TABLE[c as usize] != 255
}

pub fn encode_z85(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut input_pos = 0;

    // Process complete 4-byte blocks
    while input_pos + 4 <= input.len() {
        let block = &input[input_pos..input_pos + 4];
        encode_block(block, &mut output);
        input_pos += 4;
    }

    // Handle partial final block
    let remaining = input.len() - input_pos;
    if remaining > 0 {
        let mut block = [0u8; 4];
        block[..remaining].copy_from_slice(&input[input_pos..]);
        encode_partial_block(&block, remaining, &mut output);
    }

    output
}

fn encode_block(bytes: &[u8], output: &mut Vec<u8>) {
    debug_assert_eq!(bytes.len(), 4);
    let value = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    encode_u32(value, output);
}

pub fn encode_u32(value: u32, output: &mut Vec<u8>) {
    // Extract 5 base-85 digits (most significant first)
    let digit4 = (value % 85) as u8;
    let digit3 = ((value / 85) % 85) as u8;
    let digit2 = ((value / (85 * 85)) % 85) as u8;
    let digit1 = ((value / (85 * 85 * 85)) % 85) as u8;
    let digit0 = (value / (85 * 85 * 85 * 85)) as u8;

    output.push(Z85_ALPHABET[digit0 as usize]);
    output.push(Z85_ALPHABET[digit1 as usize]);
    output.push(Z85_ALPHABET[digit2 as usize]);
    output.push(Z85_ALPHABET[digit3 as usize]);
    output.push(Z85_ALPHABET[digit4 as usize]);
}

/// Encode a partial block with K bytes, emitting K+1 leading Z85 characters.
///
/// This interprets the K bytes as the high-order bytes of a 4-byte value
/// (padding with zeros on the right) and emits the leading K+1 Z85 characters
/// that would encode them.
pub fn encode_partial_block_leading(bytes: &[u8], output: &mut Vec<u8>) {
    debug_assert!(bytes.len() > 0 && bytes.len() < 4);
    let mut block = [0u8; 4];
    block[..bytes.len()].copy_from_slice(bytes);
    let value = u32::from_be_bytes(block);

    let _digit4 = (value % 85) as u8;
    let digit3 = ((value / 85) % 85) as u8;
    let digit2 = ((value / (85 * 85)) % 85) as u8;
    let digit1 = ((value / (85 * 85 * 85)) % 85) as u8;
    let digit0 = (value / (85 * 85 * 85 * 85)) as u8;

    output.push(Z85_ALPHABET[digit0 as usize]);
    if bytes.len() >= 1 {
        output.push(Z85_ALPHABET[digit1 as usize]);
    }
    if bytes.len() >= 2 {
        output.push(Z85_ALPHABET[digit2 as usize]);
    }
    if bytes.len() >= 3 {
        output.push(Z85_ALPHABET[digit3 as usize]);
    }
}

/// Encode trailing bytes (used for exit boundaries).
/// The bytes are the low-order bytes of a 4-byte block, with prefix bytes known.
/// Given the known prefix bytes, we can compute the trailing Z85 characters.
pub fn encode_trailing_bytes(prefix_bytes: &[u8], suffix_bytes: &[u8], output: &mut Vec<u8>) {
    debug_assert!(prefix_bytes.len() + suffix_bytes.len() == 4);
    debug_assert!(suffix_bytes.len() > 0 && suffix_bytes.len() < 4);

    let mut block = [0u8; 4];
    block[..prefix_bytes.len()].copy_from_slice(prefix_bytes);
    block[prefix_bytes.len()..].copy_from_slice(suffix_bytes);
    let value = u32::from_be_bytes(block);

    let digit4 = (value % 85) as u8;
    let digit3 = ((value / 85) % 85) as u8;
    let digit2 = ((value / (85 * 85)) % 85) as u8;
    let _digit1 = ((value / (85 * 85 * 85)) % 85) as u8;
    let _digit0 = (value / (85 * 85 * 85 * 85)) as u8;

    let suffix_len = suffix_bytes.len();
    let _prefix_len = prefix_bytes.len();

    // Emit trailing characters. For a suffix of length K, we emit K trailing chars
    // counting from the right (char 4, chars 3-4, chars 2-4 for K=1,2,3 respectively)
    if suffix_len == 1 {
        output.push(Z85_ALPHABET[digit4 as usize]);
    } else if suffix_len == 2 {
        output.push(Z85_ALPHABET[digit3 as usize]);
        output.push(Z85_ALPHABET[digit4 as usize]);
    } else if suffix_len == 3 {
        output.push(Z85_ALPHABET[digit2 as usize]);
        output.push(Z85_ALPHABET[digit3 as usize]);
        output.push(Z85_ALPHABET[digit4 as usize]);
    }
}

fn encode_partial_block(bytes: &[u8], len: usize, output: &mut Vec<u8>) {
    debug_assert!(len > 0 && len < 4);
    debug_assert_eq!(bytes.len(), 4);
    encode_partial_block_leading(&bytes[..len], output);
}

pub fn decode_z85(input: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut input_pos = 0;

    while input_pos < input.len() {
        if input_pos + 5 <= input.len() {
            // Try to decode a complete block
            let chunk = &input[input_pos..input_pos + 5];
            let bytes = decode_block(chunk)?;
            output.extend_from_slice(&bytes);
            input_pos += 5;
        } else {
            // Partial block at the end
            let remaining = input.len() - input_pos;
            let chunk = &input[input_pos..];
            let bytes = decode_partial_block(chunk, remaining)?;
            output.extend_from_slice(&bytes);
            input_pos += remaining;
        }
    }

    Ok(output)
}

fn decode_block(chars: &[u8]) -> Result<[u8; 4]> {
    debug_assert_eq!(chars.len(), 5);

    let mut digits = [0u8; 5];
    for (i, &c) in chars.iter().enumerate() {
        let digit = Z85_DECODE_TABLE[c as usize];
        if digit == 255 {
            return Err(Error::InvalidZ85Character {
                character: c as char,
                position: i,
            });
        }
        digits[i] = digit;
    }

    let value = digits[0] as u32 * 85u32.pow(4)
        + digits[1] as u32 * 85u32.pow(3)
        + digits[2] as u32 * 85u32.pow(2)
        + digits[3] as u32 * 85
        + digits[4] as u32;

    Ok(value.to_be_bytes())
}

pub fn decode_partial_block(chars: &[u8], len: usize) -> Result<Vec<u8>> {
    debug_assert!(len > 0 && len < 5);

    let mut target_digits = [0u8; 5];
    for (i, &c) in chars.iter().enumerate() {
        let digit = Z85_DECODE_TABLE[c as usize];
        if digit == 255 {
            return Err(Error::InvalidZ85Character {
                character: c as char,
                position: i,
            });
        }
        target_digits[i] = digit;
    }

    // For K characters, we need to find the K unknown bytes such that
    // encoding [known_bytes || unknown_bytes] produces the K target digits.
    // Brute force: try all possible values for unknown bytes.
    let num_known = len - 1;  // K characters encode K bytes, but the last byte is ambiguous
    let num_unknown = 4 - num_known;

    // Try all combinations of unknown bytes
    match num_unknown {
        3 => {
            // 1 known byte, 3 unknown bytes (padded with zeros)
            // K=2 characters (target_digits[0] and target_digits[1])
            // The encoder pads with zeros, so we should find b1=0, b2=0 first
            for b0 in 0..=255u8 {
                let block = [b0, 0u8, 0u8, 0u8];
                let value = u32::from_be_bytes(block);
                let d0 = (value / (85u32.pow(4))) as u8;
                let d1 = ((value / (85u32.pow(3))) % 85) as u8;
                if d0 == target_digits[0] && d1 == target_digits[1] {
                    return Ok(vec![b0]);
                }
            }
            Err(Error::DecodingError {
                message: "could not decode 1-byte partial block".to_string(),
            })
        }
        2 => {
            // 2 known bytes, 2 unknown bytes
            // K=3 characters
            for b0 in 0..=255u8 {
                for b1 in 0..=255u8 {
                    let block = [b0, b1, 0u8, 0u8];
                    let value = u32::from_be_bytes(block);
                    let d0 = (value / (85u32.pow(4))) as u8;
                    let d1 = ((value / (85u32.pow(3))) % 85) as u8;
                    let d2 = ((value / (85u32.pow(2))) % 85) as u8;
                    if d0 == target_digits[0] && d1 == target_digits[1] && d2 == target_digits[2] {
                        return Ok(vec![b0, b1]);
                    }
                }
            }
            Err(Error::DecodingError {
                message: "could not decode 2-byte partial block".to_string(),
            })
        }
        1 => {
            // 3 known bytes, 1 unknown byte
            // K=4 characters
            for b0 in 0..=255u8 {
                for b1 in 0..=255u8 {
                    for b2 in 0..=255u8 {
                        let block = [b0, b1, b2, 0u8];
                        let value = u32::from_be_bytes(block);
                        let d0 = (value / (85u32.pow(4))) as u8;
                        let d1 = ((value / (85u32.pow(3))) % 85) as u8;
                        let d2 = ((value / (85u32.pow(2))) % 85) as u8;
                        let d3 = ((value / 85) % 85) as u8;
                        if d0 == target_digits[0] && d1 == target_digits[1] && d2 == target_digits[2] && d3 == target_digits[3] {
                            return Ok(vec![b0, b1, b2]);
                        }
                    }
                }
            }
            Err(Error::DecodingError {
                message: "could not decode 3-byte partial block".to_string(),
            })
        }
        _ => Err(Error::DecodingError {
            message: "invalid partial block length".to_string(),
        })
    }
}

/// Decode trailing Z85 characters given the prefix bytes.
/// This is used at exit boundaries where the prefix bytes are known (from raw section).
pub fn decode_trailing_chars(
    prefix_bytes: &[u8],
    trailing_chars: &[u8],
) -> Result<Vec<u8>> {
    debug_assert!(prefix_bytes.len() + trailing_chars.len() == 4);
    debug_assert!(trailing_chars.len() > 0 && trailing_chars.len() < 4);

    let mut digits = [0u8; 5];
    for (i, &c) in trailing_chars.iter().enumerate() {
        let digit = Z85_DECODE_TABLE[c as usize];
        if digit == 255 {
            return Err(Error::InvalidZ85Character {
                character: c as char,
                position: i,
            });
        }
        // Trailing chars are at the end positions
        let pos = 5 - trailing_chars.len() + i;
        digits[pos] = digit;
    }

    // The trailing digit is a modular sum: (b0 + b1 + b2 + b3) mod 85
    // We know b0, b1, b2 (the prefix) and the digit value.
    // We need to find b3 (or b2, b3 or b1, b2, b3 depending on suffix length).

    let prefix_sum: u32 = prefix_bytes.iter().map(|&b| b as u32).sum();
    let _digit_sum = digits.iter().take(trailing_chars.len()).map(|&d| d as u32).sum::<u32>();

    // Reconstruct by solving the modular arithmetic
    let mut suffix_bytes = Vec::new();
    for _i in 0..trailing_chars.len() {
        let _byte_index = prefix_bytes.len() + _i;

        // For each byte, we need b_i such that when combined with others,
        // the trailing digit equation holds
        // This requires solving: value = prefix_be_bytes_as_u32 | (suffix_bytes_as_u32)
        // And extracting the right trailing character

        // For simplicity with the constraint that we know all prefix bytes,
        // we can extract bytes from high to low in the suffix
        if trailing_chars.len() == 1 {
            // Single trailing char: digit4 = V mod 85
            // V = (b0*2^24 + b1*2^16 + b2*2^8 + b3)
            // By the modular property: V mod 85 = (b0 + b1 + b2 + b3) mod 85
            let target_sum = digits[4] as u32;
            let b3 = (target_sum as i32 - prefix_sum as i32).rem_euclid(85) as u8;
            suffix_bytes.push(b3);
        } else if trailing_chars.len() == 2 {
            // Two trailing chars: digit3, digit4
            // We need to reconstruct b2 and b3
            // This is more complex - we need to solve the full u32 equation
            let mut reconstructed = [0u8; 4];
            reconstructed[..prefix_bytes.len()].copy_from_slice(prefix_bytes);

            // Try all possible 2-byte combinations
            for b2 in 0..=255u8 {
                for b3 in 0..=255u8 {
                    reconstructed[2] = b2;
                    reconstructed[3] = b3;
                    let value = u32::from_be_bytes(reconstructed);
                    let d3 = ((value / 85) % 85) as u8;
                    let d4 = (value % 85) as u8;
                    if d3 == digits[3] && d4 == digits[4] {
                        suffix_bytes = vec![b2, b3];
                        return Ok(suffix_bytes);
                    }
                }
            }
            return Err(Error::DecodingError {
                message: "could not reconstruct trailing bytes".to_string(),
            });
        } else if trailing_chars.len() == 3 {
            // Three trailing chars: digit2, digit3, digit4
            let mut reconstructed = [0u8; 4];
            reconstructed[0] = prefix_bytes[0];

            // Try all possible 3-byte combinations
            for b1 in 0..=255u8 {
                for b2 in 0..=255u8 {
                    for b3 in 0..=255u8 {
                        reconstructed[1] = b1;
                        reconstructed[2] = b2;
                        reconstructed[3] = b3;
                        let value = u32::from_be_bytes(reconstructed);
                        let d2 = ((value / (85 * 85)) % 85) as u8;
                        let d3 = ((value / 85) % 85) as u8;
                        let d4 = (value % 85) as u8;
                        if d2 == digits[2] && d3 == digits[3] && d4 == digits[4] {
                            suffix_bytes = vec![b1, b2, b3];
                            return Ok(suffix_bytes);
                        }
                    }
                }
            }
            return Err(Error::DecodingError {
                message: "could not reconstruct trailing bytes".to_string(),
            });
        }
    }

    Ok(suffix_bytes)
}
