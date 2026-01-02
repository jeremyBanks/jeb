#![doc = include_str!("../README.md")]
#![doc = ::document_features::document_features!()]
#![allow(
    unused_imports,
    clippy::approx_constant
)]
mod bytes;
mod float;
mod from;
#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "serde_json")]
mod serde_json;
mod text;
mod value;
#[cfg(feature = "serde")]
pub use self::serde::*;
#[cfg(feature = "serde_json")]
pub use self::serde_json::*;
pub use self::{
    bytes::*,
    float::*,
    text::*,
    value::*,
};
