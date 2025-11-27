use {
    crate::Panic,
    core::hash::Hash,
    derive_more::{
        AsMut, AsRef, Deref, DerefMut, Display, From, Index, IndexMut, Into, IntoIterator, TryInto,
    },
    indexmap::IndexMap,
    serde::{Deserialize, Serialize, de::value},
    tokio::{io::BufReader, task::JoinHandle},
    tokio_util::codec::{BytesCodec, FramedRead},
};
 use tokio_stream::StreamExt;

pub type Sender = tokio_util::sync::PollSender<Item>;

pub type Receiver = tokio_stream::wrappers::ReceiverStream<Item>;

#[must_use]
pub fn channel() -> (Sender, Receiver) {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    let sender = tokio_util::sync::PollSender::new(sender);
    let receiver = tokio_stream::wrappers::ReceiverStream::new(receiver);

    (sender, receiver)
}

pub trait Node {
    fn spawn(stack: Vec<Receiver>) -> (Vec<Receiver>, JoinHandle<Result<(), Panic>>);
}

struct Stdin;
impl Node for Stdin {
    fn spawn(mut stack: Vec<Receiver>) -> (Vec<Receiver>, JoinHandle<Result<(), Panic>>) {
        let (sender, receiver) = channel();

        let handle = tokio::spawn(async move {
            let mut stdin = tokio::io::stdin();

            let mut stdin_bytes: FramedRead<tokio::io::Stdin, BytesCodec> = FramedRead::new(stdin, BytesCodec::new());

            while let Some(value) = stdin_bytes.next().await {
                let vec = value?.to_vec();
                let bytes = Bytes::from(vec);
                sender.get_ref().unwrap().send(bytes.into()).await?;
            }

            Ok(())
        });

        stack.push(receiver);

        (stack, handle)
    }
}

#[derive(Debug, Clone, From, Serialize, Deserialize)]
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

#[derive(Debug, Clone, From, Serialize, Deserialize, Default)]
#[serde(untagged)]
#[must_use]
pub enum Value {
    Unsigned(u64),
    Signed(i64),
    Float(f64),
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
pub struct FiniteFloat(f64);

impl FiniteFloat {
    #[must_use]
    pub const fn new(value: f64) -> Option<Self> {
        if value.is_finite() {
            Some(FiniteFloat(value))
        } else {
            None
        }
    }
}

impl<'de> Deserialize<'de> for FiniteFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        FiniteFloat::new(value).ok_or_else(|| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Float(value),
                &"a finite floating point number",
            )
        })
    }
}

impl TryFrom<f64> for FiniteFloat {
    type Error = f64;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        FiniteFloat::new(value).ok_or(value)
    }
}

impl TryFrom<f32> for FiniteFloat {
    type Error = f32;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        FiniteFloat::new(value.into()).ok_or(value)
    }
}

impl From<i32> for FiniteFloat {
    fn from(value: i32) -> Self {
        FiniteFloat(value.into())
    }
}

impl From<u32> for FiniteFloat {
    fn from(value: u32) -> Self {
        FiniteFloat(value.into())
    }
}

impl From<i16> for FiniteFloat {
    fn from(value: i16) -> Self {
        FiniteFloat(value.into())
    }
}

impl From<u16> for FiniteFloat {
    fn from(value: u16) -> Self {
        FiniteFloat(value.into())
    }
}

impl From<i8> for FiniteFloat {
    fn from(value: i8) -> Self {
        FiniteFloat(value.into())
    }
}

impl From<u8> for FiniteFloat {
    fn from(value: u8) -> Self {
        FiniteFloat(value.into())
    }
}

impl Ord for FiniteFloat {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for FiniteFloat {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == core::cmp::Ordering::Equal
    }
}

impl Eq for FiniteFloat {}

impl PartialOrd for FiniteFloat {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for FiniteFloat {
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
