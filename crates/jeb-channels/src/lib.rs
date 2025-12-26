use derive_more::{Deref, DerefMut};
use jeb_values::Item;

#[derive(Deref, DerefMut)]
pub struct Input<T = Item> {
    sender: tokio::sync::mpsc::Sender<T>,
}

pub struct Output<T = Item> {
    receiver: tokio::sync::mpsc::Receiver<T>,
}

pub type Task = tokio::task::JoinHandle<()>;

#[must_use]
pub fn channel<T>() -> (Input<T>, Output<T>) {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    (Input { sender }, Output { receiver })
}
