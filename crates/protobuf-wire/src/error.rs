//! Error types for protobuf wire format parsing and serialization.

use std::fmt;

/// Errors that can occur during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Reached end of input unexpectedly.
    UnexpectedEof,
    /// Varint encoding is invalid (too many bytes or malformed).
    InvalidVarint,
    /// Wire type value is not in the valid range 0-5.
    InvalidWireType(u8),
    /// Field number is not in the valid range 1 to 2^29-1.
    InvalidFieldNumber,
    /// EGROUP field number doesn't match the corresponding SGROUP.
    MismatchedGroupEnd { expected: u32, found: u32 },
    /// Reached end of input while inside a group.
    UnterminatedGroup { field_number: u32 },
    /// Length prefix would exceed available data or overflow.
    LengthOverflow,
    /// Group nesting is too deep.
    NestingTooDeep,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEof => write!(f, "unexpected end of input"),
            ParseError::InvalidVarint => write!(f, "invalid varint encoding"),
            ParseError::InvalidWireType(wt) => write!(f, "invalid wire type: {}", wt),
            ParseError::InvalidFieldNumber => {
                write!(f, "field number must be in range 1 to 536870911")
            }
            ParseError::MismatchedGroupEnd { expected, found } => {
                write!(
                    f,
                    "mismatched group end: expected field {}, found field {}",
                    expected, found
                )
            }
            ParseError::UnterminatedGroup { field_number } => {
                write!(f, "unterminated group at field {}", field_number)
            }
            ParseError::LengthOverflow => write!(f, "length prefix exceeds available data"),
            ParseError::NestingTooDeep => write!(f, "group nesting exceeds maximum depth"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Errors that can occur during serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializeError {
    /// Field number is not in the valid range 1 to 2^29-1.
    InvalidFieldNumber(u32),
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializeError::InvalidFieldNumber(n) => {
                write!(
                    f,
                    "invalid field number {}: must be in range 1 to 536870911",
                    n
                )
            }
        }
    }
}

impl std::error::Error for SerializeError {}
