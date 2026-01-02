use {
    super::{
        bytes::Bytes,
        float::Number,
        text::Text,
    },
    derive_more::{
        From,
        IsVariant,
        TryInto,
        TryUnwrap,
        Unwrap,
    },
    indexmap::IndexMap,
};
/// The core Value enum representing all supported data types.
/// [impl value.types.null]
/// [impl value.types.bool]
/// [impl value.types.number]
/// [impl value.types.bytes]
/// [impl value.types.text]
/// [impl value.types.array]
/// [impl value.types.bytes-map]
/// [impl value.types.text-map]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    serde(untagged)
)]
#[derive(Debug, Clone, From, Default, TryInto, IsVariant, TryUnwrap, Unwrap)]
#[must_use]
pub enum Value {
    #[default]
    Null,
    Bool(bool),
    Number(Number),
    Bytes(Bytes),
    Text(Text),
    Array(Vec<Value>),
    BytesMap(IndexMap<Bytes, Value>),
    TextMap(IndexMap<Text, Value>),
}
/// Implements structural equality for values.
/// [impl value.equality.structural]
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            (Unsigned(a), Unsigned(b)) => a == b,
            (Signed(a), Signed(b)) => a == b,
            (Unsigned(a), Signed(b)) => {
                if *b < 0 {
                    false
                } else {
                    u64::try_from(*b) == Ok(*a)
                }
            }
            (Signed(a), Unsigned(b)) => {
                if *a < 0 {
                    false
                } else {
                    u64::try_from(*a) == Ok(*b)
                }
            }
            (Number(a), Number(b)) => a == b,
            (Bytes(a), Bytes(b)) => a == b,
            (Text(a), Text(b)) => a == b,
            (Array(a), Array(b)) => a == b,
            (BytesMap(a), BytesMap(b)) => a == b,
            (TextMap(a), TextMap(b)) => a == b,
            _ => false,
        }
    }
}
/// [impl value.equality.hashable]
impl Eq for Value {}
/// Implements hashing with type discriminants.
/// [impl value.equality.hashable]
/// [impl value.hash.discriminant]
impl core::hash::Hash for Value {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => {
                0u8.hash(state);
            }
            Value::Bool(value) => {
                1u8.hash(state);
                value.hash(state);
            }
            Value::Unsigned(value) => {
                2u8.hash(state);
                value.hash(state);
            }
            Value::Signed(value) => {
                2u8.hash(state);
                value.hash(state);
            }
            Value::Number(value) => {
                3u8.hash(state);
                value.hash(state);
            }
            Value::Bytes(value) => {
                4u8.hash(state);
                value.hash(state);
            }
            Value::Text(value) => {
                5u8.hash(state);
                value.hash(state);
            }
            Value::Array(value) => {
                6u8.hash(state);
                value.hash(state);
            }
            Value::TextMap(value) => {
                7u8.hash(state);
                value.len().hash(state);
                for item in value {
                    item.hash(state);
                }
            }
            Value::BytesMap(value) => {
                8u8.hash(state);
                value.len().hash(state);
                for item in value {
                    item.hash(state);
                }
            }
        }
    }
}
impl Ord for Value {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        use {
            Value::*,
            core::cmp::Ordering::*,
        };
        fn type_rank(value: &Value) -> usize {
            use Value::*;
            match value {
                Bytes(_) => 0,
                Text(_) => 1,
                Unsigned(_) | Signed(_) | Number(_) => 2,
                Array(_) => 3,
                Bool(false) => 4,
                Null => 5,
                Bool(true) => 6,
                BytesMap(_) => 7,
                TextMap(_) => 8,
            }
        }
        let self_rank = type_rank(self);
        let other_rank = type_rank(other);
        match self_rank.cmp(&other_rank) {
            Equal => match (self, other) {
                (Null, Null) => Equal,
                (Bool(left), Bool(right)) => left.cmp(right),
                (Unsigned(left), Unsigned(right)) => left.cmp(right),
                (Signed(left), Signed(right)) => left.cmp(right),
                (Number(left), Number(right)) => left.cmp(right),
                (Bytes(left), Bytes(right)) => left.cmp(right),
                (Text(left), Text(right)) => left.cmp(right),
                (Array(left), Array(right)) => left.cmp(right),
                (BytesMap(left), BytesMap(right)) => left.iter().cmp(right),
                (TextMap(left), TextMap(right)) => left.iter().cmp(right),
                (Unsigned(left), Signed(right)) => {
                    if *right < 0 {
                        Greater
                    } else {
                        match u64::try_from(*right) {
                            Ok(b_as_u64) => left.cmp(&b_as_u64),
                            Err(_) => Less,
                        }
                    }
                }
                (Signed(left), Unsigned(right)) => {
                    if *left < 0 {
                        Less
                    } else {
                        match u64::try_from(*left) {
                            Ok(a_as_u64) => a_as_u64.cmp(right),
                            Err(_) => Greater,
                        }
                    }
                }
                (Unsigned(left), Number(right)) => {
                    if **right < 0.0 {
                        return Greater;
                    }
                    const U64_MAX_PLUS_1: f64 = 18446744073709551616.0;
                    if **right >= U64_MAX_PLUS_1 {
                        return Less;
                    }
                    let right_trunc = right.trunc();
                    let right_int = right_trunc as u64;
                    match left.cmp(&right_int) {
                        Less => Less,
                        Greater => Greater,
                        Equal => Less,
                    }
                }
                (Number(_), Unsigned(_)) => other.cmp(self).reverse(),
                (Signed(left), Number(right)) => {
                    const I64_MAX_PLUS_1: f64 = 9223372036854775808.0;
                    const I64_MIN: f64 = -9223372036854775808.0;
                    if **right >= I64_MAX_PLUS_1 {
                        return Less;
                    }
                    if **right < I64_MIN {
                        return Greater;
                    }
                    let right_trunc = right.trunc();
                    let right_int = right_trunc as i64;
                    match left.cmp(&right_int) {
                        Less => Less,
                        Greater => Greater,
                        Equal => {
                            if **right > right_trunc {
                                Less
                            } else if **right < right_trunc {
                                Greater
                            } else {
                                Less
                            }
                        }
                    }
                }
                (Number(_), Signed(_)) => other.cmp(self).reverse(),
                _ => unreachable!("type_rank equality should prevent this"),
            },
            ord => ord,
        }
    }
}
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
