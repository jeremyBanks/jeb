use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum Error {
    Message(Box<str>),
    InvalidType {
        unexpected: Unexpected,
        expected: Box<str>,
    },
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

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::InvalidType { unexpected, expected } => {
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

impl std::error::Error for Error {}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string().into_boxed_str())
    }
}

impl Error {
    pub(crate) fn invalid_type(unexp: Unexpected, exp: &str) -> Self {
        Error::InvalidType {
            unexpected: unexp,
            expected: exp.into(),
        }
    }

    pub(crate) fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string().into_boxed_str())
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string().into_boxed_str())
    }

    fn invalid_type(unexp: serde::de::Unexpected, exp: &dyn serde::de::Expected) -> Self {
        let unexpected = match unexp {
            serde::de::Unexpected::Bool(b) => Unexpected::Bool(b),
            serde::de::Unexpected::Unsigned(u) => Unexpected::Unsigned(u),
            serde::de::Unexpected::Signed(i) => Unexpected::Signed(i),
            serde::de::Unexpected::Float(f) => Unexpected::Float(f),
            serde::de::Unexpected::Char(c) => Unexpected::Char(c),
            serde::de::Unexpected::Str(s) => Unexpected::Str(s.into()),
            serde::de::Unexpected::Bytes(b) => Unexpected::Bytes(b.into()),
            serde::de::Unexpected::Unit => Unexpected::Unit,
            serde::de::Unexpected::Option => {
                return Error::Message("invalid type: option value".into());
            }
            serde::de::Unexpected::NewtypeStruct => {
                return Error::Message("invalid type: newtype struct".into());
            }
            serde::de::Unexpected::Seq => Unexpected::Seq,
            serde::de::Unexpected::Map => Unexpected::Map,
            serde::de::Unexpected::Enum => {
                return Error::Message("invalid type: enum".into());
            }
            serde::de::Unexpected::UnitVariant => {
                return Error::Message("invalid type: unit variant".into());
            }
            serde::de::Unexpected::NewtypeVariant => {
                return Error::Message("invalid type: newtype variant".into());
            }
            serde::de::Unexpected::TupleVariant => {
                return Error::Message("invalid type: tuple variant".into());
            }
            serde::de::Unexpected::StructVariant => {
                return Error::Message("invalid type: struct variant".into());
            }
            serde::de::Unexpected::Other(other) => {
                return Error::Message(format!("invalid type: {}", other).into());
            }
        };

        Error::InvalidType {
            unexpected,
            expected: exp.to_string().into_boxed_str(),
        }
    }
}
