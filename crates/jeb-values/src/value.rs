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
                            Ok(b_as_u64) => match left.cmp(&b_as_u64) {
                                Equal => Less,
                                ord => ord,
                            },
                            Err(_) => Less,
                        }
                    }
                }
                (Signed(_), Unsigned(_)) => other.cmp(self).reverse(),

                (Unsigned(left), Float(right)) => {
                    let left_as_f64 = *left as f64;
                    match left_as_f64.total_cmp(right) {
                        Equal => Less,
                        ord => ord,
                    }
                }
                (Float(_), Unsigned(_)) => other.cmp(self).reverse(),

                (Signed(left), Float(right)) => {
                    let left_as_f64 = *left as f64;
                    match left_as_f64.total_cmp(right) {
                        Equal => Less,
                        ord => ord,
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
