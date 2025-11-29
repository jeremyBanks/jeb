use derive_more::{From, IsVariant, TryUnwrap, Unwrap};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::{bytes::Bytes, float::Float, text::Text};



#[derive(Debug, Clone, From, Serialize, Deserialize, Default, TryUnwrap, IsVariant, Unwrap)]
#[serde(untagged)]
#[must_use]
pub enum Value {
    Unsigned(u64),
    Signed(i64),
    Float(Float),
    Bool(bool),
    #[default]
    Null,
    Text(Text),
    Bytes(Bytes),
    Array(Vec<Value>),
    BytesMap(IndexMap<Bytes, Value>),
    TextMap(IndexMap<Text, Value>),
}

impl TryFrom<f32> for Value {
    type Error = f32;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Float::try_from(value).map(Value::from)
    }
}

impl TryFrom<f64> for Value {
    type Error = f64;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Float::try_from(value).map(Value::from)
    }
}

impl TryFrom<u128> for Value {
    type Error = u128;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        u64::try_from(value).map(Value::Unsigned).map_err(|_| value)
    }
}

impl TryFrom<i128> for Value {
    type Error = i128;

    fn try_from(value: i128) -> Result<Self, Self::Error> {
        i64::try_from(value).map(Value::Signed).map_err(|_| value)
    }
}

impl From<()> for Value {
    fn from((): ()) -> Self {
        Value::Null
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::Unsigned(value.into())
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::Unsigned(value.into())
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::Unsigned(value.into())
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Signed(value.into())
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::Signed(value.into())
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::Signed(value.into())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value.into())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Text(value.to_string().into())
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Bytes(value.into())
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Value::Bytes(value.into())
    }
}

impl FromIterator<Value> for Value {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Value::Array(iter.into_iter().collect())
    }
}

impl<const N: usize> From<[Value; N]> for Value {
    fn from(value: [Value; N]) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<(Text, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (Text, Value)>>(iter: T) -> Self {
        Value::TextMap(iter.into_iter().collect())
    }
}

impl FromIterator<(String, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Value::TextMap(iter.into_iter().map(|(k, v)| (Text::from(k), v)).collect())
    }
}

impl<'a> FromIterator<(&'a str, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (&'a str, Value)>>(iter: T) -> Self {
        Value::TextMap(
            iter.into_iter()
                .map(|(k, v)| (Text::from(k.to_string()), v))
                .collect(),
        )
    }
}

impl<const N: usize> From<[(Text, Value); N]> for Value {
    fn from(value: [(Text, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(String, Value); N]> for Value {
    fn from(value: [(String, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(&str, Value); N]> for Value {
    fn from(value: [(&str, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<(Bytes, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (Bytes, Value)>>(iter: T) -> Self {
        Value::BytesMap(iter.into_iter().collect())
    }
}

impl FromIterator<(Vec<u8>, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (Vec<u8>, Value)>>(iter: T) -> Self {
        Value::BytesMap(iter.into_iter().map(|(k, v)| (Bytes::from(k), v)).collect())
    }
}

impl<'a> FromIterator<(&'a [u8], Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (&'a [u8], Value)>>(iter: T) -> Self {
        Value::BytesMap(iter.into_iter().map(|(k, v)| (Bytes::from(k), v)).collect())
    }
}

impl<const N: usize> From<[(Bytes, Value); N]> for Value {
    fn from(value: [(Bytes, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(Vec<u8>, Value); N]> for Value {
    fn from(value: [(Vec<u8>, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(&[u8], Value); N]> for Value {
    fn from(value: [(&[u8], Value); N]) -> Self {
        value.into_iter().collect()
    }
}
