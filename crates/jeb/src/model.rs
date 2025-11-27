use {
    crate::Panic,
    core::hash::Hash,
    derive_more::{
        AsMut, AsRef, Deref, DerefMut, Display, From, Index, IndexMut, Into, IntoIterator,
        IsVariant, TryUnwrap, Unwrap,
    },
    indexmap::IndexMap,
    serde::{Deserialize, Serialize},
    tokio::task::JoinHandle,
};



pub type Task = JoinHandle<Result<(), Panic>>;

pub type Sender = tokio::sync::mpsc::Sender<Item>;

pub type Receiver = tokio_stream::wrappers::ReceiverStream<Item>;

#[must_use]
pub fn channel() -> (Sender, Receiver) {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    let receiver = tokio_stream::wrappers::ReceiverStream::new(receiver);

    (sender, receiver)
}

pub trait Node {
    fn spawn(&self, stack: Vec<Receiver>) -> (Vec<Receiver>, Task);
}



#[derive(Debug, Clone, From, Serialize, Deserialize, TryUnwrap, IsVariant, Unwrap)]
#[serde(untagged)]
#[must_use]
pub enum Item {
    Bytes(Bytes),
    Text(Text),
    Value(Value),
}

impl Default for Item {
    fn default() -> Self {
        Item::Bytes(Bytes::default())
    }
}



#[derive(Debug, Clone, From, Serialize, Deserialize, Default, TryUnwrap, IsVariant, Unwrap)]
#[serde(untagged)]
#[must_use]
pub enum Value {
    Unsigned(u64),
    Signed(i64),
    Float(Float),
    Bool(bool),
    #[default]
    Null,
    Text(Text),
    Bytes(Bytes),
    Array(Vec<Value>),
    BytesMap(IndexMap<Bytes, Value>),
    TextMap(IndexMap<Text, Value>),
}



#[derive(AsRef, Clone, Debug, Default, Deref, Copy, Display, Index, Into, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
#[must_use]
pub struct Float(f64);

impl Float {
    #[must_use]
    pub const fn new(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(Float(value))
        } else {
            None
        }
    }
}

impl<'de> Deserialize<'de> for Float {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Float::new(value).ok_or_else(|| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Float(value),
                &"a finite floating point number",
            )
        })
    }
}

impl Ord for Float {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == core::cmp::Ordering::Equal
    }
}

impl Eq for Float {}

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Float {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.to_bits());
    }
}



#[derive(
    AsMut,
    AsRef,
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    Deserialize,
    Eq,
    From,
    Hash,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[repr(transparent)]
#[serde(transparent)]
#[must_use]
#[into_iterator(owned, ref, ref_mut)]
pub struct Bytes(Vec<u8>);



#[derive(
    AsMut,
    AsRef,
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    Deserialize,
    Display,
    Eq,
    From,
    Hash,
    Index,
    Into,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(transparent)]
#[repr(transparent)]
#[must_use]
pub struct Text(String);



// MARK: Bytes conversions

impl From<&[u8]> for Bytes {
    fn from(value: &[u8]) -> Self {
        Bytes(value.to_vec())
    }
}

impl From<&str> for Bytes {
    fn from(value: &str) -> Self {
        Bytes(value.as_bytes().to_vec())
    }
}

impl From<Text> for Bytes {
    fn from(value: Text) -> Self {
        Bytes(value.0.into_bytes())
    }
}

// MARK: Text conversions

impl TryFrom<Bytes> for Text {
    type Error = core::str::Utf8Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let s = core::str::from_utf8(&value)?;
        Ok(Text(s.to_string()))
    }
}

// MARK: Float conversions

impl TryFrom<f64> for Float {
    type Error = f64;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Float::new(value).ok_or(value)
    }
}

impl TryFrom<f32> for Float {
    type Error = f32;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Float::new(value.into()).ok_or(value)
    }
}

impl From<i32> for Float {
    fn from(value: i32) -> Self {
        Float(value.into())
    }
}

impl From<u32> for Float {
    fn from(value: u32) -> Self {
        Float(value.into())
    }
}

impl From<i16> for Float {
    fn from(value: i16) -> Self {
        Float(value.into())
    }
}

impl From<u16> for Float {
    fn from(value: u16) -> Self {
        Float(value.into())
    }
}

impl From<i8> for Float {
    fn from(value: i8) -> Self {
        Float(value.into())
    }
}

impl From<u8> for Float {
    fn from(value: u8) -> Self {
        Float(value.into())
    }
}


// MARK: Value conversions

impl TryFrom<f32> for Value {
    type Error = f32;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Float::try_from(value).map(Value::from)
    }
}

impl TryFrom<f64> for Value {
    type Error = f64;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Float::try_from(value).map(Value::from)
    }
}

impl TryFrom<u128> for Value {
    type Error = u128;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        u64::try_from(value).map(Value::Unsigned).map_err(|_| value)
    }
}

impl TryFrom<i128> for Value {
    type Error = i128;

    fn try_from(value: i128) -> Result<Self, Self::Error> {
        i64::try_from(value).map(Value::Signed).map_err(|_| value)
    }
}

impl From<()> for Value {
    fn from((): ()) -> Self {
        Value::Null
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::Unsigned(value.into())
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::Unsigned(value.into())
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::Unsigned(value.into())
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Signed(value.into())
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::Signed(value.into())
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::Signed(value.into())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value.into())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Text(value.to_string().into())
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Bytes(value.into())
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Self {
        Value::Bytes(value.into())
    }
}

impl FromIterator<Value> for Value {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Value::Array(iter.into_iter().collect())
    }
}

impl<const N: usize> From<[Value; N]> for Value {
    fn from(value: [Value; N]) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<(Text, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (Text, Value)>>(iter: T) -> Self {
        Value::TextMap(iter.into_iter().collect())
    }
}

impl FromIterator<(String, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Value::TextMap(iter.into_iter().map(|(k, v)| (Text::from(k), v)).collect())
    }
}

impl<'a> FromIterator<(&'a str, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (&'a str, Value)>>(iter: T) -> Self {
        Value::TextMap(
            iter.into_iter()
                .map(|(k, v)| (Text::from(k.to_string()), v))
                .collect(),
        )
    }
}

impl<const N: usize> From<[(Text, Value); N]> for Value {
    fn from(value: [(Text, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(String, Value); N]> for Value {
    fn from(value: [(String, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(&str, Value); N]> for Value {
    fn from(value: [(&str, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl FromIterator<(Bytes, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (Bytes, Value)>>(iter: T) -> Self {
        Value::BytesMap(iter.into_iter().collect())
    }
}

impl FromIterator<(Vec<u8>, Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (Vec<u8>, Value)>>(iter: T) -> Self {
        Value::BytesMap(iter.into_iter().map(|(k, v)| (Bytes::from(k), v)).collect())
    }
}

impl<'a> FromIterator<(&'a [u8], Value)> for Value {
    fn from_iter<T: IntoIterator<Item = (&'a [u8], Value)>>(iter: T) -> Self {
        Value::BytesMap(iter.into_iter().map(|(k, v)| (Bytes::from(k), v)).collect())
    }
}

impl<const N: usize> From<[(Bytes, Value); N]> for Value {
    fn from(value: [(Bytes, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(Vec<u8>, Value); N]> for Value {
    fn from(value: [(Vec<u8>, Value); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<const N: usize> From<[(&[u8], Value); N]> for Value {
    fn from(value: [(&[u8], Value); N]) -> Self {
        value.into_iter().collect()
    }
}
