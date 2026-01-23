//! Error types for the ideated-encoding crate.

use thiserror::Error;

/// Errors that can occur during decoding.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum DecodeError {
    /// An invalid character was encountered in the encoded stream.
    #[error("invalid character at position {position}: byte {byte:#04x} ('{}')", char::from(*byte).escape_default())]
    InvalidCharacter { position: usize, byte: u8 },

    /// An escape character appeared at an invalid position within a block.
    #[error("escape character '{escape}' at invalid position {position} within block")]
    InvalidEscapePosition { position: usize, escape: char },

    /// A length value in a `|` escape was invalid (e.g., 1-7 which should use
    /// simpler escapes).
    #[error("invalid length {length} for | escape (must be 0 or >= 8)")]
    InvalidLength { length: usize },

    /// The input ended unexpectedly.
    #[error("unexpected end of input")]
    UnexpectedEndOfInput,

    /// A prefix value was too large to fit in the available character slots.
    #[error("prefix value {value} exceeds maximum {max} for {chars} character(s)")]
    PrefixValueTooLarge { value: u64, max: u64, chars: usize },

    /// Invalid Z85 digit value encountered in base-42 length decoding.
    #[error("invalid Z85 digit in length encoding at position {position}")]
    InvalidLengthDigit { position: usize },

    /// The padding was invalid or incomplete.
    #[error("invalid padding at position {position}")]
    InvalidPadding { position: usize },
}

/// Errors that can occur during encoding.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EncodeError {
    /// Internal error - should not happen in normal operation.
    #[error("internal encoding error: {message}")]
    Internal { message: String },
}

/// A general error type for the crate.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum Error {
    /// A decoding error occurred.
    #[error(transparent)]
    Decode(#[from] DecodeError),

    /// An encoding error occurred.
    #[error(transparent)]
    Encode(#[from] EncodeError),
}
