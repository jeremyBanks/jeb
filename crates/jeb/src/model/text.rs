use core::hash::Hash;

use derive_more::{AsMut, AsRef, Deref, DerefMut, Display, From, Index, Into};
use serde::{Deserialize, Serialize};

use super::bytes::Bytes;


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(
    AsMut,
    AsRef,
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    Deserialize,
    Display,
    Eq,
    From,
    Hash,
    Index,
    Into,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(transparent)]
#[repr(transparent)]
#[must_use]
pub struct Text(pub(in crate::model) String);

impl TryFrom<Bytes> for Text {
    type Error = core::str::Utf8Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let s = core::str::from_utf8(&value)?;
        Ok(Text(s.to_string()))
    }
}
