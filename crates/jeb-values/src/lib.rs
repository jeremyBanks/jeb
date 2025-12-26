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
//! Item is a top-level enum which may contain Text, Binary, or an arbitrary
//! Value.
//!
//! This crate just contains the type definitions, optional serde annotations,
//! and a bunch of helper functions for converting between different runtime
//! types. This doesn't define any serialization scheme or anything like that.
mod bytes;
mod float;
mod item;
mod text;
mod value;

pub use self::{bytes::Bytes, float::Float, item::Item, text::Text, value::Value};
