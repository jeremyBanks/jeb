//! JSON Encoded Binary (JEB85)
//!
//! A library for encoding binary data in JSON-compatible formats using Z85 with extensions.
//!
//! JEB85 supports two modes:
//! - **Text mode**: Valid UTF-8 strings without control characters (except those with single-char JSON escapes), ≤64 KiB
//! - **Binary mode**: Prefixed with `\b` (0x08), uses Z85 encoding with raw chunk extensions for readability
//!
//! # Encoding Strategy
//!
//! Binary mode uses Z85 encoding with special raw chunk markers for preserving readable ASCII:
//! - Single raw block: `|xxxx` (4 bytes of raw data)
//! - Multi-block raw: `N|xxxx...` where N is Z85-encoded (count-2), followed by count×4 bytes
//! - Terminal raw: `||...` (rest of data is raw, no length limit)
//!
//! All raw chunks are padded with `.` to maintain 5-character block alignment.

#![warn(missing_docs)]

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while_m_n},
    combinator::{map, map_res},
    multi::many0,
    sequence::preceded,
    IResult,
};

// For future use: better error handling and position tracking
#[allow(unused_imports)]
use nom_locate::LocatedSpan;
#[allow(unused_imports)]
use nom_supreme::error::ErrorTree;

/// Maximum size for text mode strings (64 KiB)
pub const MAX_TEXT_SIZE: usize = 64 * 1024;

/// Default maximum chunk size for binary mode (64 KiB)
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Z85 alphabet (85 characters) - note that | is NOT in this alphabet
pub const Z85_ALPHABET: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

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
    /// Invalid UTF-8 in text mode
    InvalidUtf8,
    /// Text contains prohibited control characters
    ProhibitedControlChar(u8),
    /// Text too large for text mode
    TextTooLarge,
    /// Parse error
    ParseError(String),
}

impl std::fmt::Display for Jeb85Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidZ85Character(c) => write!(f, "Invalid Z85 character: 0x{:02X}", c),
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8 in text mode"),
            Self::ProhibitedControlChar(c) => {
                write!(f, "Prohibited control character: 0x{:02X}", c)
            }
            Self::TextTooLarge => write!(f, "Text too large for text mode (max {} bytes)", MAX_TEXT_SIZE),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for Jeb85Error {}

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

/// Encode a count as Z85 digits (1-4 characters)
fn encode_z85_count(count: usize) -> Vec<u8> {
    let mut result = Vec::new();
    let mut val = count;

    if val == 0 {
        return vec![Z85_ALPHABET[0]];
    }

    while val > 0 {
        result.push(Z85_ALPHABET[val % 85]);
        val /= 85;
    }

    result.reverse();
    result
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
    if std::str::from_utf8(data).is_err() {
        return false;
    }

    // Check for prohibited control characters
    for &byte in data {
        match byte {
            // Allow only the three common whitespace control chars
            b'\t' | b'\n' | b'\r' => continue,
            // Prohibit backspace (our binary marker) and all other control chars
            0x00..=0x08 | 0x0B..=0x1F | 0x7F => return false,
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
        // Look ahead to find runs of JSON-safe blocks
        let mut run_start = i;
        let mut run_blocks = 0;

        while i < data.len() && (i - run_start) / 4 < 85usize.pow(4) {
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
            if run_blocks == 1 {
                // Single block: |xxxx
                output.push('|');
                output.push_str(std::str::from_utf8(&data[run_start..run_start + 4]).unwrap());
            } else {
                // Multi-block: N|xxxx... (N = blocks - 2)
                let count_encoded = encode_z85_count(run_blocks - 2);
                output.push_str(std::str::from_utf8(&count_encoded).unwrap());
                output.push('|');

                // Emit the raw blocks
                let raw_len = run_blocks * 4;
                output.push_str(std::str::from_utf8(&data[run_start..run_start + raw_len]).unwrap());

                // Pad to 5-char alignment if needed
                let total_len = count_encoded.len() + 1 + raw_len; // count + | + data
                let padding_needed = (5 - (total_len % 5)) % 5;
                for _ in 0..padding_needed {
                    output.push('.');
                }
            }
        } else {
            // No raw run, encode as Z85
            let block_end = (i + 4).min(data.len());
            let block_size = block_end - i;

            if block_size == 4 {
                // Full block
                let mut block_data = [0u8; 4];
                block_data.copy_from_slice(&data[i..i + 4]);
                let encoded = encode_z85_block(&block_data);
                output.push_str(std::str::from_utf8(&encoded).unwrap());
                i += 4;
            } else {
                // Partial block at end - use terminal raw chunk
                output.push_str("||");
                // Emit remaining bytes as-is (may not be valid UTF-8, but that's okay for raw data)
                for &byte in &data[i..] {
                    output.push(byte as char);
                }
                i = data.len();
            }
        }
    }
}

/// Parse a Z85 block (5 characters)
fn parse_z85_block(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    map_res(
        take_while_m_n(5, 5, |c: u8| Z85_DECODE[c as usize] != 255),
        |bytes: &[u8]| {
            let mut arr = [0u8; 5];
            arr.copy_from_slice(bytes);
            decode_z85_block(&arr).map(|b| b.to_vec())
        },
    )(input)
}

/// Parse a single raw block (|xxxx)
fn parse_single_raw_block(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    preceded(tag(b"|"), map(take(4usize), |bytes: &[u8]| bytes.to_vec()))(input)
}

/// Parse a terminal raw block (||...)
fn parse_terminal_raw_block(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, _) = tag(b"||")(input)?;
    // Take everything remaining
    Ok((&b""[..], input.to_vec()))
}

