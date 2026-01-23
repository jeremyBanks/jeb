use derive_more::{
    AsMut,
    AsRef,
    Deref,
    DerefMut,
    From,
    Index,
    IndexMut,
    Into,
    IntoIterator,
};

mod from;
mod serde;
mod value;

// Note: Serde impls are in bytes/serde.rs (manual impl for serde_bytes
// compatibility) [impl jeb-value.variant.common.clone]
// [impl jeb-value.variant.common.debug]
// [impl jeb-value.variant.common.deref]
// [impl jeb-value.variant.common.as-ref]
// [impl jeb-value.variant.common.eq-delegate-inner]
// [impl jeb-value.variant.common.partial-eq-delegate-inner]
// [impl jeb-value.variant.common.ord-delegate-inner]
// [impl jeb-value.variant.common.partial-ord-delegate-inner]
// [impl jeb-value.variant.common.hash-delegate-inner]
// [impl jeb-value.variant.common.borrow]
// [impl jeb-value.variant.common.mut]
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
    IntoIterator,
    Ord,
    PartialEq,
    PartialOrd,
)]
// [impl jeb-value.variant.common.transparent]
#[repr(transparent)]
#[as_ref(Vec<u8>, [u8])]
#[into_iterator(
    owned, ref, ref_mut
)]
// [impl jeb-value.variant.common.must-use]
#[must_use]
// [impl jeb-value.bytes.struct]
pub struct Bytes(pub(crate) Vec<u8>);

// [impl jeb-value.variant.common.constructor]
// [impl jeb-value.variant.common.try-from-inner]
// [impl jeb-value.bytes.from-inner]
impl Bytes {
    /// Creates a new `Bytes` from a `Vec<u8>`.
    #[must_use]
    pub fn new(value: Vec<u8>) -> Self {
        Bytes(value)
    }
}

// [impl jeb-value.variant.common.into-inner]
// [impl jeb-value.variant.common.to-inner]
// [impl jeb-value.variant.common.as-inner]
impl Bytes {
    /// Consumes the `Bytes` and returns the inner `Vec<u8>`.
    #[must_use]
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    /// Returns a clone of the inner `Vec<u8>`.
    #[must_use]
    pub fn to_inner(&self) -> Vec<u8> {
        self.0.clone()
    }

    /// Returns a reference to the inner `Vec<u8>`.
    #[must_use]
    pub fn as_inner(&self) -> &Vec<u8> {
        &self.0
    }
}

// [impl jeb-value.variant.common.into-named-inner]
// [impl jeb-value.variant.common.to-named-inner]
// [impl jeb-value.variant.common.as-named-inner]
impl Bytes {
    /// Consumes the `Bytes` and returns the inner `Vec<u8>`.
    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }

    /// Returns a clone of the inner `Vec<u8>`.
    #[must_use]
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }

    /// Returns a reference to the inner `Vec<u8>`.
    #[must_use]
    pub fn as_vec(&self) -> &Vec<u8> {
        &self.0
    }
}

// [impl jeb-value.variant.common.inner-from]
impl From<Bytes> for Vec<u8> {
    fn from(value: Bytes) -> Self {
        value.0
    }
}

// [impl jeb-value.bytes.from-slice]
impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Self {
        Bytes(value.to_vec())
    }
}
