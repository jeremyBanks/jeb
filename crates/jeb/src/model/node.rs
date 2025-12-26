use derive_more::{From, IsVariant, TryUnwrap, Unwrap};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use super::{Bytes, Text, Value};
use crate::Panic;



pub trait Node {
    fn spawn(&self, stack: Vec<Receiver>) -> (Vec<Receiver>, Task);
}

pub type Task = JoinHandle<Result<(), Panic>>;

pub type Sender = tokio::sync::mpsc::Sender<Item>;

pub type Receiver = tokio_stream::wrappers::ReceiverStream<Item>;

#[must_use]
pub fn channel() -> (Sender, Receiver) {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    let receiver = tokio_stream::wrappers::ReceiverStream::new(receiver);

    (sender, receiver)
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
