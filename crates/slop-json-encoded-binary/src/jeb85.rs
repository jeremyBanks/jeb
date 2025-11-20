// JEB85 implementation - Enhanced Z85 with text mode and raw chunks
//
// JEB85 adds two enhancements over pure Z85:
// 1. Text mode: If input is text-safe (valid UTF-8, no prohibited control
//    chars, ≤64 KiB), pass through as-is
// 2. Raw chunks: For binary mode, preserve readable ASCII blocks using .
//    markers
//
// This implementation does NOT use the \b prefix - that's for JSON
// serialization.

use crate::{
    BLOCK_BYTES_4, BLOCK_BYTES_BY_DIGITS, BLOCK_DIGITS_5, BLOCK_DIGITS_BY_BYTES, TARGET_RAW_BYTES,
    Z85_DECODE, decode_z85_block, encode_z85_block,
};

/// Check if a byte is JSON-safe printable ASCII
const fn is_json_safe_ascii(byte: u8) -> bool {
    matches!(byte, 0x20..=0x7E)
}

/// Check if data is text-safe (can use text mode)
fn is_text_safe(data: &[u8]) -> bool {
    if data.len() > TARGET_RAW_BYTES {
        return false;
    }

    // Must be valid UTF-8
    if core::str::from_utf8(data).is_err() {
        return false;
    }

    // Check for prohibited control characters
    for &byte in data {
        match byte {
            // Allow common whitespace
            b'\t' | b'\n' | b'\r' => continue,
            // Prohibit all other control chars
            0x00..=0x1F | 0x7F => return false,
            _ => continue,
        }
    }

    true
}

/// JEB85 encoder
pub struct Jeb85Encoder;

impl Jeb85Encoder {
    /// Encode bytes using JEB85 (enhanced Z85 with text mode)
    #[must_use]
    pub fn encode_bytes(bytes: &[u8]) -> Vec<u8> {
        // Binary mode: use Z85 encoding with raw chunks
        Self::encode_binary(bytes)
    }

    fn encode_binary(data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len() * 5 / 4 + 10);
        let mut i = 0;

        while i < data.len() {
            // Look for runs of JSON-safe 4-byte blocks
            let run_start = i;
            let mut run_blocks = 0;

            while i < data.len() && run_blocks < 85 {
                let block_end = (i + 4).min(data.len());
                if block_end - i == 4 && data[i..block_end].iter().all(|&b| is_json_safe_ascii(b)) {
                    run_blocks += 1;
                    i += 4;
                } else {
                    break;
                }
            }

            // Emit raw chunk if we found any
            if run_blocks > 0 {
                let raw_len = run_blocks * 4;
                let is_at_end = i >= data.len();

                if run_blocks == 1 {
                    // Single block: .xxxx
                    result.push(b'_');
                    result.extend_from_slice(&data[run_start..run_start + 4]);
                } else if is_at_end {
                    // Multi-block at end: ..xxxx... (no padding needed)
                    result.extend_from_slice(b"__");
                    result.extend_from_slice(&data[run_start..run_start + raw_len]);
                } else {
                    // Multi-block in middle: N.xxxx... where N = blocks - 2
                    let count = run_blocks - 2;
                    let count_encoded = encode_count(count);
                    result.extend_from_slice(&count_encoded);
                    result.push(b'_');

                    result.extend_from_slice(&data[run_start..run_start + raw_len]);

                    // Pad to 5-char alignment with . or preview of next bytes
                    let total_len = count_encoded.len() + 1 + raw_len;
                    let padding = (5 - (total_len % 5)) % 5;

                    // Try to use next bytes as padding for readability
                    let mut padding_used = 0;
                    while padding_used < padding && i < data.len() {
                        if is_json_safe_ascii(data[i]) {
                            result.push(data[i]);
                            padding_used += 1;
                            i += 1;
                        } else {
                            break;
                        }
                    }

                    // Fill remaining padding with .
                    result.extend(core::iter::repeat_n(b'_', padding - padding_used));

                    // Rewind i - those bytes are still to be processed
                    i -= padding_used;
                }
            } else {
                // Encode as Z85
                let block_end = (i + 4).min(data.len());
                let remaining = block_end - i;

                if remaining == 4 {
                    let mut block = [0u8; 4];
                    block.copy_from_slice(&data[i..i + 4]);
                    let encoded = encode_z85_block(block);
                    result.extend_from_slice(&encoded);
                    i += 4;
                } else if remaining > 0 {
                    // Partial block - use terminal raw chunk ..
                    result.extend_from_slice(b"__");
                    result.extend_from_slice(&data[i..]);
                    i = data.len();
                }
            }
        }

        result
    }
}

/// Encode a count as Z85 digits
fn encode_count(count: usize) -> Vec<u8> {
    if count == 0 {
        return vec![crate::Z85_ALPHABET[0]];
    }

    let mut result = Vec::new();
    let mut val = count;

    while val > 0 {
        result.push(crate::Z85_ALPHABET[val % 85]);
        val /= 85;
    }

    result.reverse();
    result
}

