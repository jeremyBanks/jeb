// Z85 Encoding/Decoding Implementation
// =====================================
//
// Z85 is a binary-to-text encoding scheme defined by ZeroMQ (RFC 32).
// It encodes binary data into printable ASCII characters, similar to Base64
// but with a different alphabet and encoding ratio.
//
// Key Properties:
// - Alphabet: 85 printable ASCII characters (excluding characters that might
//   cause issues in various contexts like quotes, backslash, etc.)
// - Encoding ratio: 4 bytes -> 5 characters (80% efficiency vs 75% for Base64)
// - Big-endian byte order for the 4-byte blocks
//
// This implementation extends standard Z85 to support arbitrary-length input
// (not just multiples of 4 bytes) using a scheme similar to unpadded Base64:
// - 1 byte  -> 2 characters
// - 2 bytes -> 3 characters
// - 3 bytes -> 4 characters
// - 4 bytes -> 5 characters

/// The Z85 alphabet: 85 printable ASCII characters in a specific order.
/// Characters are chosen to be safe in most contexts (no quotes, backslash, etc.)
/// Index 0 = '0', Index 84 = '#'
const Z85_ALPHABET: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/// Lookup table for decoding: maps ASCII byte value -> Z85 digit value (0-84)
/// Invalid characters are marked with 0xFF
const Z85_DECODE_TABLE: [u8; 256] = build_decode_table();

