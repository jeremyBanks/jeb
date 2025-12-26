use std::any::Any;

use derive_more::{Deref, DerefMut};
use jeb_values::Item;

pub type Task = tokio::task::JoinHandle<()>;

#[derive(Deref, DerefMut)]
pub struct Input<T = Item> {
    #[deref]
    sender: tokio::sync::mpsc::Sender<T>,
}

#[derive(Deref, DerefMut)]
pub struct Output<T = Item> {
    #[deref]
    receiver: tokio::sync::mpsc::Receiver<T>,
}

#[must_use]
pub fn channel<T>() -> (Input<T>, Output<T>) {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    (Input { sender }, Output { receiver })
}
