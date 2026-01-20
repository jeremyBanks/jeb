//! Decoder for extended Z85 with raw passthrough.
//!
//! The decoder processes input in 5-character blocks, detecting escape
//! characters and handling raw byte sequences that may span multiple blocks.

use crate::alphabet::{
    decode_z85_block, escape_info_at_position_1_to_3, escape_raw_bytes_at_position_0,
    is_escape_char, z85_digit_value, ESCAPE_PIPE,
};
use crate::base42::{decode_length, Endianness};
use crate::error::DecodeError;

/// Decoder state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Normal decoding (Z85 or looking for escapes)
    Normal,
    /// Currently consuming raw bytes
    InRaw,
    /// Infinite raw mode (until end of input)
    InfiniteRaw,
}

/// Streaming decoder for extended Z85 with raw passthrough.
pub struct Decoder {
    state: State,
    raw_bytes_remaining: usize,
    output: Vec<u8>,
    buffer: Vec<u8>,
    position: usize, // global position for error reporting
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder {
    /// Create a new decoder.
    pub fn new() -> Self {
        Self {
            state: State::Normal,
            raw_bytes_remaining: 0,
            output: Vec::new(),
            buffer: Vec::new(),
            position: 0,
        }
    }

    /// Write encoded data to the decoder.
    pub fn write(&mut self, data: &[u8]) -> Result<(), DecodeError> {
        for &byte in data {
            self.buffer.push(byte);
            self.position += 1;

            // Process complete blocks
            while self.buffer.len() >= 5 {
                self.process_block()?;
            }
        }
        Ok(())
    }

    /// Finish decoding and return the decoded output.
    pub fn finish(mut self) -> Result<Vec<u8>, DecodeError> {
        // Process any remaining partial block
        if !self.buffer.is_empty() {
            self.process_partial()?;
        }

        Ok(self.output)
    }

    /// Process a complete 5-character block.
    fn process_block(&mut self) -> Result<(), DecodeError> {
        debug_assert!(self.buffer.len() >= 5);

        // Extract the block
        let block: [u8; 5] = [
            self.buffer[0],
            self.buffer[1],
            self.buffer[2],
            self.buffer[3],
            self.buffer[4],
        ];

        // If we're in raw mode, consume raw bytes from this block
        if self.state == State::InRaw || self.state == State::InfiniteRaw {
            let to_consume = if self.state == State::InfiniteRaw {
                5 // consume all
            } else {
                self.raw_bytes_remaining.min(5)
            };

            // Raw bytes are consumed verbatim - padding only appears AFTER raw sequence ends
            for &byte in block.iter().take(to_consume) {
                self.output.push(byte);
            }

            if self.state == State::InRaw {
                self.raw_bytes_remaining -= to_consume;
                if self.raw_bytes_remaining == 0 {
                    self.state = State::Normal;
                    // Remaining chars in this block are padding - skip them all
                    // (drain the entire block, not just the consumed raw bytes)
                }
            }

            self.buffer.drain(..5);
            return Ok(());
        }

        // Normal mode: scan for escape characters
        let escape_pos = block.iter().position(|&b| is_escape_char(b));

        match escape_pos {
            Some(pos) => {
                let escape = block[pos];
                self.process_escape(&block, pos, escape)?;
            }
            None => {
                // Standard Z85 block - decode all 5 characters
                // Note: '.' is both the padding character AND Z85 digit 62
                // In a full 5-char block, all characters are meaningful
                let decoded = decode_z85_block(&block)
                    .ok_or_else(|| DecodeError::InvalidCharacter {
                        position: self.position - 5,
                        byte: block[0], // First invalid char
                    })?;
                self.output.extend_from_slice(&decoded);
            }
        }

        self.buffer.drain(..5);
        Ok(())
    }

    /// Process an escape character found at the given position.
    fn process_escape(&mut self, block: &[u8; 5], pos: usize, escape: u8) -> Result<(), DecodeError> {
        if escape == ESCAPE_PIPE {
            self.process_pipe_escape(block, pos)?;
        } else {
            self.process_standard_escape(block, pos, escape)?;
        }
        Ok(())
    }

    /// Process a standard escape (`, ` ; ~ _).
    fn process_standard_escape(
        &mut self,
        block: &[u8; 5],
        pos: usize,
        escape: u8,
    ) -> Result<(), DecodeError> {
        let (raw_bytes, _is_little_endian) = if pos == 0 {
            // At position 0, no prefix
            let raw = escape_raw_bytes_at_position_0(escape)
                .ok_or(DecodeError::InvalidEscapePosition {
                    position: self.position - 5 + pos,
                    escape: escape as char,
                })?;
            (raw, true) // Endianness doesn't matter for position 0
        } else if pos <= 3 {
            // At positions 1-3, decode prefix
            let (raw, little_endian) = escape_info_at_position_1_to_3(escape)
                .ok_or(DecodeError::InvalidEscapePosition {
                    position: self.position - 5 + pos,
                    escape: escape as char,
                })?;

            // Decode prefix bytes
            let prefix_chars = &block[..pos];
            let prefix_value = self.decode_prefix(prefix_chars, little_endian)?;

            // Convert prefix value to bytes
            self.emit_prefix_bytes(prefix_value, pos, little_endian);

            (raw, little_endian)
        } else {
            // Position 4 is invalid for standard escapes
            return Err(DecodeError::InvalidEscapePosition {
                position: self.position - 5 + pos,
                escape: escape as char,
            });
        };

        // Consume raw bytes from remainder of block
        let raw_in_block = (5 - pos - 1).min(raw_bytes);
        for i in 0..raw_in_block {
            self.output.push(block[pos + 1 + i]);
        }

        // Set up cross-block continuation if needed
        let raw_remaining = raw_bytes - raw_in_block;
        if raw_remaining > 0 {
            self.state = State::InRaw;
            self.raw_bytes_remaining = raw_remaining;
        }

        Ok(())
    }

    /// Process the `|` pipe escape.
    fn process_pipe_escape(&mut self, block: &[u8; 5], pos: usize) -> Result<(), DecodeError> {
        // Characters before | encode the length
        let length_chars = &block[..pos];
        let block_aligned = pos == 0 || pos == 4;

        let (length, _endianness) = if length_chars.is_empty() {
            // | at position 0 means we need to look at prior digits
            // For now, treat as infinite (the prior block would have set this up)
            (0, Endianness::Little)
        } else {
            decode_length(length_chars, block_aligned)?
        };

        // Set up raw byte consumption
        if length == 0 {
            self.state = State::InfiniteRaw;
        } else {
            self.state = State::InRaw;
            self.raw_bytes_remaining = length;
        }

        // Consume raw bytes from remainder of this block
        let raw_in_block = if self.state == State::InfiniteRaw {
            5 - pos - 1
        } else {
            (5 - pos - 1).min(self.raw_bytes_remaining)
        };

        for i in 0..raw_in_block {
            self.output.push(block[pos + 1 + i]);
        }

        if self.state == State::InRaw {
            self.raw_bytes_remaining -= raw_in_block;
            if self.raw_bytes_remaining == 0 {
                self.state = State::Normal;
            }
        }

        Ok(())
    }

    /// Decode a prefix value from Z85 characters.
    fn decode_prefix(&self, chars: &[u8], little_endian: bool) -> Result<u64, DecodeError> {
        let mut value: u64 = 0;

        for (i, &c) in chars.iter().enumerate() {
            let digit = z85_digit_value(c).ok_or(DecodeError::InvalidCharacter {
                position: self.position - 5 + i,
                byte: c,
            })?;
            value = value * 85 + digit as u64;
        }

        // The prefix represents a big-endian value that will be converted to bytes
        // If little_endian flag is set, the bytes themselves are little-endian
        // (the digit interpretation is still big-endian, it's the byte order that differs)
        let _ = little_endian; // Used when emitting bytes, not here

        Ok(value)
    }

    /// Emit prefix bytes from a decoded value.
    fn emit_prefix_bytes(&mut self, value: u64, num_chars: usize, little_endian: bool) {
        let num_bytes = num_chars; // 1 char = 1 byte for prefix

        // Convert value to bytes
        let bytes = value.to_be_bytes();
        let significant_bytes = &bytes[8 - num_bytes..];

        if little_endian {
            // Little-endian: reverse byte order
            for &b in significant_bytes.iter().rev() {
                self.output.push(b);
            }
        } else {
            // Big-endian: keep byte order
            for &b in significant_bytes {
                self.output.push(b);
            }
        }
    }

    /// Process a partial block at end of input.
    fn process_partial(&mut self) -> Result<(), DecodeError> {
        // In infinite raw mode, consume all remaining bytes verbatim
        if self.state == State::InfiniteRaw {
            self.output.extend_from_slice(&self.buffer);
            self.buffer.clear();
            return Ok(());
        }

        // In raw mode, consume remaining raw bytes verbatim
        if self.state == State::InRaw {
            let to_consume = self.raw_bytes_remaining.min(self.buffer.len());
            self.output.extend_from_slice(&self.buffer[..to_consume]);
            self.buffer.drain(..to_consume);
            self.raw_bytes_remaining -= to_consume;

            if self.raw_bytes_remaining > 0 {
                return Err(DecodeError::UnexpectedEndOfInput);
            }

            if self.buffer.is_empty() {
                return Ok(());
            }
        }

        // Check for escape in partial block
        if let Some(pos) = self.buffer.iter().position(|&b| is_escape_char(b)) {
            // Handle escape in partial block
            // This is complex - for now, treat as error
            return Err(DecodeError::InvalidEscapePosition {
                position: self.position - self.buffer.len() + pos,
                escape: self.buffer[pos] as char,
            });
        }

        // Decode as partial Z85 (all remaining characters are meaningful)
        if !self.buffer.is_empty() {
            // Clone to avoid borrow issues
            let chars: Vec<u8> = self.buffer.drain(..).collect();
            self.decode_partial_z85(&chars)?;
        }

        Ok(())
    }

    /// Decode a partial Z85 block (fewer than 5 characters).
    fn decode_partial_z85(&mut self, chars: &[u8]) -> Result<(), DecodeError> {
        if chars.is_empty() {
            return Ok(());
        }

        // Pad at beginning (high-order positions) with '0' (value 0)
        let mut padded = [b'0'; 5];
        let start = 5 - chars.len();
        for (i, &c) in chars.iter().enumerate() {
            padded[start + i] = c;
        }

        // Decode as full block
        let decoded = decode_z85_block(&padded).ok_or_else(|| DecodeError::InvalidCharacter {
            position: self.position - chars.len(),
            byte: chars[0],
        })?;

        // Calculate how many output bytes based on input chars
        // c chars encodes floor(c * 4 / 5) bytes
        // Take from end (low-order positions)
        let num_bytes = chars.len() * 4 / 5;
        self.output.extend_from_slice(&decoded[4 - num_bytes..]);

        Ok(())
    }
}

/// Decode data in one shot.
pub fn decode(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let mut decoder = Decoder::new();
    decoder.write(data)?;
    decoder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::{encode_z85_block, z85_digit_char, ESCAPE_COMMA, ESCAPE_BACKTICK, PADDING_CHAR};

    #[test]
    fn test_decode_empty() {
        let result = decode(b"").unwrap();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_decode_z85_block() {
        // Encode a known value and decode it
        let original = [0x86, 0x4F, 0xD2, 0x6F];
        let encoded = encode_z85_block(&original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_decode_zeros() {
        let decoded = decode(b"00000").unwrap();
        assert_eq!(decoded, [0u8; 4]);
    }

    #[test]
    fn test_decode_escape_at_position_0() {
        // ` at position 0 = 3 raw bytes follow
        // Block: `abc. (backtick + 3 raw bytes + padding)
        let input = vec![ESCAPE_BACKTICK, b'a', b'b', b'c', PADDING_CHAR];
        let decoded = decode(&input).unwrap();
        assert_eq!(decoded, b"abc");
    }

    #[test]
    fn test_decode_comma_at_position_0() {
        // , at position 0 = 4 raw bytes follow
        // Block: ,abcd
        let input = vec![ESCAPE_COMMA, b'a', b'b', b'c', b'd'];
        let decoded = decode(&input).unwrap();
        assert_eq!(decoded, b"abcd");
    }

    #[test]
    fn test_decode_escape_with_prefix() {
        // At position 2: "AB," means decode AB as LE u16, then 4 raw bytes follow
        // A = 10, B = 11 in Z85
        // Value = 10*85 + 11 = 861
        // As LE u16: 861 = 0x035D, little-endian bytes are [0x5D, 0x03]
        //
        // Block 1: a b , x y  (positions 0-4)
        // Block 2: z w (partial - 2 more raw bytes)
        //
        // No trailing padding (encoder doesn't add it)
        let input = vec![
            z85_digit_char(10), // 'a'
            z85_digit_char(11), // 'b'
            ESCAPE_COMMA,       // , at position 2
            b'x', b'y',         // 2 raw bytes in this block
            b'z', b'w',         // 2 more raw bytes (partial block)
        ];

        let decoded = decode(&input).unwrap();
        // Expected: [0x5D, 0x03] (LE prefix) + [x, y, z, w] (raw)
        assert_eq!(decoded, &[0x5D, 0x03, b'x', b'y', b'z', b'w']);
    }

    #[test]
    fn test_decode_pipe_escape() {
        // | with length encoding
        // Length 10: single digit, value 10 + 1 = 11
        // Block 1: [len_digit] | 0 1 2  (positions 0-4)
        //          3 raw bytes consumed from positions 2-4
        // Block 2: 3 4 5 6 7  (5 more raw bytes, but we need 7)
        // Remaining: 8 9 (2 more raw bytes in partial block)
        //
        // Total input: 5 + 5 + 2 = 12 chars
        // No trailing padding (encoder doesn't add it, decoder doesn't expect it)
        let input = vec![
            z85_digit_char(11), // length digit (position 0)
            ESCAPE_PIPE,        // | (position 1)
            b'0', b'1', b'2',   // raw bytes (positions 2-4)
            b'3', b'4', b'5', b'6', b'7', // 5 more raw (block 2)
            b'8', b'9',         // 2 more raw (partial block 3)
        ];

        let decoded = decode(&input).unwrap();
        assert_eq!(decoded, b"0123456789");
    }
}
