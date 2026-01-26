use {
    crate::{
        String,
        Value,
    },
    core::hash::Hash,
    derive_more::{
        AsMut,
        AsRef,
        Deref,
        DerefMut,
        From,
        IntoIterator,
    },
    ordermap::OrderMap,
};

// [impl jeb-value.features.core.cfg]
// [impl jeb-value.features.serde.optional]
#[cfg_attr(
    feature = "serde",
    derive(
        serde::Serialize,
        serde::Deserialize
    ),
    serde(transparent)
)]
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
#[into_iterator(
    owned, ref, ref_mut
)]
// [impl jeb-value.variant.common.must-use]
#[must_use]
// [impl jeb-value.string-map.struct]
pub struct StringMap(pub(crate) OrderMap<String, Value>);

// [impl jeb-value.variant.common.constructor]
// [impl jeb-value.variant.common.try-from-inner]
// [impl jeb-value.string-map.from-inner]
impl StringMap {
    /// Creates a new `StringMap` from an `OrderMap<String, Value>`.
    pub fn new(value: OrderMap<String, Value>) -> Self {
        StringMap(value)
    }
}

// [impl jeb-value.variant.common.into-inner]
// [impl jeb-value.variant.common.to-inner]
// [impl jeb-value.variant.common.as-inner]
impl StringMap {
    /// Consumes the `StringMap` and returns the inner map.
    #[must_use]
    pub fn into_inner(self) -> OrderMap<String, Value> {
        self.0
    }

    /// Returns a clone of the inner map.
    #[must_use]
    pub fn to_inner(&self) -> OrderMap<String, Value> {
        self.0.clone()
    }

    /// Returns a reference to the inner map.
    #[must_use]
    pub fn as_inner(&self) -> &OrderMap<String, Value> {
        &self.0
    }
}

// [impl jeb-value.variant.common.into-named-inner]
// [impl jeb-value.variant.common.to-named-inner]
// [impl jeb-value.variant.common.as-named-inner]
impl StringMap {
    /// Consumes the `StringMap` and returns the inner map.
    #[must_use]
    pub fn into_index_map(self) -> OrderMap<String, Value> {
        self.0
    }

    /// Returns a clone of the inner map.
    #[must_use]
    pub fn to_index_map(&self) -> OrderMap<String, Value> {
        self.0.clone()
    }

    /// Returns a reference to the inner map.
    #[must_use]
    pub fn as_index_map(&self) -> &OrderMap<String, Value> {
        &self.0
    }
}

// [impl jeb-value.string-map.len]
// [impl jeb-value.string-map.is-empty]
// [impl jeb-value.string-map.keys]
// [impl jeb-value.string-map.values]
// [impl jeb-value.string-map.iter]
impl StringMap {
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
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }

    /// Returns an iterator over the values of the map.
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.0.values()
    }

    /// Returns an iterator over the key-value pairs of the map.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0.iter()
    }
}

// [impl jeb-value.string-map.get]
// [impl jeb-value.string-map.contains-key]
// [impl jeb-value.string-map.generic-access]
impl StringMap {
    /// Returns a reference to the value associated with the key.
    pub fn get(&self, key: &String) -> Option<&Value> {
        self.0.get(key)
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key(&self, key: &String) -> bool {
        self.0.contains_key(key)
    }
}

// [impl jeb-value.variant.common.inner-from]
impl From<StringMap> for OrderMap<String, Value> {
    fn from(value: StringMap) -> Self {
        value.0
    }
}

// [impl jeb-value.variant.common.hash]
// Note: ordermap doesn't implement Hash, so we implement it manually
impl Hash for StringMap {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.len().hash(state);
        for item in &self.0 {
            item.hash(state);
        }
    }
}

// [impl jeb-value.variant.common.ord]
impl Ord for StringMap {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.iter().cmp(other.0.iter())
    }
}

// [impl jeb-value.variant.common.partial-ord]
impl PartialOrd for StringMap {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// [impl jeb-value.string-map.index]
impl core::ops::Index<&String> for StringMap {
    type Output = Value;

    fn index(&self, key: &String) -> &Self::Output {
        &self.0[key]
    }
}

impl FromIterator<(String, Value)> for StringMap {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        StringMap(iter.into_iter().collect())
    }
}
