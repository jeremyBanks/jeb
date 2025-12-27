use {
    super::{
        bytes::Bytes,
        float::Float,
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
    Unsigned(u64),
    Signed(i64),
    Float(Float),
    Bytes(Bytes),
    Text(Text),
    Array(Vec<Value>),
    BytesMap(IndexMap<Bytes, Value>),
    TextMap(IndexMap<Text, Value>),
}


impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            (Unsigned(a), Unsigned(b)) => a == b,
            (Signed(a), Signed(b)) => a == b,
            // Cross-type integer equality: treat Unsigned and Signed as same type
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
            (Float(a), Float(b)) => a == b,
            (Bytes(a), Bytes(b)) => a == b,
            (Text(a), Text(b)) => a == b,
            (Array(a), Array(b)) => a == b,
            (BytesMap(a), BytesMap(b)) => a == b,
            (TextMap(a), TextMap(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

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
                2u8.hash(state); // Integer type discriminant
                value.hash(state);
            }
            Value::Signed(value) => {
                2u8.hash(state); // Same as Unsigned - treat as same type
                value.hash(state);
            }
            Value::Float(value) => {
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
                /* "\b__ */ Bytes(_) => 0,
                /* "____ */ Text(_) => 1,
                /* 0____ */ Unsigned(_) | Signed(_) | Float(_) => 2,
                /* [____ */ Array(_) => 3,
                /* false */ Bool(false) => 4,
                /* null_ */ Null => 5,
                /* true_ */ Bool(true) => 6,
                /* {"\b_ */ BytesMap(_) => 7,
                /* {"___ */ TextMap(_) => 8,
            }
        }

        let self_rank = type_rank(self);
        let other_rank = type_rank(other);

        // First compare by type rank
        match self_rank.cmp(&other_rank) {
            Equal => match (self, other) {
                (Null, Null) => Equal,
                (Bool(left), Bool(right)) => left.cmp(right),
                (Unsigned(left), Unsigned(right)) => left.cmp(right),
                (Signed(left), Signed(right)) => left.cmp(right),
                (Float(left), Float(right)) => left.cmp(right),
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

                (Unsigned(left), Float(right)) => {
                    // Float is guaranteed finite (no NaN/Infinity)
                    if **right < 0.0 {
                        return Greater; // u64 >= 0, so u64 > negative float
                    }

                    // Check if float exceeds u64 range
                    const U64_MAX_PLUS_1: f64 = 18446744073709551616.0; // 2^64
                    if **right >= U64_MAX_PLUS_1 {
                        return Less; // u64 < float (float exceeds u64::MAX)
                    }

                    // Float is in [0, 2^64), safe to truncate and convert
                    let right_trunc = right.trunc();
                    let right_int = right_trunc as u64;

                    match left.cmp(&right_int) {
                        Less => Less,
                        Greater => Greater,
                        Equal => {
                            // Integer parts equal
                            // If right has fractional part: left < right
                            // If exactly equal: Unsigned < Float (tiebreaker)
                            // Either way: Less
                            Less
                        }
                    }
                }
                (Float(_), Unsigned(_)) => other.cmp(self).reverse(),

                (Signed(left), Float(right)) => {
                    // Float is guaranteed finite (no NaN/Infinity)
                    // Check if float exceeds i64 range
                    const I64_MAX_PLUS_1: f64 = 9223372036854775808.0; // 2^63
                    const I64_MIN: f64 = -9223372036854775808.0; // -2^63

                    if **right >= I64_MAX_PLUS_1 {
                        return Less; // i64 < float (float exceeds i64::MAX)
                    }
                    if **right < I64_MIN {
                        return Greater; // i64 > float (float below i64::MIN)
                    }

                    // Float is in [i64::MIN, i64::MAX + 1), safe to truncate and convert
                    let right_trunc = right.trunc();
                    let right_int = right_trunc as i64;

                    match left.cmp(&right_int) {
                        Less => Less,
                        Greater => Greater,
                        Equal => {
                            // Integer parts equal; check fractional part
                            if **right > right_trunc {
                                Less // left < right (right has positive fractional part)
                            } else if **right < right_trunc {
                                Greater // left > right (right has negative fractional part)
                            } else {
                                // Exactly equal: Signed < Float (tiebreaker)
                                Less
                            }
                        }
                    }
                }
                (Float(_), Signed(_)) => other.cmp(self).reverse(),

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
