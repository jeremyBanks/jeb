//! Extended Z85 encoding with raw ASCII passthrough support.

use crate::error::{Error, Result};
use crate::z85::{self, is_z85_char, decode_partial_block, encode_partial_block_leading, encode_trailing_bytes, encode_u32};

// Escape characters (Tier 1 - completely free per §5)
const ESCAPE_BE: u8 = b'_';    // Big-endian entry boundary convention
const ESCAPE_LE: u8 = b'~';    // Little-endian entry boundary convention

// Special overhead character value indicating "raw to end of stream" (not currently used)
#[allow(dead_code)]
const RAW_TO_END: u8 = 255;

// Minimum raw section length (matching Z85 block size)
const MIN_RAW_LEN: usize = 4;

/// Check if a byte is eligible for raw passthrough (default policy - R3).
fn is_raw_eligible(byte: u8) -> bool {
    // Printable ASCII (0x20-0x7E)
    (byte >= 0x20 && byte <= 0x7E) ||
    // Also allow escape characters (length-prefixed sections are unambiguous)
    byte == ESCAPE_BE || byte == ESCAPE_LE
}

/// Check if a region contains only raw-eligible bytes.
fn all_raw_eligible(data: &[u8]) -> bool {
    data.iter().all(|&b| is_raw_eligible(b))
}

/// Find the longest run of raw-eligible bytes starting at position, up to max_len.
fn find_raw_run(data: &[u8], start: usize, max_len: usize) -> usize {
    let mut len = 0;
    for i in 0..(max_len.min(data.len() - start)) {
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
        // Try to find a raw section starting at current position
        // First, figure out block alignment relative to current output position
        let z85_output_chars_so_far = count_z85_output_chars(&output);
        let block_offset = z85_output_chars_so_far % 5;
        let remaining_in_block = (5 - block_offset) % 5;

        // Check if we're mid-block
        if remaining_in_block > 0 && remaining_in_block <= 3 && input_pos < input.len() {
            // We're mid-block. Can we start a raw section with an entry boundary cut?
            let byte_offset = remaining_in_block; // How many bytes remain in current block (1-3)

            if byte_offset <= 3 && input_pos + byte_offset <= input.len() {
                let prefix_bytes = &input[input_pos..input_pos + byte_offset];
                // Check stability of the entry boundary
                if is_stable_entry(prefix_bytes) {
                    // Try to encode a raw section starting here
                    let max_raw = input.len() - input_pos - byte_offset;
                    if max_raw >= MIN_RAW_LEN {
                        let raw_len = find_raw_run(&input, input_pos + byte_offset, max_raw);
                        if raw_len > 0 {
                            // Encode the prefix bytes as partial Z85 chars
                            encode_partial_block_leading(prefix_bytes, &mut output);
                            // Now encode the raw section
                            let raw_data = &input[input_pos + byte_offset..input_pos + byte_offset + raw_len];
                            encode_raw_section(raw_data, &mut output, false);
                            input_pos += byte_offset + raw_len;
                            continue;
                        }
                    }
                }
            }
        }

        // Not in the middle of a block, or couldn't use a mid-block entry
        // Try block-aligned raw section
        if input_pos % 4 == 0 && all_raw_eligible(&input[input_pos..]) && input.len() - input_pos >= MIN_RAW_LEN {
            // Entire remaining input is raw-eligible
            let raw_data = &input[input_pos..];
            encode_raw_section(raw_data, &mut output, true);
            input_pos = input.len();
            continue;
        }

        // Look for block-aligned raw sections
        if input_pos % 4 == 0 {
            let raw_len = find_raw_run(&input, input_pos, input.len() - input_pos);
            if raw_len >= MIN_RAW_LEN && raw_len % 4 == 0 {
                // Block-aligned raw section
                let raw_data = &input[input_pos..input_pos + raw_len];
                encode_raw_section(raw_data, &mut output, true);
                input_pos += raw_len;
                continue;
            }

            // Try mid-block exit raw section
            if raw_len >= MIN_RAW_LEN && raw_len > 4 {
                // Extract complete blocks first
                let complete_blocks = raw_len / 4;
                let remainder = raw_len % 4;

                if remainder > 0 && is_stable_exit(&input[input_pos + raw_len - remainder..input_pos + raw_len]) {
                    // Encode complete blocks as raw
                    let blocks_bytes = &input[input_pos..input_pos + complete_blocks * 4];
                    let suffix_bytes = &input[input_pos + complete_blocks * 4..input_pos + raw_len];

                    encode_raw_section_with_exit(blocks_bytes, suffix_bytes, &mut output);
                    input_pos += raw_len;
                    continue;
                }
            }
        }

        // Fall back to Z85 encoding for the next block or partial block
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
            encode_partial_block_leading(&input[input_pos..], &mut output);
            input_pos += remaining;
        }
    }

    output
}

