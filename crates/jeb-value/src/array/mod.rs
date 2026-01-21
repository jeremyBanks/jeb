use {
    crate::Value,
    derive_more::{AsMut, AsRef, Deref, DerefMut, From, Index, IndexMut, IntoIterator},
};

// [impl jeb-value.features.core.cfg]
// [impl jeb-value.features.serde.optional]
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
// [impl jeb-value.variant.common.borrow]
// [impl jeb-value.variant.common.mut]
#[derive(
    AsMut, AsRef, Clone, Debug, Default, Deref, DerefMut, Eq, From, Hash, Index, IndexMut,
    IntoIterator, Ord, PartialEq, PartialOrd,
)]
// [impl jeb-value.variant.common.transparent]
#[repr(transparent)]
#[into_iterator(owned, ref, ref_mut)]
// [impl jeb-value.variant.common.must-use]
#[must_use]
// [impl jeb-value.array.struct]
pub struct Array(pub(crate) Vec<Value>);

// [impl jeb-value.variant.common.constructor]
// [impl jeb-value.variant.common.try-from-inner]
// [impl jeb-value.array.from-inner]
impl Array {
    /// Creates a new `Array` from a `Vec<Value>`.
    #[must_use]
    pub fn new(value: Vec<Value>) -> Self {
        Array(value)
    }
}

// [impl jeb-value.variant.common.into-inner]
// [impl jeb-value.variant.common.to-inner]
// [impl jeb-value.variant.common.as-inner]
impl Array {
    /// Consumes the `Array` and returns the inner `Vec<Value>`.
    #[must_use]
    pub fn into_inner(self) -> Vec<Value> {
        self.0
    }

    /// Returns a clone of the inner `Vec<Value>`.
    #[must_use]
    pub fn to_inner(&self) -> Vec<Value> {
        self.0.clone()
    }

    /// Returns a reference to the inner `Vec<Value>`.
    #[must_use]
    pub fn as_inner(&self) -> &Vec<Value> {
        &self.0
    }
}

// [impl jeb-value.variant.common.into-named-inner]
// [impl jeb-value.variant.common.to-named-inner]
// [impl jeb-value.variant.common.as-named-inner]
impl Array {
    /// Consumes the `Array` and returns the inner `Vec<Value>`.
    #[must_use]
    pub fn into_vec(self) -> Vec<Value> {
        self.0
    }

    /// Returns a clone of the inner `Vec<Value>`.
    #[must_use]
    pub fn to_vec(&self) -> Vec<Value> {
        self.0.clone()
    }

    /// Returns a reference to the inner `Vec<Value>`.
    #[must_use]
    pub fn as_vec(&self) -> &Vec<Value> {
        &self.0
    }
}

// [impl jeb-value.array.len]
// [impl jeb-value.array.is-empty]
// [impl jeb-value.array.iter]
impl Array {
    /// Returns the number of elements in the array.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the array contains no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the values in the array.
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.0.iter()
    }
}

// [impl jeb-value.variant.common.inner-from]
impl From<Array> for Vec<Value> {
    fn from(value: Array) -> Self {
        value.0
    }
}

// [impl jeb-value.array.from-slice]
impl From<&[Value]> for Array {
    fn from(value: &[Value]) -> Self {
        Array(value.to_vec())
    }
}

// [impl jeb-value.array.from-iterator]
impl FromIterator<Value> for Array {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Array(iter.into_iter().collect())
    }
}
