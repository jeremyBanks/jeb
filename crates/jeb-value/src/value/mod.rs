use {
    super::{boolean::Boolean, bytes::Bytes, null::Null, number::Number, string::String},
    derive_more::{From, IsVariant, TryInto, TryUnwrap, Unwrap},
    indexmap::IndexMap,
};

// [impl jeb-value.features.core.cfg]
// [impl jeb-value.features.serde.optional]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(untagged))]
// [impl jeb-value.value.traits.clone]
// [impl jeb-value.value.traits.debug]
#[derive(Debug, Clone, From, IsVariant, TryUnwrap, Unwrap)]
// [impl jeb-value.value.traits.must-use]
#[must_use]
// [impl jeb-value.value.def.enum-variants]
// [impl jeb-value.value.def.variant-types]
pub enum Value {
    Null(#[from] Null),
    Boolean(Boolean),
    Number(Number),
    Bytes(Bytes),
    String(String),
    Array(Vec<Value>),
    BytesMap(IndexMap<Bytes, Value>),
    StringMap(IndexMap<String, Value>),
}

// [impl jeb-value.value.traits.partial-eq]
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Null(a), Null(b)) => a == b,
            (Boolean(a), Boolean(b)) => a == b,
            (Number(a), Number(b)) => a == b,
            (Bytes(a), Bytes(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Array(a), Array(b)) => a == b,
            (BytesMap(a), BytesMap(b)) => a == b,
            (StringMap(a), StringMap(b)) => a == b,
            _ => false,
        }
    }
}

// [impl jeb-value.value.traits.eq]
impl Eq for Value {}

// [impl jeb-value.value.traits.hash]
impl core::hash::Hash for Value {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::Null(value) => value.hash(state),
            Value::Boolean(value) => value.hash(state),
            Value::Number(value) => value.hash(state),
            Value::Bytes(value) => value.hash(state),
            Value::String(value) => value.hash(state),
            Value::Array(value) => value.hash(state),
            Value::BytesMap(value) => {
                value.len().hash(state);
                for item in value {
                    item.hash(state);
                }
            }
            Value::StringMap(value) => {
                value.len().hash(state);
                for item in value {
                    item.hash(state);
                }
            }
        }
    }
}

// [impl jeb-value.value.traits.ord]
// [impl jeb-value.variant.common.cmp-delegate-variants]
// [impl jeb-value.variant.common.cmp-mixed-variants]
impl Ord for Value {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        use {core::cmp::Ordering::*, Value::*};

        // [impl jeb-value.src.ordering.spec]
        // Order: Null, Boolean, Number, Bytes, String, Array, BytesMap, StringMap
        fn type_rank(value: &Value) -> usize {
            match value {
                Null(_) => 0,
                Boolean(_) => 1,
                Number(_) => 2,
                Bytes(_) => 3,
                String(_) => 4,
                Array(_) => 5,
                BytesMap(_) => 6,
                StringMap(_) => 7,
            }
        }

        let self_rank = type_rank(self);
        let other_rank = type_rank(other);

        match self_rank.cmp(&other_rank) {
            Equal => match (self, other) {
                (Null(left), Null(right)) => left.cmp(right),
                (Boolean(left), Boolean(right)) => left.cmp(right),
                (Number(left), Number(right)) => left.cmp(right),
                (Bytes(left), Bytes(right)) => left.cmp(right),
                (String(left), String(right)) => left.cmp(right),
                (Array(left), Array(right)) => left.cmp(right),
                (BytesMap(left), BytesMap(right)) => left.iter().cmp(right.iter()),
                (StringMap(left), StringMap(right)) => left.iter().cmp(right.iter()),
                _ => unreachable!("type_rank equality should prevent this"),
            },
            ord => ord,
        }
    }
}

// [impl jeb-value.value.traits.partial-ord]
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// [impl jeb-value.value.traits.default]
impl Default for Value {
    fn default() -> Self {
        Value::Null(Null::new())
    }
}

// [impl jeb-value.value.accessors.from]
// (covered by derive(From))

// [impl jeb-value.value.accessors.as]
// [impl jeb-value.value.accessors.to]
// [impl jeb-value.value.accessors.into]
// [impl jeb-value.value.accessors.unwrap]
// (covered by derive(TryInto, TryUnwrap, Unwrap, IsVariant))
