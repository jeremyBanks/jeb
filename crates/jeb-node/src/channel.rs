use derive_more::{Deref, DerefMut};
use jeb_values::Item;

pub struct Sender<T = Result<Item, &'static str>> {
    sender: tokio::sync::mpsc::Sender<T>,
}

impl<T> Sender<T> {
    pub async fn send(&self, item: T) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
        self.sender.send(item).await
    }
}

pub struct Receiver<T = Item> {
    receiver: tokio::sync::mpsc::Receiver<T>,
}

impl<T> Receiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        self.receiver.recv().await
    }
}

#[must_use]
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    (Sender { sender }, Receiver { receiver })
}