/// Build the decode lookup table at compile time.
/// For each byte 0-255, stores either the Z85 digit value (0-84) or 0xFF if invalid.
const fn build_decode_table() -> [u8; 256] {
    let mut table = [0xFFu8; 256];
    let mut i = 0usize;
    while i < 85 {
        table[Z85_ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    table
}

/// Error type for Z85 decoding failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Input contains a character not in the Z85 alphabet
    InvalidCharacter(u8),
    /// The decoded value overflows the expected byte count
    /// (e.g., 5 characters that decode to > 0xFFFFFFFF)
    Overflow,
    /// Input length is invalid (1 char or 6 chars - can't map to valid byte counts)
    InvalidLength,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::InvalidCharacter(c) => {
                write!(f, "invalid character in Z85 input: 0x{:02X}", c)
            }
            DecodeError::Overflow => write!(f, "Z85 value overflow"),
            DecodeError::InvalidLength => write!(f, "invalid Z85 input length"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Encode arbitrary bytes into a Z85 string.
///
/// # Algorithm
///
/// For each 4-byte block:
/// 1. Interpret the 4 bytes as a big-endian u32
/// 2. Convert to base-85 by repeatedly dividing by 85
/// 3. Map each base-85 digit to the corresponding alphabet character
///
/// The division produces digits in reverse order (least significant first),
/// so we fill the output buffer from right to left.
///
/// For trailing bytes (1-3 bytes), we:
/// 1. Pad conceptually with zeros on the right to form a partial block
/// 2. Encode only the significant portion:
///    - 1 byte  (8 bits)  -> 2 chars (ceil(8 * 5/4) / 5 = 2 chars needed)
///    - 2 bytes (16 bits) -> 3 chars
///    - 3 bytes (24 bits) -> 4 chars
///
/// # Example
///
/// Input: [0x86, 0x4F, 0xD2, 0x6F]
/// As u32 (big-endian): 0x864FD26F = 2,252,698,223
///
/// Successive division by 85:
/// 2252698223 / 85 = 26502332 remainder 3  -> alphabet[3] = '3' (but this goes last)
/// ... (continue division)
///
pub fn encode(input: &[u8]) -> String {
    // Handle empty input
    if input.is_empty() {
        return String::new();
    }

    // Calculate output size:
    // - Full 4-byte blocks: each produces 5 characters
    // - Trailing n bytes (1-3): produces n+1 characters
    let full_blocks = input.len() / 4;
    let trailing = input.len() % 4;
    let trailing_chars = if trailing > 0 { trailing + 1 } else { 0 };
    let output_len = full_blocks * 5 + trailing_chars;

    let mut output = vec![0u8; output_len];
    let mut out_idx = 0;

    // Process full 4-byte blocks
    for chunk in input.chunks(4) {
        if chunk.len() == 4 {
            // Convert 4 bytes to big-endian u32
            let value = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);

            // Convert to base-85, filling 5 characters from right to left
            // This naturally handles the big-endian ordering
            encode_block_to_slice(value, &mut output[out_idx..out_idx + 5]);
            out_idx += 5;
        } else {
            // Handle trailing bytes (1, 2, or 3 bytes)
            let num_bytes = chunk.len();
            let num_chars = num_bytes + 1;

            // Construct the value from available bytes (big-endian, left-aligned)
            // Example: for 2 bytes [0xAB, 0xCD], treat as 0xABCD (16-bit value)
            let value = match num_bytes {
                1 => chunk[0] as u32,
                2 => u16::from_be_bytes([chunk[0], chunk[1]]) as u32,
                3 => {
                    // 3 bytes: construct 24-bit value
                    ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32)
                }
                _ => unreachable!(),
            };

            // Encode the partial block
            encode_partial_to_slice(value, num_chars, &mut output[out_idx..out_idx + num_chars]);
            out_idx += num_chars;
        }
    }

    // Convert to String (all characters are ASCII, so this is safe)
    String::from_utf8(output).expect("Z85 output should be valid UTF-8")
}

/// Encode a full 32-bit value into exactly 5 Z85 characters.
/// Fills the slice from right to left with base-85 digits.
fn encode_block_to_slice(mut value: u32, output: &mut [u8]) {
    debug_assert!(output.len() == 5);

    // Fill from right to left (least significant digit first)
    for i in (0..5).rev() {
        output[i] = Z85_ALPHABET[(value % 85) as usize];
        value /= 85;
    }
}

/// Encode a partial value (from 1-3 bytes) into the appropriate number of Z85 characters.
///
/// The math: for n input bytes, we need n+1 output characters.
/// We encode as if the value represents the most significant bits of a larger number.
fn encode_partial_to_slice(mut value: u32, num_chars: usize, output: &mut [u8]) {
    debug_assert!(num_chars >= 2 && num_chars <= 4);
    debug_assert!(output.len() == num_chars);

    // Fill from right to left
    for i in (0..num_chars).rev() {
        output[i] = Z85_ALPHABET[(value % 85) as usize];
        value /= 85;
    }
}

/// Decode a Z85 string back into bytes.
///
/// # Algorithm
///
/// For each 5-character block:
/// 1. Map each character to its base-85 digit value (0-84)
/// 2. Accumulate: value = d0*85^4 + d1*85^3 + d2*85^2 + d3*85 + d4
/// 3. Convert the u32 value to 4 big-endian bytes
///
/// For trailing characters (2-4 chars), we:
/// 1. Decode to get the partial value
/// 2. Extract only the appropriate number of bytes:
///    - 2 chars -> 1 byte
///    - 3 chars -> 2 bytes
///    - 4 chars -> 3 bytes
///
/// # Errors
///
/// - `InvalidCharacter`: Input contains a byte not in the Z85 alphabet
/// - `Overflow`: The accumulated value exceeds what can fit in the target bytes
/// - `InvalidLength`: Input length is 1 or 6 (mod 5), which can't map to valid byte counts
///
pub fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    let input = input.as_bytes();

    // Handle empty input
    if input.is_empty() {
        return Ok(Vec::new());
    }

    // Validate input length
    // Valid lengths: 0, 2, 3, 4, 5, 7, 8, 9, 10, 12, ...
    // Invalid: 1, 6, 11, 16, ... (1 mod 5 and 6 mod 5... wait, let's think about this)
    //
    // For n output bytes:
    // - 0 bytes -> 0 chars
    // - 1 byte  -> 2 chars
    // - 2 bytes -> 3 chars
    // - 3 bytes -> 4 chars
    // - 4 bytes -> 5 chars
    // - 5 bytes -> 7 chars (5 + 2)
    // - 6 bytes -> 8 chars (5 + 3)
    // ...
    //
    // So valid char counts: 0, 2, 3, 4, 5, 7, 8, 9, 10, 12, 13, 14, 15, 17, ...
    // Invalid: 1, 6, 11, 16, ... (i.e., 1 mod 5)
    let trailing_chars = input.len() % 5;
    if trailing_chars == 1 {
        return Err(DecodeError::InvalidLength);
    }

    // Calculate output size
    let full_blocks = input.len() / 5;
    let trailing_bytes = if trailing_chars > 0 { trailing_chars - 1 } else { 0 };
    let output_len = full_blocks * 4 + trailing_bytes;

    let mut output = vec![0u8; output_len];
    let mut out_idx = 0;
    let mut in_idx = 0;

    // Process full 5-character blocks
    while in_idx + 5 <= input.len() {
        let block = &input[in_idx..in_idx + 5];
        let value = decode_block(block)?;

        // Convert u32 to 4 big-endian bytes
        let bytes = value.to_be_bytes();
        output[out_idx..out_idx + 4].copy_from_slice(&bytes);

        in_idx += 5;
        out_idx += 4;
    }

    // Process trailing characters (2, 3, or 4 chars)
    if trailing_chars > 0 {
        let block = &input[in_idx..];
        let num_bytes = trailing_chars - 1;

        // Decode the partial block
        let value = decode_partial_block(block)?;

        // Extract the appropriate number of bytes (most significant first)
        // The value represents a number that should fit in num_bytes bytes
        match num_bytes {
            1 => {
                if value > 0xFF {
                    return Err(DecodeError::Overflow);
                }
                output[out_idx] = value as u8;
            }
            2 => {
                if value > 0xFFFF {
                    return Err(DecodeError::Overflow);
                }
                let bytes = (value as u16).to_be_bytes();
                output[out_idx..out_idx + 2].copy_from_slice(&bytes);
            }
            3 => {
                if value > 0xFFFFFF {
                    return Err(DecodeError::Overflow);
                }
                output[out_idx] = (value >> 16) as u8;
                output[out_idx + 1] = (value >> 8) as u8;
                output[out_idx + 2] = value as u8;
            }
            _ => unreachable!(),
        }
    }

    Ok(output)
}

