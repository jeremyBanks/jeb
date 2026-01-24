//! Internal choices for algorithms and representations that could be changed
//! without breaking external APIs, or breaking any of our internal flows if
//! data is migrated and required properties are maintained.

/// Encodes arbitrary bytes as text using an augmented hex encoding.
///
/// The encoding scheme:
/// - Graphic ASCII (0x21-0x7E visible glyphs): space + character (e.g., ` A`)
/// - Tab (0x09): `\t`
/// - Newline (0x0A): `\n`
/// - Carriage return (0x0D): `\r`
/// - All other bytes (control chars, space, high bytes): uppercase hex (e.g., `00`, `FF`)
///
/// This encoding maintains a consistent 2-character width per byte while keeping
/// printable characters readable.
pub fn bytes_to_text(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| match b {
            0x09 => r"\t".to_string(),
            0x0A => r"\n".to_string(),
            0x0D => r"\r".to_string(),
            0x21..=0x7E => format!(" {}", b as char),
            _ => format!("{:02X}", b),
        })
        .collect()
}

/// Decodes text produced by [`bytes_to_text`] back into bytes.
///
/// The decoding scheme:
/// - ` X` (space + char): literal ASCII character (0x21-0x7E only)
/// - `\n`, `\t`, `\r`: respective escape sequences
/// - `XX` (two hex digits): byte value
///
/// Returns `None` if the input is malformed.
pub fn text_to_bytes(text: &str) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == ' ' {
            // Space followed by literal character
            if i + 1 >= chars.len() {
                return None; // trailing space
            }
            let c = chars[i + 1];
            // Only accept 0x21-0x7E (graphic ASCII)
            if !c.is_ascii() || (c as u8) < 0x21 || (c as u8) > 0x7E {
                return None; // invalid literal character
            }
            result.push(c as u8);
            i += 2;
        } else if chars[i] == '\\' {
            // Backslash escape sequence
            if i + 1 >= chars.len() {
                return None; // trailing backslash
            }
            let escaped = chars[i + 1];
            let byte = match escaped {
                'n' => 0x0A,
                't' => 0x09,
                'r' => 0x0D,
                _ => return None, // unknown escape
            };
            result.push(byte);
            i += 2;
        } else {
            // Two hex digits
            if i + 1 >= chars.len() {
                return None; // odd number of hex characters
            }
            let hex: String = chars[i..i + 2].iter().collect();
            let byte = u8::from_str_radix(&hex, 16).ok()?;
            result.push(byte);
            i += 2;
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty() {
        let bytes = b"";
        let encoded = bytes_to_text(bytes);
        assert_eq!(encoded, "");
        assert_eq!(text_to_bytes(&encoded), Some(bytes.to_vec()));
    }

    #[test]
    fn roundtrip_printable() {
        let bytes = b"Hello, World!";
        let encoded = bytes_to_text(bytes);
        assert_eq!(encoded, " H e l l o ,20 W o r l d !");
        assert_eq!(text_to_bytes(&encoded), Some(bytes.to_vec()));
    }

    #[test]
    fn roundtrip_escapes() {
        let bytes = b"a\t\n\rb";
        let encoded = bytes_to_text(bytes);
        assert_eq!(encoded, r" a\t\n\r b");
        assert_eq!(text_to_bytes(&encoded), Some(bytes.to_vec()));
    }

    #[test]
    fn roundtrip_binary() {
        let bytes: Vec<u8> = (0..=255).collect();
        let encoded = bytes_to_text(&bytes);
        assert_eq!(text_to_bytes(&encoded), Some(bytes));
    }

    #[test]
    fn invalid_trailing_space() {
        assert_eq!(text_to_bytes(" "), None);
    }

    #[test]
    fn invalid_trailing_backslash() {
        assert_eq!(text_to_bytes(r"\"), None);
    }

    #[test]
    fn invalid_escape() {
        assert_eq!(text_to_bytes(r"\x"), None);
    }

    #[test]
    fn invalid_hex() {
        assert_eq!(text_to_bytes("GG"), None);
    }

    #[test]
    fn invalid_odd_hex() {
        assert_eq!(text_to_bytes("0"), None);
    }
}
