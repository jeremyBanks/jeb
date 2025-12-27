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
pub mod serde;
#[cfg(feature = "serde")]
pub use self::serde::{from_value, to_value, Error as SerdeError};
