use {
    derive_more::{
        AsRef,
        From,
        IsVariant,
        TryInto,
        TryUnwrap,
        Unwrap,
    },
    jeb_values::{
        Bytes,
        Text,
        Value,
    },
};

#[cfg_attr(
    feature = "serde",
    derive(
        serde::Deserialize,
        serde::Serialize
    ),
    serde(untagged)
)]
#[must_use]
#[derive(
    Clone, Debug, From, IsVariant, TryInto, TryUnwrap, Hash, Eq, PartialEq, PartialOrd, Ord,
)]
pub enum Item {
    Bytes(Bytes),
    Text(Text),
    Value(Value),
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
