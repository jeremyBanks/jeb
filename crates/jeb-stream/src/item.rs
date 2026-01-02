use {
    derive_more::{
        From,
        IsVariant,
        TryInto,
        TryUnwrap,
        Unwrap,
    },
    jeb_value::{
        Bytes,
        Text,
        Value,
    },
};

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    serde(untagged)
)]
#[derive(
    Debug, Clone, From, TryInto, IsVariant, TryUnwrap, Unwrap, Hash, Eq, PartialEq, PartialOrd, Ord,
)]
#[must_use]
pub enum Item {
    Bytes(Bytes),
    Text(Text),
    Value(Value),
}

impl Item {
    pub fn into_value(self) -> Value {
        match self {
            Item::Value(value) => value,
            Item::Bytes(bytes) => bytes.into(),
            Item::Text(text) => text.into(),
        }
    }

    pub fn try_as_bytes(&self) -> Result<&[u8], &Item> {
        match self {
            Item::Bytes(bytes) | Item::Value(Value::Bytes(bytes)) => Ok(bytes),
            Item::Text(text) | Item::Value(Value::Text(text)) => Ok(text.as_ref()),
            _ => Err(self),
        }
    }

    pub fn try_as_str(&self) -> Result<&str, &Item> {
        match self {
            Item::Text(text) | Item::Value(Value::Text(text)) => Ok(text.as_ref()),
            Item::Bytes(bytes) | Item::Value(Value::Bytes(bytes)) => {
                core::str::from_utf8(bytes.as_ref()).map_err(|_| self)
            }
            _ => Err(self),
        }
    }
}

impl<'a> TryFrom<&'a Item> for &'a [u8] {
    type Error = &'a Value;

    fn try_from(item: &'a Item) -> Result<&'a [u8], Self::Error> {
        match item {
            Item::Bytes(b) => Ok(b.as_ref()),
            Item::Text(t) => Ok(t.as_bytes()),
            Item::Value(v) => Err(v),
        }
    }
}

impl Default for Item {
    fn default() -> Self {
        Item::Text(Text::default())
    }
}

impl From<&str> for Item {
    fn from(s: &str) -> Self {
        Item::Text(s.into())
    }
}

impl From<String> for Item {
    fn from(s: String) -> Self {
        Item::Text(s.into())
    }
}

impl From<&[u8]> for Item {
    fn from(b: &[u8]) -> Self {
        Item::Bytes(b.into())
    }
}

impl From<Vec<u8>> for Item {
    fn from(b: Vec<u8>) -> Self {
        Item::Bytes(b.into())
    }
}
