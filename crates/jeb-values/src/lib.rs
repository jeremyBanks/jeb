//! The value types used by `jeb`.
//!
//! This is an extension of the standard JavaScript JSON data model. Like
//! JavaScript JSON, it includes null, booleans, UTF-8 text strings, finite
//! 64-bit floating-point numbers (excluding NaN or Infinity, including -0),
//! arrays of arbitrary values, and order-preserving maps with text string keys
//! and arbitrary values. This adds signed and unsigned 64-bit integers,
//! binary byte strings, and order-preserving maps with binary byte string keys
//! and arbitrary values.
//!
//! These types support serde, but they also can be used as an in-memory serde
//! serialization format (similar to `serde_json::Value`, but with more direct
//! support for more of serde's JSON model) for interop with other types.
//!
//! This crate does _not_ define a text or binary representation for this data.
#![doc = ::document_features::document_features!()]

mod bytes;
mod float;
mod text;
mod value;

pub use self::{bytes::Bytes, float::Float, text::Text, value::Value};

#[cfg(feature = "serde")]
mod deserialize;
#[cfg(feature = "serde")]
pub use self::deserialize::*;

#[cfg(feature = "serde")]
mod serialize;
#[cfg(feature = "serde")]
pub use self::serialize::*;