/// Decode a full 5-character block into a u32.
/// Returns error if any character is invalid or if the value overflows u32.
fn decode_block(block: &[u8]) -> Result<u32, DecodeError> {
    debug_assert!(block.len() == 5);

    let mut value: u64 = 0;

    // Accumulate: value = d0*85^4 + d1*85^3 + d2*85^2 + d3*85 + d4
    // We use u64 to detect overflow before converting to u32
    for &byte in block {
        let digit = Z85_DECODE_TABLE[byte as usize];
        if digit == 0xFF {
            return Err(DecodeError::InvalidCharacter(byte));
        }
        value = value * 85 + digit as u64;
    }

    // Check for overflow (max valid Z85 5-char value is 85^5 - 1 = 4,437,053,124)
    // But we need it to fit in u32 (max 4,294,967,295 = 0xFFFFFFFF)
    if value > u32::MAX as u64 {
        return Err(DecodeError::Overflow);
    }

    Ok(value as u32)
}

/// Decode a partial block (2, 3, or 4 characters) into a u32.
/// The value represents a partial number (1, 2, or 3 bytes worth).
fn decode_partial_block(block: &[u8]) -> Result<u32, DecodeError> {
    debug_assert!(block.len() >= 2 && block.len() <= 4);

    let mut value: u64 = 0;

    for &byte in block {
        let digit = Z85_DECODE_TABLE[byte as usize];
        if digit == 0xFF {
            return Err(DecodeError::InvalidCharacter(byte));
        }
        value = value * 85 + digit as u64;
    }

    // No overflow check here - we check in the caller based on expected byte count
    Ok(value as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(encode(&[]), "");
        assert_eq!(decode("").unwrap(), vec![]);
    }

    #[test]
    fn test_zeros_4bytes() {
        assert_eq!(encode(&[0, 0, 0, 0]), "00000");
        assert_eq!(decode("00000").unwrap(), vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_max_4bytes() {
        assert_eq!(encode(&[0xff, 0xff, 0xff, 0xff]), "%nSc0");
        assert_eq!(decode("%nSc0").unwrap(), vec![0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn test_one_byte() {
        assert_eq!(encode(&[0x00]), "00");
        assert_eq!(decode("00").unwrap(), vec![0x00]);

        assert_eq!(encode(&[0xff]), "c%");
        assert_eq!(decode("c%").unwrap(), vec![0xff]);
    }

    #[test]
    fn test_roundtrip() {
        let test_cases: &[&[u8]] = &[
            &[],
            &[0],
            &[0, 0],
            &[0, 0, 0],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0, 0],
            &[0xff],
            &[0xff, 0xff],
            &[0xff, 0xff, 0xff],
            &[0xff, 0xff, 0xff, 0xff],
            &[1, 2, 3, 4],
            &[1, 2, 3, 4, 5, 6, 7, 8],
        ];

        for input in test_cases {
            let encoded = encode(input);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, *input, "roundtrip failed for {:?}", input);
        }
    }

    #[test]
    fn test_invalid_character() {
        assert!(matches!(decode("hello\"world"), Err(DecodeError::InvalidCharacter(b'"'))));
        assert!(matches!(decode("hel lo"), Err(DecodeError::InvalidCharacter(b' '))));
    }

    #[test]
    fn test_invalid_length() {
        // Length 1 is invalid
        assert!(matches!(decode("0"), Err(DecodeError::InvalidLength)));
        // Length 6 is invalid (would be 1 mod 5)
        assert!(matches!(decode("000000"), Err(DecodeError::InvalidLength)));
    }

    #[test]
    fn test_overflow() {
        // "#####" = 84*85^4 + 84*85^3 + 84*85^2 + 84*85 + 84 = 4,437,053,124 > 0xFFFFFFFF
        assert!(matches!(decode("#####"), Err(DecodeError::Overflow)));

        // "##" for 1 byte: 84*85 + 84 = 7224 > 255
        assert!(matches!(decode("##"), Err(DecodeError::Overflow)));
    }
}
