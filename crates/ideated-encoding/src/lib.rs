//! Extended Z85 encoding with raw passthrough support.
//!
//! This crate implements an extension to Z85 encoding that allows raw
//! (unencoded) byte sequences to pass through while maintaining the standard
//! Z85 overhead guarantees of +25% (4 bytes → 5 characters).
//!
//! # Overview
//!
//! The encoding uses 6 escape characters outside the Z85 alphabet to signal
//! transitions between encoded and raw data:
//!
//! - `,` (comma) - 4 raw bytes
//! - `` ` `` (backtick) - 4 raw bytes (or 3 at position 0)
//! - `;` (semicolon) - 6 raw bytes
//! - `~` (tilde) - 6 raw bytes (or 5 at position 0)
//! - `_` (underscore) - 7 raw bytes
//! - `|` (pipe) - Variable length (8+ bytes)
//!
//! # Usage
//!
//! ## Simple encoding/decoding
//!
//! ```
//! use ideated_encoding::{
//!     decode,
//!     encode,
//! };
//!
//! let original = b"Hello, World!";
//! let encoded = encode(original);
//! let decoded = decode(&encoded).unwrap();
//! assert_eq!(decoded, original);
//! ```
//!
//! ## Streaming encoding
//!
//! ```
//! use ideated_encoding::Encoder;
//!
//! let mut encoder = Encoder::new();
//! encoder.write(b"Hello");
//! encoder.write(b", World!");
//! let encoded = encoder.finish();
//! ```
//!
//! ## Streaming decoding
//!
//! ```
//! use ideated_encoding::Decoder;
//!
//! let mut decoder = Decoder::new();
//! decoder.write(b"HelloWorld").unwrap();
//! let decoded = decoder.finish().unwrap();
//! ```
//!
//! # Design
//!
//! The encoding maintains block alignment (5-character blocks) throughout the
//! stream. Raw byte sequences are followed by padding to maintain the +25%
//! overhead invariant, ensuring that following content appears at the same
//! character position as it would with pure Z85 encoding.
//!
//! Escape characters are position-dependent within blocks:
//! - At position 0: No prefix, raw bytes follow immediately
//! - At positions 1-3: Prefix bytes are decoded as a base-85 integer
//! - Position 4 is invalid for standard escapes (but valid for `|`)
//!
//! The `|` escape uses a backward-looking base-42 encoding for the length,
//! allowing sequences of 8 or more bytes (or infinite until end of stream).

mod alphabet;
mod base42;
mod decode;
mod encode;
mod error;

// Re-export public API
// Re-export alphabet constants for advanced usage
// Re-export base42 utilities for advanced usage
pub use {
    alphabet::{
        ESCAPE_BACKTICK,
        ESCAPE_CHARS,
        ESCAPE_COMMA,
        ESCAPE_PIPE,
        ESCAPE_SEMICOLON,
        ESCAPE_TILDE,
        ESCAPE_UNDERSCORE,
        PADDING_CHAR,
        Z85_ALPHABET,
        is_escape_char,
        is_safe_for_raw,
        is_z85_char,
    },
    base42::{
        Endianness,
        MAX_LENGTH,
    },
    decode::{
        Decoder,
        decode,
    },
    encode::{
        Encoder,
        encode,
    },
    error::{
        DecodeError,
        EncodeError,
        Error,
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_roundtrip() {
        let original = b"Hello, World!";
        let encoded = encode(original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_empty() {
        let encoded = encode(b"");
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, vec![]);
    }

    #[test]
    fn test_single_byte() {
        for byte in 0..=255u8 {
            let original = [byte];
            let encoded = encode(&original);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, original, "failed for byte {}", byte);
        }
    }

    #[test]
    fn test_all_zeros() {
        for len in 1..=16 {
            let original = vec![0u8; len];
            let encoded = encode(&original);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, original, "failed for len {}", len);
        }
    }

    #[test]
    fn test_all_ones() {
        for len in 1..=16 {
            let original = vec![0xFFu8; len];
            let encoded = encode(&original);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, original, "failed for len {}", len);
        }
    }

    #[test]
    fn test_streaming_encoder() {
        let mut encoder = Encoder::new();
        encoder.write(b"Hello");
        encoder.write(b", ");
        encoder.write(b"World!");
        let encoded = encoder.finish();

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, b"Hello, World!");
    }

    #[test]
    fn test_streaming_decoder() {
        let original = b"Hello, World!";
        let encoded = encode(original);

        let mut decoder = Decoder::new();
        for chunk in encoded.chunks(3) {
            decoder.write(chunk).unwrap();
        }
        let decoded = decoder.finish().unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_dot_in_raw_sequence() {
        // '.' is both Z85 digit 62 AND the padding character.
        // It must be preserved in raw sequences, not skipped as padding.
        let original = b"ab.cd.ef";
        let encoded = encode(original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_dot_in_cross_block_continuation() {
        // Test case where '.' lands in a continuation block.
        // 7 raw bytes using _ escape at position 0:
        // Block 1: _abcd (escape + 4 raw bytes)
        // Block 2: e.g.. (3 more raw bytes + 2 padding)
        // The '.' at position 1 of block 2 must NOT be skipped.
        let original = b"abcde.g";
        let encoded = encode(original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(
            decoded, original,
            "dot was incorrectly skipped in continuation"
        );
    }

    #[test]
    fn test_multiple_dots_in_raw() {
        // Stress test with many dots
        let original = b"a.b.c.d.e.f.g.h";
        let encoded = encode(original);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_all_dots() {
        // Edge case: entire content is dots
        for len in 1..=16 {
            let original = vec![b'.'; len];
            let encoded = encode(&original);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, original, "failed for len {} all-dots", len);
        }
    }
}