/// Count the number of Z85 characters (non-escape) in the output so far.
fn count_z85_output_chars(output: &[u8]) -> usize {
    output.iter().filter(|&&c| c != ESCAPE_BE && c != ESCAPE_LE).count()
}

/// Check if an entry boundary byte is stable (per §6).
/// A byte is stable if its leading Z85 character doesn't depend on unknown low bytes.
fn is_stable_entry(bytes: &[u8]) -> bool {
    match bytes.len() {
        1 => {
            // For 1-byte entry: 68% of values are stable
            // These are values where [b, 0, 0, 0] and [b, 255, 255, 255] produce the same leading digit
            let b = bytes[0] as u32;
            let low = b << 24;
            let high = ((b + 1) << 24) - 1;
            low / (85_u32.pow(4)) == high / (85_u32.pow(4))
        }
        2 => {
            // For 2-byte entry: 89% of values are stable
            let b01 = u16::from_be_bytes([bytes[0], bytes[1]]) as u32;
            let low = b01 << 16;
            let high = ((b01 + 1) << 16) - 1;
            low / (85_u32.pow(3)) == high / (85_u32.pow(3))
        }
        3 => {
            // For 3-byte entry: 96% of values are stable
            let mut b012 = [0u8; 4];
            b012[..3].copy_from_slice(bytes);
            let val = u32::from_be_bytes(b012);
            let low = val & !0xFF;
            let high = val | 0xFF;
            low / (85_u32.pow(2)) == high / (85_u32.pow(2))
        }
        _ => false,
    }
}

/// Check if an exit boundary is stable (simpler: check if raw context can disambiguate).
fn is_stable_exit(_bytes: &[u8]) -> bool {
    // Exit boundaries always use raw context for disambiguation, so always "stable" in that sense
    // We return true to indicate they're safe to use
    true
}

/// Encode a raw section with implicit length (block-aligned).
fn encode_raw_section(raw_data: &[u8], output: &mut Vec<u8>, is_block_aligned: bool) {
    debug_assert!(raw_data.len() >= MIN_RAW_LEN);

    // Choose escape character (BE convention by default)
    output.push(ESCAPE_BE);

    // Encode length in overflow character
    if is_block_aligned {
        let blocks = raw_data.len() / 4;
        encode_length(blocks as u16, output);
    } else {
        encode_length(raw_data.len() as u16, output);
    }

    // Raw data passes through as-is
    output.extend_from_slice(raw_data);
}

/// Encode a raw section with explicit mid-block exit boundary.
fn encode_raw_section_with_exit(blocks_data: &[u8], suffix_bytes: &[u8], output: &mut Vec<u8>) {
    debug_assert!(blocks_data.len() % 4 == 0);
    debug_assert!(suffix_bytes.len() > 0 && suffix_bytes.len() < 4);

    output.push(ESCAPE_BE);

    let total_bytes = blocks_data.len() + suffix_bytes.len();
    encode_length(total_bytes as u16, output);

    // Raw data
    output.extend_from_slice(blocks_data);

    // Trailing Z85 characters for suffix bytes
    encode_trailing_bytes(&[], suffix_bytes, output);
}

