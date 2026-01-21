#![doc = include_str!("../README.md")]
#![doc = ::document_features::document_features!()]
#![allow(unused_imports, clippy::approx_constant)]

mod array;
mod boolean;
mod bytes;
mod bytes_map;
mod from;
mod null;
mod number;
// [impl jeb-value.features.core.cfg]
// [impl jeb-value.features.serde.optional]
#[cfg(feature = "serde")]
mod serde;
// [impl jeb-value.features.core.cfg]
// [impl jeb-value.features.serde-json.depends]
#[cfg(feature = "serde_json")]
mod serde_json;
mod string;
mod string_map;
mod value;

// [impl jeb-value.features.core.cfg]
#[cfg(feature = "serde")]
pub use self::serde::*;
// [impl jeb-value.features.core.cfg]
#[cfg(feature = "serde_json")]
pub use self::serde_json::*;

// [impl jeb-value.value.def.pub]
// [impl jeb-value.variant.common.pub]
pub use self::{
    array::Array,
    boolean::{from::NotBooleanError, Boolean},
    bytes::Bytes,
    bytes_map::BytesMap,
    null::{from::NotNullError, Null},
    number::{NotFiniteError, Number},
    string::String,
    string_map::StringMap,
    value::Value,
};
