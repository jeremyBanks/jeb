#![doc = include_str!("../README.md")]
#![doc = ::document_features::document_features!()]
#![allow(unused_imports)]

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
pub use self::serde::*;

#[cfg(feature = "serde_json")]
pub mod serde_json;
#[cfg(feature = "serde_json")]
pub use self::serde_json::*;
