use {
    crate::text::Text,
    core::hash::Hash,
    derive_more::{
        AsMut,
        AsRef,
        Deref,
        DerefMut,
        From,
        Index,
        IndexMut,
        Into,
        IntoIterator,
    },
};

#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen
)]
#[cfg_attr(
    feature = "serde",
    derive(
        serde::Serialize,
        serde::Deserialize,
    ),
    serde(transparent)
)]
#[derive(
    AsMut,
    AsRef,
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    Eq,
    From,
    Hash,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Ord,
    PartialEq,
    PartialOrd,
)]
#[repr(transparent)]
#[must_use]
#[into_iterator(
    owned, ref, ref_mut
)]
pub struct Bytes(pub(crate) Vec<u8>);

impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Self {
        Bytes(value.to_vec())
    }
}

impl From<&str> for Bytes {
    fn from(value: &str) -> Self {
        Bytes(value.as_bytes().to_vec())
    }
}

impl From<Text> for Bytes {
    fn from(value: Text) -> Self {
        Bytes(value.0.into_bytes())
    }
}
