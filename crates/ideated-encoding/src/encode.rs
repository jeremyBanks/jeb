//! Encoder for extended Z85 with raw passthrough.
//!
//! The encoder uses a lookahead buffer to decide whether to use raw passthrough
//! or standard Z85 encoding. It chooses the escape form that maximizes the
//! length of raw passthrough sequences.

use crate::alphabet::{
    encode_z85_block, is_safe_for_raw, z85_digit_char, ESCAPE_BACKTICK, ESCAPE_COMMA,
    ESCAPE_PIPE, ESCAPE_SEMICOLON, ESCAPE_TILDE, ESCAPE_UNDERSCORE, PADDING_CHAR,
};
use crate::base42::{encode_length, Endianness};

/// Encoding strategy determined by buffer analysis.
enum EncodingStrategy {
    /// Use standard Z85 encoding.
    StandardZ85 {
        /// Number of bytes to encode (1-4).
        bytes: usize,
    },
    /// Use raw passthrough with a standard escape.
    RawPassthrough {
        /// Number of prefix bytes to encode (0-3).
        prefix_bytes: usize,
        /// The escape character to use.
        escape: u8,
        /// Number of raw bytes following the escape.
        raw_bytes: usize,
        /// Whether to use little-endian for prefix encoding.
        is_little_endian: bool,
    },
    /// Use the pipe escape for 8+ bytes.
    PipeRaw {
        /// Total length of raw bytes (0 for infinite).
        length: usize,
        /// Position of | within the block.
        pipe_position: u8,
        /// Endianness hint.
        is_little_endian: bool,
    },
}

/// Default lookahead buffer size.
const DEFAULT_LOOKAHEAD: usize = 64;

/// Streaming encoder for extended Z85 with raw passthrough.
pub struct Encoder {
    /// Lookahead buffer for analyzing upcoming bytes
    buffer: Vec<u8>,
    /// Maximum lookahead size
    lookahead_size: usize,
    /// Output buffer
    output: Vec<u8>,
    /// Current position within a 5-char block (0-4)
    block_position: usize,
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder {
    /// Create a new encoder with default lookahead size.
    pub fn new() -> Self {
        Self::with_lookahead_size(DEFAULT_LOOKAHEAD)
    }

    /// Create a new encoder with a specific lookahead size.
    pub fn with_lookahead_size(size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(size),
            lookahead_size: size,
            output: Vec::new(),
            block_position: 0,
        }
    }

    /// Write input data to the encoder.
    pub fn write(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);

