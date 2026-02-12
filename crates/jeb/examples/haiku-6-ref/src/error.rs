//! Error types for Z85 encoding/decoding.

use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidCharacter { character: char, position: usize },
    InvalidZ85Character { character: char, position: usize },
    TruncatedRawSection { expected: usize, got: usize },
    TruncatedInput { position: usize },
    InvalidEscapeSequence { position: usize, reason: String },
    DecodingError { message: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCharacter { character, position } => {
                write!(f, "invalid character '{}' at position {}", character, position)
            }
            Error::InvalidZ85Character { character, position } => {
                write!(f, "invalid Z85 character '{}' at position {}", character, position)
            }
            Error::TruncatedRawSection { expected, got } => {
                write!(f, "truncated raw section: expected {} bytes, got {}", expected, got)
            }
            Error::TruncatedInput { position } => {
                write!(f, "truncated input at position {}", position)
            }
            Error::InvalidEscapeSequence { position, reason } => {
                write!(f, "invalid escape sequence at position {}: {}", position, reason)
            }
            Error::DecodingError { message } => {
                write!(f, "decoding error: {}", message)
            }
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
