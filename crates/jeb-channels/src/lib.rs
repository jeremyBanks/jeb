use jeb_values::Item;

pub type Sender = tokio::sync::mpsc::Sender<Item>;
pub type Receiver = tokio_stream::wrappers::ReceiverStream<Item>;

#[must_use]
pub fn channel() -> (Sender, Receiver) {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    let receiver = tokio_stream::wrappers::ReceiverStream::new(receiver);

    (sender, receiver)
}