        // Process data while we have enough in the buffer
        while self.buffer.len() >= self.lookahead_size {
            self.process_buffer(false);
        }
    }

    /// Finish encoding and return the output.
    pub fn finish(mut self) -> Vec<u8> {
        // Process all remaining data
        while !self.buffer.is_empty() {
            self.process_buffer(true);
        }

        // Add final padding if needed to complete a block
        if self.block_position != 0 {
            while self.block_position < 5 {
                self.output.push(PADDING_CHAR);
                self.block_position += 1;
            }
            self.block_position = 0;
        }

        self.output
    }

    /// Process data from the buffer.
    fn process_buffer(&mut self, is_final: bool) {
        if self.buffer.is_empty() {
            return;
        }

        // Analyze the buffer to find the best encoding strategy
        let strategy = self.analyze_buffer(is_final);

        match strategy {
            EncodingStrategy::StandardZ85 { bytes } => {
                self.emit_z85_block(bytes);
            }
            EncodingStrategy::RawPassthrough {
                prefix_bytes,
                escape,
                raw_bytes,
                is_little_endian,
            } => {
                self.emit_raw_sequence(prefix_bytes, escape, raw_bytes, is_little_endian);
            }
            EncodingStrategy::PipeRaw {
                length,
                pipe_position,
                is_little_endian,
            } => {
                self.emit_pipe_sequence(length, pipe_position, is_little_endian);
            }
        }
    }

    /// Analyze the buffer to determine the best encoding strategy.
    fn analyze_buffer(&self, is_final: bool) -> EncodingStrategy {
        // First, check if standard escapes would be beneficial
        if let Some(strategy) = self.try_standard_escape() {
            return strategy;
        }

        // Check if pipe escape would be beneficial for longer sequences
        if let Some(strategy) = self.try_pipe_escape(is_final) {
            return strategy;
        }

        // Fall back to standard Z85 encoding
        let bytes_to_encode = self.buffer.len().min(4);
        EncodingStrategy::StandardZ85 {
            bytes: bytes_to_encode,
        }
    }

    /// Try to use a standard escape (`, ` ; ~ _).
    fn try_standard_escape(&self) -> Option<EncodingStrategy> {
        // For each possible escape position and type, calculate the benefit

        // Standard escapes and their raw byte counts at position 0
        let escapes_pos_0 = [
            (ESCAPE_BACKTICK, 3),    // `
            (ESCAPE_COMMA, 4),       // ,
            (ESCAPE_TILDE, 5),       // ~
            (ESCAPE_SEMICOLON, 6),   // ;
            (ESCAPE_UNDERSCORE, 7),  // _
        ];

        // Check position 0 (no prefix)
        for &(escape, raw_count) in &escapes_pos_0 {
            if self.buffer.len() >= raw_count {
                if self.all_safe_for_raw(&self.buffer[..raw_count]) {
                    return Some(EncodingStrategy::RawPassthrough {
                        prefix_bytes: 0,
                        escape,
                        raw_bytes: raw_count,
                        is_little_endian: true,
                    });
                }
            }
        }

        // Check positions 1-3 with prefix encoding
        for prefix_len in 1..=3 {
            if self.buffer.len() < prefix_len {
                continue;
            }

            // Check if prefix can be encoded (value fits in prefix_len chars)
            let prefix_bytes = &self.buffer[..prefix_len];
            let (can_encode_le, can_encode_be) = self.can_encode_prefix(prefix_bytes);

            if !can_encode_le && !can_encode_be {
                continue;
            }

            // Standard escapes at positions 1-3
            let escapes_with_prefix = [
                (ESCAPE_COMMA, 4, true),     // LE
                (ESCAPE_BACKTICK, 4, false), // BE
                (ESCAPE_SEMICOLON, 6, true), // LE
                (ESCAPE_TILDE, 6, false),    // BE
                (ESCAPE_UNDERSCORE, 7, true), // LE only
            ];

            for &(escape, raw_count, is_le) in &escapes_with_prefix {
                if is_le && !can_encode_le {
                    continue;
                }
                if !is_le && !can_encode_be {
                    continue;
                }

                let total_bytes = prefix_len + raw_count;
                if self.buffer.len() >= total_bytes {
                    if self.all_safe_for_raw(&self.buffer[prefix_len..prefix_len + raw_count]) {
                        return Some(EncodingStrategy::RawPassthrough {
                            prefix_bytes: prefix_len,
                            escape,
                            raw_bytes: raw_count,
                            is_little_endian: is_le,
                        });
                    }
                }
            }
        }

        None
    }

    /// Try to use the pipe escape for 8+ bytes.
    fn try_pipe_escape(&self, is_final: bool) -> Option<EncodingStrategy> {
        // Find how many safe bytes we have starting from various positions
        let safe_count = self.count_safe_bytes(0);

        if safe_count >= 8 {
            // Determine length to use
            let length = if is_final && safe_count == self.buffer.len() {
                // Use infinite (0) if we're at the end
                0
            } else {
                safe_count.min(self.buffer.len())
            };

            // Use position that minimizes overhead
            // Position 4 allows maximum length encoding digits
            return Some(EncodingStrategy::PipeRaw {
                length,
                pipe_position: 1, // Simple position for now
                is_little_endian: true,
            });
        }

        None
    }

    /// Check if all bytes are safe for raw passthrough.
    fn all_safe_for_raw(&self, bytes: &[u8]) -> bool {
        bytes.iter().all(|&b| is_safe_for_raw(b))
    }

    /// Count how many consecutive safe bytes from a starting position.
    fn count_safe_bytes(&self, start: usize) -> usize {
        self.buffer[start..]
            .iter()
            .take_while(|&&b| is_safe_for_raw(b))
            .count()
    }

    /// Check if prefix bytes can be encoded as a base-85 value.
    /// Returns (can_encode_little_endian, can_encode_big_endian).
    fn can_encode_prefix(&self, bytes: &[u8]) -> (bool, bool) {
        let max_values = [85u64, 85 * 85, 85 * 85 * 85]; // For 1, 2, 3 chars
        let max_value = max_values.get(bytes.len() - 1).copied().unwrap_or(0);

        if max_value == 0 {
            return (false, false);
        }

        // Calculate LE and BE values
        let le_value = bytes_to_u64_le(bytes);
        let be_value = bytes_to_u64_be(bytes);

        (le_value < max_value, be_value < max_value)
    }

    /// Emit a standard Z85 block.
    fn emit_z85_block(&mut self, byte_count: usize) {
        debug_assert!(byte_count <= 4);
        debug_assert!(byte_count <= self.buffer.len());

        if byte_count == 4 {
            // Full block
            let bytes: [u8; 4] = [
                self.buffer[0],
                self.buffer[1],
                self.buffer[2],
                self.buffer[3],
            ];
            let encoded = encode_z85_block(&bytes);
            self.output.extend_from_slice(&encoded);
            self.buffer.drain(..4);
            // block_position stays at 0 (we emitted a full 5-char block)
        } else {
            // Partial block
            let mut bytes = [0u8; 4];
            for (i, &b) in self.buffer[..byte_count].iter().enumerate() {
                bytes[i] = b;
            }
            let encoded = encode_z85_block(&bytes);

            // Output only the needed characters
            // For b bytes, we need ceil(b * 5 / 4) characters
            let char_count = (byte_count * 5 + 3) / 4;
            self.output.extend_from_slice(&encoded[..char_count]);
            self.buffer.drain(..byte_count);
            self.block_position = (self.block_position + char_count) % 5;
        }
    }

    /// Emit a raw sequence using a standard escape.
    fn emit_raw_sequence(
        &mut self,
        prefix_bytes: usize,
        escape: u8,
        raw_bytes: usize,
        is_little_endian: bool,
    ) {
        // Emit prefix if any
        if prefix_bytes > 0 {
            let prefix = &self.buffer[..prefix_bytes];
            let value = if is_little_endian {
                bytes_to_u64_le(prefix)
            } else {
                bytes_to_u64_be(prefix)
            };

            // Encode value in base-85
            self.emit_base85_value(value, prefix_bytes);
        }

        // Emit escape
        self.output.push(escape);
        self.block_position = (self.block_position + 1) % 5;

        // Emit raw bytes
        let raw_start = prefix_bytes;
        let raw_end = raw_start + raw_bytes;
        self.output
            .extend_from_slice(&self.buffer[raw_start..raw_end]);
        self.block_position = (self.block_position + raw_bytes) % 5;

        // Calculate total bytes used
        let total_bytes = prefix_bytes + raw_bytes;

        // Calculate padding needed
        // Standard encoding would use: ceil(total_bytes * 5 / 4) chars
        // We used: prefix_bytes (chars) + 1 (escape) + raw_bytes (chars)
        let chars_used = prefix_bytes + 1 + raw_bytes;
        let standard_chars = (total_bytes * 5 + 3) / 4;

        // Padding to maintain alignment
        if chars_used < standard_chars {
            let padding = standard_chars - chars_used;
            for _ in 0..padding {
                self.output.push(PADDING_CHAR);
                self.block_position = (self.block_position + 1) % 5;
            }
        }

        self.buffer.drain(..total_bytes);
    }

    /// Emit base-85 encoded value.
    fn emit_base85_value(&mut self, value: u64, num_chars: usize) {
        let mut digits = vec![0u8; num_chars];
        let mut v = value;

        for i in (0..num_chars).rev() {
            digits[i] = z85_digit_char((v % 85) as u8);
            v /= 85;
        }

        self.output.extend_from_slice(&digits);
        self.block_position = (self.block_position + num_chars) % 5;
    }

    /// Emit a pipe escape with length-encoded raw sequence.
    fn emit_pipe_sequence(&mut self, length: usize, _pipe_position: u8, is_little_endian: bool) {
        // Encode the length
        let endianness = if is_little_endian {
            Endianness::Little
        } else {
            Endianness::Big
        };

        // Determine how many length digits we need
        let length_chars = encode_length(length, 1, endianness);
        let pipe_position = length_chars.len();

        // Emit length digits
        self.output.extend_from_slice(&length_chars);
        self.block_position = (self.block_position + length_chars.len()) % 5;

        // Emit pipe
        self.output.push(ESCAPE_PIPE);
        self.block_position = (self.block_position + 1) % 5;

        // Emit raw bytes
        let raw_count = if length == 0 {
            self.buffer.len() // Infinite = all remaining
        } else {
            length
        };

        self.output.extend_from_slice(&self.buffer[..raw_count]);
        self.block_position = (self.block_position + raw_count) % 5;

        // Calculate padding
        // Total chars used = length_chars + 1 (pipe) + raw_count
        // Standard would use ceil(raw_count * 5 / 4)
        let chars_used = length_chars.len() + 1 + raw_count;

        // We need to pad to maintain alignment relative to what standard encoding
        // would have produced for the same number of bytes
        // But actually, raw bytes don't produce the same overhead as Z85...
        // Let me reconsider.

        // The invariant is: following content should appear at the same position
        // as if everything before it was standard Z85 encoded.
        // Raw bytes use 1:1 mapping but standard Z85 uses 5:4 mapping.
        // So for N raw bytes, standard would produce ceil(N * 5 / 4) chars.
        // We produced: length_chars + 1 + N chars.
        // The difference is the padding needed.

        let standard_chars = (raw_count * 5 + 3) / 4;
        let prefix_overhead = length_chars.len() + 1;
        let total_chars = prefix_overhead + raw_count;

        // Padding needed to reach the same total as standard encoding
        if total_chars < standard_chars {
            let padding = standard_chars - total_chars;
            for _ in 0..padding {
                self.output.push(PADDING_CHAR);
                self.block_position = (self.block_position + 1) % 5;
            }
        }

        self.buffer.drain(..raw_count);
    }
}

