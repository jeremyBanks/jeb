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

mod from;

#[cfg_attr(
    feature = "serde",
    derive(
        serde::Serialize,
        serde::Deserialize
    ),
    serde(untagged)
)]
#[derive(Debug, Clone, From, Default, TryInto, IsVariant, TryUnwrap, Unwrap, Eq, PartialEq)]
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


impl core::hash::Hash for Value {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Value::Null => {}
            Value::Bool(value) => value.hash(state),
            Value::Unsigned(value) => value.hash(state),
            Value::Signed(value) => value.hash(state),
            Value::Float(value) => value.hash(state),
            Value::Bytes(value) => value.hash(state),
            Value::Text(value) => value.hash(state),
            Value::Array(value) => value.hash(state),
            Value::TextMap(value) => {
                value.len().hash(state);
                for item in value {
                    item.hash(state);
                }
            }
            Value::BytesMap(value) => {
                value.len().hash(state);
                for item in value {
                    item.hash(state);
                }
            }
        }
    }
}

impl Value {
    /// Total ordering by type complexity and value.
    ///
    /// Order hierarchy:
    /// 1. Null (least complex)
    /// 2. Bool
    /// 3. Numbers (compared numerically when possible, then by type: Unsigned < Signed < Float)
    /// 4. Bytes
    /// 5. Text
    /// 6. Array
    /// 7. BytesMap
    /// 8. TextMap (most complex)
    #[must_use]
    pub fn cmp_by_complexity(&self, other: &Self) -> core::cmp::Ordering {
        use core::cmp::Ordering;

        // Helper to get type rank
        let type_rank = |v: &Value| match v {
            Value::Null => 0,
            Value::Bool(_) => 1,
            Value::Unsigned(_) | Value::Signed(_) | Value::Float(_) => 2,
            Value::Bytes(_) => 3,
            Value::Text(_) => 4,
            Value::Array(_) => 5,
            Value::BytesMap(_) => 6,
            Value::TextMap(_) => 7,
        };

        let self_rank = type_rank(self);
        let other_rank = type_rank(other);

        // First compare by type rank
        match self_rank.cmp(&other_rank) {
            Ordering::Equal => {
                // Same rank, compare within type
                match (self, other) {
                    (Value::Null, Value::Null) => Ordering::Equal,
                    (Value::Bool(a), Value::Bool(b)) => a.cmp(b),

                    // Numbers: try numeric comparison first, then fall back to type ordering
                    (Value::Unsigned(a), Value::Unsigned(b)) => a.cmp(b),
                    (Value::Signed(a), Value::Signed(b)) => a.cmp(b),
                    (Value::Float(a), Value::Float(b)) => a.cmp(b),

                    // Cross-number comparisons: compare numerically if possible
                    (Value::Unsigned(a), Value::Signed(b)) => {
                        // If signed is negative, unsigned is always greater
                        if *b < 0 {
                            Ordering::Greater
                        } else {
                            // Both non-negative, compare as u64 if possible
                            match u64::try_from(*b) {
                                Ok(b_as_u64) => match a.cmp(&b_as_u64) {
                                    Ordering::Equal => Ordering::Less, // Unsigned < Signed for same value
                                    ord => ord,
                                },
                                Err(_) => Ordering::Less, // b too large for u64
                            }
                        }
                    }
                    (Value::Signed(_), Value::Unsigned(_)) => {
                        other.cmp_by_complexity(self).reverse()
                    }

                    (Value::Unsigned(a), Value::Float(b)) => {
                        let a_as_f64 = *a as f64;
                        match a_as_f64.total_cmp(&**b) {
                            Ordering::Equal => Ordering::Less, // Unsigned < Float for same value
                            ord => ord,
                        }
                    }
                    (Value::Float(_), Value::Unsigned(_)) => {
                        other.cmp_by_complexity(self).reverse()
                    }

                    (Value::Signed(a), Value::Float(b)) => {
                        let a_as_f64 = *a as f64;
                        match a_as_f64.total_cmp(&**b) {
                            Ordering::Equal => Ordering::Less, // Signed < Float for same value
                            ord => ord,
                        }
                    }
                    (Value::Float(_), Value::Signed(_)) => {
                        other.cmp_by_complexity(self).reverse()
                    }

                    (Value::Bytes(a), Value::Bytes(b)) => a.cmp(b),
                    (Value::Text(a), Value::Text(b)) => a.cmp(b),
                    (Value::Array(a), Value::Array(b)) => {
                        // Lexicographic comparison
                        for (a_item, b_item) in a.iter().zip(b.iter()) {
                            match a_item.cmp_by_complexity(b_item) {
                                Ordering::Equal => continue,
                                ord => return ord,
                            }
                        }
                        a.len().cmp(&b.len())
                    }
                    (Value::BytesMap(a), Value::BytesMap(b)) => {
                        // Lexicographic comparison by key-value pairs
                        for (a_item, b_item) in a.iter().zip(b.iter()) {
                            match a_item.0.cmp(b_item.0) {
                                Ordering::Equal => match a_item.1.cmp_by_complexity(&b_item.1) {
                                    Ordering::Equal => continue,
                                    ord => return ord,
                                },
                                ord => return ord,
                            }
                        }
                        a.len().cmp(&b.len())
                    }
                    (Value::TextMap(a), Value::TextMap(b)) => {
                        // Lexicographic comparison by key-value pairs
                        for (a_item, b_item) in a.iter().zip(b.iter()) {
                            match a_item.0.cmp(b_item.0) {
                                Ordering::Equal => match a_item.1.cmp_by_complexity(&b_item.1) {
                                    Ordering::Equal => continue,
                                    ord => return ord,
                                },
                                ord => return ord,
                            }
                        }
                        a.len().cmp(&b.len())
                    }

                    _ => unreachable!("type_rank equality should prevent this"),
                }
            }
            ord => ord,
        }
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.cmp_by_complexity(other)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
