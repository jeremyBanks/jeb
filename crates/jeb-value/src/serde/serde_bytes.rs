//! Converting from `serde_bytes` types to `jeb-value` types.
use crate::{Bytes, Value};
impl From<serde_bytes::ByteBuf> for Bytes {
    fn from(value: serde_bytes::ByteBuf) -> Self {
        Bytes(value.into_vec())
    }
}
impl From<&serde_bytes::Bytes> for Bytes {
    fn from(value: &serde_bytes::Bytes) -> Self {
        Bytes(value.to_vec())
    }
}
impl<const N: usize> From<&serde_bytes::ByteArray<N>> for Bytes {
    fn from(value: &serde_bytes::ByteArray<N>) -> Self {
        Bytes(value.to_vec())
    }
}
impl From<serde_bytes::ByteBuf> for Value {
    fn from(value: serde_bytes::ByteBuf) -> Self {
        Value::Bytes(value.into())
    }
}
impl From<&serde_bytes::Bytes> for Value {
    fn from(value: &serde_bytes::Bytes) -> Self {
        Value::Bytes(value.into())
    }
}
impl<const N: usize> From<&serde_bytes::ByteArray<N>> for Value {
    fn from(value: &serde_bytes::ByteArray<N>) -> Self {
        Value::Bytes(value.into())
    }
}
