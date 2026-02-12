//! Extended Z85 encoding with raw ASCII passthrough support.
//!
//! Simple approach: For raw sections, use a fixed format:
//! ESCAPE + (2-byte big-endian length) + raw data
//! This is simple to parse and allows arbitrary length raw sections.

use crate::error::{Error, Result};
use crate::z85::{self, is_z85_char, encode_u32, decode_partial_block};

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

    let _digit4 = (value % 85) as u8;
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

/// Encode a raw section with Z85-encoded length.
fn encode_raw_section(raw_data: &[u8], output: &mut Vec<u8>) {
    debug_assert!(raw_data.len() >= MIN_RAW_LEN);

    output.push(ESCAPE);

    // Length encoded in Z85 alphabet (3 characters encode up to 614,125)
    let len = raw_data.len() as u32;
    encode_length_z85(len, output);

    // Raw data passes through as-is
    output.extend_from_slice(raw_data);
}

/// Encode a length value using Z85 alphabet.
fn encode_length_z85(len: u32, output: &mut Vec<u8>) {
    // Use 3 Z85 characters to encode the length
    let d0 = (len / (85 * 85)) as u8;
    let d1 = ((len / 85) % 85) as u8;
    let d2 = (len % 85) as u8;
    output.push(z85::Z85_ALPHABET[d0 as usize]);
    output.push(z85::Z85_ALPHABET[d1 as usize]);
    output.push(z85::Z85_ALPHABET[d2 as usize]);
}

/// Decode a Z85-encoded length value.
fn decode_length_z85(chars: &[u8]) -> Result<u32> {
    if chars.len() < 3 {
        return Err(Error::InvalidEscapeSequence {
            position: 0,
            reason: "length encoding requires 3 Z85 characters".to_string(),
        });
    }

    let d0 = z85::Z85_DECODE_TABLE[chars[0] as usize];
    let d1 = z85::Z85_DECODE_TABLE[chars[1] as usize];
    let d2 = z85::Z85_DECODE_TABLE[chars[2] as usize];

    if d0 == 255 || d1 == 255 || d2 == 255 {
        return Err(Error::InvalidEscapeSequence {
            position: 0,
            reason: "invalid Z85 character in length".to_string(),
        });
    }

    let len = (d0 as u32) * 85 * 85 + (d1 as u32) * 85 + (d2 as u32);
    Ok(len)
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

            if input_pos + 3 > input.len() {
                return Err(Error::InvalidEscapeSequence {
                    position: input_pos - 1,
                    reason: "escape requires 3-character Z85 length".to_string(),
                });
            }

            // Decode 3-character Z85-encoded length
            let len = decode_length_z85(&input[input_pos..input_pos + 3])? as usize;
            input_pos += 3;

            if input_pos + len > input.len() {
                return Err(Error::TruncatedRawSection {
                    expected: len,
                    got: input.len().saturating_sub(input_pos),
                });
            }

            // Raw data passes through
            output.extend_from_slice(&input[input_pos..input_pos + len]);
            input_pos += len;
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
