//! Error type for serde-value operations.

use std::fmt;

/// Error type for serde-value serialization and deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A custom error message.
    Message(String),

    /// Type mismatch during deserialization.
    TypeMismatch {
        expected: &'static str,
        found: &'static str,
    },

    /// A required field is missing.
    MissingField(&'static str),

    /// An unknown field was encountered.
    UnknownField(String),

    /// A field appeared more than once.
    DuplicateField(&'static str),

    /// Integer coercion is not yet implemented.
    IntegerCoercion {
        from: &'static str,
        to: &'static str,
    },

    /// Expected a different length.
    LengthMismatch { expected: usize, found: usize },

    /// Invalid enum variant.
    InvalidVariant {
        expected: &'static [&'static str],
        found: String,
    },

    /// Value out of range for target type.
    OutOfRange(&'static str),

    /// Invalid UTF-8 in bytes-to-string conversion.
    InvalidUtf8,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{msg}"),
            Error::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected {expected}, found {found}")
            }
            Error::MissingField(field) => write!(f, "missing field: {field}"),
            Error::UnknownField(field) => write!(f, "unknown field: {field}"),
            Error::DuplicateField(field) => write!(f, "duplicate field: {field}"),
            Error::IntegerCoercion { from, to } => {
                write!(f, "integer coercion from {from} to {to} not implemented")
            }
            Error::LengthMismatch { expected, found } => {
                write!(f, "length mismatch: expected {expected}, found {found}")
            }
            Error::InvalidVariant { expected, found } => {
                write!(f, "invalid variant: expected one of {expected:?}, found {found}")
            }
            Error::OutOfRange(target) => {
                write!(f, "value out of range for {target}")
            }
            Error::InvalidUtf8 => {
                write!(f, "invalid UTF-8 in bytes")
            }
        }
    }
}

impl std::error::Error for Error {}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Error {
    /// Create a type mismatch error.
    pub fn type_mismatch(expected: &'static str, found: &'static str) -> Self {
        Error::TypeMismatch { expected, found }
    }

    /// Create an out-of-range error.
    pub fn out_of_range(target: &'static str) -> Self {
        Error::OutOfRange(target)
    }

    /// Create an invalid UTF-8 error.
    pub fn invalid_utf8() -> Self {
        Error::InvalidUtf8
    }
}