/// Convert bytes to u64 in little-endian order.
fn bytes_to_u64_le(bytes: &[u8]) -> u64 {
    let mut value = 0u64;
    for (i, &b) in bytes.iter().enumerate() {
        value |= (b as u64) << (i * 8);
    }
    value
}

/// Convert bytes to u64 in big-endian order.
fn bytes_to_u64_be(bytes: &[u8]) -> u64 {
    let mut value = 0u64;
    for &b in bytes {
        value = (value << 8) | (b as u64);
    }
    value
}

/// Encode data in one shot.
pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut encoder = Encoder::new();
    encoder.write(data);
    encoder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::decode;

    #[test]
    fn test_encode_empty() {
        let result = encode(b"");
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_encode_4_bytes() {
        let original = [0x86, 0x4F, 0xD2, 0x6F];
        let encoded = encode(&original);
        // Should be standard Z85
        assert_eq!(encoded.len(), 5);

        // Roundtrip
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_encode_zeros() {
        let original = [0u8; 4];
        let encoded = encode(&original);
        assert_eq!(&encoded, b"00000");
    }

    #[test]
    fn test_encode_safe_bytes() {
        // Safe bytes that could use raw passthrough
        let original = b"abcd";
        let encoded = encode(original);

        // Roundtrip
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_roundtrip_random_data() {
        // Test with some random-ish data
        let original: Vec<u8> = (0..100).map(|i| (i * 17 + 23) as u8).collect();
        let encoded = encode(&original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_roundtrip_ascii_text() {
        let original = b"Hello, World! This is a test of the extended Z85 encoding.";
        let encoded = encode(original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_bytes_to_u64() {
        assert_eq!(bytes_to_u64_le(&[0x48, 0x00]), 0x0048);
        assert_eq!(bytes_to_u64_be(&[0x48, 0x00]), 0x4800);

        assert_eq!(bytes_to_u64_le(&[0x01, 0x02, 0x03]), 0x030201);
        assert_eq!(bytes_to_u64_be(&[0x01, 0x02, 0x03]), 0x010203);
    }
}
