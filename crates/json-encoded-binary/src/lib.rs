//! JSON Encoded Binary (JEB85)
//!
//! A library for encoding binary data in JSON-compatible formats using Z85 with extensions.
//!
//! JEB85 supports two modes:
//! - **Text mode**: Valid UTF-8 strings without control characters (except those with single-char JSON escapes), ≤64 KiB
//! - **Binary mode**: Prefixed with `\b` (0x08), uses Z85 encoding with raw chunk extensions for readability
//!
//! See the [JEB85 specification](https://github.com/jeremyBanks/json-entity-bucket) for details.

#![warn(missing_docs)]

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while, take_while_m_n},
    character::complete::one_of,
    combinator::{map, map_res, opt, value},
    multi::many0,
    sequence::{preceded, tuple},
    IResult,
};
use nom_locate::LocatedSpan;
use nom_supreme::error::ErrorTree;

/// Maximum size for text mode strings (64 KiB)
pub const MAX_TEXT_SIZE: usize = 64 * 1024;

/// Default maximum chunk size for binary mode (64 KiB)
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Z85 alphabet (85 characters)
const Z85_ALPHABET: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/// Reverse lookup table for Z85 decoding
const Z85_DECODE: [u8; 256] = {
    let mut table = [255u8; 256];
    let mut i = 0;
    while i < 85 {
        table[Z85_ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    table
};

/// Error type for JEB85 operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Jeb85Error {
    /// Invalid Z85 character
    InvalidZ85Character(u8),
    /// Invalid input length for Z85 (must be multiple of 5)
    InvalidZ85Length,
    /// Invalid UTF-8 in text mode
    InvalidUtf8,
    /// Text too large for text mode
    TextTooLarge,
    /// Invalid raw block count
    InvalidRawBlockCount,
    /// Incomplete data
    IncompleteData,
    /// Parse error
    ParseError(String),
}

impl std::fmt::Display for Jeb85Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidZ85Character(c) => write!(f, "Invalid Z85 character: {}", c),
            Self::InvalidZ85Length => write!(f, "Invalid Z85 length (must be multiple of 5)"),
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8 in text mode"),
            Self::TextTooLarge => write!(f, "Text too large for text mode (max 64 KiB)"),
            Self::InvalidRawBlockCount => write!(f, "Invalid raw block count"),
            Self::IncompleteData => write!(f, "Incomplete data"),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for Jeb85Error {}

/// Represents a decoded JEB85 value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Jeb85Value {
    /// Text mode: plain UTF-8 string
    Text(String),
    /// Binary mode: arbitrary bytes
    Binary(Vec<u8>),
}

/// Encode 4 bytes as 5 Z85 characters
fn encode_z85_block(input: &[u8; 4]) -> [u8; 5] {
    let value = u32::from_be_bytes(*input);
    let mut output = [0u8; 5];
    let mut val = value;

    for i in (0..5).rev() {
        output[i] = Z85_ALPHABET[(val % 85) as usize];
        val /= 85;
    }

    output
}

/// Decode 5 Z85 characters to 4 bytes
fn decode_z85_block(input: &[u8; 5]) -> Result<[u8; 4], Jeb85Error> {
    let mut value = 0u32;

    for &byte in input {
        let digit = Z85_DECODE[byte as usize];
        if digit == 255 {
            return Err(Jeb85Error::InvalidZ85Character(byte));
        }
        value = value * 85 + digit as u32;
    }

    Ok(value.to_be_bytes())
}

/// Check if a byte is a JSON-safe printable ASCII character (no escape needed)
fn is_json_safe_ascii(byte: u8) -> bool {
    matches!(byte, 0x20..=0x21 | 0x23..=0x5B | 0x5D..=0x7E)
    // Excludes: 0x22 ("), 0x5C (\), and all control chars
}

/// Check if data should use text mode
fn should_use_text_mode(data: &[u8]) -> bool {
    if data.len() > MAX_TEXT_SIZE {
        return false;
    }

    // Check if valid UTF-8
    let Ok(s) = std::str::from_utf8(data) else {
        return false;
    };

    // Check for prohibited control characters
    for byte in data {
        match byte {
            // Allow these control chars (have single-char JSON escapes)
            b'\t' | b'\n' | b'\r' => continue,
            // Prohibit backspace and other control chars
            0x00..=0x08 | 0x0E..=0x1F | 0x7F => return false,
            _ => continue,
        }
    }

    true
}

/// Encode data using JEB85
pub fn encode(data: &[u8]) -> String {
    if should_use_text_mode(data) {
        // Text mode: return as-is (will be JSON-escaped by the JSON encoder)
        return String::from_utf8(data.to_vec()).expect("validated UTF-8");
    }

    // Binary mode: prefix with \b and encode
    let mut result = String::from("\u{0008}"); // \b character
    encode_binary(data, &mut result);
    result
}

/// Encode binary data (without the \b prefix)
fn encode_binary(data: &[u8], output: &mut String) {
    let mut i = 0;

    while i < data.len() {
        let remaining = data.len() - i;
        let chunk_size = remaining.min(DEFAULT_CHUNK_SIZE);
        let chunk = &data[i..i + chunk_size];

        // Process chunk in 4-byte blocks
        let mut j = 0;
        while j < chunk.len() {
            let block_remaining = chunk.len() - j;
            let block_size = block_remaining.min(4);
            let block = &chunk[j..j + block_size];

            // Check if this 4-byte block is all JSON-safe ASCII
            if block.len() == 4 && block.iter().all(|&b| is_json_safe_ascii(b)) {
                // Emit as raw chunk
                output.push('|');
                output.push_str(std::str::from_utf8(block).unwrap());
            } else {
                // Pad to 4 bytes if needed and encode as Z85
                let mut padded = [0u8; 4];
                padded[..block_size].copy_from_slice(block);
                let encoded = encode_z85_block(&padded);

                // Output only the needed characters (5 for full block, less for final partial)
                let output_len = if block_size == 4 { 5 } else {
                    // For partial blocks, calculate output length
                    // This is approximate - Z85 doesn't have a standard for partial blocks
                    ((block_size * 5) + 3) / 4
                };
                output.push_str(std::str::from_utf8(&encoded[..output_len]).unwrap());
            }

            j += block_size;
        }

        i += chunk_size;
    }
}

