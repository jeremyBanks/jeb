//! Internal choices for algorithms and representations that could be changed
//! without breaking external APIs, or breaking any of our internal flows if
//! data is migrated and required properties are maintained.

/// Encodes arbitrary bytes as text using JSON-style escaping (without quotes).
///
/// The encoding:
/// - Uses JSON standard escapes: `\n`, `\t`, `\r`, `\\`
/// - Uses `\xHH` hex escapes for other non-printable ASCII (0x00-0x1F, 0x7F)
/// - Uses `\xHH` hex escapes for high bytes (0x80-0xFF)
/// - Printable ASCII (0x20-0x7E except `\`) pass through directly
pub fn bytes_to_text(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        match b {
            b'\n' => result.push_str("\\n"),
            b'\t' => result.push_str("\\t"),
            b'\r' => result.push_str("\\r"),
            b'\\' => result.push_str("\\\\"),
            0x20..=0x5B | 0x5D..=0x7E => {
                // Printable ASCII except \ (0x5C)
                result.push(b as char);
            }
            _ => {
                // Control chars, DEL, or high bytes: use \xHH
                result.push_str(&format!("\\x{:02X}", b));
            }
        }
    }
    result
}

/// Decodes text produced by [`bytes_to_text`] back into bytes.
///
/// Handles:
/// - JSON escapes: `\n`, `\t`, `\r`, `\\`
/// - Hex escapes: `\xHH`
///
/// Returns `None` if the input is malformed.
pub fn text_to_bytes(text: &str) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            let escaped = chars.next()?;
            match escaped {
                'n' => result.push(b'\n'),
                't' => result.push(b'\t'),
                'r' => result.push(b'\r'),
                '\\' => result.push(b'\\'),
                'x' => {
                    // \xHH
                    let h1 = chars.next()?;
                    let h2 = chars.next()?;
                    let hex: String = [h1, h2].iter().collect();
                    let byte = u8::from_str_radix(&hex, 16).ok()?;
                    result.push(byte);
                }
                _ => return None, // Unknown escape
            }
        } else if c.is_ascii() {
            result.push(c as u8);
        } else {
            return None; // Non-ASCII not allowed
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
        assert_eq!(encoded, "Hello, World!");
        assert_eq!(text_to_bytes(&encoded), Some(bytes.to_vec()));
    }

    #[test]
    fn roundtrip_escapes() {
        let bytes = b"line1\nline2\ttab\\slash";
        let encoded = bytes_to_text(bytes);
        assert_eq!(encoded, r"line1\nline2\ttab\\slash");
        assert_eq!(text_to_bytes(&encoded), Some(bytes.to_vec()));
    }

    #[test]
    fn roundtrip_binary() {
        let bytes: Vec<u8> = (0..=255).collect();
        let encoded = bytes_to_text(&bytes);
        assert_eq!(text_to_bytes(&encoded), Some(bytes));
    }

    #[test]
    fn hex_escapes() {
        let bytes = &[0x00, 0x1F, 0x7F, 0x80, 0xFF];
        let encoded = bytes_to_text(bytes);
        assert_eq!(encoded, r"\x00\x1F\x7F\x80\xFF");
        assert_eq!(text_to_bytes(&encoded), Some(bytes.to_vec()));
    }

    #[test]
    fn invalid_trailing_backslash() {
        assert_eq!(text_to_bytes(r"\"), None);
    }

    #[test]
    fn invalid_escape() {
        assert_eq!(text_to_bytes(r"\q"), None);
    }

    #[test]
    fn invalid_hex() {
        assert_eq!(text_to_bytes(r"\xGG"), None);
    }
}
