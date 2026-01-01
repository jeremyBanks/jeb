use {crate::Panic, jeb_stream::Item, tokio::task::JoinHandle};
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