/// Parse a Z85 block (5 characters)
fn parse_z85_block(input: &[u8]) -> IResult<&[u8], [u8; 4]> {
    map_res(
        take_while_m_n(5, 5, |c: u8| Z85_DECODE[c as usize] != 255),
        |bytes: &[u8]| {
            let mut arr = [0u8; 5];
            arr.copy_from_slice(bytes);
            decode_z85_block(&arr)
        },
    )(input)
}

/// Parse a single raw block (|xxxx)
fn parse_single_raw_block(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    preceded(
        tag(b"|"),
        map(take(4usize), |bytes: &[u8]| bytes.to_vec()),
    )(input)
}

/// Parse a terminal raw block (||...)
fn parse_terminal_raw_block(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    preceded(
        tag(b"||"),
        map(take_while(|_| true), |bytes: &[u8]| bytes.to_vec()),
    )(input)
}

/// Parse a multi-block raw chunk (N|...)
fn parse_multi_raw_block(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, count_chars) = take_while_m_n(1, 4, |c: u8| Z85_DECODE[c as usize] != 255)(input)?;
    let (input, _) = tag(b"|")(input)?;

    // Decode the count (number of blocks - 2)
    let mut count_value = 0u32;
    for &byte in count_chars {
        let digit = Z85_DECODE[byte as usize];
        count_value = count_value * 85 + digit as u32;
    }
    let num_blocks = count_value as usize + 2;

    // Read num_blocks * 4 bytes, accounting for padding
    let (input, bytes) = take(num_blocks * 4)(input)?;

    // Remove trailing padding (.) characters
    let mut result = bytes.to_vec();
    while result.last() == Some(&b'.') {
        result.pop();
    }

    Ok((input, result))
}

/// Parse a binary mode chunk (Z85 block or raw chunk)
fn parse_binary_chunk(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    alt((
        parse_terminal_raw_block,
        parse_multi_raw_block,
        parse_single_raw_block,
        map(parse_z85_block, |block| block.to_vec()),
    ))(input)
}

/// Parse binary mode data (after the \b prefix)
fn parse_binary_mode(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, chunks) = many0(parse_binary_chunk)(input)?;

    // Flatten chunks into a single byte vector
    let mut result = Vec::new();
    for chunk in chunks {
        result.extend_from_slice(&chunk);
    }

    Ok((input, result))
}

/// Decode a JEB85-encoded string
pub fn decode(input: &str) -> Result<Vec<u8>, Jeb85Error> {
    let bytes = input.as_bytes();

    // Check for binary mode prefix (\b = 0x08)
    if bytes.starts_with(&[0x08]) {
        let (_remaining, data) = parse_binary_mode(&bytes[1..])
            .map_err(|e| Jeb85Error::ParseError(e.to_string()))?;
        Ok(data)
    } else {
        // Text mode: validate UTF-8 and return as-is
        if !should_use_text_mode(bytes) {
            return Err(Jeb85Error::InvalidUtf8);
        }
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z85_encode_decode() {
        let input = [0x86, 0x4F, 0xD2, 0x6F];
        let encoded = encode_z85_block(&input);
        assert_eq!(&encoded, b"HelloW");

        let decoded = decode_z85_block(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_text_mode_simple() {
        let data = b"Hello, World!";
        let encoded = encode(data);
        assert_eq!(encoded, "Hello, World!");

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_text_mode_with_escapes() {
        let data = b"Line 1\nLine 2\tTabbed";
        let encoded = encode(data);
        assert_eq!(encoded, "Line 1\nLine 2\tTabbed");

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_binary_mode_basic() {
        let data = b"\x00\x01\x02\x03";
        let encoded = encode(data);
        assert!(encoded.starts_with('\u{0008}'));

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_binary_mode_with_ascii() {
        let data = b"Test";
        // Even though it's ASCII, if it triggers binary mode, should work
        let encoded = format!("\u{0008}|Test");
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_round_trip_empty() {
        let data = b"";
        let encoded = encode(data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_round_trip_binary() {
        let data = b"\x00\x01\x02\x03\x04\x05\x06\x07";
        let encoded = encode(data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_should_use_text_mode() {
        assert!(should_use_text_mode(b"Hello"));
        assert!(should_use_text_mode(b"Hello\nWorld"));
        assert!(!should_use_text_mode(b"Hello\x00World")); // null byte
        assert!(!should_use_text_mode(b"Hello\x08World")); // backspace
        assert!(!should_use_text_mode(&vec![b'a'; MAX_TEXT_SIZE + 1])); // too large
    }

    #[test]
    fn test_is_json_safe_ascii() {
        assert!(is_json_safe_ascii(b'a'));
        assert!(is_json_safe_ascii(b'Z'));
        assert!(is_json_safe_ascii(b' '));
        assert!(is_json_safe_ascii(b'!'));
        assert!(!is_json_safe_ascii(b'"')); // needs escape
        assert!(!is_json_safe_ascii(b'\\')); // needs escape
        assert!(!is_json_safe_ascii(b'\n')); // control char
        assert!(!is_json_safe_ascii(0x00)); // control char
    }
}