/// Encode a length value in Z85 alphabet space.
fn encode_length(len: u16, output: &mut Vec<u8>) {
    // Use a simple scheme: map length to Z85 alphabet (0-84)
    // For lengths > 84, use multiple characters
    if len <= 84 {
        output.push(z85::Z85_ALPHABET[len as usize]);
    } else if len <= 7140 {
        // 84 * 85 = 7140
        let hi = (len / 85) as u8;
        let lo = (len % 85) as u8;
        output.push(z85::Z85_ALPHABET[hi as usize]);
        output.push(z85::Z85_ALPHABET[lo as usize]);
    } else {
        // For very long sections, use all available alphabet
        let val0 = (len / (85 * 85)) as u8;
        let val1 = ((len / 85) % 85) as u8;
        let val2 = (len % 85) as u8;
        output.push(z85::Z85_ALPHABET[val0 as usize]);
        output.push(z85::Z85_ALPHABET[val1 as usize]);
        output.push(z85::Z85_ALPHABET[val2 as usize]);
    }
}

/// Decode a length value from Z85 alphabet.
fn decode_length(chars: &[u8]) -> Result<(u16, usize)> {
    if chars.is_empty() {
        return Err(Error::InvalidEscapeSequence {
            position: 0,
            reason: "missing length encoding".to_string(),
        });
    }

    let z85_decode = z85::Z85_DECODE_TABLE;

    // Try to decode 1, 2, or 3 characters
    let c0 = z85_decode[chars[0] as usize];
    if c0 == 255 {
        return Err(Error::InvalidZ85Character {
            character: chars[0] as char,
            position: 0,
        });
    }

    if chars.len() >= 3 && c0 < 85 {
        let c1 = z85_decode[chars[1] as usize];
        let c2 = z85_decode[chars[2] as usize];
        if c1 != 255 && c2 != 255 {
            let len = c0 as u16 * 85 * 85 + c1 as u16 * 85 + c2 as u16;
            return Ok((len, 3));
        }
    }

    if chars.len() >= 2 {
        let c1 = z85_decode[chars[1] as usize];
        if c1 != 255 {
            let len = c0 as u16 * 85 + c1 as u16;
            return Ok((len, 2));
        }
    }

    // Single character
    let len = c0 as u16;
    Ok((len, 1))
}

/// Decode extended Z85 with raw sections.
pub fn decode(input: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut input_pos = 0;

    while input_pos < input.len() {
        let c = input[input_pos];

        if c == ESCAPE_BE || c == ESCAPE_LE {
            // Raw section
            input_pos += 1;

            if input_pos >= input.len() {
                return Err(Error::InvalidEscapeSequence {
                    position: input_pos - 1,
                    reason: "escape at end of input".to_string(),
                });
            }

            // Decode length
            let (len, len_chars) = decode_length(&input[input_pos..])?;
            input_pos += len_chars;

            if input_pos + len as usize > input.len() {
                return Err(Error::TruncatedRawSection {
                    expected: len as usize,
                    got: input.len() - input_pos,
                });
            }

            // Check for mid-block entry/exit boundaries
            let z85_chars_before = count_z85_chars_in_output(&output);
            let block_offset = z85_chars_before % 5;

            if block_offset > 0 {
                // Mid-block entry: we have partial Z85 characters for the entry boundary
                // The number of partial Z85 characters is (5 - block_offset)
                // They encode (5 - block_offset - 1) bytes
                let partial_z85_count = 5 - block_offset;
                let entry_bytes = partial_z85_count - 1;

                if entry_bytes > 0 && len as usize >= entry_bytes {
                    // Decode the leading Z85 characters (already in output as the incomplete block)
                    // The raw data starts right after them
                    // We've already emitted the leading characters, now add raw data
                    output.extend_from_slice(&input[input_pos..input_pos + len as usize]);
                    input_pos += len as usize;
                    continue;
                }
            }

            // Normal (block-aligned or simple) raw section
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

/// Count Z85 characters (non-escape, non-raw) in output.
fn count_z85_chars_in_output(output: &[u8]) -> usize {
    output.iter().filter(|&&c| c != ESCAPE_BE && c != ESCAPE_LE).count()
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

// Export Z85 alphabet for length encoding
pub use z85::Z85_ALPHABET;
