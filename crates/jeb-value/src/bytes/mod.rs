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

mod serde;
mod std;
mod value;

#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen
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
#[as_ref(Vec<u8>, [u8])]
#[into_iterator(
    owned, ref, ref_mut
)]
#[must_use]
pub struct Bytes(pub(crate) Vec<u8>);
impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Self {
        Bytes(value.to_vec())
    }
}
