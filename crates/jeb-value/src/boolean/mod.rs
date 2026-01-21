use derive_more::{AsRef, Deref, Display, From, Into};

pub mod std;

// [impl jeb-value.features.core.cfg]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(transparent))]
// [impl jeb-value.variant.common.clone]
// [impl jeb-value.variant.common.debug]
// [impl jeb-value.variant.common.deref]
// [impl jeb-value.variant.common.as-ref]
// [impl jeb-value.variant.common.eq-delegate-inner]
// [impl jeb-value.variant.common.partial-eq-delegate-inner]
// [impl jeb-value.variant.common.ord-delegate-inner]
// [impl jeb-value.variant.common.partial-ord-delegate-inner]
// [impl jeb-value.variant.common.hash-delegate-inner]
#[derive(
    AsRef, Clone, Copy, Debug, Default, Deref, Display, Eq, From, Hash, Ord, PartialEq, PartialOrd,
)]
// [impl jeb-value.variant.common.transparent]
#[repr(transparent)]
// [impl jeb-value.variant.common.must-use]
#[must_use]
// [impl jeb-value.boolean.struct]
pub struct Boolean(pub(crate) bool);

// [impl jeb-value.variant.common.constructor]
// [impl jeb-value.variant.common.try-from-inner]
impl Boolean {
    /// Creates a new `Boolean` from a `bool`.
    #[must_use]
    pub const fn new(value: bool) -> Self {
        Boolean(value)
    }
}

// [impl jeb-value.variant.common.into-inner]
// [impl jeb-value.variant.common.to-inner]
// [impl jeb-value.variant.common.as-inner]
impl Boolean {
    /// Consumes the `Boolean` and returns the inner `bool`.
    #[must_use]
    pub fn into_inner(self) -> bool {
        self.0
    }

    /// Returns a copy of the inner `bool`.
    #[must_use]
    pub fn to_inner(&self) -> bool {
        self.0
    }

    /// Returns a reference to the inner `bool`.
    #[must_use]
    pub fn as_inner(&self) -> &bool {
        &self.0
    }
}

// [impl jeb-value.variant.common.into-named-inner]
// [impl jeb-value.variant.common.to-named-inner]
// [impl jeb-value.variant.common.as-named-inner]
impl Boolean {
    /// Consumes the `Boolean` and returns the inner `bool`.
    #[must_use]
    pub fn into_bool(self) -> bool {
        self.0
    }

    /// Returns a copy of the inner `bool`.
    #[must_use]
    pub fn to_bool(&self) -> bool {
        self.0
    }

    /// Returns a reference to the inner `bool`.
    #[must_use]
    pub fn as_bool(&self) -> &bool {
        &self.0
    }
}

// [impl jeb-value.variant.common.inner-from]
impl From<Boolean> for bool {
    fn from(value: Boolean) -> Self {
        value.0
    }
}

// [impl jeb-value.boolean.from-inner]
// (already covered by derive(From))

// [impl jeb-value.boolean.from-false]
impl From<()> for Boolean {
    fn from(_: ()) -> Self {
        Boolean(false)
    }
}
