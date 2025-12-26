#![doc = include_str!("../README.md")]
#![doc = ::document_features::document_features!()]

mod bytes;
mod float;
mod text;
mod value;

pub use self::{
    bytes::Bytes,
    float::Float,
    text::Text,
    value::Value,
};

#[cfg(feature = "serde")]
mod deserialize;
#[cfg(feature = "serde")]
pub use self::deserialize::*;

#[cfg(feature = "serde")]
mod serialize;
#[cfg(feature = "serde")]
pub use self::serialize::*;
