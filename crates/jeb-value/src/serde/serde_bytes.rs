//! Conversions between serde_bytes types and jeb-value types.

use crate::{Bytes, Value};

// From serde_bytes::ByteBuf to Bytes
impl From<serde_bytes::ByteBuf> for Bytes {
    fn from(value: serde_bytes::ByteBuf) -> Self {
        Bytes(value.into_vec())
    }
}

// From serde_bytes::ByteBuf to Value
impl From<serde_bytes::ByteBuf> for Value {
    fn from(value: serde_bytes::ByteBuf) -> Self {
        Value::Bytes(Bytes::from(value))
    }
}

// From &serde_bytes::Bytes to Bytes
impl From<&serde_bytes::Bytes> for Bytes {
    fn from(value: &serde_bytes::Bytes) -> Self {
        Bytes(value.to_vec())
    }
}

// From &serde_bytes::Bytes to Value
impl From<&serde_bytes::Bytes> for Value {
    fn from(value: &serde_bytes::Bytes) -> Self {
        Value::Bytes(Bytes::from(value))
    }
}