/// JEB85 decoder
pub struct Jeb85Decoder;

impl Jeb85Decoder {
    /// Decode JEB85-encoded bytes
    ///
    /// # Panics
    ///
    /// Panics on invalid encoding
    #[must_use]
    pub fn decode_bytes(encoded: &[u8]) -> Vec<u8> {
        if encoded.is_empty() {
            return Vec::new();
        }

        // If it contains . markers, it's binary mode with raw chunks
        if encoded.contains(&b'_') {
            return Self::decode_binary(encoded);
        }

        // Check if all characters are Z85-valid
        let all_z85 = encoded.iter().all(|&b| Z85_DECODE[b as usize] != 255);

        // If text-safe and NOT all Z85 chars, it's text mode
        if is_text_safe(encoded) && !all_z85 {
            return encoded.to_vec();
        }

        // Otherwise decode as binary (pure Z85)
        Self::decode_binary(encoded)
    }

    fn decode_binary(encoded: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(encoded.len() * 4 / 5);
        let mut i = 0;

        while i < encoded.len() {
            if encoded[i] == b'_' {
                // Raw chunk marker
                if i + 1 < encoded.len() && encoded[i + 1] == b'_' {
                    // Terminal raw: rest is raw data
                    result.extend_from_slice(&encoded[i + 2..]);
                    break;
                } else if i + 5 <= encoded.len() {
                    // Single raw block .xxxx
                    result.extend_from_slice(&encoded[i + 1..i + 5]);
                    i += 5;
                } else {
                    panic!("Invalid raw chunk");
                }
            } else {
                // Check if this is a count prefix for multi-block raw
                let mut count_len = 0;
                let mut j = i;
                while j < encoded.len() && j - i < 4 && encoded[j] != b'_' {
                    if Z85_DECODE[encoded[j] as usize] == 255 {
                        break;
                    } else {
                        count_len += 1;
                        j += 1;
                    }
                }

                if j < encoded.len() && encoded[j] == b'_' && count_len > 0 {
                    // Multi-block raw: decode count
                    let mut count = 0usize;
                    for k in i..j {
                        count = count * 85 + Z85_DECODE[encoded[k] as usize] as usize;
                    }
                    let num_blocks = count + 2;
                    let raw_bytes = num_blocks * 4;

                    // Skip past count and .
                    i = j + 1;

                    // Read raw data (may have . padding)
                    let prefix_len = count_len + 1;
                    let total_len = prefix_len + raw_bytes;
                    let padding = (5 - (total_len % 5)) % 5;
                    let chars_to_read = raw_bytes + padding;

                    if i + chars_to_read <= encoded.len() {
                        result.extend_from_slice(&encoded[i..i + raw_bytes]);
                        i += chars_to_read;
                    } else {
                        panic!("Invalid multi-block raw chunk");
                    }
                } else {
                    // Regular Z85 block
                    if i + 5 <= encoded.len() {
                        let mut block = [0u8; 5];
                        block.copy_from_slice(&encoded[i..i + 5]);
                        let decoded = decode_z85_block(block).expect("Invalid Z85 block");
                        result.extend_from_slice(&decoded);
                        i += 5;
                    } else if i + 1 < encoded.len() {
                        // Partial Z85 block
                        let remaining = encoded.len() - i;
                        let mut block = [b'0'; 5];
                        let start_pos = 5 - remaining;
                        block[start_pos..].copy_from_slice(&encoded[i..]);
                        let decoded = decode_z85_block(block).expect("Invalid Z85 partial block");
                        let bytes_decoded = BLOCK_BYTES_BY_DIGITS[remaining];
                        if bytes_decoded == usize::MAX {
                            panic!("Invalid partial block size");
                        }
                        let start_byte = 4 - bytes_decoded;
                        result.extend_from_slice(&decoded[start_byte..]);
                        break;
                    } else {
                        break;
                    }
                }
            }
        }

        result
    }
}

#[must_use]
pub fn encode_jeb85(bytes: &[u8]) -> Vec<u8> {
    Jeb85Encoder::encode_bytes(bytes)
}

#[must_use]
pub fn decode_jeb85(encoded: &[u8]) -> Vec<u8> {
    Jeb85Decoder::decode_bytes(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_mode() {
        let data = b"Hello, World!";
        let encoded = encode_jeb85(data);
        // Text is now encoded in binary mode with raw chunks, not passed through
        assert_ne!(encoded, data);

        let decoded = decode_jeb85(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_binary_mode() {
        let data = b"\x00\x01\x02\x03\x04\x05";
        let encoded = encode_jeb85(data);
        assert_ne!(encoded, data); // Should be encoded

        let decoded = decode_jeb85(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_mixed_text_binary() {
        let data = b"Hello\x00World";
        let encoded = encode_jeb85(data);
        let decoded = decode_jeb85(&encoded);
        assert_eq!(decoded, data);
    }
}
