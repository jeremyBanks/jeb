use derive_more::{AsMut, AsRef, Deref, DerefMut, From};

mod std;
mod value;

// Note: Serde impls would go in null/serde.rs
// [impl jeb-value.variant.common.clone]
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
    AsMut, AsRef, Clone, Copy, Debug, Default, Deref, DerefMut, Eq, From, Hash, Ord, PartialEq,
    PartialOrd,
)]
// [impl jeb-value.variant.common.transparent]
#[repr(transparent)]
// [impl jeb-value.null.must-use]
// Note: Null is NOT marked #[must_use] per spec
// [impl jeb-value.null.struct]
// [impl jeb-value.null.from-inner]
pub struct Null(pub(crate) ());

// [impl jeb-value.variant.common.constructor]
impl Null {
    /// Creates a new `Null`.
    pub const fn new() -> Self {
        Null(())
    }
}

// [impl jeb-value.variant.common.into-inner]
// [impl jeb-value.variant.common.to-inner]
// [impl jeb-value.variant.common.as-inner]
impl Null {
    /// Consumes the `Null` and returns the inner `()`.
    pub fn into_inner(self) -> () {
        self.0
    }

    /// Returns a copy of the inner `()`.
    pub fn to_inner(&self) -> () {
        self.0
    }

    /// Returns a reference to the inner `()`.
    pub fn as_inner(&self) -> &() {
        &self.0
    }
}

// [impl jeb-value.variant.common.into-named-inner]
// [impl jeb-value.variant.common.to-named-inner]
// [impl jeb-value.variant.common.as-named-inner]
impl Null {
    /// Consumes the `Null` and returns the inner `()`.
    pub fn into_unit(self) -> () {
        self.0
    }

    /// Returns a copy of the inner `()`.
    pub fn to_unit(&self) -> () {
        self.0
    }

    /// Returns a reference to the inner `()`.
    pub fn as_unit(&self) -> &() {
        &self.0
    }
}

// [impl jeb-value.variant.common.inner-from]
// [impl jeb-value.null.into-unit]
impl From<Null> for () {
    fn from(_value: Null) -> Self {}
}
