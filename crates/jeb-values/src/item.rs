use derive_more::{From, IsVariant, TryUnwrap, Unwrap};

#[cfg_attr(
    feature = "serde",
    derive(
        serde::Deserialize,
        serde::Serialize
    ),
    serde(untagged)
)]
#[derive(Debug, Clone, From, TryUnwrap, IsVariant, Unwrap)]
#[must_use]
pub enum Item {
    Bytes(crate::Bytes),
    Text(crate::Text),
    Value(crate::Value),
}
