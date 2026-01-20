//! Z85 alphabet and escape character definitions.

/// The Z85 alphabet (85 characters).
/// Characters 0-9, a-z, A-Z, and 23 special characters.
pub const Z85_ALPHABET: &[u8; 85] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/// Escape character: comma - LE 4 raw bytes (positions 1-3), or 4 raw (position 0)
pub const ESCAPE_COMMA: u8 = b',';

/// Escape character: backtick - BE 4 raw bytes (positions 1-3), or 3 raw (position 0)
pub const ESCAPE_BACKTICK: u8 = b'`';

/// Escape character: semicolon - LE 6 raw bytes (positions 1-3), or 6 raw (position 0)
pub const ESCAPE_SEMICOLON: u8 = b';';

/// Escape character: tilde - BE 6 raw bytes (positions 1-3), or 5 raw (position 0)
pub const ESCAPE_TILDE: u8 = b'~';

/// Escape character: underscore - LE 7 raw bytes (positions 1-3), or 7 raw (position 0)
pub const ESCAPE_UNDERSCORE: u8 = b'_';

/// Escape character: pipe - Variable length (8+ bytes), any position
pub const ESCAPE_PIPE: u8 = b'|';

/// Padding character (used after raw sequences to maintain alignment)
pub const PADDING_CHAR: u8 = b'.';

/// All escape characters (6 total, outside Z85 alphabet)
pub const ESCAPE_CHARS: [u8; 6] = [
    ESCAPE_COMMA,
    ESCAPE_BACKTICK,
    ESCAPE_SEMICOLON,
    ESCAPE_TILDE,
    ESCAPE_UNDERSCORE,
    ESCAPE_PIPE,
];

/// Lookup table: Z85 character -> digit value (255 = invalid)
pub static Z85_DECODE: [u8; 256] = {
    let mut table = [255u8; 256];
    let mut i = 0;
    while i < 85 {
        table[Z85_ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    table
};

/// Check if a byte is a valid Z85 digit character.
#[inline]
pub fn is_z85_char(byte: u8) -> bool {
    Z85_DECODE[byte as usize] != 255
}

/// Check if a byte is an escape character.
#[inline]
pub fn is_escape_char(byte: u8) -> bool {
    matches!(
        byte,
        ESCAPE_COMMA
            | ESCAPE_BACKTICK
            | ESCAPE_SEMICOLON
            | ESCAPE_TILDE
            | ESCAPE_UNDERSCORE
            | ESCAPE_PIPE
    )
}

/// Check if a byte is "safe" for raw passthrough.
/// This includes all Z85 characters plus all escape characters (91 total).
/// The encoder restricts to these; the decoder accepts any byte in raw mode.
#[inline]
pub fn is_safe_for_raw(byte: u8) -> bool {
    is_z85_char(byte) || is_escape_char(byte)
}

/// Get the Z85 digit value for a character, or None if invalid.
#[inline]
pub fn z85_digit_value(byte: u8) -> Option<u8> {
    let value = Z85_DECODE[byte as usize];
    if value == 255 { None } else { Some(value) }
}

/// Get the Z85 character for a digit value (0-84).
/// Panics if value >= 85.
#[inline]
pub fn z85_digit_char(value: u8) -> u8 {
    debug_assert!(value < 85, "Z85 digit value must be < 85");
    Z85_ALPHABET[value as usize]
}

/// Encode bytes as a Z85 4-byte block -> 5 chars.
/// Input must be exactly 4 bytes.
pub fn encode_z85_block(bytes: &[u8; 4]) -> [u8; 5] {
    let value = u32::from_be_bytes(*bytes);
    let mut result = [0u8; 5];
    let mut v = value;
    for i in (0..5).rev() {
        result[i] = z85_digit_char((v % 85) as u8);
        v /= 85;
    }
    result
}

/// Decode a Z85 5-char block -> 4 bytes.
/// Returns None if any character is invalid.
pub fn decode_z85_block(chars: &[u8; 5]) -> Option<[u8; 4]> {
    let mut value: u32 = 0;
    for &c in chars {
        let digit = z85_digit_value(c)?;
        value = value.checked_mul(85)?.checked_add(digit as u32)?;
    }
    Some(value.to_be_bytes())
}

/// Raw byte counts for standard escapes at position 0.
/// Returns None if the escape is not valid at position 0 (only `|` is valid elsewhere).
pub fn escape_raw_bytes_at_position_0(escape: u8) -> Option<usize> {
    match escape {
        ESCAPE_BACKTICK => Some(3),
        ESCAPE_COMMA => Some(4),
        ESCAPE_TILDE => Some(5),
        ESCAPE_SEMICOLON => Some(6),
        ESCAPE_UNDERSCORE => Some(7),
        _ => None, // ESCAPE_PIPE is special, handled differently
    }
}

/// Information about a standard escape at positions 1-3.
/// Returns (raw_bytes, is_little_endian).
pub fn escape_info_at_position_1_to_3(escape: u8) -> Option<(usize, bool)> {
    match escape {
        ESCAPE_COMMA => Some((4, true)),     // LE
        ESCAPE_BACKTICK => Some((4, false)), // BE
        ESCAPE_SEMICOLON => Some((6, true)), // LE
        ESCAPE_TILDE => Some((6, false)),    // BE
        ESCAPE_UNDERSCORE => Some((7, true)), // LE only, no BE variant
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z85_alphabet_length() {
        assert_eq!(Z85_ALPHABET.len(), 85);
    }

    #[test]
    fn test_z85_alphabet_unique() {
        let mut seen = [false; 256];
        for &c in Z85_ALPHABET {
            assert!(!seen[c as usize], "duplicate character in Z85: {}", c as char);
            seen[c as usize] = true;
        }
    }

    #[test]
    fn test_escape_chars_outside_z85() {
        for &escape in &ESCAPE_CHARS {
            assert!(
                !is_z85_char(escape),
                "escape char '{}' should not be in Z85",
                escape as char
            );
        }
    }

    #[test]
    fn test_safe_for_raw_count() {
        let count = (0u8..=255).filter(|&b| is_safe_for_raw(b)).count();
        assert_eq!(count, 91, "should have 85 Z85 + 6 escape = 91 safe chars");
    }

    #[test]
    fn test_z85_roundtrip() {
        for digit in 0..85 {
            let c = z85_digit_char(digit);
            let decoded = z85_digit_value(c).unwrap();
            assert_eq!(digit, decoded);
        }
    }

    #[test]
    fn test_encode_decode_z85_block() {
        let bytes = [0x86, 0x4F, 0xD2, 0x6F];
        let encoded = encode_z85_block(&bytes);
        let decoded = decode_z85_block(&encoded).unwrap();
        assert_eq!(bytes, decoded);
    }

    #[test]
    fn test_z85_known_value() {
        // From ZeroMQ spec: HelloWorld -> xK#0@zY<mym
        // But that's for "HelloWorld" which is 10 bytes = not aligned
        // Let's use a simpler test: 0x00000000 -> "00000"
        let zeros = [0u8; 4];
        let encoded = encode_z85_block(&zeros);
        assert_eq!(&encoded, b"00000");

        // And max value: 0xFFFFFFFF -> "#####" (84 in all positions)
        let max = [0xFFu8; 4];
        let encoded = encode_z85_block(&max);
        // 0xFFFFFFFF = 4294967295
        // 4294967295 / 85^4 = 82 remainder ...
        // Actually let's just verify roundtrip
        let decoded = decode_z85_block(&encoded).unwrap();
        assert_eq!(max, decoded);
    }
}
