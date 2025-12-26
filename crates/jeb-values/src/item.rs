use core::hash::Hash;

use derive_more::{
    AsMut, AsRef, Deref, DerefMut, Display, From, Index, Into, IsVariant, TryUnwrap, Unwrap,
};


#[derive(Debug, Clone, From, TryUnwrap, IsVariant, Unwrap)]
#[cfg_attr(
    feature = "serde",
    derive(
        serde::Deserialize,
        serde::Serialize
    ),
    serde(untagged)
)]
#[must_use]
pub enum Item {
    Bytes(crate::Bytes),
    Text(crate::Text),
    Value(crate::Value),
}
