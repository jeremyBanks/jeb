#![deny(unsafe_code)]
#![warn(missing_docs, unused_crate_dependencies)]
#![allow(clippy::unusual_byte_groupings)]
#![cfg_attr(
    all(debug_assertions, any(not(test), feature = "phony")),
    allow(unused, unused_crate_dependencies, missing_docs)
)]

//! serproto encodes a serde subset as a protobuf subset.
mod de;
mod error;
mod ser;
mod wire;

#[cfg(test)]
mod tests;

pub use de::Deserializer;
pub use error::Error;
pub use error::Result;
pub use ser::Serializer;
use serde::Deserialize;
use serde::Serialize;

// TODO: add all of the byte interfaces from serde_json

/// Serialize a value into a new byte vector.
#[inline]
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    let mut v = Vec::new();
    to_writer(&mut v, value)?;
    Ok(v)
}

/// Serialize a value to a [`io::Write`](std::io::Write) implementation.
///
/// Use this to extend a `Vec<u8>`, or feed into some compressor.
#[inline]
pub fn to_writer<T, W>(w: &mut W, value: &T) -> Result<()>
where
    T: Serialize + ?Sized,
    W: std::io::Write,
{
    value.serialize(Serializer::new(w))
}

/// Deserialize a value from a byte slice.
pub fn from_bytes<'de, T>(data: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut de = Deserializer::from_bytes(data);
    let value = T::deserialize(&mut de)?;
    if de.remaining_len() > 0 {
        return Err(Error::DataBeyondEnd);
    }
    Ok(value)
}

/// Deserialize a value from a byte slice that may have more data.
///
/// Returns a pair of (value, size_read).
pub fn from_bytes_more_data<'de, T>(data: &'de [u8]) -> Result<(T, usize)>
where
    T: Deserialize<'de>,
{
    let mut de = Deserializer::from_bytes(data);
    let value = T::deserialize(&mut de)?;
    let consumed = data.len() - de.remaining_len();
    Ok((value, consumed))
}
