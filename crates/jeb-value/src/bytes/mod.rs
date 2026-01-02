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

// [impl jeb-value.dependencies.cfg]
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen
)]
// [impl jeb-value.variants.clone]
// [impl jeb-value.variants.debug]
// [impl jeb-value.variants.deref]
// [impl jeb-value.variants.as-ref]
// [impl jeb-value.variants.cmp]
// [impl jeb-value.variants.cmp.delegate-inner]
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
// [impl jeb-value.variants.transparent]
#[repr(transparent)]
#[as_ref(Vec<u8>, [u8])]
#[into_iterator(
    owned, ref, ref_mut
)]
// [impl jeb-value.variant.must-use]
#[must_use]
// [impl jeb-value.bytes]
pub struct Bytes(pub(crate) Vec<u8>);
impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Self {
        Bytes(value.to_vec())
    }
}
