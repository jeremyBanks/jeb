#![doc = include_str!("../README.md")]
#![doc = ::document_features::document_features!()]
#![allow(
    unused_imports,
    clippy::approx_constant
)]
mod bytes;
mod float;
mod from;
// [impl jeb-value.dependencies.cfg]
#[cfg(feature = "serde")]
mod serde;
// [impl jeb-value.dependencies.cfg]
#[cfg(feature = "serde_json")]
mod serde_json;
mod text;
mod value;
// [impl jeb-value.dependencies.cfg]
#[cfg(feature = "serde")]
pub use self::serde::*;
// [impl jeb-value.dependencies.cfg]
#[cfg(feature = "serde_json")]
pub use self::serde_json::*;
// [impl jeb-value.value.pub]
// [impl jeb-value.variants.pub]
pub use self::{
    bytes::*,
    float::*,
    text::*,
    value::*,
};