/// Parse a multi-block raw chunk (N|...)
fn parse_multi_raw_block(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, count_chars) = take_while_m_n(1, 4, |c: u8| Z85_DECODE[c as usize] != 255)(input)?;
    let (input, _) = tag(b"|")(input)?;

    // Decode the count (number of blocks - 2)
    let mut count_value = 0usize;
    for &byte in count_chars {
        let digit = Z85_DECODE[byte as usize] as usize;
        count_value = count_value * 85 + digit;
    }
    let num_blocks = count_value + 2;

    // Calculate total chars needed (data + padding to 5-char alignment)
    let raw_bytes = num_blocks * 4;
    let prefix_len = count_chars.len() + 1; // count + |
    let total_len = prefix_len + raw_bytes;
    let padding_needed = (5 - (total_len % 5)) % 5;
    let chars_to_read = raw_bytes + padding_needed;

    // Read the data + padding
    let (input, bytes) = take(chars_to_read)(input)?;

    // Remove trailing padding (.) characters
    let mut result = bytes.to_vec();
    while result.last() == Some(&b'.') && result.len() > raw_bytes {
        result.pop();
    }

    // Trim to exact size
    result.truncate(raw_bytes);

    Ok((input, result))
}

/// Parse a binary mode chunk (Z85 block or raw chunk)
fn parse_binary_chunk(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    alt((
        parse_terminal_raw_block,
        parse_multi_raw_block,
        parse_single_raw_block,
        parse_z85_block,
    ))(input)
}

/// Parse binary mode data (after the \b prefix)
fn parse_binary_mode(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, chunks) = many0(parse_binary_chunk)(input)?;

    // Flatten chunks into a single byte vector
    let result = chunks.into_iter().flatten().collect();

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
        // Text mode: validate and return as-is
        if !should_use_text_mode(bytes) {
            // Check specifically what failed
            if bytes.len() > MAX_TEXT_SIZE {
                return Err(Jeb85Error::TextTooLarge);
            }
            if std::str::from_utf8(bytes).is_err() {
                return Err(Jeb85Error::InvalidUtf8);
            }
            // Must be a prohibited control char
            for &byte in bytes {
                if !matches!(byte, b'\t' | b'\n' | b'\r') && byte < 0x20 || byte == 0x7F {
                    return Err(Jeb85Error::ProhibitedControlChar(byte));
                }
            }
            return Err(Jeb85Error::InvalidUtf8); // Fallback
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
    fn test_form_feed_forces_binary() {
        // Form feed (\f) should force binary mode
        let data = b"Before\x0CAfter";
        let encoded = encode(data);

        // Should use binary mode (contains backspace marker)
        assert!(encoded.contains('\x08'), "Form feed should force binary mode");

        // Should round-trip correctly
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
        let encoded = encode(data);
        // ASCII should trigger text mode
        assert_eq!(encoded, "Test");

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_binary_with_embedded_text() {
        // Mix of binary and text-safe blocks
        let mut data = vec![0x00, 0x01, 0x02, 0x03]; // Binary
        data.extend_from_slice(b"Test");              // Text-safe
        data.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0xFC]); // Binary

        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_multi_block_raw() {
        // 8 bytes of JSON-safe ASCII (2 blocks)
        let data = b"TestData";
        // Force binary mode by adding a null byte
        let mut binary_data = vec![0x00];
        binary_data.extend_from_slice(data);

        let encoded = encode(&binary_data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, binary_data);
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
    fn test_round_trip_partial_block() {
        // 5 bytes - not a multiple of 4
        let data = b"\x00\x01\x02\x03\x04";
        let encoded = encode(data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data, "Failed to round-trip partial block");
    }

    #[test]
    fn test_should_use_text_mode() {
        assert!(should_use_text_mode(b"Hello"));
        assert!(should_use_text_mode(b"Hello\nWorld"));
        assert!(should_use_text_mode(b"Hello\tWorld")); // tab is allowed
        assert!(should_use_text_mode(b"Hello\rWorld")); // carriage return is allowed
        assert!(!should_use_text_mode(b"Hello\x00World")); // null byte
        assert!(!should_use_text_mode(b"Hello\x08World")); // backspace
        assert!(!should_use_text_mode(b"Hello\x0CWorld")); // form feed (\f) - rejected
        assert!(!should_use_text_mode(b"Hello\x0BWorld")); // vertical tab (\v) - rejected
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

    #[test]
    fn test_terminal_chunk() {
        // Test terminal chunk with non-UTF8 bytes
        let data = vec![0xFF, 0xFE, 0xFD];
        let encoded = encode(&data);
        assert!(encoded.contains("||"));
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_large_raw_run() {
        // Test many consecutive JSON-safe blocks
        let mut data = Vec::new();
        for _ in 0..10 {
            data.extend_from_slice(b"Test");
        }
        // Force binary mode
        data.insert(0, 0x00);

        let encoded = encode(&data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_count_encoding() {
        assert_eq!(encode_z85_count(0), vec![Z85_ALPHABET[0]]);
        assert_eq!(encode_z85_count(1), vec![Z85_ALPHABET[1]]);
        assert_eq!(encode_z85_count(84), vec![Z85_ALPHABET[84]]);
        // 85 = "10" in base85
        assert_eq!(encode_z85_count(85), vec![Z85_ALPHABET[1], Z85_ALPHABET[0]]);
    }
}
