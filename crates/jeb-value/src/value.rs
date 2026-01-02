use {
    super::{
        bytes::Bytes,
        float::Float,
        text::Text,
    },
    derive_more::{
        From,
        IsVariant,
        TryInto,
        TryUnwrap,
        Unwrap,
    },
    indexmap::IndexMap,
};

mod cmp;

/// The core Value enum representing all supported data types.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    serde(untagged)
)]
#[derive(Debug, Clone, From, Default, TryInto, IsVariant, TryUnwrap, Unwrap)]
#[must_use]
pub enum Value {
    // [impl jeb-value.null]
    #[default]
    Null,
    Bool(bool),
    Float(f64),
    Signed(i64),
    Unsigned(u64),
    // [impl jeb-value.integer.bytes]
    Bytes(Bytes),
    // [impl jeb-value.text]
    Text(Text),
    // [impl jeb-value.array]
    Array(Vec<Value>),
    // [impl jeb-value.map.bytes-map]
    BytesMap(IndexMap<Bytes, Value>),
    // [impl jeb-value.map.text-map]
    TextMap(IndexMap<Text, Value>),
}
