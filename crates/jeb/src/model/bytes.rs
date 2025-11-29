use core::hash::Hash;

use derive_more::{AsMut, AsRef, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator};
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::model::text::Text;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(
    AsMut,
    AsRef,
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    Deserialize,
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
    Serialize,
)]
#[repr(transparent)]
#[serde(transparent)]
#[must_use]
#[into_iterator(owned, ref, ref_mut)]
pub struct Bytes(pub(in crate::model) Vec<u8>);

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
