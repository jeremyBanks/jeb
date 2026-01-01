use std::fmt::{self, Display};
/// Error type for both serialization and deserialization of `Value`.
///
/// This error type is used by both:
/// - `impl Serializer` in serializer.rs (for serializing to `Value`)
/// - `impl Deserializer for Value` in deserializer.rs (for deserializing from
///   `Value`)
#[derive(Debug, Clone)]
pub enum SerdeError {
    Message(Box<str>),
    InvalidType { unexpected: Unexpected, expected: Box<str> },
}
#[derive(Debug, Clone)]
pub enum Unexpected {
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Char(char),
    Str(Box<str>),
    Bytes(Box<[u8]>),
    Unit,
    Seq,
    Map,
}
impl Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerdeError::Message(msg) => f.write_str(msg),
            SerdeError::InvalidType { unexpected, expected } => {
                write!(f, "invalid type: {}, expected {}", unexpected, expected)
            }
        }
    }
}
impl Display for Unexpected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unexpected::Bool(b) => write!(f, "boolean `{}`", b),
            Unexpected::Unsigned(u) => write!(f, "unsigned integer `{}`", u),
            Unexpected::Signed(i) => write!(f, "signed integer `{}`", i),
            Unexpected::Float(fl) => write!(f, "float `{}`", fl),
            Unexpected::Char(c) => write!(f, "character `{}`", c),
            Unexpected::Str(s) => write!(f, "string {:?}", s),
            Unexpected::Bytes(b) => write!(f, "byte array of length {}", b.len()),
            Unexpected::Unit => write!(f, "unit value"),
            Unexpected::Seq => write!(f, "sequence"),
            Unexpected::Map => write!(f, "map"),
        }
    }
}
impl std::error::Error for SerdeError {}
impl serde::ser::Error for SerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeError::Message(msg.to_string().into_boxed_str())
    }
}
impl SerdeError {
    pub(crate) fn invalid_type(unexp: Unexpected, exp: &str) -> Self {
        SerdeError::InvalidType {
            unexpected: unexp,
            expected: exp.into(),
        }
    }
    pub(crate) fn custom<T: Display>(msg: T) -> Self {
        SerdeError::Message(msg.to_string().into_boxed_str())
    }
}
impl serde::de::Error for SerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        SerdeError::Message(msg.to_string().into_boxed_str())
    }
    fn invalid_type(
        unexpected: serde::de::Unexpected,
        exp: &dyn serde::de::Expected,
    ) -> Self {
        let unexpected = match unexpected {
            serde::de::Unexpected::Bool(b) => Unexpected::Bool(b),
            serde::de::Unexpected::Unsigned(u) => Unexpected::Unsigned(u),
            serde::de::Unexpected::Signed(i) => Unexpected::Signed(i),
            serde::de::Unexpected::Float(f) => Unexpected::Float(f),
            serde::de::Unexpected::Char(c) => Unexpected::Char(c),
            serde::de::Unexpected::Str(s) => Unexpected::Str(s.into()),
            serde::de::Unexpected::Bytes(b) => Unexpected::Bytes(b.into()),
            serde::de::Unexpected::Unit => Unexpected::Unit,
            serde::de::Unexpected::Option => {
                return SerdeError::Message("invalid type: option value".into());
            }
            serde::de::Unexpected::NewtypeStruct => {
                return SerdeError::Message("invalid type: newtype struct".into());
            }
            serde::de::Unexpected::Seq => Unexpected::Seq,
            serde::de::Unexpected::Map => Unexpected::Map,
            serde::de::Unexpected::Enum => {
                return SerdeError::Message("invalid type: enum".into());
            }
            serde::de::Unexpected::UnitVariant => {
                return SerdeError::Message("invalid type: unit variant".into());
            }
            serde::de::Unexpected::NewtypeVariant => {
                return SerdeError::Message("invalid type: newtype variant".into());
            }
            serde::de::Unexpected::TupleVariant => {
                return SerdeError::Message("invalid type: tuple variant".into());
            }
            serde::de::Unexpected::StructVariant => {
                return SerdeError::Message("invalid type: struct variant".into());
            }
            serde::de::Unexpected::Other(other) => {
                return SerdeError::Message(format!("invalid type: {}", other).into());
            }
        };
        SerdeError::InvalidType {
            unexpected,
            expected: exp.to_string().into_boxed_str(),
        }
    }
}
