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

mod from;
mod ord;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    serde(untagged)
)]
#[derive(Debug, Clone, From, Default, TryInto, IsVariant, TryUnwrap, Unwrap, Eq, PartialEq)]
#[must_use]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Float(Float),
    Bytes(Bytes),
    Text(Text),
    Array(Vec<Value>),
    BytesMap(IndexMap<Bytes, Value>),
    TextMap(IndexMap<Text, Value>),
}


impl core::hash::Hash for Value {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::Null => {}
            Value::Bool(value) => value.hash(state),
            Value::Unsigned(value) => value.hash(state),
            Value::Signed(value) => value.hash(state),
            Value::Float(value) => value.hash(state),
            Value::Bytes(value) => value.hash(state),
            Value::Text(value) => value.hash(state),
            Value::Array(value) => value.hash(state),
            Value::TextMap(value) => {
                value.len().hash(state);
                for item in value {
                    item.hash(state);
                }
            }
            Value::BytesMap(value) => {
                value.len().hash(state);
                for item in value {
                    item.hash(state);
                }
            }
        }
    }
}
