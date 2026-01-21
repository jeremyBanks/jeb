use {
    crate::{Bytes, Value},
    core::hash::Hash,
    derive_more::{AsMut, AsRef, Deref, DerefMut, From, IntoIterator},
    indexmap::IndexMap,
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
    AsMut, AsRef, Clone, Debug, Default, Deref, DerefMut, Eq, From, IntoIterator, PartialEq,
)]
// [impl jeb-value.variant.common.transparent]
#[repr(transparent)]
#[into_iterator(owned, ref, ref_mut)]
// [impl jeb-value.variant.common.must-use]
#[must_use]
// [impl jeb-value.bytes-map.struct]
pub struct BytesMap(pub(crate) IndexMap<Bytes, Value>);

// [impl jeb-value.variant.common.constructor]
// [impl jeb-value.variant.common.try-from-inner]
// [impl jeb-value.bytes-map.from-inner]
impl BytesMap {
    /// Creates a new `BytesMap` from an `IndexMap<Bytes, Value>`.
    #[must_use]
    pub fn new(value: IndexMap<Bytes, Value>) -> Self {
        BytesMap(value)
    }
}

// [impl jeb-value.variant.common.into-inner]
// [impl jeb-value.variant.common.to-inner]
// [impl jeb-value.variant.common.as-inner]
impl BytesMap {
    /// Consumes the `BytesMap` and returns the inner map.
    #[must_use]
    pub fn into_inner(self) -> IndexMap<Bytes, Value> {
        self.0
    }

    /// Returns a clone of the inner map.
    #[must_use]
    pub fn to_inner(&self) -> IndexMap<Bytes, Value> {
        self.0.clone()
    }

    /// Returns a reference to the inner map.
    #[must_use]
    pub fn as_inner(&self) -> &IndexMap<Bytes, Value> {
        &self.0
    }
}

// [impl jeb-value.variant.common.into-named-inner]
// [impl jeb-value.variant.common.to-named-inner]
// [impl jeb-value.variant.common.as-named-inner]
impl BytesMap {
    /// Consumes the `BytesMap` and returns the inner map.
    #[must_use]
    pub fn into_index_map(self) -> IndexMap<Bytes, Value> {
        self.0
    }

    /// Returns a clone of the inner map.
    #[must_use]
    pub fn to_index_map(&self) -> IndexMap<Bytes, Value> {
        self.0.clone()
    }

    /// Returns a reference to the inner map.
    #[must_use]
    pub fn as_index_map(&self) -> &IndexMap<Bytes, Value> {
        &self.0
    }
}

// [impl jeb-value.bytes-map.len]
// [impl jeb-value.bytes-map.is-empty]
// [impl jeb-value.bytes-map.keys]
// [impl jeb-value.bytes-map.values]
// [impl jeb-value.bytes-map.iter]
impl BytesMap {
    /// Returns the number of key-value pairs in the map.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the map contains no key-value pairs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the keys of the map.
    pub fn keys(&self) -> impl Iterator<Item = &Bytes> {
        self.0.keys()
    }

    /// Returns an iterator over the values of the map.
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.0.values()
    }

    /// Returns an iterator over the key-value pairs of the map.
    pub fn iter(&self) -> impl Iterator<Item = (&Bytes, &Value)> {
        self.0.iter()
    }
}

// [impl jeb-value.bytes-map.get]
// [impl jeb-value.bytes-map.contains-key]
// [impl jeb-value.bytes-map.generic-access]
impl BytesMap {
    /// Returns a reference to the value associated with the key.
    pub fn get(&self, key: &Bytes) -> Option<&Value> {
        self.0.get(key)
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key(&self, key: &Bytes) -> bool {
        self.0.contains_key(key)
    }
}

// [impl jeb-value.variant.common.inner-from]
impl From<BytesMap> for IndexMap<Bytes, Value> {
    fn from(value: BytesMap) -> Self {
        value.0
    }
}

// [impl jeb-value.variant.common.hash]
// Note: IndexMap doesn't implement Hash, so we implement it manually
impl core::hash::Hash for BytesMap {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.len().hash(state);
        for item in &self.0 {
            item.hash(state);
        }
    }
}

// [impl jeb-value.variant.common.ord]
impl Ord for BytesMap {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.iter().cmp(other.0.iter())
    }
}

// [impl jeb-value.variant.common.partial-ord]
impl PartialOrd for BytesMap {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// [impl jeb-value.bytes-map.index]
impl core::ops::Index<&Bytes> for BytesMap {
    type Output = Value;

    fn index(&self, key: &Bytes) -> &Self::Output {
        &self.0[key]
    }
}

impl FromIterator<(Bytes, Value)> for BytesMap {
    fn from_iter<T: IntoIterator<Item = (Bytes, Value)>>(iter: T) -> Self {
        BytesMap(iter.into_iter().collect())
    }
}
