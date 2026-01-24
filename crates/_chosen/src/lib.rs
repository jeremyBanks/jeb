//! Internal choices for algorithms and representations that could be changed
//! without breaking external APIs, or breaking any of our internal flows if
//! data is migrated and required properties are maintained.

/// Encodes arbitrary bytes as text using ideated-encoding (extended Z85).
///
/// This provides a compact text representation of binary data with ~25% overhead,
/// while preserving readable ASCII sequences where possible.
pub fn bytes_to_text(bytes: &[u8]) -> String {
    // ideated-encoding output is always valid ASCII/UTF-8
    String::from_utf8(ideated_encoding::encode(bytes)).expect("ideated-encoding produces valid UTF-8")
}

/// Decodes text produced by [`bytes_to_text`] back into bytes.
///
/// Returns `None` if the input is malformed.
pub fn text_to_bytes(text: &str) -> Option<Vec<u8>> {
    ideated_encoding::decode(text.as_bytes()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty() {
        let bytes = b"";
        let encoded = bytes_to_text(bytes);
        assert_eq!(text_to_bytes(&encoded), Some(bytes.to_vec()));
    }

    #[test]
    fn roundtrip_printable() {
        let bytes = b"Hello, World!";
        let encoded = bytes_to_text(bytes);
        assert_eq!(text_to_bytes(&encoded), Some(bytes.to_vec()));
    }

    #[test]
    fn roundtrip_binary() {
        let bytes: Vec<u8> = (0..=255).collect();
        let encoded = bytes_to_text(&bytes);
        assert_eq!(text_to_bytes(&encoded), Some(bytes));
    }

    #[test]
    fn invalid_input() {
        // Characters outside Z85 alphabet (like space or quotes)
        assert_eq!(text_to_bytes("\"invalid\""), None);
    }
}
