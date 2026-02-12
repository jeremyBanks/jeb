//! Extended Z85 encoding with raw ASCII passthrough support.
//!
//! Simple approach: For raw sections, use a fixed format:
//! ESCAPE + (2-byte big-endian length) + raw data
//! This is simple to parse and allows arbitrary length raw sections.

use crate::error::{Error, Result};
use crate::z85::{self, is_z85_char, encode_u32};

// Escape character (Tier 1 - completely free per §5)
const ESCAPE: u8 = b'_';

// Minimum raw section length (matching Z85 block size)
const MIN_RAW_LEN: usize = 4;

/// Check if a byte is eligible for raw passthrough (default policy - R3).
/// Printable ASCII (0x20-0x7E) including space.
fn is_raw_eligible(byte: u8) -> bool {
    (byte >= 0x20 && byte <= 0x7E) && byte != ESCAPE
}

/// Find the longest run of raw-eligible bytes starting at position.
fn find_raw_run(data: &[u8], start: usize) -> usize {
    let mut len = 0;
    for i in 0..(data.len().saturating_sub(start)) {
        if is_raw_eligible(data[start + i]) {
            len += 1;
        } else {
            break;
        }
    }
    len
}

/// Encode data using extended Z85 with opportunistic raw sections.
pub fn encode(input: &[u8]) -> Vec<u8> {
    if input.is_empty() {
        return Vec::new();
    }

    let mut output = Vec::new();
    let mut input_pos = 0;

    while input_pos < input.len() {
        // Check if we can start a raw section here
        let raw_run = find_raw_run(input, input_pos);

        if raw_run >= MIN_RAW_LEN {
            // Encode as raw section
            encode_raw_section(&input[input_pos..input_pos + raw_run], &mut output);
            input_pos += raw_run;
        } else {
            // Encode as Z85
            if input_pos + 4 <= input.len() {
                let block = &input[input_pos..input_pos + 4];
                let mut block_bytes = [0u8; 4];
                block_bytes.copy_from_slice(block);
                let value = u32::from_be_bytes(block_bytes);
                encode_u32(value, &mut output);
                input_pos += 4;
            } else {
                // Partial block at end
                let remaining = input.len() - input_pos;
                let block = &input[input_pos..];
                encode_partial_block_leading(block, &mut output);
                input_pos += remaining;
            }
        }
    }

    output
}

/// Encode a partial block with K bytes, emitting K+1 leading Z85 characters.
fn encode_partial_block_leading(bytes: &[u8], output: &mut Vec<u8>) {
    debug_assert!(bytes.len() > 0 && bytes.len() < 4);
    let mut block = [0u8; 4];
    block[..bytes.len()].copy_from_slice(bytes);
    let value = u32::from_be_bytes(block);

    let digit4 = (value % 85) as u8;
    let digit3 = ((value / 85) % 85) as u8;
    let digit2 = ((value / (85 * 85)) % 85) as u8;
    let digit1 = ((value / (85 * 85 * 85)) % 85) as u8;
    let digit0 = (value / (85 * 85 * 85 * 85)) as u8;

    output.push(z85::Z85_ALPHABET[digit0 as usize]);
    if bytes.len() >= 1 {
        output.push(z85::Z85_ALPHABET[digit1 as usize]);
    }
    if bytes.len() >= 2 {
        output.push(z85::Z85_ALPHABET[digit2 as usize]);
    }
    if bytes.len() >= 3 {
        output.push(z85::Z85_ALPHABET[digit3 as usize]);
    }
}

/// Encode a raw section with simple length encoding.
fn encode_raw_section(raw_data: &[u8], output: &mut Vec<u8>) {
    debug_assert!(raw_data.len() >= MIN_RAW_LEN);

    output.push(ESCAPE);

    // Length encoded as 2-byte big-endian u16
    let len = raw_data.len() as u16;
    output.push((len >> 8) as u8);
    output.push((len & 0xFF) as u8);

    // Raw data passes through as-is
    output.extend_from_slice(raw_data);
}

/// Decode extended Z85 with raw sections.
pub fn decode(input: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut input_pos = 0;

    while input_pos < input.len() {
        let c = input[input_pos];

        if c == ESCAPE {
            // Raw section
            input_pos += 1;

            if input_pos + 2 > input.len() {
                return Err(Error::InvalidEscapeSequence {
                    position: input_pos - 1,
                    reason: "escape requires 2-byte length".to_string(),
                });
            }

            // Decode 2-byte big-endian length
            let len_hi = input[input_pos] as u16;
            let len_lo = input[input_pos + 1] as u16;
            let len = (len_hi << 8) | len_lo;
            input_pos += 2;

            if input_pos + len as usize > input.len() {
                return Err(Error::TruncatedRawSection {
                    expected: len as usize,
                    got: input.len().saturating_sub(input_pos),
                });
            }

            // Raw data passes through
            output.extend_from_slice(&input[input_pos..input_pos + len as usize]);
            input_pos += len as usize;
        } else if is_z85_char(c) {
            // Z85 character
            if input_pos + 5 <= input.len() {
                // Try complete block
                let chunk = &input[input_pos..input_pos + 5];
                let bytes = decode_z85_block(chunk)?;
                output.extend_from_slice(&bytes);
                input_pos += 5;
            } else {
                // Partial block at end
                let remaining = input.len() - input_pos;
                let chunk = &input[input_pos..];
                let bytes = decode_partial_block(chunk, remaining)?;
                output.extend_from_slice(&bytes);
                input_pos += remaining;
            }
        } else {
            return Err(Error::InvalidCharacter {
                character: c as char,
                position: input_pos,
            });
        }
    }

    Ok(output)
}

/// Decode a partial Z85 block.
fn decode_partial_block(chars: &[u8], len: usize) -> Result<Vec<u8>> {
    debug_assert!(len > 0 && len < 5);

    let mut digits = [0u8; 5];
    for (i, &c) in chars.iter().enumerate() {
        let digit = z85::Z85_DECODE_TABLE[c as usize];
        if digit == 255 {
            return Err(Error::InvalidZ85Character {
                character: c as char,
                position: i,
            });
        }
        digits[i] = digit;
    }

    // For a partial block of K characters, we decode K base-85 digits.
    // These encode K bytes in the most-significant positions.
    let mut value = 0u32;
    for i in 0..len {
        value = value * 85 + digits[i] as u32;
    }

    // Shift left to align to high-order positions
    let shift = (5 - len) * 8;
    value = value.wrapping_shl(shift as u32);

    let full_bytes = value.to_be_bytes();
    Ok(full_bytes[..(len - 1).min(3)].to_vec())
}

/// Decode a complete Z85 block.
fn decode_z85_block(chars: &[u8]) -> Result<Vec<u8>> {
    debug_assert_eq!(chars.len(), 5);

    let mut digits = [0u8; 5];
    for (i, &c) in chars.iter().enumerate() {
        let digit = z85::Z85_DECODE_TABLE[c as usize];
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

    Ok(value.to_be_bytes().to_vec())
}
